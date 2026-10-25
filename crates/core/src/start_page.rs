//! The start page (story B of start page and season): the open items (what still needs the
//! coordinator), the last training, and the events of this week and the next.

use chrono::{Datelike, Days, NaiveDate};

use crate::schedule::ScheduleReason;
use crate::season::Season;
use crate::season_view::{Event, EventKind, SeasonEvents};
use crate::training::{DayStatus, SeasonList, SeasonRow, not_checked, not_opened};

/// The state of the game list for the open items (Dutch dates).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReadingState {
    /// A held-back reading waits for the coordinator.
    pub held_back: bool,
    /// The date of the last good reading, when the games may be out of date.
    pub out_of_date_since: Option<NaiveDate>,
}

/// Something that still needs the coordinator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenItem {
    ReadingHeldBack,
    OutOfDate {
        since: NaiveDate,
    },
    ChangesNotSeen {
        count: u32,
    },
    /// Training days before today that were never opened; `first` is the oldest.
    NotOpened {
        first: NaiveDate,
        count: usize,
    },
    /// Trainings before today that are neither cancelled nor checked; `first` is the oldest.
    NotChecked {
        first: NaiveDate,
        count: usize,
    },
}

/// The seasons the open items and the last training cover: the season of `today`, and in July
/// and August also the season before, so the record of June is not dropped on 1 July.
pub fn open_item_seasons(today: NaiveDate) -> Vec<Season> {
    let current = Season::of(today);
    if matches!(today.month(), 7 | 8) {
        vec![current, current.previous()]
    } else {
        vec![current]
    }
}

/// The open items in the order of the start page: the game list, the changes not seen, the
/// training days not opened, the trainings not checked. A kind without items gives nothing. The
/// training items count over all `lists`, with the oldest date as `first`.
pub fn open_items(reading: &ReadingState, unseen: u32, lists: &[SeasonList<'_>]) -> Vec<OpenItem> {
    let mut items = Vec::new();
    if reading.held_back {
        items.push(OpenItem::ReadingHeldBack);
    }
    if let Some(since) = reading.out_of_date_since {
        items.push(OpenItem::OutOfDate { since });
    }
    if unseen > 0 {
        items.push(OpenItem::ChangesNotSeen { count: unseen });
    }
    let days: Vec<NaiveDate> = lists.iter().flat_map(not_opened).collect();
    if let Some(&first) = days.iter().min() {
        items.push(OpenItem::NotOpened {
            first,
            count: days.len(),
        });
    }
    let trainings: Vec<NaiveDate> = lists.iter().flat_map(not_checked).collect();
    if let Some(&first) = trainings.iter().min() {
        items.push(OpenItem::NotChecked {
            first,
            count: trainings.len(),
        });
    }
    items
}

/// The newest row before today with a training that is not cancelled, over all `lists`; checked
/// or not.
pub fn last_training<'a>(lists: &'a [SeasonList<'a>]) -> Option<&'a SeasonRow<'a>> {
    lists
        .iter()
        .flat_map(|list| &list.past)
        .filter(|row| row.training.is_some_and(|training| !training.cancelled))
        .max_by_key(|row| row.date)
}

/// The games of the team and the training rows from `today` until the Sunday of next week (on a
/// Sunday: today and the next week). Cancelled trainings show but do not count: when the window
/// has nothing else, the first coming event after it that is not cancelled follows them.
/// Possible duty games are left out.
pub fn next_events<'a>(events: &'a SeasonEvents<'a>, today: NaiveDate) -> Vec<&'a Event<'a>> {
    let days_left = 6 - u64::from(today.weekday().num_days_from_monday()) + 7;
    let until = today
        .checked_add_days(Days::new(days_left))
        .unwrap_or(today);
    let of_the_team = |event: &&Event<'_>| match event.kind {
        EventKind::Game(scheduled) => scheduled.reason == ScheduleReason::TeamGame,
        EventKind::Training(_) => true,
    };
    let cancelled = |event: &&Event<'_>| matches!(event.kind, EventKind::Training(row) if row.status == DayStatus::Cancelled);
    let mut within: Vec<&Event<'_>> = events
        .coming
        .iter()
        .filter(of_the_team)
        .filter(|event| event.date <= until)
        .collect();
    if within.iter().all(cancelled) {
        within.extend(
            events
                .coming
                .iter()
                .filter(of_the_team)
                .filter(|event| event.date > until && !cancelled(event))
                .take(1),
        );
    }
    within
}

#[cfg(test)]
mod tests {
    use chrono::NaiveTime;

    use super::*;
    use crate::schedule::ScheduledGame;
    use crate::schedule::tests::{CLUB_TEAM, OPPONENT, TEAM, game};
    use crate::training::{Training, TrainingId};

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    fn time(hour: u32, minute: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(hour, minute, 0).expect("valid time")
    }

    fn training(id: i64, on: NaiveDate) -> Training {
        Training {
            id: TrainingId(id),
            date: on,
            start: time(19, 45),
            end: time(21, 0),
            cancelled: false,
            checked: false,
        }
    }

