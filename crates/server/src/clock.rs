//! The current time, behind a trait so tests can set it.

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
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

/// Development only (`DEV_NOW`): starts at a set moment and then runs with the real time, so
/// a later reading still starts later (decision 0015).
#[cfg(debug_assertions)]
pub struct ShiftedClock {
    shift: chrono::TimeDelta,
}

#[cfg(debug_assertions)]
impl ShiftedClock {
    pub fn starting_at(start: DateTime<Utc>) -> Self {
        Self {
            shift: start - Utc::now(),
        }
    }
}

#[cfg(debug_assertions)]
impl Clock for ShiftedClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now() + self.shift
    }
}

/// Today's date in the Netherlands: the source's dates and the season follow Dutch local time.
pub fn today_in_netherlands(clock: &dyn Clock) -> NaiveDate {
    clock.now().with_timezone(&Amsterdam).date_naive()
}

/// A moment as Dutch local time, for the rules of the nightly reading.
pub fn in_netherlands(at: DateTime<Utc>) -> NaiveDateTime {
    at.with_timezone(&Amsterdam).naive_local()
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
    fn a_shifted_clock_starts_at_its_moment_and_runs_on() {
        let start = at(2026, 10, 12, 10, 0).0;
        let clock = ShiftedClock::starting_at(start);
        let first = clock.now();
        let second = clock.now();
        assert!(first >= start);
        assert!(first - start < chrono::TimeDelta::seconds(5));
        assert!(second >= first);
        assert_eq!(current_season(&clock).to_string(), "2026-2027");
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
    fn the_nights_the_clock_changes_in_dutch_time() {
        let local = |at: At| in_netherlands(at.0).format("%Y-%m-%d %H:%M").to_string();
        // Summer time ends at 01:00 UTC on 25 October 2026: 02:00–03:00 happens twice.
        assert_eq!(local(at(2026, 10, 25, 0, 30)), "2026-10-25 02:30");
        assert_eq!(local(at(2026, 10, 25, 1, 0)), "2026-10-25 02:00");
        assert_eq!(local(at(2026, 10, 25, 2, 0)), "2026-10-25 03:00");
        // Summer time starts at 01:00 UTC on 28 March 2027: 02:00–03:00 is skipped.
        assert_eq!(local(at(2027, 3, 28, 0, 59)), "2027-03-28 01:59");
        assert_eq!(local(at(2027, 3, 28, 1, 0)), "2027-03-28 03:00");
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
