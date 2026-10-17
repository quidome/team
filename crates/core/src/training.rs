//! Weekly trainings: a day of the week with a start and an end time, in a window from a date until
//! a date inside one season. A weekly training only suggests training days; a training exists only
//! when the coordinator opens it (story B), and a later change of the weekly training does not
//! touch it.

use chrono::{Datelike, Days, NaiveDate, NaiveTime, Weekday};

use crate::season::Season;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WeeklyTrainingId(pub i64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeeklyTraining {
    pub id: WeeklyTrainingId,
    pub season: Season,
    pub weekday: Weekday,
    pub start: NaiveTime,
    pub end: NaiveTime,
    /// The first day of the window.
    pub from: NaiveDate,
    /// The last day of the window.
    pub until: NaiveDate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TrainingError {
    #[error("the end time is not after the start time")]
    EndBeforeStart,
    #[error("the last training date is before the first")]
    LastBeforeFirst,
    #[error("a training date is outside the season")]
    OutsideSeason,
    #[error("another weekly training on the same day overlaps")]
    WeeklyTrainingsOverlap,
}

/// Checks a weekly training of `season`: the window inside the season, `until` not before `from`,
/// `end` after `start`, and no overlap with `others` (the other weekly trainings of the season,
/// without the one that is changed) on the same day of the week.
pub fn check_weekly_training(
    season: Season,
    weekday: Weekday,
    start: NaiveTime,
    end: NaiveTime,
    from: NaiveDate,
    until: NaiveDate,
    others: &[WeeklyTraining],
) -> Result<(), TrainingError> {
    if end <= start {
        return Err(TrainingError::EndBeforeStart);
    }
    if until < from {
        return Err(TrainingError::LastBeforeFirst);
    }
    if Season::of(from) != season || Season::of(until) != season {
        return Err(TrainingError::OutsideSeason);
    }
    if others
        .iter()
        .any(|other| other.weekday == weekday && other.from <= until && from <= other.until)
    {
        return Err(TrainingError::WeeklyTrainingsOverlap);
    }
    Ok(())
}

/// The proposed window for a weekly training on `weekday`: that day on or after 1 September of the
/// season until that day on or before 31 May.
pub fn proposed_window(season: Season, weekday: Weekday) -> Option<(NaiveDate, NaiveDate)> {
    let year = season.start_year();
    let mut from = NaiveDate::from_ymd_opt(year, 9, 1)?;
    while from.weekday() != weekday {
        from = from.succ_opt()?;
    }
    let mut until = NaiveDate::from_ymd_opt(year + 1, 5, 31)?;
    while until.weekday() != weekday {
        until = until.pred_opt()?;
    }
    Some((from, until))
}

/// The training days: every date in a window on the day of the week of its weekly training, sorted
/// by date. A window without its day of the week in it suggests no training days.
pub fn training_days(weekly: &[WeeklyTraining]) -> Vec<(NaiveDate, &WeeklyTraining)> {
    let mut days = Vec::new();
    for training in weekly {
        let mut date = training.from;
        while date.weekday() != training.weekday {
            match date.succ_opt() {
                Some(next) => date = next,
                None => break,
            }
        }
        while date <= training.until && date.weekday() == training.weekday {
            days.push((date, training));
            match date.checked_add_days(Days::new(7)) {
                Some(next) => date = next,
                None => break,
            }
        }
    }
    days.sort_by_key(|(date, _)| *date);
    days
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    fn time(hour: u32, minute: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(hour, minute, 0).expect("valid time")
    }

    fn season() -> Season {
        Season::from_start_year(2026)
    }

    fn weekly(id: i64, weekday: Weekday, from: NaiveDate, until: NaiveDate) -> WeeklyTraining {
        WeeklyTraining {
            id: WeeklyTrainingId(id),
            season: season(),
            weekday,
            start: time(19, 45),
            end: time(21, 0),
            from,
            until,
        }
    }

    fn check(
        weekday: Weekday,
        from: NaiveDate,
        until: NaiveDate,
        others: &[WeeklyTraining],
    ) -> Result<(), TrainingError> {
        check_weekly_training(
            season(),
            weekday,
            time(19, 45),
            time(21, 0),
            from,
            until,
            others,
        )
    }

    #[test]
    fn two_weekly_trainings_on_the_same_day_never_overlap() {
        let tuesday = [weekly(1, Weekday::Tue, date(2026, 9, 1), date(2027, 5, 25))];
        assert_eq!(
            check(Weekday::Tue, date(2027, 1, 5), date(2027, 5, 25), &tuesday),
            Err(TrainingError::WeeklyTrainingsOverlap)
        );
        assert_eq!(
            check(Weekday::Thu, date(2026, 9, 1), date(2027, 5, 25), &tuesday),
            Ok(())
        );
        // The weekly training that is changed is not in `others`.
        assert_eq!(
            check(Weekday::Tue, date(2026, 9, 8), date(2027, 5, 25), &[]),
            Ok(())
        );
        // A window that ends the day before the other starts.
        let from_january = [weekly(2, Weekday::Tue, date(2027, 1, 5), date(2027, 5, 25))];
        assert_eq!(
            check(
                Weekday::Tue,
                date(2026, 9, 1),
                date(2027, 1, 4),
                &from_january
            ),
            Ok(())
        );
    }

    #[test]
    fn a_weekly_training_is_checked() {
        assert_eq!(
            check(Weekday::Tue, date(2026, 6, 30), date(2027, 5, 25), &[]),
            Err(TrainingError::OutsideSeason)
        );
        assert_eq!(
            check(Weekday::Tue, date(2026, 9, 1), date(2027, 7, 1), &[]),
            Err(TrainingError::OutsideSeason)
        );
        assert_eq!(
            check(Weekday::Tue, date(2026, 9, 1), date(2026, 8, 31), &[]),
            Err(TrainingError::LastBeforeFirst)
        );
        assert_eq!(
            check_weekly_training(
                season(),
                Weekday::Tue,
                time(21, 0),
                time(19, 45),
                date(2026, 9, 1),
                date(2027, 5, 25),
                &[]
            ),
            Err(TrainingError::EndBeforeStart)
        );
    }

    #[test]
    fn the_proposed_window_is_the_first_day_of_september_and_the_last_of_may() {
        assert_eq!(
            proposed_window(season(), Weekday::Tue),
            Some((date(2026, 9, 1), date(2027, 5, 25)))
        );
        assert_eq!(
            proposed_window(season(), Weekday::Thu),
            Some((date(2026, 9, 3), date(2027, 5, 27)))
        );
    }

    #[test]
    fn a_tuesday_in_the_proposed_window_has_39_training_days() {
        let tuesday = [weekly(1, Weekday::Tue, date(2026, 9, 1), date(2027, 5, 25))];
        let days = training_days(&tuesday);
        assert_eq!(days.len(), 39);
        assert_eq!(days[0].0, date(2026, 9, 1));
        assert_eq!(days[38].0, date(2027, 5, 25));
        assert!(days.iter().all(|(day, _)| day.weekday() == Weekday::Tue));
    }

    #[test]
    fn training_days_of_two_weekly_trainings_are_sorted_by_date() {
        let both = [
            weekly(1, Weekday::Thu, date(2026, 9, 1), date(2026, 9, 30)),
            weekly(2, Weekday::Tue, date(2026, 9, 1), date(2026, 9, 30)),
        ];
        let days: Vec<NaiveDate> = training_days(&both).iter().map(|(day, _)| *day).collect();
        assert_eq!(
            days,
            [
                date(2026, 9, 1),
                date(2026, 9, 3),
                date(2026, 9, 8),
                date(2026, 9, 10),
                date(2026, 9, 15),
                date(2026, 9, 17),
                date(2026, 9, 22),
                date(2026, 9, 24),
                date(2026, 9, 29),
            ]
        );
    }

    #[test]
    fn a_window_without_its_day_has_no_training_days() {
        // Wednesday 2 Sep to Monday 7 Sep holds no Tuesday.
        let none = [weekly(1, Weekday::Tue, date(2026, 9, 2), date(2026, 9, 7))];
        assert!(training_days(&none).is_empty());
    }
}
