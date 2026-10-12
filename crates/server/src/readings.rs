//! The record of every reading of the game list, also the failed and held-back ones (table
//! `readings`).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgExecutor;
use team_core::{ClubNumber, Counts, HoldReason, Season, TeamNumber};

use crate::db::StoreError;

/// Why a reading failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureKind {
    /// The source could not be reached, or answered with an error.
    SourceUnavailable,
    /// The source answered with data the tool does not understand.
    SourceFormat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadingResult {
    /// The games were written. `counts` is `None` for the first reading after a choice.
    Good {
        games_read: u32,
        first_reading: bool,
        counts: Option<Counts>,
    },
    /// The guard held the reading back; the games were not changed.
    HeldBack(HoldReason),
    Failed(FailureKind),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reading {
    pub id: i64,
    pub club: ClubNumber,
    pub season: Season,
    /// The team chosen when it was read; `None` while no team was chosen.
    pub team: Option<TeamNumber>,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub result: ReadingResult,
}

/// Records a reading and gives its id.
pub async fn insert(
    db: impl PgExecutor<'_>,
    club: ClubNumber,
    season: Season,
    team: Option<TeamNumber>,
    started_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
    result: &ReadingResult,
) -> Result<i64, StoreError> {
    let columns = Columns::of(result)?;
    let id = sqlx::query_scalar!(
        "INSERT INTO readings
             (club_number, season_year, team_number, started_at, finished_at, result, error,
              games_read, first_reading, counts, hold_reason, hold_before)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
         RETURNING id",
        club.0,
        season.start_year(),
        team.map(|team| team.0),
        started_at,
        finished_at,
        columns.result,
        columns.error,
        columns.games_read,
        columns.first_reading,
        columns.counts,
        columns.hold_reason,
        columns.hold_before,
    )
    .fetch_one(db)
    .await?;
    Ok(id)
}

/// The newest reading of `club` and `season`, of any result.
pub async fn latest(
    db: impl PgExecutor<'_>,
    club: ClubNumber,
    season: Season,
) -> Result<Option<Reading>, StoreError> {
    let row = sqlx::query_as!(
        Row,
        "SELECT id, club_number, season_year, team_number, started_at, finished_at, result, error,
                games_read, first_reading, counts, hold_reason, hold_before
         FROM readings WHERE club_number = $1 AND season_year = $2
         ORDER BY id DESC LIMIT 1",
        club.0,
        season.start_year(),
    )
    .fetch_optional(db)
    .await?;
    row.map(Row::into_reading).transpose()
}

/// The newest good reading of `club` and `season`.
pub async fn last_good(
    db: impl PgExecutor<'_>,
    club: ClubNumber,
    season: Season,
) -> Result<Option<Reading>, StoreError> {
    let row = sqlx::query_as!(
        Row,
        "SELECT id, club_number, season_year, team_number, started_at, finished_at, result, error,
                games_read, first_reading, counts, hold_reason, hold_before
         FROM readings WHERE club_number = $1 AND season_year = $2 AND result = 'good'
         ORDER BY id DESC LIMIT 1",
        club.0,
        season.start_year(),
    )
    .fetch_optional(db)
    .await?;
    row.map(Row::into_reading).transpose()
}

/// The newest reading of `club` and `season` that did not fail. A failed reading changes nothing,
/// so a held-back reading stays open behind it.
pub async fn newest_not_failed(
    db: impl PgExecutor<'_>,
    club: ClubNumber,
    season: Season,
) -> Result<Option<Reading>, StoreError> {
    let row = sqlx::query_as!(
        Row,
        "SELECT id, club_number, season_year, team_number, started_at, finished_at, result, error,
                games_read, first_reading, counts, hold_reason, hold_before
         FROM readings WHERE club_number = $1 AND season_year = $2 AND result <> 'failed'
         ORDER BY id DESC LIMIT 1",
        club.0,
        season.start_year(),
    )
    .fetch_optional(db)
    .await?;
    row.map(Row::into_reading).transpose()
}

/// The newest good reading of `club` and `season` that started after `at`: it read the source
/// later, so its games are newer.
pub async fn good_started_after(
    db: impl PgExecutor<'_>,
    club: ClubNumber,
    season: Season,
    at: DateTime<Utc>,
) -> Result<Option<Reading>, StoreError> {
    let row = sqlx::query_as!(
        Row,
        "SELECT id, club_number, season_year, team_number, started_at, finished_at, result, error,
                games_read, first_reading, counts, hold_reason, hold_before
         FROM readings
         WHERE club_number = $1 AND season_year = $2 AND result = 'good' AND started_at > $3
         ORDER BY id DESC LIMIT 1",
        club.0,
        season.start_year(),
        at,
    )
    .fetch_optional(db)
    .await?;
    row.map(Row::into_reading).transpose()
}

pub async fn by_id(db: impl PgExecutor<'_>, id: i64) -> Result<Option<Reading>, StoreError> {
    let row = sqlx::query_as!(
        Row,
        "SELECT id, club_number, season_year, team_number, started_at, finished_at, result, error,
                games_read, first_reading, counts, hold_reason, hold_before
         FROM readings WHERE id = $1",
        id,
    )
    .fetch_optional(db)
    .await?;
    row.map(Row::into_reading).transpose()
}

/// `Counts` as stored in the `counts` column.
#[derive(Serialize, Deserialize)]
struct StoredCounts {
    new: u32,
    moved: u32,
    start_time_known: u32,
    home_and_away: u32,
    team_changed: u32,
    gone: u32,
    came_back: u32,
    into_schedule: u32,
    out_of_schedule: u32,
}

impl From<Counts> for StoredCounts {
    fn from(counts: Counts) -> Self {
        Self {
            new: counts.new,
            moved: counts.moved,
            start_time_known: counts.start_time_known,
            home_and_away: counts.home_and_away,
            team_changed: counts.team_changed,
            gone: counts.gone,
            came_back: counts.came_back,
            into_schedule: counts.into_schedule,
            out_of_schedule: counts.out_of_schedule,
        }
    }
}

impl From<StoredCounts> for Counts {
    fn from(counts: StoredCounts) -> Self {
        Self {
            new: counts.new,
            moved: counts.moved,
            start_time_known: counts.start_time_known,
            home_and_away: counts.home_and_away,
            team_changed: counts.team_changed,
            gone: counts.gone,
            came_back: counts.came_back,
            into_schedule: counts.into_schedule,
            out_of_schedule: counts.out_of_schedule,
        }
    }
}

/// The columns that hold a `ReadingResult`.
struct Columns {
    result: &'static str,
    error: Option<&'static str>,
    games_read: Option<i32>,
    first_reading: bool,
    counts: Option<serde_json::Value>,
    hold_reason: Option<&'static str>,
    hold_before: Option<i32>,
}

impl Columns {
    fn of(result: &ReadingResult) -> Result<Self, StoreError> {
        let none = Columns {
            result: "",
            error: None,
            games_read: None,
            first_reading: false,
            counts: None,
            hold_reason: None,
            hold_before: None,
        };
        Ok(match result {
            ReadingResult::Good {
                games_read,
                first_reading,
                counts,
            } => Columns {
                result: "good",
                games_read: Some(to_i32(*games_read)?),
                first_reading: *first_reading,
                counts: counts
                    .map(|counts| serde_json::to_value(StoredCounts::from(counts)))
                    .transpose()
                    .map_err(|error| StoreError::Invalid(error.to_string()))?,
                ..none
            },
            ReadingResult::HeldBack(HoldReason::Empty { before }) => Columns {
                result: "held_back",
                games_read: Some(0),
                hold_reason: Some("empty"),
                hold_before: Some(to_i32(*before)?),
                ..none
            },
            ReadingResult::HeldBack(HoldReason::FarFewer { before, now }) => Columns {
                result: "held_back",
                games_read: Some(to_i32(*now)?),
                hold_reason: Some("far_fewer"),
                hold_before: Some(to_i32(*before)?),
                ..none
            },
            ReadingResult::Failed(kind) => Columns {
                result: "failed",
                error: Some(match kind {
                    FailureKind::SourceUnavailable => "source_unavailable",
                    FailureKind::SourceFormat => "source_format",
                }),
                ..none
            },
        })
    }
}

fn to_i32(number: u32) -> Result<i32, StoreError> {
    i32::try_from(number).map_err(|_| StoreError::Invalid(format!("number too large: {number}")))
}

fn to_u32(number: Option<i32>, column: &str) -> Result<u32, StoreError> {
    number
        .and_then(|number| u32::try_from(number).ok())
        .ok_or_else(|| StoreError::Invalid(format!("readings.{column}: {number:?}")))
}

struct Row {
    id: i64,
    club_number: i64,
    season_year: i32,
    team_number: Option<i64>,
    started_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
    result: String,
    error: Option<String>,
    games_read: Option<i32>,
    first_reading: bool,
    counts: Option<serde_json::Value>,
    hold_reason: Option<String>,
    hold_before: Option<i32>,
}

impl Row {
    fn into_reading(self) -> Result<Reading, StoreError> {
        let invalid = |column: &str, value: &Option<String>| {
            StoreError::Invalid(format!("readings.{column}: {value:?}"))
        };
        let result = match self.result.as_str() {
            "good" => ReadingResult::Good {
                games_read: to_u32(self.games_read, "games_read")?,
                first_reading: self.first_reading,
                counts: self
                    .counts
                    .map(serde_json::from_value::<StoredCounts>)
                    .transpose()
                    .map_err(|error| StoreError::Invalid(format!("readings.counts: {error}")))?
                    .map(Counts::from),
            },
            "held_back" => {
                let before = to_u32(self.hold_before, "hold_before")?;
                match self.hold_reason.as_deref() {
                    Some("empty") => ReadingResult::HeldBack(HoldReason::Empty { before }),
                    Some("far_fewer") => ReadingResult::HeldBack(HoldReason::FarFewer {
                        before,
                        now: to_u32(self.games_read, "games_read")?,
                    }),
                    _ => return Err(invalid("hold_reason", &self.hold_reason)),
                }
            }
            "failed" => ReadingResult::Failed(match self.error.as_deref() {
                Some("source_unavailable") => FailureKind::SourceUnavailable,
                Some("source_format") => FailureKind::SourceFormat,
                _ => return Err(invalid("error", &self.error)),
            }),
            _ => return Err(invalid("result", &Some(self.result))),
        };
        Ok(Reading {
            id: self.id,
            club: ClubNumber(self.club_number),
            season: Season::from_start_year(self.season_year),
            team: self.team_number.map(TeamNumber),
            started_at: self.started_at,
            finished_at: self.finished_at,
            result,
        })
    }
}
