//! One reading of the game list: read the source, compare with the stored games, store the
//! result. Readings take a PostgreSQL advisory lock, so two at the same time (a tap and the
//! nightly reading) never count a change twice or lose one (decision 0005).

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use team_core::{
    Change, ClubNumber, GameChange, Guard, Season, TeamNumber, compare, count, guard,
    same_number_pairs,
};

use crate::clock::{Clock, current_season};
use crate::db::StoreError;
use crate::followed_teams;
use crate::games;
use crate::readings::{self, FailureKind, Reading, ReadingResult};
use crate::source::{Source, SourceError};

/// The key of the advisory lock that every reading takes ("team" "read" in ASCII).
const READING_LOCK: i64 = 0x7465_616d_7265_6164;

#[derive(Debug, thiserror::Error)]
pub enum ReadingError {
    #[error("no club chosen")]
    NoChoice,
    #[error(transparent)]
    Store(#[from] StoreError),
    /// The source client could not be made or used; not a failure of the source itself.
    #[error(transparent)]
    Client(SourceError),
}

impl From<sqlx::Error> for ReadingError {
    fn from(error: sqlx::Error) -> Self {
        Self::Store(StoreError::Database(error))
    }
}

/// The club and team a reading is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Target {
    pub season: Season,
    pub club: ClubNumber,
    /// `None` while no team is chosen for the season.
    pub team: Option<TeamNumber>,
    /// When the choice that gives this club and team was saved.
    pub chosen_at: DateTime<Utc>,
}

/// The club and team of the current season: the choice of the current season; or, in a new season
/// with no choice yet, the club of the newest earlier choice without its team. `None` when nothing
/// was ever chosen.
pub async fn target(pool: &PgPool, clock: &dyn Clock) -> Result<Option<Target>, StoreError> {
    let season = current_season(clock);
    let (choice, team) = match followed_teams::for_season(pool, season).await? {
        Some(choice) => {
            let team = choice.team.as_ref().map(|team| team.number);
            (choice, team)
        }
        None => match followed_teams::newest_before(pool, season).await? {
            Some(earlier) => (earlier, None),
            None => return Ok(None),
        },
    };
    let Some(chosen_at) = followed_teams::chosen_at(pool, choice.season).await? else {
        return Ok(None);
    };
    Ok(Some(Target {
        season,
        club: choice.club.number,
        team,
        chosen_at,
    }))
}

/// Reads the game list of the current club and season and records the reading. With `guard_on`
/// false, a reading with no or far fewer games is not held back ("these games are really gone").
/// A failed or held-back reading is a recorded result, not an error.
pub async fn read(
    pool: &PgPool,
    source: &Source,
    clock: &dyn Clock,
    guard_on: bool,
) -> Result<Reading, ReadingError> {
    let started_at = clock.now();
    let target = target(pool, clock).await?.ok_or(ReadingError::NoChoice)?;
    let Target { season, club, .. } = target;

    // The source is read before the lock, so a slow source never holds the lock.
    let read = match source.club_games(club, season).await {
        Ok(games) => games,
        Err(SourceError::Client(error)) => {
            return Err(ReadingError::Client(SourceError::Client(error)));
        }
        Err(error) => {
            let kind = match error {
                SourceError::Format { .. } => FailureKind::SourceFormat,
                _ => FailureKind::SourceUnavailable,
            };
            let result = ReadingResult::Failed(kind);
            return record(pool, target, started_at, clock.now(), result).await;
        }
    };

    let mut tx = pool.begin().await?;
    sqlx::query!("SELECT pg_advisory_xact_lock($1)", READING_LOCK)
        .execute(&mut *tx)
        .await?;

    let stored = games::stored(&mut tx, club, season).await?;
    let last_good = readings::last_good(&mut *tx, club, season).await?;
    let games_read = u32::try_from(read.len())
        .map_err(|_| StoreError::Invalid(format!("too many games: {}", read.len())))?;

    if guard_on {
        let before = last_good.as_ref().and_then(|reading| match reading.result {
            ReadingResult::Good { games_read, .. } => Some(games_read),
            _ => None,
        });
        if let Guard::HoldBack(reason) = guard(before, games_read) {
            let result = ReadingResult::HeldBack(reason);
            let finished_at = clock.now();
            let id =
                readings::insert(&mut *tx, club, season, started_at, finished_at, &result).await?;
            tx.commit().await?;
            return Ok(reading(id, target, started_at, finished_at, result));
        }
    }

    let first_reading = !readings::good_since(&mut *tx, club, season, target.chosen_at).await?;
    let counts = if first_reading {
        None
    } else {
        let changes = compare(&stored, &read, club, target.team);
        log_same_numbers(&changes);
        Some(count(&changes))
    };

    let read_ids: BTreeSet<i64> = read.iter().map(|game| game.source_id).collect();
    let gone: Vec<i64> = stored
        .iter()
        .filter(|stored| stored.in_list && !read_ids.contains(&stored.game.source_id))
        .map(|stored| stored.game.source_id)
        .collect();
    let finished_at = clock.now();
    games::write(&mut tx, club, season, &read, &gone, finished_at).await?;

    let result = ReadingResult::Good {
        games_read,
        first_reading,
        counts,
    };
    let id = readings::insert(&mut *tx, club, season, started_at, finished_at, &result).await?;
    tx.commit().await?;
    Ok(reading(id, target, started_at, finished_at, result))
}

async fn record(
    pool: &PgPool,
    target: Target,
    started_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
    result: ReadingResult,
) -> Result<Reading, ReadingError> {
    let id = readings::insert(
        pool,
        target.club,
        target.season,
        started_at,
        finished_at,
        &result,
    )
    .await?;
    Ok(reading(id, target, started_at, finished_at, result))
}

fn reading(
    id: i64,
    target: Target,
    started_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
    result: ReadingResult,
) -> Reading {
    Reading {
        id,
        club: target.club,
        season: target.season,
        started_at,
        finished_at,
        result,
    }
}

/// A gone and a new game with the same game number may be one game whose source id changed.
/// Only ids and game numbers go to the log.
fn log_same_numbers(changes: &[GameChange]) {
    for (gone_id, new_id) in same_number_pairs(changes) {
        let game_number = changes
            .iter()
            .find(|change| change.source_id == gone_id && change.change == Change::Gone)
            .and_then(|change| change.before.as_ref())
            .map(|game| game.game_number.as_str())
            .unwrap_or_default();
        tracing::warn!(
            gone_id,
            new_id,
            game_number,
            "a gone and a new game have the same game number"
        );
    }
}