    /// A row of `training`, or a training day not opened when `None`.
    fn row(on: NaiveDate, training: Option<&Training>) -> SeasonRow<'_> {
        SeasonRow {
            date: on,
            start: time(19, 45),
            end: time(21, 0),
            training,
            extra: false,
            status: match training {
                None => DayStatus::NotOpened,
                Some(t) if t.cancelled => DayStatus::Cancelled,
                Some(_) => DayStatus::AllThere,
            },
        }
    }

    /// A list with these past rows (newest first, as `season_list` gives them).
    fn past(mut rows: Vec<SeasonRow<'_>>) -> SeasonList<'_> {
        rows.sort_by(|a, b| b.date.cmp(&a.date));
        SeasonList {
            coming: Vec::new(),
            past: rows,
        }
    }

    fn scheduled(source_id: i64, on: NaiveDate, reason: ScheduleReason) -> ScheduledGame {
        let mut game = match reason {
            ScheduleReason::TeamGame => game(source_id, TEAM, OPPONENT),
            ScheduleReason::DutyGame => game(source_id, CLUB_TEAM, OPPONENT),
        };
        game.date = on;
        ScheduledGame {
            game,
            reason,
            in_list: true,
        }
    }

    fn dates(events: &[&Event<'_>]) -> Vec<NaiveDate> {
        events.iter().map(|event| event.date).collect()
    }

    #[test]
    fn the_seasons_of_the_open_items_include_the_season_before_in_july_and_august() {
        let season = Season::from_start_year;
        assert_eq!(open_item_seasons(date(2027, 6, 30)), [season(2026)]);
        assert_eq!(
            open_item_seasons(date(2027, 7, 1)),
            [season(2027), season(2026)]
        );
        assert_eq!(
            open_item_seasons(date(2027, 8, 31)),
            [season(2027), season(2026)]
        );
        assert_eq!(open_item_seasons(date(2027, 9, 1)), [season(2027)]);
    }

    #[test]
    fn open_items_come_in_the_order_of_the_spec() {
        let not_checked = training(1, date(2026, 10, 6));
        let list = past(vec![
            row(date(2026, 10, 6), Some(&not_checked)),
            row(date(2026, 9, 29), None),
        ]);
        let reading = ReadingState {
            held_back: true,
            out_of_date_since: Some(date(2026, 10, 8)),
        };
        assert_eq!(
            open_items(&reading, 4, &[list]),
            [
                OpenItem::ReadingHeldBack,
                OpenItem::OutOfDate {
                    since: date(2026, 10, 8)
                },
                OpenItem::ChangesNotSeen { count: 4 },
                OpenItem::NotOpened {
                    first: date(2026, 9, 29),
                    count: 1
                },
                OpenItem::NotChecked {
                    first: date(2026, 10, 6),
                    count: 1
                },
            ]
        );
    }

    #[test]
    fn one_and_three_training_days_not_opened() {
        let one = past(vec![row(date(2026, 10, 6), None)]);
        assert_eq!(
            open_items(&ReadingState::default(), 0, &[one]),
            [OpenItem::NotOpened {
                first: date(2026, 10, 6),
                count: 1
            }]
        );
        let three = past(vec![
            row(date(2026, 10, 6), None),
            row(date(2026, 9, 29), None),
            row(date(2026, 9, 22), None),
        ]);
        assert_eq!(
            open_items(&ReadingState::default(), 0, &[three]),
            [OpenItem::NotOpened {
                first: date(2026, 9, 22),
                count: 3
            }]
        );
    }

    #[test]
    fn the_days_of_two_seasons_add_up() {
        let this = past(vec![row(date(2027, 7, 6), None)]);
        let before = past(vec![row(date(2027, 6, 29), None)]);
        assert_eq!(
            open_items(&ReadingState::default(), 0, &[this, before]),
            [OpenItem::NotOpened {
                first: date(2027, 6, 29),
                count: 2
            }]
        );
    }

    #[test]
    fn nothing_open_gives_no_items() {
        let mut checked = training(1, date(2026, 10, 6));
        checked.checked = true;
        let list = past(vec![row(date(2026, 10, 6), Some(&checked))]);
        assert_eq!(open_items(&ReadingState::default(), 0, &[list]), []);
    }

    #[test]
    fn a_cancelled_training_is_never_an_open_item() {
        let mut cancelled = training(1, date(2026, 10, 6));
        cancelled.cancelled = true;
        let list = past(vec![row(date(2026, 10, 6), Some(&cancelled))]);
        assert_eq!(open_items(&ReadingState::default(), 0, &[list]), []);
    }

    #[test]
    fn last_is_the_newest_past_training_also_when_checked() {
        let mut checked = training(1, date(2026, 10, 6));
        checked.checked = true;
        let mut cancelled = training(2, date(2026, 10, 8));
        cancelled.cancelled = true;
        let older = training(3, date(2026, 9, 29));
        let list = past(vec![
            row(date(2026, 10, 9), None),
            row(date(2026, 10, 8), Some(&cancelled)),
            row(date(2026, 10, 6), Some(&checked)),
            row(date(2026, 9, 29), Some(&older)),
        ]);
        let lists = [list];
        assert_eq!(
            last_training(&lists).map(|row| row.date),
            Some(date(2026, 10, 6))
        );
    }

    #[test]
    fn on_2_july_last_is_the_training_of_30_june() {
        let june = training(1, date(2027, 6, 30));
        let lists = [
            past(vec![row(date(2027, 7, 1), None)]),
            past(vec![row(date(2027, 6, 30), Some(&june))]),
        ];
        assert_eq!(
            last_training(&lists).map(|row| row.date),
            Some(date(2027, 6, 30))
        );
    }

    #[test]
    fn no_past_training_gives_no_last() {
        let lists = [past(vec![row(date(2026, 10, 6), None)])];
        assert_eq!(last_training(&lists), None);
    }

    /// The events of the season from `games` and training days on `days`, all coming.
    fn coming<'a>(games: &'a [ScheduledGame], rows: &'a [SeasonRow<'a>]) -> SeasonEvents<'a> {
        let mut coming: Vec<Event<'a>> = games
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
            .collect();
        coming.sort_by_key(|event| event.date);
        SeasonEvents {
            coming,
            past: Vec::new(),
        }
    }

    #[test]
    fn on_monday_26_october_next_runs_until_sunday_8_november() {
        let games = [
            scheduled(1, date(2026, 10, 31), ScheduleReason::TeamGame),
            scheduled(2, date(2026, 11, 8), ScheduleReason::TeamGame),
            scheduled(3, date(2026, 11, 9), ScheduleReason::TeamGame),
        ];
        let rows = [row(date(2026, 10, 27), None), row(date(2026, 11, 10), None)];
        let events = coming(&games, &rows);
        assert_eq!(
            dates(&next_events(&events, date(2026, 10, 26))),
            [date(2026, 10, 27), date(2026, 10, 31), date(2026, 11, 8)]
        );
    }

    #[test]
    fn on_sunday_25_october_next_runs_until_sunday_1_november() {
        let games = [
            scheduled(1, date(2026, 10, 25), ScheduleReason::TeamGame),
            scheduled(2, date(2026, 11, 1), ScheduleReason::TeamGame),
            scheduled(3, date(2026, 11, 2), ScheduleReason::TeamGame),
        ];
        let events = coming(&games, &[]);
        assert_eq!(
            dates(&next_events(&events, date(2026, 10, 25))),
            [date(2026, 10, 25), date(2026, 11, 1)]
        );
    }

    #[test]
    fn next_leaves_out_possible_duty_games() {
        let games = [
            scheduled(1, date(2026, 10, 31), ScheduleReason::DutyGame),
            scheduled(2, date(2026, 10, 31), ScheduleReason::TeamGame),
        ];
        let events = coming(&games, &[]);
        let next = next_events(&events, date(2026, 10, 26));
        assert_eq!(next.len(), 1);
        assert!(matches!(next[0].kind, EventKind::Game(game) if game.game.source_id == 2));
    }

    #[test]
    fn nothing_in_two_weeks_gives_the_first_coming_event() {
        let games = [
            scheduled(1, date(2026, 12, 5), ScheduleReason::DutyGame),
            scheduled(2, date(2026, 12, 12), ScheduleReason::TeamGame),
            scheduled(3, date(2026, 12, 19), ScheduleReason::TeamGame),
        ];
        let events = coming(&games, &[]);
        assert_eq!(
            dates(&next_events(&events, date(2026, 10, 26))),
            [date(2026, 12, 12)]
        );
    }

    /// A cancelled training on `on`.
    fn cancelled_day(on: NaiveDate, training: &Training) -> SeasonRow<'_> {
        let mut row = row(on, Some(training));
        row.status = DayStatus::Cancelled;
        row
    }

    #[test]
    fn a_cancelled_training_in_the_window_does_not_count() {
        let mut cancelled = training(1, date(2026, 10, 27));
        cancelled.cancelled = true;
        let games = [scheduled(1, date(2026, 12, 12), ScheduleReason::TeamGame)];
        let rows = [cancelled_day(date(2026, 10, 27), &cancelled)];
        let events = coming(&games, &rows);
        assert_eq!(
            dates(&next_events(&events, date(2026, 10, 26))),
            [date(2026, 10, 27), date(2026, 12, 12)]
        );
    }

    #[test]
    fn the_first_coming_event_is_not_a_cancelled_training() {
        let mut cancelled = training(1, date(2026, 12, 1));
        cancelled.cancelled = true;
        let games = [scheduled(1, date(2026, 12, 12), ScheduleReason::TeamGame)];
        let rows = [cancelled_day(date(2026, 12, 1), &cancelled)];
        let events = coming(&games, &rows);
        assert_eq!(
            dates(&next_events(&events, date(2026, 10, 26))),
            [date(2026, 12, 12)]
        );
    }

    #[test]
    fn nothing_coming_gives_nothing_next() {
        let events = SeasonEvents::default();
        assert!(next_events(&events, date(2026, 10, 26)).is_empty());
    }
}
