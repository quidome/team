//! The training settings of each season: the first and the last training date (table
//! `training_seasons`) and the sets of training days (table `training_days`).

use chrono::{NaiveDate, Weekday};
use sqlx::PgConnection;
use team_core::{Season, TrainingDay, TrainingDaySet, TrainingSeason};

use crate::db::StoreError;

/// The settings of `season`, with all its sets in order of `valid_from`; `None` when nothing is
/// saved for it.
pub async fn for_season(
    db: &mut PgConnection,
    season: Season,
) -> Result<Option<TrainingSeason>, StoreError> {
    let Some(dates) = sqlx::query!(
        "SELECT first_date, last_date FROM training_seasons WHERE season_year = $1",
        season.start_year(),
    )
    .fetch_optional(&mut *db)
    .await?
    else {
        return Ok(None);
    };
    let rows = sqlx::query!(
        "SELECT valid_from, weekday, start_time, end_time FROM training_days
         WHERE season_year = $1 ORDER BY valid_from, weekday",
        season.start_year(),
    )
    .fetch_all(&mut *db)
    .await?;
    let mut sets: Vec<TrainingDaySet> = Vec::new();
    for row in rows {
        let day = TrainingDay {
            weekday: weekday(row.weekday)?,
            start: row.start_time,
            end: row.end_time,
        };
        match sets.last_mut() {
            Some(set) if set.valid_from == row.valid_from => set.days.push(day),
            _ => sets.push(TrainingDaySet {
                valid_from: row.valid_from,
                days: vec![day],
            }),
        }
    }
    Ok(Some(TrainingSeason {
        season,
        first: dates.first_date,
        last: dates.last_date,
        sets,
    }))
}

/// Saves the first and the last training date of `season`, and `set`: a stored set with the same
/// `valid_from` is replaced. Run it in a transaction.
pub async fn save(
    tx: &mut PgConnection,
    season: Season,
    first: NaiveDate,
    last: NaiveDate,
    set: &TrainingDaySet,
) -> Result<(), StoreError> {
    sqlx::query!(
        "INSERT INTO training_seasons (season_year, first_date, last_date) VALUES ($1, $2, $3)
         ON CONFLICT (season_year) DO UPDATE SET
             first_date = EXCLUDED.first_date, last_date = EXCLUDED.last_date",
        season.start_year(),
        first,
        last,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "DELETE FROM training_days WHERE season_year = $1 AND valid_from = $2",
        season.start_year(),
        set.valid_from,
    )
    .execute(&mut *tx)
    .await?;
    for day in &set.days {
        insert_day(&mut *tx, season, set.valid_from, day).await?;
    }
    Ok(())
}

async fn insert_day(
    tx: &mut PgConnection,
    season: Season,
    valid_from: NaiveDate,
    day: &TrainingDay,
) -> Result<(), StoreError> {
    let weekday = i16::try_from(day.weekday.number_from_monday())
        .map_err(|_| StoreError::Invalid(format!("weekday {}", day.weekday)))?;
    sqlx::query!(
        "INSERT INTO training_days (season_year, valid_from, weekday, start_time, end_time)
         VALUES ($1, $2, $3, $4, $5)",
        season.start_year(),
        valid_from,
        weekday,
        day.start,
        day.end,
    )
    .execute(tx)
    .await?;
    Ok(())
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

fn weekday(number: i16) -> Result<Weekday, StoreError> {
    weekday_from_number(number).ok_or_else(|| StoreError::Invalid(format!("weekday {number}")))
}
