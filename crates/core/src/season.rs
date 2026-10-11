//! The season: a season starts on 1 August and is written as "2026-2027".

use std::fmt;
use std::str::FromStr;

use chrono::{Datelike, NaiveDate};

/// The month in which a new season starts (August).
const FIRST_MONTH: u32 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Season {
    start_year: i32,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("not a season: {0:?} (expected for example \"2026-2027\")")]
pub struct SeasonError(pub String);

impl Season {
    /// The season of a date in Dutch local time.
    pub fn of(date: NaiveDate) -> Season {
        let start_year = if date.month() >= FIRST_MONTH {
            date.year()
        } else {
            date.year() - 1
        };
        Season { start_year }
    }

    pub fn start_year(self) -> i32 {
        self.start_year
    }
}

impl fmt::Display for Season {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.start_year, self.start_year + 1)
    }
}

impl FromStr for Season {
    type Err = SeasonError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let error = || SeasonError(text.to_owned());
        let (first, second) = text.split_once('-').ok_or_else(error)?;
        let is_year = |part: &str| part.len() == 4 && part.bytes().all(|b| b.is_ascii_digit());
        if !is_year(first) || !is_year(second) {
            return Err(error());
        }
        let start_year: i32 = first.parse().map_err(|_| error())?;
        let end_year: i32 = second.parse().map_err(|_| error())?;
        if end_year != start_year + 1 {
            return Err(error());
        }
        Ok(Season { start_year })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    #[test]
    fn the_last_day_of_july_is_still_the_old_season() {
        assert_eq!(Season::of(date(2027, 7, 31)).to_string(), "2026-2027");
    }

    #[test]
    fn the_first_of_august_starts_a_new_season() {
        assert_eq!(Season::of(date(2027, 8, 1)).to_string(), "2027-2028");
    }

    #[test]
    fn january_belongs_to_the_season_that_started_last_year() {
        assert_eq!(Season::of(date(2027, 1, 16)).start_year(), 2026);
    }

    #[test]
    fn reads_back_what_it_shows() {
        let season: Result<Season, _> = "2026-2027".parse();
        assert_eq!(season, Ok(Season::of(date(2026, 10, 11))));
    }

    #[test]
    fn refuses_years_that_do_not_follow_each_other() {
        assert!("2026-2028".parse::<Season>().is_err());
        assert!("2027-2026".parse::<Season>().is_err());
    }

    #[test]
    fn refuses_other_forms() {
        for text in ["2026", "", "26-27", "2026-2027 ", "+026-2027", "2026–2027"] {
            assert!(text.parse::<Season>().is_err(), "{text:?} was accepted");
        }
    }

    #[test]
    fn seasons_sort_by_start_year() {
        assert!(Season::of(date(2025, 9, 1)) < Season::of(date(2026, 9, 1)));
    }
}
