//! One reading of the game list: read the source, compare with the stored games, store the
//! result and the open changes. Readings take a PostgreSQL advisory lock, so two at the same
//! time (a tap and the nightly reading) never count a change twice or lose one (decision 0005).

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use team_core::{
    Change, ClubNumber, GameChange, Guard, HoldReason, Season, TeamNumber, compare, count, guard,
    guard_after_confirm, is_first_reading, no_longer_in_list, same_number_pairs,
    update_open_changes,
};

use crate::changes;
use crate::clock::{Clock, current_season, today_in_netherlands};
use crate::db::{self, StoreError};
use crate::followed_teams;
use crate::games;
use crate::readings::{self, FailureKind, Reading, ReadingResult};
use crate::source::{Source, SourceError};

#[derive(Debug, thiserror::Error)]
pub enum ReadingError {
    #[error("no club chosen")]
    NoChoice,
    /// The reading to confirm is not the newest reading that did not fail, or is not held back.
    #[error("not held back")]
    NotHeldBack,
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

/// How the guard applies to a reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// A reading with no or far fewer games is held back.
    Guarded,
    /// The coordinator confirmed that the games of this held-back reading are really gone.
    Confirm { held_id: i64 },
}

/// The club and team a reading is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Target {
    pub season: Season,
    pub club: ClubNumber,
    /// `None` while no team is chosen for the season.
    pub team: Option<TeamNumber>,
}

/// The club and team of the current season: the choice of the current season; or, in a new season
/// with no choice yet, the club of the newest earlier choice without its team. `None` when nothing
/// was ever chosen.
pub async fn target(pool: &PgPool, clock: &dyn Clock) -> Result<Option<Target>, StoreError> {
    let season = current_season(clock);
    let (club, team) = match followed_teams::for_season(pool, season).await? {
        Some(choice) => (
            choice.club.number,
            choice.team.as_ref().map(|team| team.number),
        ),
        None => match followed_teams::newest_before(pool, season).await? {
            Some(earlier) => (earlier.club.number, None),
            None => return Ok(None),
        },
    };
    Ok(Some(Target { season, club, team }))
}

/// Reads the game list of the current club and season and records the reading. A failed or
/// held-back reading is a recorded result, not an error.
pub async fn read(
    pool: &PgPool,
    source: &Source,
    clock: &dyn Clock,
    mode: Mode,
) -> Result<Reading, ReadingError> {
    let started_at = clock.now();
    let target = target(pool, clock).await?.ok_or(ReadingError::NoChoice)?;
    let Target { season, club, team } = target;

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
            let finished_at = clock.now();
            let id = readings::insert(pool, club, season, team, started_at, finished_at, &result)
                .await?;
            return Ok(reading(id, target, started_at, finished_at, result));
        }
    };

    let mut tx = pool.begin().await?;
    db::lock_readings(&mut tx).await?;

    // A reading that read the source later already stored newer games: storing this older
    // answer would count the changes in between backwards, and the next reading forwards again.
    if let Some(newer) = readings::good_started_after(&mut *tx, club, season, started_at).await? {
        tx.commit().await?;
        return Ok(newer);
    }

    let games_read = u32::try_from(read.len())
        .map_err(|_| StoreError::Invalid(format!("too many games: {}", read.len())))?;
    let last_good = readings::last_good(&mut *tx, club, season).await?;
    let last_good_count = last_good.as_ref().and_then(|reading| match reading.result {
        ReadingResult::Good { games_read, .. } => Some(games_read),
        _ => None,
    });
    let verdict = match mode {
        Mode::Guarded => guard(last_good_count, games_read),
        Mode::Confirm { held_id } => {
            let held = readings::newest_not_failed(&mut *tx, club, season).await?;
            let confirmed = match held {
                Some(Reading {
                    id,
                    result: ReadingResult::HeldBack(reason),
                    ..
                }) if id == held_id => match reason {
                    HoldReason::Empty { .. } => 0,
                    HoldReason::FarFewer { now, .. } => now,
                },
                _ => return Err(ReadingError::NotHeldBack),
            };
            guard_after_confirm(last_good_count, confirmed, games_read)
        }
    };
    if let Guard::HoldBack(reason) = verdict {
        let result = ReadingResult::HeldBack(reason);
        let finished_at = clock.now();
        let id = readings::insert(
            &mut *tx,
            club,
            season,
            team,
            started_at,
            finished_at,
            &result,
        )
        .await?;
        tx.commit().await?;
        return Ok(reading(id, target, started_at, finished_at, result));
    }

    let stored = games::stored(&mut tx, club, season).await?;
    let open = changes::open_for_update(&mut tx, club, season).await?;
    let today = today_in_netherlands(clock);
    let first_reading = is_first_reading(last_good.map(|reading| reading.team), team);
    let counts = if first_reading {
        None
    } else {
        let changes = compare(&stored, &read, club, team, today);
        log_same_numbers(&changes);
        Some(count(&changes))
    };
    // The first reading after a choice keeps no changes; without a team there is no schedule.
    // The save of another choice already removed the open changes, under the same lock.
    let updates = match team {
        Some(team) if !first_reading => {
            update_open_changes(&open, &stored, &read, club, team, today)
        }
        _ => Vec::new(),
    };
    let gone: Vec<i64> = no_longer_in_list(&stored, &read)
        .iter()
        .map(|stored| stored.game.source_id)
        .collect();
    let finished_at = clock.now();
    games::write(&mut tx, club, season, &read, &gone, finished_at).await?;
    changes::apply(&mut tx, club, season, &updates, finished_at).await?;

    let result = ReadingResult::Good {
        games_read,
        first_reading,
        counts,
    };
    let id = readings::insert(
        &mut *tx,
        club,
        season,
        team,
        started_at,
        finished_at,
        &result,
    )
    .await?;
    tx.commit().await?;
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
        team: target.team,
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
