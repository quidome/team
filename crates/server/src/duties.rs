//! The duties of games (tables `game_duties` and `duty_teams`, story A of duties; and
//! `duty_assignments`, the players on the places, story B). Every write takes the row lock of its
//! game first (`lock`): two screens wait for each other.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgExecutor};
use team_core::{
    Assignment, ClubNumber, DutyKind, Outcome, PlaceKind, PlayerId, Season, StoredDuties,
    TeamNumber, TeamRef,
};

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

fn duty_kind_text(kind: DutyKind) -> &'static str {
    match kind {
        DutyKind::Drive => "drive",
        DutyKind::Table => "table",
        DutyKind::Referee => "referee",
    }
}

fn duty_kind_of(text: &str) -> Result<DutyKind, StoreError> {
    match text {
        "drive" => Ok(DutyKind::Drive),
        "table" => Ok(DutyKind::Table),
        "referee" => Ok(DutyKind::Referee),
        other => Err(StoreError::Invalid(format!("duty kind {other}"))),
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

struct AssignmentRow {
    source_id: i64,
    kind: String,
    number: i16,
    planned_player: i64,
    outcome: String,
    done_by: Option<i64>,
}

fn assignment_of(row: AssignmentRow) -> Result<(i64, Assignment), StoreError> {
    let outcome = match (row.outcome.as_str(), row.done_by) {
        ("as_planned", None) => Outcome::AsPlanned,
        ("done_by", Some(player)) => Outcome::DoneBy(PlayerId(player)),
        ("not_done", None) => Outcome::NotDone,
        (other, _) => return Err(StoreError::Invalid(format!("duty outcome {other}"))),
    };
    Ok((
        row.source_id,
        Assignment {
            kind: duty_kind_of(&row.kind)?,
            number: small(row.number)?,
            planned: PlayerId(row.planned_player),
            outcome,
        },
    ))
}

/// The players on the places of the games of `club` in `season`, by source id, in the order of
/// kind and number.
pub async fn assignments_for_season(
    db: &mut PgConnection,
    club: ClubNumber,
    season: Season,
) -> Result<BTreeMap<i64, Vec<Assignment>>, StoreError> {
    let rows = sqlx::query_as!(
        AssignmentRow,
        "SELECT source_id, kind, number, planned_player, outcome, done_by
         FROM duty_assignments WHERE club_number = $1 AND season_year = $2
         ORDER BY source_id, kind, number",
        club.0,
        season.start_year(),
    )
    .fetch_all(&mut *db)
    .await?;
    let mut assignments: BTreeMap<i64, Vec<Assignment>> = BTreeMap::new();
    for row in rows {
        let (source_id, assignment) = assignment_of(row)?;
        assignments.entry(source_id).or_default().push(assignment);
    }
    Ok(assignments)
}

/// The players on the places of one game; the game must be locked.
pub async fn assignments_of_game(
    tx: &mut PgConnection,
    key: GameKey,
) -> Result<Vec<Assignment>, StoreError> {
    let rows = sqlx::query_as!(
        AssignmentRow,
        "SELECT source_id, kind, number, planned_player, outcome, done_by
         FROM duty_assignments
         WHERE club_number = $1 AND season_year = $2 AND source_id = $3
         ORDER BY kind, number",
        key.club.0,
        key.season.start_year(),
        key.source_id,
    )
    .fetch_all(&mut *tx)
    .await?;
    rows.into_iter()
        .map(|row| assignment_of(row).map(|(_, assignment)| assignment))
        .collect()
}

/// Puts `player` on a place of the game, as planned (a correction is reset); the game must be
/// locked.
pub async fn set_assignment(
    tx: &mut PgConnection,
    key: GameKey,
    kind: DutyKind,
    number: u8,
    player: PlayerId,
    now: DateTime<Utc>,
) -> Result<(), StoreError> {
    sqlx::query!(
        "INSERT INTO duty_assignments
             (club_number, season_year, source_id, kind, number, planned_player)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (club_number, season_year, source_id, kind, number)
         DO UPDATE SET planned_player = EXCLUDED.planned_player, outcome = 'as_planned',
                       done_by = NULL",
        key.club.0,
        key.season.start_year(),
        key.source_id,
        duty_kind_text(kind),
        i16::from(number),
        player.0,
    )
    .execute(&mut *tx)
    .await?;
    touch(tx, key, now).await
}

/// Makes a place of the game open again; the game must be locked.
pub async fn clear_assignment(
    tx: &mut PgConnection,
    key: GameKey,
    kind: DutyKind,
    number: u8,
    now: DateTime<Utc>,
) -> Result<(), StoreError> {
    sqlx::query!(
        "DELETE FROM duty_assignments
         WHERE club_number = $1 AND season_year = $2 AND source_id = $3
           AND kind = $4 AND number = $5",
        key.club.0,
        key.season.start_year(),
        key.source_id,
        duty_kind_text(kind),
        i16::from(number),
    )
    .execute(&mut *tx)
    .await?;
    touch(tx, key, now).await
}

/// The outcome of a filled place after the game; the game must be locked.
pub async fn set_outcome(
    tx: &mut PgConnection,
    key: GameKey,
    kind: DutyKind,
    number: u8,
    outcome: Outcome,
    now: DateTime<Utc>,
) -> Result<(), StoreError> {
    let (text, done_by) = match outcome {
        Outcome::AsPlanned => ("as_planned", None),
        Outcome::DoneBy(player) => ("done_by", Some(player.0)),
        Outcome::NotDone => ("not_done", None),
    };
    sqlx::query!(
        "UPDATE duty_assignments SET outcome = $6, done_by = $7
         WHERE club_number = $1 AND season_year = $2 AND source_id = $3
           AND kind = $4 AND number = $5",
        key.club.0,
        key.season.start_year(),
        key.source_id,
        duty_kind_text(kind),
        i16::from(number),
        text,
        done_by,
    )
    .execute(&mut *tx)
    .await?;
    touch(tx, key, now).await
}

/// Whether `player` is on a place, or did one, in any season ("Delete player" is then refused).
pub async fn player_has_duties(
    db: impl PgExecutor<'_>,
    player: PlayerId,
) -> Result<bool, StoreError> {
    let found = sqlx::query_scalar!(
        r#"SELECT EXISTS (
               SELECT 1 FROM duty_assignments WHERE planned_player = $1 OR done_by = $1
           ) AS "found!""#,
        player.0
    )
    .fetch_one(db)
    .await?;
    Ok(found)
}

/// The games of `club` in `season` with duties of `team`: a place that names the team, a number
/// of drivers above 0, or a player on a place. A change to such a game stays in the list of
/// changes also when the game is past (`team_core::change_between`).
pub async fn with_duties(
    db: impl PgExecutor<'_>,
    club: ClubNumber,
    season: Season,
    team: TeamNumber,
) -> Result<BTreeSet<i64>, StoreError> {
    let ids = sqlx::query_scalar!(
        r#"SELECT source_id AS "source_id!" FROM duty_teams
           WHERE club_number = $1 AND season_year = $2 AND team_number = $3
           UNION
           SELECT source_id FROM game_duties
           WHERE club_number = $1 AND season_year = $2 AND drivers > 0
           UNION
           SELECT source_id FROM duty_assignments
           WHERE club_number = $1 AND season_year = $2"#,
        club.0,
        season.start_year(),
        team.0,
    )
    .fetch_all(db)
    .await?;
    Ok(ids.into_iter().collect())
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
