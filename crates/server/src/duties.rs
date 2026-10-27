//! The duties of games (tables `game_duties` and `duty_teams`; story A of duties). Every write
//! takes the row lock of its game first (`lock`): two screens wait for each other.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use sqlx::PgConnection;
use team_core::{ClubNumber, PlaceKind, Season, StoredDuties, TeamNumber, TeamRef};

use crate::db::StoreError;

/// The key of a game: club, season and the source's game id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameKey {
    pub club: ClubNumber,
    pub season: Season,
    pub source_id: i64,
}

fn kind_text(kind: PlaceKind) -> &'static str {
    match kind {
        PlaceKind::Referee => "referee",
        PlaceKind::Table => "table",
    }
}

fn kind_of(text: &str) -> Result<PlaceKind, StoreError> {
    match text {
        "referee" => Ok(PlaceKind::Referee),
        "table" => Ok(PlaceKind::Table),
        other => Err(StoreError::Invalid(format!("duty place kind {other}"))),
    }
}

fn small(value: i16) -> Result<u8, StoreError> {
    u8::try_from(value).map_err(|_| StoreError::Invalid(format!("duty number {value}")))
}

struct DutiesRow {
    source_id: i64,
    entered: bool,
    drivers: Option<i16>,
}

struct TeamRow {
    source_id: i64,
    kind: String,
    number: i16,
    team_number: i64,
    team_name: String,
}

fn duties_of(row: &DutiesRow) -> Result<StoredDuties, StoreError> {
    Ok(StoredDuties {
        schedule_entered: row.entered,
        drivers: row.drivers.map(small).transpose()?,
        teams: BTreeMap::new(),
    })
}

/// The place and the team of a row of `duty_teams`.
fn place_of(row: TeamRow) -> Result<((PlaceKind, u8), TeamRef), StoreError> {
    Ok((
        (kind_of(&row.kind)?, small(row.number)?),
        TeamRef {
            number: TeamNumber(row.team_number),
            name: row.team_name,
        },
    ))
}

/// The stored duties of the games of `club` in `season`, by source id.
pub async fn for_season(
    db: &mut PgConnection,
    club: ClubNumber,
    season: Season,
) -> Result<BTreeMap<i64, StoredDuties>, StoreError> {
    let rows = sqlx::query_as!(
        DutiesRow,
        r#"SELECT source_id, schedule_entered_at IS NOT NULL AS "entered!", drivers
           FROM game_duties WHERE club_number = $1 AND season_year = $2"#,
        club.0,
        season.start_year(),
    )
    .fetch_all(&mut *db)
    .await?;
    let mut duties = BTreeMap::new();
    for row in &rows {
        duties.insert(row.source_id, duties_of(row)?);
    }
    let teams = sqlx::query_as!(
        TeamRow,
        "SELECT source_id, kind, number, team_number, team_name
         FROM duty_teams WHERE club_number = $1 AND season_year = $2",
        club.0,
        season.start_year(),
    )
    .fetch_all(&mut *db)
    .await?;
    for row in teams {
        let source_id = row.source_id;
        let (place, team) = place_of(row)?;
        duties
            .entry(source_id)
            .or_default()
            .teams
            .insert(place, team);
    }
    Ok(duties)
}

