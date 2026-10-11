//! The current time, behind a trait so tests can set it.

use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Europe::Amsterdam;
use team_core::Season;

pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

/// The real clock.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Today's date in the Netherlands: the source's dates and the season follow Dutch local time.
pub fn today_in_netherlands(clock: &dyn Clock) -> NaiveDate {
    clock.now().with_timezone(&Amsterdam).date_naive()
}

/// The current season, from today's Dutch date.
pub fn current_season(clock: &dyn Clock) -> Season {
    Season::of(today_in_netherlands(clock))
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    struct At(DateTime<Utc>);

    impl Clock for At {
        fn now(&self) -> DateTime<Utc> {
            self.0
        }
    }

    fn at(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> At {
        At(Utc
            .with_ymd_and_hms(year, month, day, hour, minute, 0)
            .single()
            .expect("valid time"))
    }

    #[test]
    fn a_summer_evening_is_already_the_next_day_in_the_netherlands() {
        // 22:00 UTC on 30 June is 00:00 on 1 July in Dutch summer time.
        assert_eq!(
            current_season(&at(2027, 6, 30, 21, 59)).to_string(),
            "2026-2027"
        );
        assert_eq!(
            current_season(&at(2027, 6, 30, 22, 0)).to_string(),
            "2027-2028"
        );
    }

    #[test]
    fn winter_time_is_one_hour_ahead_of_utc() {
        let date = today_in_netherlands(&at(2026, 12, 31, 23, 0));
        assert_eq!(
            date,
            NaiveDate::from_ymd_opt(2027, 1, 1).expect("valid date")
        );
    }
}
