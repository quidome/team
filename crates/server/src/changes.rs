//! The open changes: one row for each game with a change the coordinator has not seen yet (table
//! `changes`, decision 0016). A row keeps the game as the coordinator last saw it; the kind of
//! change is worked out from it and the stored game (`team_core::change_between`). "Seen" deletes
//! the row.

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use team_core::{
    ChangeUpdate, ClubNumber, Game, OpenChange, Place, PlaceNumber, Season, Side, StoredGame,
    TeamNumber,
};

use crate::db::StoreError;

/// The open changes of `club` and `season`, locked until the end of the transaction, so a "seen"
/// waits for the reading that holds them.
pub async fn open_for_update(
    db: &mut PgConnection,
    club: ClubNumber,
    season: Season,
) -> Result<Vec<OpenChange>, StoreError> {
    let rows = sqlx::query!(
        "SELECT source_id, last_seen FROM changes
         WHERE club_number = $1 AND season_year = $2
         ORDER BY source_id
         FOR UPDATE",
        club.0,
        season.start_year(),
    )
    .fetch_all(db)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(OpenChange {
                source_id: row.source_id,
                last_seen: row
                    .last_seen
                    .map(|value| last_seen_from_json(value, season))
                    .transpose()?,
            })
        })
        .collect()
}

/// The open changes of `club` and `season` with their versions, for the list of changes.
pub async fn open(
    db: &mut PgConnection,
    club: ClubNumber,
    season: Season,
) -> Result<Vec<(OpenChange, i64)>, StoreError> {
    let rows = sqlx::query!(
        "SELECT source_id, last_seen, version FROM changes
         WHERE club_number = $1 AND season_year = $2
         ORDER BY source_id",
        club.0,
        season.start_year(),
    )
    .fetch_all(db)
    .await?;
    rows.into_iter()
        .map(|row| {
            let open = OpenChange {
                source_id: row.source_id,
                last_seen: row
                    .last_seen
                    .map(|value| last_seen_from_json(value, season))
                    .transpose()?,
            };
            Ok((open, row.version))
        })
        .collect()
}

/// Marks changes as seen: deletes each one whose version is still `version`. A change that a
/// reading updated or made again after the screen loaded it has another version and stays. The
/// caller holds the reading lock, so a reading never runs in between.
pub async fn mark_seen(
    db: &mut PgConnection,
    club: ClubNumber,
    season: Season,
    seen: &[(i64, i64)],
) -> Result<(), StoreError> {
    let (ids, versions): (Vec<i64>, Vec<i64>) = seen.iter().copied().unzip();
    sqlx::query!(
        "DELETE FROM changes c
         USING unnest($3::bigint[], $4::bigint[]) AS seen(source_id, version)
         WHERE c.club_number = $1 AND c.season_year = $2
           AND c.source_id = seen.source_id AND c.version = seen.version",
        club.0,
        season.start_year(),
        &ids,
        &versions,
    )
    .execute(db)
    .await?;
    Ok(())
}

/// Applies the updates of one reading. The games must be written first: a change points to its
/// stored game.
pub async fn apply(
    db: &mut PgConnection,
    club: ClubNumber,
    season: Season,
    updates: &[ChangeUpdate],
    now: DateTime<Utc>,
) -> Result<(), StoreError> {
    for update in updates {
        match update {
            ChangeUpdate::Insert {
                source_id,
                last_seen,
            } => {
                let last_seen = last_seen.as_deref().map(last_seen_to_json).transpose()?;
                sqlx::query!(
                    "INSERT INTO changes (club_number, season_year, source_id, last_seen, updated_at)
                     VALUES ($1, $2, $3, $4, $5)",
                    club.0,
                    season.start_year(),
                    source_id,
                    last_seen,
                    now,
                )
                .execute(&mut *db)
                .await?;
            }
            ChangeUpdate::Touch { source_id } => {
                sqlx::query!(
                    "UPDATE changes SET version = nextval('change_versions'), updated_at = $4
                     WHERE club_number = $1 AND season_year = $2 AND source_id = $3",
                    club.0,
                    season.start_year(),
                    source_id,
                    now,
                )
                .execute(&mut *db)
                .await?;
            }
            ChangeUpdate::Remove { source_id } => {
                sqlx::query!(
                    "DELETE FROM changes
                     WHERE club_number = $1 AND season_year = $2 AND source_id = $3",
                    club.0,
                    season.start_year(),
                    source_id,
                )
                .execute(&mut *db)
                .await?;
            }
        }
    }
    Ok(())
}

/// Removes every open change (another club or team was chosen).
pub async fn remove_all(db: &mut PgConnection) -> Result<(), StoreError> {
    sqlx::query!("DELETE FROM changes").execute(db).await?;
    Ok(())
}

/// The game as last seen, as stored in the `last_seen` column. The hall and town names are kept
/// as they were, so a place with a new name still shows its old name as the old value.
#[derive(Debug, Serialize, Deserialize)]
struct SeenGame {
    source_id: i64,
    game_number: String,
    date: NaiveDate,
    start_time: Option<NaiveTime>,
    home: SeenSide,
    away: SeenSide,
    place_number: i64,
    hall: String,
    town: String,
    court: Option<String>,
    competition: String,
    in_list: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct SeenSide {
    team: i64,
    team_name: String,
    club: i64,
}

impl From<&Side> for SeenSide {
    fn from(side: &Side) -> Self {
        Self {
            team: side.team.0,
            team_name: side.team_name.clone(),
            club: side.club.0,
        }
    }
}

impl From<SeenSide> for Side {
    fn from(side: SeenSide) -> Self {
        Self {
            team: TeamNumber(side.team),
            team_name: side.team_name,
            club: ClubNumber(side.club),
        }
    }
}

fn last_seen_to_json(stored: &StoredGame) -> Result<serde_json::Value, StoreError> {
    let game = &stored.game;
    let seen = SeenGame {
        source_id: game.source_id,
        game_number: game.game_number.clone(),
        date: game.date,
        start_time: game.start_time,
        home: SeenSide::from(&game.home),
        away: SeenSide::from(&game.away),
        place_number: game.place.number.0,
        hall: game.place.hall.clone(),
        town: game.place.town.clone(),
        court: game.place.court.clone(),
        competition: game.competition.clone(),
        in_list: stored.in_list,
    };
    serde_json::to_value(seen).map_err(|error| StoreError::Invalid(error.to_string()))
}

fn last_seen_from_json(value: serde_json::Value, season: Season) -> Result<StoredGame, StoreError> {
    let seen: SeenGame = serde_json::from_value(value)
        .map_err(|error| StoreError::Invalid(format!("changes.last_seen: {error}")))?;
    Ok(StoredGame {
        game: Game {
            source_id: seen.source_id,
            game_number: seen.game_number,
            season,
            date: seen.date,
            start_time: seen.start_time,
            home: seen.home.into(),
            away: seen.away.into(),
            place: Place {
                number: PlaceNumber(seen.place_number),
                hall: seen.hall,
                town: seen.town,
                court: seen.court,
            },
            competition: seen.competition,
        },
        in_list: seen.in_list,
    })
}
