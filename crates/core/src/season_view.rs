//! The events of a season in one list: the games of the schedule and the training days and
//! trainings, for the page "Season". An event is a training or a game.

use std::cmp::Ordering;

use chrono::{Datelike, Days, NaiveDate, NaiveTime, Weekday};

use crate::schedule::{Schedule, ScheduledGame};
use crate::training::{SeasonList, SeasonRow};

/// What an event is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind<'a> {
    Game(&'a ScheduledGame),
    /// A training, or a training day that is not opened.
    Training(&'a SeasonRow<'a>),
}

/// One event of the season.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Event<'a> {
    pub date: NaiveDate,
    /// `None` only for a game whose start time is not known.
    pub start: Option<NaiveTime>,
    pub kind: EventKind<'a>,
}

impl Event<'_> {
    /// The ISO week of the date (Monday to Sunday): its number, its Monday and its Sunday.
    /// `None` is not reached for a date that chrono can hold.
    pub fn week(&self) -> Option<(u32, NaiveDate, NaiveDate)> {
        let week = self.date.iso_week();
        let monday = NaiveDate::from_isoywd_opt(week.year(), week.week(), Weekday::Mon)?;
        let sunday = monday.checked_add_days(Days::new(6))?;
        Some((week.week(), monday, sunday))
    }

    /// The order within a date: known start times in time order, then unknown; at the same time
    /// a training before a game; games by source id.
    fn within_date(&self) -> (bool, Option<NaiveTime>, u8, i64) {
        let (rank, id) = match self.kind {
            EventKind::Training(_) => (0, 0),
            EventKind::Game(scheduled) => (1, scheduled.game.source_id),
        };
        (self.start.is_none(), self.start, rank, id)
    }
}

/// The events of a season, split on today as the schedule and the list of trainings split them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SeasonEvents<'a> {
    /// Today and later, nearest date first.
    pub coming: Vec<Event<'a>>,
    /// Before today, newest date first.
    pub past: Vec<Event<'a>>,
}

/// The games of `games` and the rows of `trainings` in one list. Both were split on the same
/// today, so their parts are joined, not split again.
pub fn season_events<'a>(games: &'a Schedule, trainings: &'a SeasonList<'a>) -> SeasonEvents<'a> {
    let join = |scheduled: &'a [ScheduledGame], rows: &'a [SeasonRow<'a>]| -> Vec<Event<'a>> {
        scheduled
            .iter()
            .map(|scheduled| Event {
                date: scheduled.game.date,
                start: scheduled.game.start_time,
                kind: EventKind::Game(scheduled),
            })
            .chain(rows.iter().map(|row| Event {
                date: row.date,
                start: Some(row.start),
                kind: EventKind::Training(row),
            }))
            .collect()
    };
    let mut coming = join(&games.coming, &trainings.coming);
    coming.sort_by(|a, b| a.date.cmp(&b.date).then_with(|| order(a, b)));
    let mut past = join(&games.past, &trainings.past);
    past.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| order(a, b)));
    SeasonEvents { coming, past }
}

