//! Weekly trainings and trainings. A weekly training is a day of the week with a start and an end
//! time, in a window from a date until a date inside one season. It only suggests training days; a
//! training exists only when the coordinator opens a training day or adds one, and a later change
//! of a weekly training does not touch it. At most one training per date.

use std::collections::BTreeMap;

use chrono::{Datelike, Days, NaiveDate, NaiveTime, Weekday};

use crate::attendance::Absence;
use crate::players::{PlayerRole, role_on};
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
    #[error("there is already a training on this date")]
    TrainingExists,
    #[error("the date is not a training day")]
    NotATrainingDay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TrainingId(pub i64);

/// A training: it exists because the coordinator opened a training day or added it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Training {
    pub id: TrainingId,
    pub date: NaiveDate,
    pub start: NaiveTime,
    pub end: NaiveTime,
    /// Cancelled trainings keep their absences and do not count.
    pub cancelled: bool,
}

/// What a row of the list says about its date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DayStatus {
    /// A training day without a training.
    NotOpened,
    AllThere,
    /// The number of absences of players with a role on the date.
    Absent(usize),
    Cancelled,
}

/// One row of the list of a season: the training of its date, or else the training day.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeasonRow<'a> {
    pub date: NaiveDate,
    pub start: NaiveTime,
    pub end: NaiveTime,
    pub training: Option<&'a Training>,
    /// A training on a date that is not a training day.
    pub extra: bool,
    pub status: DayStatus,
}

/// The list of a season: today and later, nearest first; before today, newest first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeasonList<'a> {
    pub coming: Vec<SeasonRow<'a>>,
    pub past: Vec<SeasonRow<'a>>,
}

/// The list of `season` on `today`: every training day and every training of the season, one row
/// per date. A training wins over the training day of its date. "N absent" counts only the
/// absences of players with a role on the date (`roles`: at least the roles of the season).
pub fn season_list<'a>(
    season: Season,
    weekly: &'a [WeeklyTraining],
    trainings: &'a [Training],
    absences: &[Absence],
    roles: &[PlayerRole],
    today: NaiveDate,
) -> SeasonList<'a> {
    let mut rows: BTreeMap<NaiveDate, SeasonRow<'a>> = BTreeMap::new();
    for (date, day) in training_days(weekly) {
        if Season::of(date) == season {
            rows.insert(
                date,
                SeasonRow {
                    date,
                    start: day.start,
                    end: day.end,
                    training: None,
                    extra: false,
                    status: DayStatus::NotOpened,
                },
            );
        }
    }
    for training in trainings.iter().filter(|t| Season::of(t.date) == season) {
        let extra = !rows.contains_key(&training.date);
        let status = if training.cancelled {
            DayStatus::Cancelled
        } else {
            match absent_count(training, absences, roles) {
                0 => DayStatus::AllThere,
                n => DayStatus::Absent(n),
            }
        };
        rows.insert(
            training.date,
            SeasonRow {
                date: training.date,
                start: training.start,
                end: training.end,
                training: Some(training),
                extra,
                status,
            },
        );
    }
    let (coming, mut past): (Vec<_>, Vec<_>) =
        rows.into_values().partition(|row| row.date >= today);
    past.reverse();
    SeasonList { coming, past }
}

/// The training days before today that were never opened, oldest first. "Attendance" and the
/// open items of the start page both count them with this function.
pub fn not_opened(list: &SeasonList<'_>) -> Vec<NaiveDate> {
    // `past` is newest first.
    list.past
        .iter()
        .rev()
        .filter(|row| row.status == DayStatus::NotOpened)
        .map(|row| row.date)
        .collect()
}

/// The absences at `training` of players with a role on its date.
pub fn absent_count(training: &Training, absences: &[Absence], roles: &[PlayerRole]) -> usize {
    absences
        .iter()
        .filter(|absence| {
            absence.training == training.id
                && role_on(roles, absence.player, training.date).is_some()
        })
        .count()
}

/// Checks "Add training" and "Change": `date` inside `season`, no other training on `date`
/// (`except`: the training that is changed), `end` after `start`.
pub fn check_training(
    season: Season,
    date: NaiveDate,
    start: NaiveTime,
    end: NaiveTime,
    trainings: &[Training],
    except: Option<TrainingId>,
) -> Result<(), TrainingError> {
    if Season::of(date) != season {
        return Err(TrainingError::OutsideSeason);
    }
    if trainings
        .iter()
        .any(|training| training.date == date && Some(training.id) != except)
    {
        return Err(TrainingError::TrainingExists);
    }
    if end <= start {
        return Err(TrainingError::EndBeforeStart);
    }
    Ok(())
}

