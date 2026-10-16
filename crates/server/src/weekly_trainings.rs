//! The weekly trainings of each season (table `weekly_trainings`).

use chrono::{NaiveDate, NaiveTime, Weekday};
use sqlx::{PgConnection, PgExecutor};
use team_core::{Season, WeeklyTraining, WeeklyTrainingId};

use crate::db::StoreError;

/// The key of the advisory lock that every change of the weekly trainings takes ("team" "wktr" in
/// ASCII), so two changes at the same time cannot make an overlap.
const WEEKLY_TRAININGS_LOCK: i64 = 0x7465_616d_776b_7472;

/// Takes the lock of the weekly trainings until the end of the transaction. Take it before reading
/// the weekly trainings that a change is checked against.
pub async fn lock(tx: &mut PgConnection) -> Result<(), StoreError> {
    sqlx::query!("SELECT pg_advisory_xact_lock($1)", WEEKLY_TRAININGS_LOCK)
        .execute(tx)
        .await?;
    Ok(())
}

/// A new or changed weekly training, without its id and season.
#[derive(Debug, Clone, Copy)]
pub struct WeeklyTrainingData {
    pub weekday: Weekday,
    pub start: NaiveTime,
    pub end: NaiveTime,
    pub from: NaiveDate,
    pub until: NaiveDate,
}

/// The weekly trainings of `season`, sorted by the start of the window, then by the day of the week.
pub async fn for_season(
    db: impl PgExecutor<'_>,
    season: Season,
) -> Result<Vec<WeeklyTraining>, StoreError> {
    let rows = sqlx::query!(
        "SELECT id, weekday, start_time, end_time, from_date, until_date FROM weekly_trainings
         WHERE season_year = $1 ORDER BY from_date, weekday, id",
        season.start_year(),
    )
    .fetch_all(db)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(WeeklyTraining {
                id: WeeklyTrainingId(row.id),
                season,
                weekday: weekday(row.weekday)?,
                start: row.start_time,
                end: row.end_time,
                from: row.from_date,
                until: row.until_date,
            })
        })
        .collect()
}

pub async fn insert(
    db: impl PgExecutor<'_>,
    season: Season,
    data: WeeklyTrainingData,
) -> Result<WeeklyTrainingId, StoreError> {
    let id = sqlx::query_scalar!(
        "INSERT INTO weekly_trainings (season_year, weekday, start_time, end_time, from_date,
                                       until_date)
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
        season.start_year(),
        weekday_number(data.weekday)?,
        data.start,
        data.end,
        data.from,
        data.until,
    )
    .fetch_one(db)
    .await?;
    Ok(WeeklyTrainingId(id))
}

/// Changes the weekly training `id` of `season`; `false` when there is none.
pub async fn update(
    db: impl PgExecutor<'_>,
    season: Season,
    id: WeeklyTrainingId,
    data: WeeklyTrainingData,
) -> Result<bool, StoreError> {
    let result = sqlx::query!(
        "UPDATE weekly_trainings SET weekday = $3, start_time = $4, end_time = $5, from_date = $6,
                                     until_date = $7
         WHERE id = $1 AND season_year = $2",
        id.0,
        season.start_year(),
        weekday_number(data.weekday)?,
        data.start,
        data.end,
        data.from,
        data.until,
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected() == 1)
}

/// Deletes the weekly training `id` of `season`; `false` when there is none.
pub async fn delete(
    db: impl PgExecutor<'_>,
    season: Season,
    id: WeeklyTrainingId,
) -> Result<bool, StoreError> {
    let result = sqlx::query!(
        "DELETE FROM weekly_trainings WHERE id = $1 AND season_year = $2",
        id.0,
        season.start_year(),
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected() == 1)
}

/// A weekday from its number, 1 (Monday) to 7 (Sunday).
pub fn weekday_from_number(number: i16) -> Option<Weekday> {
    match number {
        1 => Some(Weekday::Mon),
        2 => Some(Weekday::Tue),
        3 => Some(Weekday::Wed),
        4 => Some(Weekday::Thu),
        5 => Some(Weekday::Fri),
        6 => Some(Weekday::Sat),
        7 => Some(Weekday::Sun),
        _ => None,
    }
}

/// The number of a weekday, 1 (Monday) to 7 (Sunday).
pub fn weekday_number(weekday: Weekday) -> Result<i16, StoreError> {
    i16::try_from(weekday.number_from_monday())
        .map_err(|_| StoreError::Invalid(format!("weekday {weekday}")))
}

fn weekday(number: i16) -> Result<Weekday, StoreError> {
    weekday_from_number(number).ok_or_else(|| StoreError::Invalid(format!("weekday {number}")))
}
