//! Training days: the days of the week the team trains, with their times, and the first and the
//! last training date of a season. A change of days applies from a date, so the past keeps the
//! days that were set then.

use std::collections::HashSet;

use chrono::{Datelike, Days, NaiveDate, NaiveTime, Weekday};

use crate::season::Season;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrainingDay {
    pub weekday: Weekday,
    pub start: NaiveTime,
    pub end: NaiveTime,
}

/// The training days that apply from `valid_from` until the next set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrainingDaySet {
    pub valid_from: NaiveDate,
    pub days: Vec<TrainingDay>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrainingSeason {
    pub season: Season,
    pub first: NaiveDate,
    pub last: NaiveDate,
    pub sets: Vec<TrainingDaySet>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TrainingError {
    #[error("the end time is not after the start time")]
    EndBeforeStart,
    #[error("the last training date is before the first")]
    LastBeforeFirst,
    #[error("no training days")]
    NoTrainingDays,
    #[error("a day of the week is a training day twice")]
    DuplicateWeekday,
    #[error("a training date is outside the season")]
    OutsideSeason,
}

/// The proposed first and last training date: the first of `weekdays` on or after 1 September of
/// the season, and the last on or before 31 May. `None` without weekdays.
pub fn proposed_dates(season: Season, weekdays: &[Weekday]) -> Option<(NaiveDate, NaiveDate)> {
    if weekdays.is_empty() {
        return None;
    }
    let year = season.start_year();
    let mut first = NaiveDate::from_ymd_opt(year, 9, 1)?;
    while !weekdays.contains(&first.weekday()) {
        first = first.succ_opt()?;
    }
    let mut last = NaiveDate::from_ymd_opt(year + 1, 5, 31)?;
    while !weekdays.contains(&last.weekday()) {
        last = last.pred_opt()?;
    }
    Some((first, last))
}

/// The set that applies on `date`: the one with the newest `valid_from` on or before it; before
/// the first set, the first set.
pub fn set_on(sets: &[TrainingDaySet], date: NaiveDate) -> Option<&TrainingDaySet> {
    sets.iter()
        .filter(|set| set.valid_from <= date)
        .max_by_key(|set| set.valid_from)
        .or_else(|| sets.iter().min_by_key(|set| set.valid_from))
}

/// Every training date of the season, from the first to the last training date, each with the
/// training day of the set that applies on it.
pub fn training_dates(season: &TrainingSeason) -> Vec<(NaiveDate, TrainingDay)> {
    let mut dates = Vec::new();
    let mut date = season.first;
    while date <= season.last {
        if let Some(day) = set_on(&season.sets, date)
            .and_then(|set| set.days.iter().find(|day| day.weekday == date.weekday()))
        {
            dates.push((date, *day));
        }
        match date.checked_add_days(Days::new(1)) {
            Some(next) => date = next,
            None => break,
        }
    }
    dates
}

/// Checks a save of the settings of `season` and gives the set to store: valid from `today`, or
/// from `first` when today is before it, or for the first save of the season (`first_save`): there
/// is no past to keep then. A stored set with the same `valid_from` is replaced by it.
pub fn settings_change(
    season: Season,
    first: NaiveDate,
    last: NaiveDate,
    days: Vec<TrainingDay>,
    today: NaiveDate,
    first_save: bool,
) -> Result<TrainingDaySet, TrainingError> {
    if Season::of(first) != season || Season::of(last) != season {
        return Err(TrainingError::OutsideSeason);
    }
    if last < first {
        return Err(TrainingError::LastBeforeFirst);
    }
    if days.is_empty() {
        return Err(TrainingError::NoTrainingDays);
    }
    if days.iter().any(|day| day.end <= day.start) {
        return Err(TrainingError::EndBeforeStart);
    }
    let mut weekdays = HashSet::new();
    if !days.iter().all(|day| weekdays.insert(day.weekday)) {
        return Err(TrainingError::DuplicateWeekday);
    }
    let mut days = days;
    days.sort_by_key(|day| day.weekday.number_from_monday());
    let valid_from = if first_save { first } else { today.max(first) };
    Ok(TrainingDaySet { valid_from, days })
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

    fn day(weekday: Weekday) -> TrainingDay {
        TrainingDay {
            weekday,
            start: time(19, 45),
            end: time(21, 0),
        }
    }

    fn season() -> Season {
        Season::from_start_year(2026)
    }

    fn tuesdays() -> TrainingSeason {
        TrainingSeason {
            season: season(),
            first: date(2026, 9, 1),
            last: date(2027, 5, 25),
            sets: vec![TrainingDaySet {
                valid_from: date(2026, 9, 1),
                days: vec![day(Weekday::Tue)],
            }],
        }
    }

    #[test]
    fn the_proposal_is_the_first_training_day_of_september_and_the_last_of_may() {
        assert_eq!(
            proposed_dates(season(), &[Weekday::Tue]),
            Some((date(2026, 9, 1), date(2027, 5, 25)))
        );
        assert_eq!(
            proposed_dates(season(), &[Weekday::Thu, Weekday::Tue]),
            Some((date(2026, 9, 1), date(2027, 5, 27)))
        );
        assert_eq!(proposed_dates(season(), &[]), None);
    }

    #[test]
    fn a_season_of_tuesdays_has_39_training_dates() {
        let dates = training_dates(&tuesdays());
        assert_eq!(dates.len(), 39);
        assert_eq!(dates.first().map(|(date, _)| *date), Some(date(2026, 9, 1)));
        assert_eq!(dates.last().map(|(date, _)| *date), Some(date(2027, 5, 25)));
        assert!(
            dates
                .iter()
                .all(|(_, day)| day.start == time(19, 45) && day.end == time(21, 0))
        );
    }

    #[test]
    fn a_change_of_days_keeps_the_past() {
        let mut season = tuesdays();
        season.sets.push(TrainingDaySet {
            valid_from: date(2027, 1, 5),
            days: vec![day(Weekday::Thu)],
        });
        let dates: Vec<NaiveDate> = training_dates(&season)
            .into_iter()
            .map(|(d, _)| d)
            .collect();
        assert!(dates.contains(&date(2026, 12, 29)));
        assert!(!dates.contains(&date(2027, 1, 5)));
        assert!(dates.contains(&date(2027, 1, 7)));
        assert!(!dates.contains(&date(2027, 1, 12)));
    }

    #[test]
    fn a_change_in_the_season_applies_from_today() {
        let set = settings_change(
            season(),
            date(2026, 9, 1),
            date(2027, 5, 25),
            vec![day(Weekday::Thu)],
            date(2027, 1, 5),
            false,
        );
        assert_eq!(set.map(|set| set.valid_from), Ok(date(2027, 1, 5)));
    }

    #[test]
    fn a_change_before_the_first_date_applies_from_the_first_date() {
        let set = settings_change(
            season(),
            date(2026, 9, 1),
            date(2027, 5, 25),
            vec![day(Weekday::Tue)],
            date(2026, 8, 15),
            false,
        );
        assert_eq!(set.map(|set| set.valid_from), Ok(date(2026, 9, 1)));
    }

    #[test]
    fn the_first_save_in_the_season_applies_from_the_first_date() {
        // The coordinator sets up the tool in October and enters the trainings since September.
        let set = settings_change(
            season(),
            date(2026, 9, 1),
            date(2027, 5, 25),
            vec![day(Weekday::Tue)],
            date(2026, 10, 15),
            true,
        );
        assert_eq!(set.map(|set| set.valid_from), Ok(date(2026, 9, 1)));
    }

    #[test]
    fn settings_are_checked() {
        let save = |first, last, days| {
            settings_change(season(), first, last, days, date(2026, 8, 15), true)
        };
        let late = TrainingDay {
            weekday: Weekday::Tue,
            start: time(21, 0),
            end: time(19, 45),
        };
        assert_eq!(
            save(date(2026, 9, 1), date(2027, 5, 25), vec![late]),
            Err(TrainingError::EndBeforeStart)
        );
        assert_eq!(
            save(date(2027, 5, 25), date(2026, 9, 1), vec![day(Weekday::Tue)]),
            Err(TrainingError::LastBeforeFirst)
        );
        assert_eq!(
            save(date(2026, 9, 1), date(2027, 5, 25), vec![]),
            Err(TrainingError::NoTrainingDays)
        );
        assert_eq!(
            save(
                date(2026, 9, 1),
                date(2027, 5, 25),
                vec![day(Weekday::Tue), day(Weekday::Tue)]
            ),
            Err(TrainingError::DuplicateWeekday)
        );
        assert_eq!(
            save(
                date(2026, 6, 30),
                date(2027, 5, 25),
                vec![day(Weekday::Tue)]
            ),
            Err(TrainingError::OutsideSeason)
        );
    }
}