/// The times of the training day `date`, for "Open"; `NotATrainingDay` when it is none.
pub fn day_times(
    weekly: &[WeeklyTraining],
    date: NaiveDate,
) -> Result<(NaiveTime, NaiveTime), TrainingError> {
    training_days(weekly)
        .into_iter()
        .find(|(day, _)| *day == date)
        .map(|(_, training)| (training.start, training.end))
        .ok_or(TrainingError::NotATrainingDay)
}

/// The times "Add training" proposes for `date`: the training day's, else those of the newest
/// weekly training of the season on the same day of the week, else none.
pub fn proposed_times(
    weekly: &[WeeklyTraining],
    date: NaiveDate,
) -> Option<(NaiveTime, NaiveTime)> {
    day_times(weekly, date).ok().or_else(|| {
        weekly
            .iter()
            .filter(|training| {
                training.season == Season::of(date) && training.weekday == date.weekday()
            })
            .max_by_key(|training| training.from)
            .map(|training| (training.start, training.end))
    })
}

/// A weekly training of last season, proposed for a new season with a new window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeeklyProposal {
    pub weekday: Weekday,
    pub start: NaiveTime,
    pub end: NaiveTime,
    pub from: NaiveDate,
    pub until: NaiveDate,
}

/// Last season's weekly trainings (`last`) proposed for `season`: those that ran until the end of
/// last season (their window ends in the last week of its training days), one per day of the week
/// (the newest), each with the proposed window of `season`, sorted by the start of the window. A day
/// of the week that already has a weekly training in `season` (`current`) gets no proposal. So a
/// day given up during last season is not proposed, and two proposals never overlap.
pub fn proposals_from(
    last: &[WeeklyTraining],
    current: &[WeeklyTraining],
    season: Season,
) -> Vec<WeeklyProposal> {
    let Some(latest) = last.iter().map(|training| training.until).max() else {
        return Vec::new();
    };
    let mut newest: BTreeMap<u32, &WeeklyTraining> = BTreeMap::new();
    for training in last {
        let to_the_end = training
            .until
            .checked_add_days(Days::new(7))
            .is_none_or(|week_later| week_later > latest);
        let taken = current
            .iter()
            .any(|other| other.weekday == training.weekday);
        if !to_the_end || taken {
            continue;
        }
        let key = training.weekday.number_from_monday();
        if newest
            .get(&key)
            .is_none_or(|seen| training.from > seen.from)
        {
            newest.insert(key, training);
        }
    }
    let mut proposals: Vec<WeeklyProposal> = newest
        .values()
        .filter_map(|training| {
            proposed_window(season, training.weekday).map(|(from, until)| WeeklyProposal {
                weekday: training.weekday,
                start: training.start,
                end: training.end,
                from,
                until,
            })
        })
        .collect();
    proposals.sort_by_key(|proposal| proposal.from);
    proposals
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

    // Trainings and the list of a season.

    use crate::players::{PlayerId, Role, RoleId};
    use crate::reasons::ReasonId;

    fn tuesdays() -> Vec<WeeklyTraining> {
        vec![weekly(1, Weekday::Tue, date(2026, 9, 1), date(2027, 5, 25))]
    }

    fn training(id: i64, on: NaiveDate) -> Training {
        Training {
            id: TrainingId(id),
            date: on,
            start: time(19, 45),
            end: time(21, 0),
            cancelled: false,
        }
    }

    fn role_from(player: i64, from: NaiveDate) -> PlayerRole {
        PlayerRole {
            id: RoleId(player),
            player: PlayerId(player),
            season: season(),
            role: Role::PlaysGames,
            from,
            until: None,
        }
    }

    fn absence(training: i64, player: i64) -> Absence {
        Absence {
            training: TrainingId(training),
            player: PlayerId(player),
            reason: ReasonId(1),
        }
    }

    #[test]
    fn the_list_has_coming_nearest_first_and_past_newest_first() {
        let weekly = tuesdays();
        let list = season_list(season(), &weekly, &[], &[], &[], date(2026, 10, 12));
        assert_eq!(list.coming[0].date, date(2026, 10, 13));
        assert_eq!(list.coming[1].date, date(2026, 10, 20));
        assert_eq!(list.past[0].date, date(2026, 10, 6));
        assert_eq!(list.past.last().map(|row| row.date), Some(date(2026, 9, 1)));
        assert_eq!(list.coming.len() + list.past.len(), 39);
        assert!(
            list.coming
                .iter()
                .chain(&list.past)
                .all(|row| row.status == DayStatus::NotOpened && row.training.is_none())
        );
        // Today's training is coming until the end of the day.
        let list = season_list(season(), &weekly, &[], &[], &[], date(2026, 10, 13));
        assert_eq!(list.coming[0].date, date(2026, 10, 13));
    }

    #[test]
    fn not_opened_is_oldest_first_and_leaves_out_today() {
        let weekly = tuesdays();
        // 29 September is opened; 13 October is today.
        let trainings = [training(1, date(2026, 9, 29))];
        let list = season_list(season(), &weekly, &trainings, &[], &[], date(2026, 10, 13));
        assert_eq!(
            not_opened(&list),
            [
                date(2026, 9, 1),
                date(2026, 9, 8),
                date(2026, 9, 15),
                date(2026, 9, 22),
                date(2026, 10, 6),
            ]
        );
    }

    #[test]
    fn a_row_shows_the_training_of_its_date() {
        let weekly = tuesdays();
        let mut cancelled = training(3, date(2026, 9, 22));
        cancelled.cancelled = true;
        let trainings = [
            training(1, date(2026, 10, 6)),
            training(2, date(2026, 9, 29)),
            cancelled,
        ];
        let roles = [
            role_from(1, date(2026, 9, 1)),
            role_from(2, date(2026, 9, 1)),
        ];
        // Player 3 has no role: its absence is kept but not counted.
        let absences = [absence(1, 1), absence(1, 2), absence(1, 3), absence(3, 1)];
        let list = season_list(
            season(),
            &weekly,
            &trainings,
            &absences,
            &roles,
            date(2026, 10, 12),
        );
        let status = |on| {
            list.past
                .iter()
                .find(|row| row.date == on)
                .map(|row| row.status)
        };
        assert_eq!(status(date(2026, 10, 6)), Some(DayStatus::Absent(2)));
        assert_eq!(status(date(2026, 9, 29)), Some(DayStatus::AllThere));
        assert_eq!(status(date(2026, 9, 22)), Some(DayStatus::Cancelled));
        assert_eq!(status(date(2026, 9, 15)), Some(DayStatus::NotOpened));
    }

    #[test]
    fn an_extra_training_has_its_own_row_and_a_training_on_a_training_day_has_one() {
        let weekly = tuesdays();
        let trainings = [
            training(1, date(2026, 10, 7)),
            training(2, date(2026, 10, 13)),
        ];
        let list = season_list(season(), &weekly, &trainings, &[], &[], date(2026, 10, 12));
        let rows: Vec<(NaiveDate, bool, bool)> = list
            .past
            .iter()
            .chain(&list.coming)
            .filter(|row| row.date >= date(2026, 10, 6) && row.date <= date(2026, 10, 13))
            .map(|row| (row.date, row.training.is_some(), row.extra))
            .collect();
        assert_eq!(
            rows,
            [
                (date(2026, 10, 7), true, true),
                (date(2026, 10, 6), false, false),
                (date(2026, 10, 13), true, false),
            ]
        );
    }

    #[test]
    fn a_season_without_weekly_trainings_or_trainings_has_an_empty_list() {
        let list = season_list(season(), &[], &[], &[], &[], date(2026, 10, 12));
        assert!(list.coming.is_empty() && list.past.is_empty());
        // A training of another season is not listed.
        let other = [training(1, date(2027, 7, 6))];
        let list = season_list(season(), &[], &other, &[], &[], date(2026, 10, 12));
        assert!(list.coming.is_empty());
    }

    #[test]
    fn a_training_is_checked() {
        let trainings = [training(1, date(2026, 10, 20))];
        let check =
            |on, start, end, except| check_training(season(), on, start, end, &trainings, except);
        assert_eq!(
            check(date(2026, 10, 22), time(19, 0), time(20, 30), None),
            Ok(())
        );
        assert_eq!(
            check(date(2026, 10, 20), time(19, 0), time(20, 30), None),
            Err(TrainingError::TrainingExists)
        );
        assert_eq!(
            check(
                date(2026, 10, 20),
                time(19, 0),
                time(20, 30),
                Some(TrainingId(1))
            ),
            Ok(())
        );
        assert_eq!(
            check(date(2027, 7, 1), time(19, 0), time(20, 30), None),
            Err(TrainingError::OutsideSeason)
        );
        assert_eq!(
            check(date(2026, 10, 22), time(20, 30), time(19, 0), None),
            Err(TrainingError::EndBeforeStart)
        );
    }

    #[test]
    fn the_times_of_a_training_day_and_the_proposed_times() {
        let weekly = tuesdays();
        assert_eq!(
            day_times(&weekly, date(2026, 10, 13)),
            Ok((time(19, 45), time(21, 0)))
        );
        assert_eq!(
            day_times(&weekly, date(2026, 10, 14)),
            Err(TrainingError::NotATrainingDay)
        );
        assert_eq!(
            proposed_times(&weekly, date(2026, 10, 13)),
            Some((time(19, 45), time(21, 0)))
        );
        // After the window, on the same day of the week.
        assert_eq!(
            proposed_times(&weekly, date(2027, 6, 1)),
            Some((time(19, 45), time(21, 0)))
        );
        assert_eq!(proposed_times(&weekly, date(2026, 10, 14)), None);
    }

    #[test]
    fn last_seasons_weekly_trainings_are_proposed_one_per_day() {
        let last = Season::from_start_year(2025);
        let of_last = |id, weekday, from, until, start| WeeklyTraining {
            id: WeeklyTrainingId(id),
            season: last,
            weekday,
            start,
            end: time(21, 0),
            from,
            until,
        };
        let one = [of_last(
            1,
            Weekday::Tue,
            date(2025, 9, 2),
            date(2026, 5, 26),
            time(19, 45),
        )];
        assert_eq!(
            proposals_from(&one, &[], season()),
            [WeeklyProposal {
                weekday: Weekday::Tue,
                start: time(19, 45),
                end: time(21, 0),
                from: date(2026, 9, 1),
                until: date(2027, 5, 25),
            }]
        );
        // Two Tuesdays (other times from January): one proposal, with the newest times.
        let two = [
            of_last(
                1,
                Weekday::Tue,
                date(2025, 9, 2),
                date(2026, 1, 6),
                time(19, 45),
            ),
            of_last(
                2,
                Weekday::Tue,
                date(2026, 1, 13),
                date(2026, 5, 26),
                time(20, 0),
            ),
        ];
        let proposals = proposals_from(&two, &[], season());
        assert_eq!(proposals.len(), 1);
        assert_eq!(proposals[0].start, time(20, 0));
        // Tuesday and Thursday until the end: two, sorted by the start of the window.
        let both = [
            of_last(
                1,
                Weekday::Thu,
                date(2025, 9, 4),
                date(2026, 5, 28),
                time(19, 0),
            ),
            of_last(
                2,
                Weekday::Tue,
                date(2025, 9, 2),
                date(2026, 5, 26),
                time(19, 45),
            ),
        ];
        let days = |proposals: Vec<WeeklyProposal>| -> Vec<Weekday> {
            proposals.iter().map(|proposal| proposal.weekday).collect()
        };
        assert_eq!(
            days(proposals_from(&both, &[], season())),
            [Weekday::Tue, Weekday::Thu]
        );
        // A day that has a weekly training in the new season is not proposed again.
        let tuesday = [weekly(9, Weekday::Tue, date(2026, 9, 1), date(2027, 5, 25))];
        assert_eq!(
            days(proposals_from(&both, &tuesday, season())),
            [Weekday::Thu]
        );
        // Moved from Tuesday to Thursday in January: the given-up Tuesday is not proposed.
        let moved = [
            of_last(
                1,
                Weekday::Tue,
                date(2025, 9, 2),
                date(2026, 1, 6),
                time(19, 45),
            ),
            of_last(
                2,
                Weekday::Thu,
                date(2026, 1, 8),
                date(2026, 5, 28),
                time(19, 45),
            ),
        ];
        assert_eq!(days(proposals_from(&moved, &[], season())), [Weekday::Thu]);
        assert!(proposals_from(&[], &[], season()).is_empty());
    }
}