/// Locks the row of the game for a write, inserting it first when it is missing; gives its
/// stored duties. Only inside the transaction of a write: when the write fails, the insert rolls
/// back with it.
pub async fn lock(
    tx: &mut PgConnection,
    key: GameKey,
    now: DateTime<Utc>,
) -> Result<StoredDuties, StoreError> {
    sqlx::query!(
        "INSERT INTO game_duties (club_number, season_year, source_id, saved_at)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (club_number, season_year, source_id) DO NOTHING",
        key.club.0,
        key.season.start_year(),
        key.source_id,
        now,
    )
    .execute(&mut *tx)
    .await?;
    let row = sqlx::query_as!(
        DutiesRow,
        r#"SELECT source_id, schedule_entered_at IS NOT NULL AS "entered!", drivers
           FROM game_duties WHERE club_number = $1 AND season_year = $2 AND source_id = $3
           FOR UPDATE"#,
        key.club.0,
        key.season.start_year(),
        key.source_id,
    )
    .fetch_one(&mut *tx)
    .await?;
    let teams = sqlx::query_as!(
        TeamRow,
        "SELECT source_id, kind, number, team_number, team_name
         FROM duty_teams WHERE club_number = $1 AND season_year = $2 AND source_id = $3",
        key.club.0,
        key.season.start_year(),
        key.source_id,
    )
    .fetch_all(&mut *tx)
    .await?;
    let mut duties = duties_of(&row)?;
    for row in teams {
        let (place, team) = place_of(row)?;
        duties.teams.insert(place, team);
    }
    Ok(duties)
}

async fn touch(tx: &mut PgConnection, key: GameKey, now: DateTime<Utc>) -> Result<(), StoreError> {
    sqlx::query!(
        "UPDATE game_duties SET saved_at = $4
         WHERE club_number = $1 AND season_year = $2 AND source_id = $3",
        key.club.0,
        key.season.start_year(),
        key.source_id,
        now,
    )
    .execute(tx)
    .await?;
    Ok(())
}

/// Puts `team` on a place of the game, or empties it (`None`); the game must be locked.
pub async fn set_team(
    tx: &mut PgConnection,
    key: GameKey,
    kind: PlaceKind,
    number: u8,
    team: Option<&TeamRef>,
    now: DateTime<Utc>,
) -> Result<(), StoreError> {
    match team {
        Some(team) => {
            sqlx::query!(
                "INSERT INTO duty_teams
                     (club_number, season_year, source_id, kind, number, team_number, team_name)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)
                 ON CONFLICT (club_number, season_year, source_id, kind, number)
                 DO UPDATE SET team_number = EXCLUDED.team_number, team_name = EXCLUDED.team_name",
                key.club.0,
                key.season.start_year(),
                key.source_id,
                kind_text(kind),
                i16::from(number),
                team.number.0,
                team.name,
            )
            .execute(&mut *tx)
            .await?;
        }
        None => {
            sqlx::query!(
                "DELETE FROM duty_teams
                 WHERE club_number = $1 AND season_year = $2 AND source_id = $3
                   AND kind = $4 AND number = $5",
                key.club.0,
                key.season.start_year(),
                key.source_id,
                kind_text(kind),
                i16::from(number),
            )
            .execute(&mut *tx)
            .await?;
        }
    }
    touch(tx, key, now).await
}

/// "Schedule entered" on (the first moment stays) or off; the game must be locked.
pub async fn set_entered(
    tx: &mut PgConnection,
    key: GameKey,
    on: bool,
    now: DateTime<Utc>,
) -> Result<(), StoreError> {
    sqlx::query!(
        "UPDATE game_duties
         SET schedule_entered_at = CASE WHEN $4 THEN COALESCE(schedule_entered_at, $5) END,
             saved_at = $5
         WHERE club_number = $1 AND season_year = $2 AND source_id = $3",
        key.club.0,
        key.season.start_year(),
        key.source_id,
        on,
        now,
    )
    .execute(tx)
    .await?;
    Ok(())
}

/// The number of drivers (`None`: the proposal); the game must be locked.
pub async fn set_drivers(
    tx: &mut PgConnection,
    key: GameKey,
    drivers: Option<u8>,
    now: DateTime<Utc>,
) -> Result<(), StoreError> {
    sqlx::query!(
        "UPDATE game_duties SET drivers = $4, saved_at = $5
         WHERE club_number = $1 AND season_year = $2 AND source_id = $3",
        key.club.0,
        key.season.start_year(),
        key.source_id,
        drivers.map(i16::from),
        now,
    )
    .execute(tx)
    .await?;
    Ok(())
}
