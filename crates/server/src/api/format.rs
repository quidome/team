//! Dates and times as the API writes and reads them: "2026-10-13" and "19:45".

use chrono::{NaiveDate, NaiveTime};
use team_core::Season;

use crate::clock::{Clock, current_season};
use crate::error::AppError;

/// A date, for example "2026-10-13".
pub(crate) fn day(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// A time of day, for example "19:45".
pub(crate) fn clock_time(time: NaiveTime) -> String {
    time.format("%H:%M").to_string()
}

/// A time of day typed as "19:45"; else `invalid_time`.
pub(crate) fn time(text: &str) -> Result<NaiveTime, AppError> {
    NaiveTime::parse_from_str(text, "%H:%M").map_err(|_| AppError::BadRequest("invalid_time"))
}

/// A date in a path, for example "2026-10-13"; else `invalid_date`.
pub(crate) fn path_date(text: &str) -> Result<NaiveDate, AppError> {
    NaiveDate::parse_from_str(text, "%Y-%m-%d").map_err(|_| AppError::BadRequest("invalid_date"))
}

/// The season of a query, for example "2026-2027"; the current season when left out; else
/// `invalid_season`.
pub(crate) fn query_season(text: Option<String>, clock: &dyn Clock) -> Result<Season, AppError> {
    match text {
        Some(text) => text
            .parse::<Season>()
            .map_err(|_| AppError::BadRequest("invalid_season")),
        None => Ok(current_season(clock)),
    }
}
