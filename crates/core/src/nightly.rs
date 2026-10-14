//! The nightly reading: when one is due, and when the games may be out of date. All times are
//! Dutch local time; the caller turns the clock and the stored times into it.

use chrono::{Days, NaiveDate, NaiveDateTime, NaiveTime};

/// The moment of the nightly reading, Dutch time. 03:00 exists exactly once on both nights the
/// clock changes (the hour that repeats or is skipped is 02:00–03:00).
pub const NIGHTLY_AT: NaiveTime = match NaiveTime::from_hms_opt(3, 0, 0) {
    Some(time) => time,
    None => panic!("03:00 is a valid time"),
};

/// The number of run moments without a good reading after which the games may be out of date.
pub const OUT_OF_DATE_NIGHTS: u32 = 3;

/// The last run moment at or before `now`: today 03:00, or yesterday 03:00 before 03:00.
pub fn last_run_moment(now: NaiveDateTime) -> NaiveDateTime {
    let today = now.date();
    let date = if now.time() >= NIGHTLY_AT {
        today
    } else {
        nights_before(today, 1)
    };
    date.and_time(NIGHTLY_AT)
}

/// A nightly reading is due when a team is chosen and no reading of the club and season, of any
/// result, started at or after the last run moment. So a tap after 03:00 makes it unnecessary,
/// and a failed nightly reading is not tried again the same night.
pub fn nightly_due(
    team_chosen: bool,
    now: NaiveDateTime,
    newest_start: Option<NaiveDateTime>,
) -> bool {
    team_chosen && newest_start.is_none_or(|start| start < last_run_moment(now))
}

/// The games may be out of date when the last good reading started before the run moment
/// `OUT_OF_DATE_NIGHTS - 1` nights before the last run moment: that many run moments passed
/// without a good reading.
pub fn out_of_date(now: NaiveDateTime, last_good_start: NaiveDateTime) -> bool {
    let last = last_run_moment(now);
    // Counted on the date, not in hours, so the nights the clock changes do not shift it.
    let oldest = nights_before(last.date(), OUT_OF_DATE_NIGHTS - 1).and_time(NIGHTLY_AT);
    last_good_start < oldest
}

/// The date `nights` days before `date`; the first date there is when that goes before it.
fn nights_before(date: NaiveDate, nights: u32) -> NaiveDate {
    date.checked_sub_days(Days::new(u64::from(nights)))
        .unwrap_or(NaiveDate::MIN)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(month: u32, day: u32, hour: u32, minute: u32) -> NaiveDateTime {
        let year = if month >= 7 { 2026 } else { 2027 };
        NaiveDate::from_ymd_opt(year, month, day)
            .and_then(|date| date.and_hms_opt(hour, minute, 0))
            .expect("valid time")
    }

    #[test]
    fn the_last_run_moment_is_today_from_three_and_yesterday_before() {
        assert_eq!(last_run_moment(at(10, 12, 2, 59)), at(10, 11, 3, 0));
        assert_eq!(last_run_moment(at(10, 12, 3, 0)), at(10, 12, 3, 0));
        assert_eq!(last_run_moment(at(10, 12, 23, 0)), at(10, 12, 3, 0));
    }

    #[test]
    fn the_last_run_moment_on_the_nights_the_clock_changes() {
        // Summer time ends on 25 October 2026 and starts on 28 March 2027.
        for (month, day) in [(10, 25), (3, 28)] {
            assert_eq!(last_run_moment(at(month, day, 4, 0)), at(month, day, 3, 0));
            assert_eq!(
                last_run_moment(at(month, day, 2, 30)),
                at(month, day - 1, 3, 0)
            );
        }
    }

    #[test]
    fn without_a_team_no_reading_is_due() {
        assert!(!nightly_due(false, at(10, 13, 3, 1), None));
        assert!(!nightly_due(
            false,
            at(10, 13, 3, 1),
            Some(at(10, 12, 14, 0))
        ));
    }

    #[test]
    fn a_reading_is_due_when_none_started_since_the_last_run_moment() {
        let now = at(10, 13, 3, 1);
        assert!(nightly_due(true, now, None));
        assert!(nightly_due(true, now, Some(at(10, 13, 2, 59))));
        assert!(!nightly_due(true, now, Some(at(10, 13, 3, 0))));
    }

    #[test]
    fn a_tap_in_the_morning_makes_the_nightly_reading_unnecessary() {
        assert!(!nightly_due(true, at(10, 13, 9, 0), Some(at(10, 13, 8, 0))));
    }

    #[test]
    fn a_pod_that_starts_after_three_reads_at_once() {
        assert!(nightly_due(true, at(10, 13, 9, 0), Some(at(10, 12, 14, 0))));
    }

    #[test]
    fn out_of_date_after_three_run_moments_without_a_good_reading() {
        let monday = at(10, 12, 3, 0);
        assert!(!out_of_date(at(10, 14, 10, 0), monday));
        assert!(!out_of_date(at(10, 15, 2, 59), monday));
        assert!(out_of_date(at(10, 15, 3, 0), monday));
    }

    #[test]
    fn a_good_tap_is_a_good_reading() {
        assert!(!out_of_date(at(10, 15, 3, 0), at(10, 14, 8, 0)));
    }

    #[test]
    fn out_of_date_over_the_night_the_clock_changes() {
        // Good on Saturday 24 October; Sunday, Monday and Tuesday 03:00 fail.
        let saturday = at(10, 24, 3, 0);
        assert!(!out_of_date(at(10, 26, 23, 0), saturday));
        assert!(out_of_date(at(10, 27, 3, 0), saturday));
    }
}
