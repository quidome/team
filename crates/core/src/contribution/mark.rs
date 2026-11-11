//! A mark that holds for a date (decision 0029): a check of an event stores the date of the event
//! it was made for, and holds only while the event is on that date. One rule for the check of a
//! team game and of a duty game.

use chrono::NaiveDate;

/// Whether a mark made for `checked_for` holds for the event on `date` now.
pub fn mark_holds(checked_for: Option<NaiveDate>, date: NaiveDate) -> bool {
    checked_for == Some(date)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, day).expect("valid date")
    }

    #[test]
    fn a_mark_holds_only_for_its_date() {
        assert!(mark_holds(Some(date(31)), date(31)));
        // The game moved: the mark has no effect.
        assert!(!mark_holds(Some(date(31)), date(24)));
        assert!(!mark_holds(None, date(31)));
    }
}