fn order(a: &Event<'_>, b: &Event<'_>) -> Ordering {
    a.within_date().cmp(&b.within_date())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schedule::ScheduleReason;
    use crate::schedule::tests::{OPPONENT, TEAM, game};
    use crate::training::DayStatus;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    fn time(hour: u32, minute: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(hour, minute, 0).expect("valid time")
    }

    /// A game of the team on `on` at `start`.
    fn scheduled(source_id: i64, on: NaiveDate, start: Option<NaiveTime>) -> ScheduledGame {
        let mut game = game(source_id, TEAM, OPPONENT);
        game.date = on;
        game.start_time = start;
        ScheduledGame {
            game,
            reason: ScheduleReason::TeamGame,
            in_list: true,
        }
    }

    /// A training day that is not opened, on `on` at 19:45.
    fn training_day(on: NaiveDate) -> SeasonRow<'static> {
        SeasonRow {
            date: on,
            start: time(19, 45),
            end: time(21, 0),
            training: None,
            extra: false,
            status: DayStatus::NotOpened,
        }
    }

    /// The source id of a game, or `None` for a training.
    fn ids(events: &[Event<'_>]) -> Vec<Option<i64>> {
        events
            .iter()
            .map(|event| match event.kind {
                EventKind::Game(scheduled) => Some(scheduled.game.source_id),
                EventKind::Training(_) => None,
            })
            .collect()
    }

    fn week_of(on: NaiveDate) -> Option<(u32, NaiveDate, NaiveDate)> {
        Event {
            date: on,
            start: None,
            kind: EventKind::Training(&training_day(on)),
        }
        .week()
    }

    #[test]
    fn a_game_at_14_30_comes_before_a_training_at_19_45() {
        let day = date(2026, 10, 31);
        let games = Schedule {
            coming: vec![scheduled(1, day, Some(time(14, 30)))],
            past: Vec::new(),
        };
        let trainings = SeasonList {
            coming: vec![training_day(day)],
            past: Vec::new(),
        };
        assert_eq!(
            ids(&season_events(&games, &trainings).coming),
            [Some(1), None]
        );
    }

    #[test]
    fn a_game_without_a_start_time_comes_after_a_training_on_its_date() {
        let day = date(2026, 10, 31);
        let games = Schedule {
            coming: vec![scheduled(1, day, None)],
            past: Vec::new(),
        };
        let trainings = SeasonList {
            coming: vec![training_day(day)],
            past: Vec::new(),
        };
        assert_eq!(
            ids(&season_events(&games, &trainings).coming),
            [None, Some(1)]
        );
    }

    #[test]
    fn at_the_same_time_a_training_comes_before_a_game() {
        let day = date(2026, 10, 31);
        let games = Schedule {
            coming: vec![
                scheduled(7, day, Some(time(19, 45))),
                scheduled(3, day, Some(time(19, 45))),
            ],
            past: Vec::new(),
        };
        let trainings = SeasonList {
            coming: vec![training_day(day)],
            past: Vec::new(),
        };
        assert_eq!(
            ids(&season_events(&games, &trainings).coming),
            [None, Some(3), Some(7)]
        );
    }

    #[test]
    fn coming_is_nearest_first_and_past_is_newest_first() {
        // Today is 12 October 2026: a game today is coming, as `schedule` splits it.
        let games = Schedule {
            coming: vec![
                scheduled(1, date(2026, 10, 12), Some(time(10, 0))),
                scheduled(2, date(2026, 10, 17), Some(time(10, 0))),
            ],
            past: vec![
                scheduled(3, date(2026, 10, 10), Some(time(10, 0))),
                scheduled(4, date(2026, 10, 3), Some(time(10, 0))),
            ],
        };
        let trainings = SeasonList {
            coming: vec![training_day(date(2026, 10, 13))],
            past: vec![training_day(date(2026, 10, 6))],
        };
        let events = season_events(&games, &trainings);
        let dates =
            |events: &[Event<'_>]| events.iter().map(|event| event.date).collect::<Vec<_>>();
        assert_eq!(
            dates(&events.coming),
            [date(2026, 10, 12), date(2026, 10, 13), date(2026, 10, 17)]
        );
        assert_eq!(
            dates(&events.past),
            [date(2026, 10, 10), date(2026, 10, 6), date(2026, 10, 3)]
        );
    }

    #[test]
    fn the_week_of_31_december_2026_is_week_53_from_28_december_to_3_january() {
        assert_eq!(
            week_of(date(2026, 12, 31)),
            Some((53, date(2026, 12, 28), date(2027, 1, 3)))
        );
        assert_eq!(
            week_of(date(2027, 1, 3)),
            Some((53, date(2026, 12, 28), date(2027, 1, 3)))
        );
    }

    #[test]
    fn the_week_of_25_october_2026_is_week_43() {
        // The day the clock changes; a date has no time zone.
        assert_eq!(
            week_of(date(2026, 10, 25)),
            Some((43, date(2026, 10, 19), date(2026, 10, 25)))
        );
        assert_eq!(
            week_of(date(2026, 10, 26)),
            Some((44, date(2026, 10, 26), date(2026, 11, 1)))
        );
    }
}
