//! The start page (story B of start page and season): the open items (what still needs the
//! coordinator), the duty games of the last date with duties, how each event looks, and the
//! events of this week and the next.

use std::collections::BTreeMap;

use chrono::{Datelike, Days, NaiveDate};

use crate::contribution::attendance::TrainingState;
use crate::contribution::duties::{
    CheckKind, DutyKind, GamePlaces, StoredDuties, check_kind, game_checked, has_team_places,
    not_entered, open_by_kind, open_places,
};
use crate::contribution::game_attendance::{GameAttendance, GameCount, GameWarnings, checked};
use crate::events::schedule::{ScheduleReason, ScheduledGame};
use crate::events::training::{SeasonList, not_checked, not_opened};
use crate::output::season_view::{Event, EventKind, SeasonEvents};
use crate::shared::season::Season;
use crate::team::players::PlayerId;

/// The state of the game list for the open items (Dutch dates).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReadingState {
    /// A held-back reading waits for the coordinator.
    pub held_back: bool,
    /// The date of the last good reading, when the games may be out of date.
    pub out_of_date_since: Option<NaiveDate>,
}

/// Something that still needs the coordinator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenItem {
    ReadingHeldBack,
    OutOfDate {
        since: NaiveDate,
    },
    ChangesNotSeen {
        count: u32,
    },
    /// Coming games of the team with too few players in their warning period, with the count, in
    /// time order (`GameWarnings::too_few`).
    TooFewPlayers {
        games: Vec<(i64, GameCount)>,
    },
    /// Coming games of the team with drivers who are not coming, in time order.
    DriverNotComing {
        games: Vec<(i64, Vec<PlayerId>)>,
    },
    /// Coming games of the team with answers from before a move, in their warning period, in
    /// time order.
    AnswersMoved {
        games: Vec<i64>,
    },
    /// Home games of the club from today to 4 weeks ahead whose schedule is not entered, in time
    /// order (by source id).
    ScheduleNotEntered {
        games: Vec<i64>,
    },
    /// Coming games with places of the team without a player (`open_places_games`), in time
    /// order.
    OpenPlaces {
        games: Vec<OpenGame>,
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
    /// Games of the team before today that are not checked (`games_not_checked`), oldest first.
    GamesNotChecked {
        games: Vec<NotCheckedGame>,
    },
    /// Duty games before today that are not checked (`duty_games_not_checked`), oldest first.
    DutyGamesNotChecked {
        games: Vec<NotCheckedGame>,
    },
}

/// A game (of the team, or a duty game) that is not checked: enough to open its page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotCheckedGame {
    pub season: Season,
    pub id: i64,
    pub date: NaiveDate,
}

/// The games of the team of `season` before `today` (Dutch date) that are not checked on their
/// date now: still in the game list and not "not played", from the start of the season.
/// `games`: the past games of the schedule; `checks`: what is stored for the attendance of each
/// game, by source id. In time order.
pub fn games_not_checked(
    season: Season,
    games: &[ScheduledGame],
    checks: &BTreeMap<i64, GameAttendance>,
    today: NaiveDate,
) -> Vec<NotCheckedGame> {
    let mut open: Vec<NotCheckedGame> = games
        .iter()
        .filter(|scheduled| scheduled.reason == ScheduleReason::TeamGame)
        .filter(|scheduled| scheduled.in_list && !scheduled.not_played)
        .filter(|scheduled| scheduled.game.date < today)
        .filter(|scheduled| {
            !checks
                .get(&scheduled.game.source_id)
                .is_some_and(|stored| checked(stored, scheduled.game.date))
        })
        .map(|scheduled| NotCheckedGame {
            season,
            id: scheduled.game.source_id,
            date: scheduled.game.date,
        })
        .collect();
    open.sort_by_key(|game| (game.date, game.id));
    open
}

/// The games not checked of the open items: of the team (`games_not_checked`) and duty games
/// (`duty_games_not_checked`), over the seasons of the open items.
#[derive(Debug, Clone, Copy, Default)]
pub struct NotCheckedGames<'a> {
    pub games: &'a [NotCheckedGame],
    pub duty_games: &'a [NotCheckedGame],
}

/// The duty games of `season` before `today` (Dutch date) that are not checked on their date now
/// (`game_checked`, decision 0035): a home game of another club team with table or referee places
/// of the team (`check_kind`), still in the game list and not "not played". `places`: the places of
/// the games; `stored`: the stored duties of each game, by source id. In time order.
pub fn duty_games_not_checked(
    season: Season,
    games: &[ScheduledGame],
    places: &PlacesOf,
    stored: &BTreeMap<i64, StoredDuties>,
    today: NaiveDate,
) -> Vec<NotCheckedGame> {
    let mut open: Vec<NotCheckedGame> = games
        .iter()
        .filter(|scheduled| scheduled.in_list && !scheduled.not_played)
        .filter(|scheduled| scheduled.game.date < today)
        .filter(|scheduled| {
            let id = scheduled.game.source_id;
            let kind = check_kind(scheduled.reason, places.get(&id));
            kind == Some(CheckKind::Duty)
                && !game_checked(kind, None, stored.get(&id), scheduled.game.date, false)
        })
        .map(|scheduled| NotCheckedGame {
            season,
            id: scheduled.game.source_id,
            date: scheduled.game.date,
        })
        .collect();
    open.sort_by_key(|game| (game.date, game.id));
    open
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

/// A game of the open item "games with open places": its open places of the team, for each kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenGame {
    pub id: i64,
    pub open: BTreeMap<DutyKind, usize>,
}

/// The duty places of the games of a season, by source id (`duties::game_places`).
pub type PlacesOf = BTreeMap<i64, GamePlaces>;

/// The days a duty open item covers: today and the next 27 days (4 weeks).
const DUTY_DAYS: u64 = 27;

/// The home games of the club from today to 4 weeks ahead whose schedule is not entered and whose
/// places count (in the game list, still a home game of the club), in time order. A past game is
/// never in it.
pub fn schedule_not_entered(
    events: &SeasonEvents<'_>,
    places: &PlacesOf,
    today: NaiveDate,
) -> Vec<i64> {
    let until = today
        .checked_add_days(Days::new(DUTY_DAYS))
        .unwrap_or(today);
    events
        .coming
        .iter()
        .filter(|event| event.date <= until)
        .filter_map(|event| match event.kind {
            EventKind::Game(scheduled) => Some(scheduled.game.source_id),
            EventKind::Training(..) => None,
        })
        .filter(|id| places.get(id).is_some_and(not_entered))
        .collect()
}

/// The coming games with places of the team that count and have no player, in time order: the
/// table and referee places from today to 4 weeks ahead, and the driver places from today until
/// the Sunday of next week (the window of "Next"). A game whose part does not count (out of the
/// game list, no longer needed) is not in it; a rising driver proposal opens places by itself.
pub fn open_places_games(
    events: &SeasonEvents<'_>,
    places: &PlacesOf,
    today: NaiveDate,
) -> Vec<OpenGame> {
    let duty_until = today
        .checked_add_days(Days::new(DUTY_DAYS))
        .unwrap_or(today);
    let drivers_until = next_until(today);
    events
        .coming
        .iter()
        .filter_map(|event| match event.kind {
            EventKind::Game(scheduled) => Some((event.date, scheduled.game.source_id)),
            EventKind::Training(..) => None,
        })
        .filter_map(|(date, id)| {
            let mut open = open_by_kind(places.get(&id)?);
            open.retain(|&kind, _| match kind {
                DutyKind::Drive => date <= drivers_until,
                DutyKind::Table | DutyKind::Referee => date <= duty_until,
            });
            (!open.is_empty()).then_some(OpenGame { id, open })
        })
        .collect()
}

/// The open items in the order of the start page: the game list, the changes not seen, too few
/// players, drivers not coming, answers from before a move (from `warnings`: the warnings of the
/// coming games of the team in time order, `game_warnings`), the home games whose schedule is not
/// entered (`schedule_not_entered`), the games with open places (`open_places_games`), the
/// training days not opened, the trainings not checked, the games not checked
/// (`games_not_checked`) and the duty games not checked (`duty_games_not_checked`), each over all
/// seasons and oldest first. A kind without items gives nothing. The training items
/// count over all `lists`, with the oldest date as `first`.
pub fn open_items(
    reading: &ReadingState,
    unseen: u32,
    schedule_not_entered: Vec<i64>,
    open_places: Vec<OpenGame>,
    lists: &[SeasonList<'_>],
    warnings: &[(i64, GameWarnings)],
    games_open: NotCheckedGames<'_>,
) -> Vec<OpenItem> {
    let NotCheckedGames {
        games: games_not_checked,
        duty_games: duty_games_not_checked,
    } = games_open;
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
    let too_few: Vec<(i64, GameCount)> = warnings
        .iter()
        .filter_map(|(id, warnings)| warnings.too_few.map(|count| (*id, count)))
        .collect();
    if !too_few.is_empty() {
        items.push(OpenItem::TooFewPlayers { games: too_few });
    }
    let drivers: Vec<(i64, Vec<PlayerId>)> = warnings
        .iter()
        .filter(|(_, warnings)| !warnings.drivers_not_coming.is_empty())
        .map(|(id, warnings)| (*id, warnings.drivers_not_coming.clone()))
        .collect();
    if !drivers.is_empty() {
        items.push(OpenItem::DriverNotComing { games: drivers });
    }
    let moved: Vec<i64> = warnings
        .iter()
        .filter(|(_, warnings)| warnings.moved)
        .map(|(id, _)| *id)
        .collect();
    if !moved.is_empty() {
        items.push(OpenItem::AnswersMoved { games: moved });
    }
    if !schedule_not_entered.is_empty() {
        items.push(OpenItem::ScheduleNotEntered {
            games: schedule_not_entered,
        });
    }
    if !open_places.is_empty() {
        items.push(OpenItem::OpenPlaces { games: open_places });
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
    let oldest_first = |games: &[NotCheckedGame]| {
        let mut games = games.to_vec();
        games.sort_by_key(|game| (game.date, game.id));
        games
    };
    if !games_not_checked.is_empty() {
        items.push(OpenItem::GamesNotChecked {
            games: oldest_first(games_not_checked),
        });
    }
    if !duty_games_not_checked.is_empty() {
        items.push(OpenItem::DutyGamesNotChecked {
            games: oldest_first(duty_games_not_checked),
        });
    }
    items
}

/// Whether a game has places of the team that count.
fn has_places(places: &PlacesOf, source_id: i64) -> bool {
    places.get(&source_id).is_some_and(has_team_places)
}

/// The games of the newest date before today that have places of the team (table, referee or
/// drivers) and are not games of the team, in time order; also games that left the game list
/// later (a past game counts). A game of the team is an open item until it is checked.
pub fn last_games<'a>(events: &'a SeasonEvents<'a>, places: &PlacesOf) -> Vec<&'a Event<'a>> {
    let with_places: Vec<&Event<'_>> = events
        .past
        .iter()
        .filter(|event| match event.kind {
            EventKind::Game(scheduled) => {
                scheduled.reason != ScheduleReason::TeamGame
                    && has_places(places, scheduled.game.source_id)
            }
            EventKind::Training(..) => false,
        })
        .collect();
    let Some(newest) = with_places.first().map(|event| event.date) else {
        return Vec::new();
    };
    with_places
        .into_iter()
        .filter(|event| event.date == newest)
        .collect()
}

/// How an event looks on the start page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventLook {
    Normal,
    /// Nothing is left to do: the row is lighter.
    Lighter,
}

/// `Lighter` when nothing is left to do for `event`: an event before `today` (its date has passed:
/// the last duty games; a past training or game of the team is an open item, not a row); a
/// training that is checked or cancelled; a game that is not played; a game of the team that is
/// checked (`game_checked`); another game in the game list whose places of the team are all
/// filled and whose schedule is entered. All else is `Normal`, also another game that left the
/// game list (its row is crossed out). `places`: `game_places` of the game.
pub fn event_look(
    event: &Event<'_>,
    places: Option<&GamePlaces>,
    game_checked: bool,
    today: NaiveDate,
) -> EventLook {
    let lighter = event.date < today
        || match event.kind {
            EventKind::Training(row, state) => {
                matches!(state, TrainingState::Cancelled)
                    || row
                        .training
                        .is_some_and(|training| training.checked || training.cancelled)
            }
            EventKind::Game(scheduled) if scheduled.not_played => true,
            EventKind::Game(scheduled) if scheduled.reason == ScheduleReason::TeamGame => {
                game_checked
            }
            EventKind::Game(scheduled) => {
                scheduled.in_list
                    && places.is_some_and(|places| {
                        has_team_places(places) && open_places(places) == 0 && !not_entered(places)
                    })
            }
        };
    if lighter {
        EventLook::Lighter
    } else {
        EventLook::Normal
    }
}

/// The last day of the window of "Next": the Sunday of next week (on a Sunday: the next Sunday).
fn next_until(today: NaiveDate) -> NaiveDate {
    let days_left = 6 - u64::from(today.weekday().num_days_from_monday()) + 7;
    today
        .checked_add_days(Days::new(days_left))
        .unwrap_or(today)
}

/// The games of the team, the home games of other club teams with places of the team, and the
/// training rows from `today` until the Sunday of next week (on a Sunday: today and the next
/// week). Cancelled trainings show but do not count: when the window has nothing else, the first
/// coming event after it that is not cancelled follows them. Other games are left out.
pub fn next_events<'a>(
    events: &'a SeasonEvents<'a>,
    places: &PlacesOf,
    today: NaiveDate,
) -> Vec<&'a Event<'a>> {
    let until = next_until(today);
    let of_the_team = |event: &&Event<'_>| match event.kind {
        EventKind::Game(scheduled) => {
            scheduled.reason == ScheduleReason::TeamGame
                || has_places(places, scheduled.game.source_id)
        }
        EventKind::Training(..) => true,
    };
    let cancelled =
        |event: &&Event<'_>| matches!(event.kind, EventKind::Training(_, TrainingState::Cancelled));
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
    use crate::contribution::duties::GameMoment;
    use chrono::NaiveTime;

    use super::*;
    use crate::contribution::attendance::training_state;
    use crate::events::schedule::ScheduledGame;
    use crate::events::schedule::tests::{CLUB_TEAM, OPPONENT, TEAM, game};
    use crate::events::training::{DayStatus, SeasonRow, Training, TrainingId};

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
                Some(_) => DayStatus::Opened,
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
            ScheduleReason::Kept => game(source_id, OPPONENT, CLUB_TEAM),
        };
        game.date = on;
        ScheduledGame {
            game,
            reason,
            in_list: true,
            not_played: false,
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
            open_items(
                &reading,
                4,
                Vec::new(),
                Vec::new(),
                &[list],
                &[],
                NotCheckedGames {
                    games: &[],
                    duty_games: &[]
                }
            ),
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
            open_items(
                &ReadingState::default(),
                0,
                Vec::new(),
                Vec::new(),
                &[one],
                &[],
                NotCheckedGames {
                    games: &[],
                    duty_games: &[]
                }
            ),
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
            open_items(
                &ReadingState::default(),
                0,
                Vec::new(),
                Vec::new(),
                &[three],
                &[],
                NotCheckedGames {
                    games: &[],
                    duty_games: &[]
                }
            ),
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
            open_items(
                &ReadingState::default(),
                0,
                Vec::new(),
                Vec::new(),
                &[this, before],
                &[],
                NotCheckedGames {
                    games: &[],
                    duty_games: &[]
                }
            ),
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
        assert_eq!(
            open_items(
                &ReadingState::default(),
                0,
                Vec::new(),
                Vec::new(),
                &[list],
                &[],
                NotCheckedGames {
                    games: &[],
                    duty_games: &[]
                }
            ),
            []
        );
    }

    #[test]
    fn a_cancelled_training_is_never_an_open_item() {
        let mut cancelled = training(1, date(2026, 10, 6));
        cancelled.cancelled = true;
        let list = past(vec![row(date(2026, 10, 6), Some(&cancelled))]);
        assert_eq!(
            open_items(
                &ReadingState::default(),
                0,
                Vec::new(),
                Vec::new(),
                &[list],
                &[],
                NotCheckedGames {
                    games: &[],
                    duty_games: &[]
                }
            ),
            []
        );
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
                kind: EventKind::Training(row, training_state(row, &[], &[])),
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
            dates(&next_events(&events, &PlacesOf::new(), date(2026, 10, 26))),
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
            dates(&next_events(&events, &PlacesOf::new(), date(2026, 10, 25))),
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
        let next = next_events(&events, &PlacesOf::new(), date(2026, 10, 26));
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
            dates(&next_events(&events, &PlacesOf::new(), date(2026, 10, 26))),
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
            dates(&next_events(&events, &PlacesOf::new(), date(2026, 10, 26))),
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
            dates(&next_events(&events, &PlacesOf::new(), date(2026, 10, 26))),
            [date(2026, 12, 12)]
        );
    }

    #[test]
    fn nothing_coming_gives_nothing_next() {
        let events = SeasonEvents::default();
        assert!(next_events(&events, &PlacesOf::new(), date(2026, 10, 26)).is_empty());
    }

    /// The places of a game: `entered` or not, with table 1 for the team or no places;
    /// `home` false: no longer a home game of the club; `in_list` and `past` as `game_places`.
    fn home_places(
        entered: bool,
        team_place: bool,
        home: bool,
        in_list: bool,
        past: bool,
    ) -> GamePlaces {
        use crate::contribution::duties::{
            PlaceKind, RequiredDuties, StoredDuties, TeamRef, game_places,
        };
        let mut stored = StoredDuties {
            schedule_entered: entered,
            ..StoredDuties::default()
        };
        if team_place {
            stored.teams.insert(
                (PlaceKind::Table, 1),
                TeamRef {
                    number: TEAM,
                    name: "M16 1".to_owned(),
                },
            );
        }
        let required = RequiredDuties {
            schedule: home,
            drivers: false,
        };
        game_places(
            required,
            Some(&stored),
            TEAM,
            &Default::default(),
            10,
            GameMoment::played(in_list, past),
        )
        .expect("places")
    }

    #[test]
    fn schedule_not_entered_covers_the_home_games_of_the_next_4_weeks() {
        let mut gone = scheduled(3, date(2026, 10, 31), ScheduleReason::DutyGame);
        gone.in_list = false;
        let games = [
            scheduled(1, date(2026, 10, 31), ScheduleReason::DutyGame),
            scheduled(2, date(2026, 11, 22), ScheduleReason::TeamGame),
            gone,
            scheduled(4, date(2026, 11, 23), ScheduleReason::DutyGame),
            scheduled(5, date(2026, 11, 1), ScheduleReason::DutyGame),
            scheduled(6, date(2026, 11, 2), ScheduleReason::Kept),
        ];
        let events = coming(&games, &[]);
        let places: PlacesOf = [
            (1, home_places(false, false, true, true, false)),
            (2, home_places(false, true, true, true, false)),
            (3, home_places(false, false, true, false, false)),
            (4, home_places(false, false, true, true, false)),
            (5, home_places(true, false, true, true, false)),
            // No longer a home game of the club (home and away swapped).
            (6, home_places(false, true, false, true, false)),
        ]
        .into_iter()
        .collect();
        // Mon 26 Oct: until Sun 22 Nov (today + 27 days). Gone, entered and swapped games are
        // left out; a home game of the team is in it.
        assert_eq!(
            schedule_not_entered(&events, &places, date(2026, 10, 26)),
            [1, 2]
        );
    }

    #[test]
    fn schedule_not_entered_comes_after_the_changes_and_before_the_trainings() {
        let list = past(vec![row(date(2026, 10, 6), None)]);
        assert_eq!(
            open_items(
                &ReadingState::default(),
                2,
                vec![7, 8],
                Vec::new(),
                &[list],
                &[],
                NotCheckedGames {
                    games: &[],
                    duty_games: &[]
                }
            ),
            [
                OpenItem::ChangesNotSeen { count: 2 },
                OpenItem::ScheduleNotEntered { games: vec![7, 8] },
                OpenItem::NotOpened {
                    first: date(2026, 10, 6),
                    count: 1
                },
            ]
        );
    }

    /// The drivers of an away game of the team with `players` on its date and `filled` driver
    /// places with a player; `in_list` as `game_places`.
    fn driver_places(players: usize, filled: u8, in_list: bool) -> GamePlaces {
        use crate::contribution::duties::{RequiredDuties, game_places};
        let required = RequiredDuties {
            schedule: false,
            drivers: true,
        };
        let filled = (1..=filled)
            .map(|number| (DutyKind::Drive, number))
            .collect();
        game_places(
            required,
            None,
            TEAM,
            &filled,
            players,
            GameMoment::played(in_list, false),
        )
        .expect("places")
    }

    fn open_game(id: i64, open: &[(DutyKind, usize)]) -> OpenGame {
        OpenGame {
            id,
            open: open.iter().copied().collect(),
        }
    }

    #[test]
    fn open_places_cover_4_weeks_for_the_table_and_this_week_and_next_for_the_drivers() {
        let mut gone = scheduled(3, date(2026, 10, 31), ScheduleReason::DutyGame);
        gone.in_list = false;
        let games = [
            // Mon 26 Oct + 27 days: Sun 22 Nov.
            scheduled(1, date(2026, 11, 22), ScheduleReason::DutyGame),
            scheduled(2, date(2026, 11, 23), ScheduleReason::DutyGame),
            gone,
            // The Sunday of next week: Sun 8 Nov.
            scheduled(4, date(2026, 11, 8), ScheduleReason::TeamGame),
            scheduled(5, date(2026, 11, 9), ScheduleReason::TeamGame),
            // Entered, but its place has no player: still open.
            scheduled(6, date(2026, 10, 31), ScheduleReason::DutyGame),
        ];
        let events = coming(&games, &[]);
        let places: PlacesOf = [
            (1, home_places(false, true, true, true, false)),
            (2, home_places(false, true, true, true, false)),
            (3, home_places(false, true, true, false, false)),
            (4, driver_places(4, 0, true)),
            (5, driver_places(4, 0, true)),
            (6, home_places(true, true, true, true, false)),
        ]
        .into_iter()
        .collect();
        assert_eq!(
            open_places_games(&events, &places, date(2026, 10, 26)),
            [
                open_game(6, &[(DutyKind::Table, 1)]),
                open_game(4, &[(DutyKind::Drive, 2)]),
                open_game(1, &[(DutyKind::Table, 1)]),
            ]
        );
    }

    #[test]
    fn a_rising_driver_proposal_makes_an_open_item_without_a_save() {
        let games = [scheduled(1, date(2026, 10, 31), ScheduleReason::TeamGame)];
        let events = coming(&games, &[]);
        let today = date(2026, 10, 26);
        // 3 players: 1 driver, on the place.
        let places: PlacesOf = [(1, driver_places(3, 1, true))].into_iter().collect();
        assert!(open_places_games(&events, &places, today).is_empty());
        // A fourth player joins: 2 drivers, one open.
        let places: PlacesOf = [(1, driver_places(4, 1, true))].into_iter().collect();
        assert_eq!(
            open_places_games(&events, &places, today),
            [open_game(1, &[(DutyKind::Drive, 1)])]
        );
        // Out of the game list: not counted.
        let places: PlacesOf = [(1, driver_places(4, 1, false))].into_iter().collect();
        assert!(open_places_games(&events, &places, today).is_empty());
    }

    #[test]
    fn open_places_come_after_the_schedule_and_before_the_trainings() {
        let list = past(vec![row(date(2026, 10, 6), None)]);
        let open = vec![open_game(9, &[(DutyKind::Table, 1)])];
        assert_eq!(
            open_items(
                &ReadingState::default(),
                0,
                vec![7],
                open.clone(),
                &[list],
                &[],
                NotCheckedGames {
                    games: &[],
                    duty_games: &[]
                }
            ),
            [
                OpenItem::ScheduleNotEntered { games: vec![7] },
                OpenItem::OpenPlaces { games: open },
                OpenItem::NotOpened {
                    first: date(2026, 10, 6),
                    count: 1
                },
            ]
        );
    }

    #[test]
    fn next_shows_the_home_games_of_other_club_teams_with_places_of_the_team() {
        let games = [
            scheduled(1, date(2026, 10, 31), ScheduleReason::DutyGame),
            scheduled(2, date(2026, 10, 31), ScheduleReason::DutyGame),
            scheduled(3, date(2026, 11, 1), ScheduleReason::DutyGame),
        ];
        let events = coming(&games, &[]);
        let places: PlacesOf = [
            (1, home_places(true, true, true, true, false)),
            (2, home_places(true, false, true, true, false)),
            (3, home_places(false, false, true, true, false)),
        ]
        .into_iter()
        .collect();
        let next = next_events(&events, &places, date(2026, 10, 26));
        assert_eq!(next.len(), 1);
        assert!(matches!(next[0].kind, EventKind::Game(game) if game.game.source_id == 1));
    }

    #[test]
    fn last_games_are_the_games_of_the_newest_date_with_places_also_when_gone() {
        let mut gone = scheduled(2, date(2026, 10, 10), ScheduleReason::DutyGame);
        gone.in_list = false;
        let games = [
            gone,
            scheduled(4, date(2026, 10, 10), ScheduleReason::DutyGame),
            scheduled(5, date(2026, 10, 10), ScheduleReason::TeamGame),
            scheduled(1, date(2026, 10, 3), ScheduleReason::DutyGame),
            scheduled(3, date(2026, 10, 11), ScheduleReason::TeamGame),
        ];
        let past_events: Vec<Event<'_>> = games
            .iter()
            .map(|scheduled| Event {
                date: scheduled.game.date,
                start: scheduled.game.start_time,
                kind: EventKind::Game(scheduled),
            })
            .collect();
        let mut past_events = past_events;
        past_events.sort_by(|a, b| b.date.cmp(&a.date));
        let events = SeasonEvents {
            coming: Vec::new(),
            past: past_events,
        };
        let places: PlacesOf = [
            (1, home_places(true, true, true, true, true)),
            (2, home_places(true, true, true, false, true)),
            (4, home_places(true, true, true, true, true)),
            // No place of the team: not among the last duty games.
            (5, home_places(true, false, true, true, true)),
        ]
        .into_iter()
        .collect();
        let ids: Vec<i64> = last_games(&events, &places)
            .iter()
            .filter_map(|event| match event.kind {
                EventKind::Game(scheduled) => Some(scheduled.game.source_id),
                EventKind::Training(..) => None,
            })
            .collect();
        // 11 Oct has no places; 10 Oct has two games with places, also the gone one.
        assert_eq!(ids, [2, 4]);
        assert!(last_games(&events, &PlacesOf::new()).is_empty());
    }

    #[test]
    fn last_games_leave_out_the_games_of_the_team() {
        let games = [
            // The newest past date has only a game of the team with drivers.
            scheduled(1, date(2026, 10, 17), ScheduleReason::TeamGame),
            scheduled(2, date(2026, 10, 10), ScheduleReason::DutyGame),
            scheduled(3, date(2026, 10, 10), ScheduleReason::TeamGame),
        ];
        let mut past_events: Vec<Event<'_>> = games
            .iter()
            .map(|scheduled| Event {
                date: scheduled.game.date,
                start: scheduled.game.start_time,
                kind: EventKind::Game(scheduled),
            })
            .collect();
        past_events.sort_by(|a, b| b.date.cmp(&a.date));
        let events = SeasonEvents {
            coming: Vec::new(),
            past: past_events,
        };
        let places: PlacesOf = [
            (1, driver_places(9, 3, true)),
            (2, home_places(true, true, true, true, true)),
            (3, driver_places(9, 3, true)),
        ]
        .into_iter()
        .collect();
        let ids: Vec<i64> = last_games(&events, &places)
            .iter()
            .filter_map(|event| match event.kind {
                EventKind::Game(scheduled) => Some(scheduled.game.source_id),
                EventKind::Training(..) => None,
            })
            .collect();
        assert_eq!(ids, [2]);
    }

    // The warnings of the games (story B of attendance at games).

    fn short(coming: usize) -> GameCount {
        GameCount {
            coming,
            not_coming: 0,
            no_answer: 7 - coming,
        }
    }

    fn warned(too_few: Option<usize>, drivers: &[i64], moved: bool) -> GameWarnings {
        GameWarnings {
            too_few: too_few.map(short),
            too_few_line: too_few.is_some(),
            drivers_not_coming: drivers.iter().copied().map(PlayerId).collect(),
            moved,
        }
    }

    #[test]
    fn the_warnings_come_after_the_changes_and_before_the_schedule() {
        let not_checked = training(1, date(2026, 10, 6));
        let list = past(vec![
            row(date(2026, 10, 6), Some(&not_checked)),
            row(date(2026, 9, 29), None),
        ]);
        let reading = ReadingState {
            held_back: true,
            out_of_date_since: Some(date(2026, 10, 8)),
        };
        let open = vec![OpenGame {
            id: 8,
            open: BTreeMap::from([(DutyKind::Table, 1)]),
        }];
        let warnings = [(5, warned(Some(5), &[3], true))];
        assert_eq!(
            open_items(
                &reading,
                4,
                vec![7],
                open.clone(),
                &[list],
                &warnings,
                NotCheckedGames {
                    games: &[],
                    duty_games: &[]
                }
            ),
            [
                OpenItem::ReadingHeldBack,
                OpenItem::OutOfDate {
                    since: date(2026, 10, 8)
                },
                OpenItem::ChangesNotSeen { count: 4 },
                OpenItem::TooFewPlayers {
                    games: vec![(5, short(5))]
                },
                OpenItem::DriverNotComing {
                    games: vec![(5, vec![PlayerId(3)])]
                },
                OpenItem::AnswersMoved { games: vec![5] },
                OpenItem::ScheduleNotEntered { games: vec![7] },
                OpenItem::OpenPlaces { games: open },
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
    fn each_warning_keeps_the_order_of_its_games_and_is_left_out_without_games() {
        let warnings = [
            (3, warned(Some(6), &[], false)),
            (1, warned(None, &[4, 2], false)),
            (2, warned(Some(4), &[9], false)),
            (4, GameWarnings::default()),
        ];
        assert_eq!(
            open_items(
                &ReadingState::default(),
                0,
                Vec::new(),
                Vec::new(),
                &[],
                &warnings,
                NotCheckedGames {
                    games: &[],
                    duty_games: &[]
                }
            ),
            [
                OpenItem::TooFewPlayers {
                    games: vec![(3, short(6)), (2, short(4))]
                },
                OpenItem::DriverNotComing {
                    games: vec![(1, vec![PlayerId(4), PlayerId(2)]), (2, vec![PlayerId(9)])]
                },
            ]
        );
        // The line on the page alone is not an open item.
        let line_only = [(
            1,
            GameWarnings {
                too_few_line: true,
                ..GameWarnings::default()
            },
        )];
        assert_eq!(
            open_items(
                &ReadingState::default(),
                0,
                Vec::new(),
                Vec::new(),
                &[],
                &line_only,
                NotCheckedGames {
                    games: &[],
                    duty_games: &[]
                }
            ),
            []
        );
    }

    // A game that was not played (story E of attendance at games).

    /// A home game of the club with table 1 of the team, open and not entered, marked "Not
    /// played".
    fn not_played_places(past: bool) -> GamePlaces {
        use crate::contribution::duties::{
            PlaceKind, RequiredDuties, StoredDuties, TeamRef, game_places,
        };
        let mut stored = StoredDuties::default();
        stored.teams.insert(
            (PlaceKind::Table, 1),
            TeamRef {
                number: TEAM,
                name: "M16 1".to_owned(),
            },
        );
        let required = RequiredDuties {
            schedule: true,
            drivers: false,
        };
        game_places(
            required,
            Some(&stored),
            TEAM,
            &Default::default(),
            10,
            GameMoment {
                not_played: true,
                ..GameMoment::played(true, past)
            },
        )
        .expect("places")
    }

    #[test]
    fn a_not_played_game_is_no_open_item_but_stays_in_next() {
        let mut marked = scheduled(1, date(2026, 10, 31), ScheduleReason::DutyGame);
        marked.not_played = true;
        let games = [marked];
        let events = coming(&games, &[]);
        let places: PlacesOf = [(1, not_played_places(false))].into_iter().collect();
        let today = date(2026, 10, 26);
        assert!(schedule_not_entered(&events, &places, today).is_empty());
        assert!(open_places_games(&events, &places, today).is_empty());
        assert_eq!(
            dates(&next_events(&events, &places, today)),
            [date(2026, 10, 31)]
        );
    }

    #[test]
    fn a_not_played_game_stays_in_last() {
        let mut marked = scheduled(1, date(2026, 10, 10), ScheduleReason::DutyGame);
        marked.not_played = true;
        let games = [marked];
        let events = SeasonEvents {
            coming: Vec::new(),
            past: games
                .iter()
                .map(|scheduled| Event {
                    date: scheduled.game.date,
                    start: scheduled.game.start_time,
                    kind: EventKind::Game(scheduled),
                })
                .collect(),
        };
        let places: PlacesOf = [(1, not_played_places(true))].into_iter().collect();
        assert_eq!(last_games(&events, &places).len(), 1);
    }

    // The games not checked (story C of attendance at games).

    fn checked_on(on: NaiveDate) -> GameAttendance {
        GameAttendance {
            extra_players: 0,
            answers_for: None,
            checked_for: Some(on),
        }
    }

    #[test]
    fn a_past_game_of_the_team_that_is_not_checked_is_open_from_the_start_of_the_season() {
        let season = Season::from_start_year(2026);
        let from = date(2026, 10, 3);
        let today = date(2026, 10, 20);
        let mut gone = scheduled(5, date(2026, 10, 10), ScheduleReason::TeamGame);
        gone.in_list = false;
        let mut cancelled = scheduled(6, date(2026, 10, 10), ScheduleReason::TeamGame);
        cancelled.not_played = true;
        let games = [
            // Newest first, as the schedule gives the past games.
            scheduled(9, today, ScheduleReason::TeamGame),
            scheduled(8, date(2026, 10, 17), ScheduleReason::TeamGame),
            scheduled(7, date(2026, 10, 17), ScheduleReason::DutyGame),
            scheduled(4, date(2026, 10, 17), ScheduleReason::Kept),
            gone,
            cancelled,
            scheduled(3, date(2026, 10, 10), ScheduleReason::TeamGame),
            scheduled(2, from, ScheduleReason::TeamGame),
            scheduled(1, date(2026, 10, 2), ScheduleReason::TeamGame),
        ];
        let checks = BTreeMap::from([
            (3, checked_on(date(2026, 10, 10))),
            // Checked on another date: the check has no effect.
            (8, checked_on(date(2026, 10, 16))),
        ]);
        let open = games_not_checked(season, &games, &checks, today);
        assert_eq!(
            open,
            [
                // Also a game of before 4 November 2026: no release day any more.
                NotCheckedGame {
                    season,
                    id: 1,
                    date: date(2026, 10, 2)
                },
                NotCheckedGame {
                    season,
                    id: 2,
                    date: from
                },
                NotCheckedGame {
                    season,
                    id: 8,
                    date: date(2026, 10, 17)
                },
            ]
        );
    }

    #[test]
    fn the_games_not_checked_come_last_with_the_oldest_first() {
        let not_checked = training(1, date(2026, 10, 6));
        let list = past(vec![row(date(2026, 10, 6), Some(&not_checked))]);
        let older = Season::from_start_year(2025);
        let current = Season::from_start_year(2026);
        let games = [
            NotCheckedGame {
                season: current,
                id: 3,
                date: date(2026, 7, 4),
            },
            NotCheckedGame {
                season: older,
                id: 8,
                date: date(2026, 6, 27),
            },
        ];
        assert_eq!(
            open_items(
                &ReadingState::default(),
                0,
                Vec::new(),
                Vec::new(),
                &[list],
                &[],
                NotCheckedGames {
                    games: &games,
                    duty_games: &[]
                }
            ),
            [
                OpenItem::NotChecked {
                    first: date(2026, 10, 6),
                    count: 1
                },
                OpenItem::GamesNotChecked {
                    games: vec![games[1], games[0]]
                },
            ]
        );
    }

    #[test]
    fn a_past_duty_game_that_is_not_checked_is_open() {
        let season = Season::from_start_year(2026);
        let today = date(2026, 10, 20);
        let on = date(2026, 10, 10);
        let mut gone = scheduled(5, on, ScheduleReason::DutyGame);
        gone.in_list = false;
        let mut not_played = scheduled(6, on, ScheduleReason::DutyGame);
        not_played.not_played = true;
        let games = [
            scheduled(9, today, ScheduleReason::DutyGame),
            scheduled(1, on, ScheduleReason::DutyGame),
            scheduled(2, on, ScheduleReason::DutyGame),
            scheduled(3, date(2026, 10, 3), ScheduleReason::DutyGame),
            scheduled(4, on, ScheduleReason::DutyGame),
            gone,
            not_played,
            scheduled(7, on, ScheduleReason::TeamGame),
        ];
        let with_place = || home_places(true, true, true, true, true);
        let places: PlacesOf = [
            (9, with_place()),
            (1, with_place()),
            (2, with_place()),
            (3, with_place()),
            // No place of the team: no duty game.
            (4, home_places(true, false, true, true, true)),
            (5, with_place()),
            (6, with_place()),
            (7, with_place()),
        ]
        .into_iter()
        .collect();
        let checked_for = |on| StoredDuties {
            checked_for: Some(on),
            ..StoredDuties::default()
        };
        let stored = BTreeMap::from([
            (2, checked_for(on)),
            // Checked for another date: the game moved, it asks again.
            (3, checked_for(date(2026, 10, 4))),
        ]);
        let ids: Vec<i64> = duty_games_not_checked(season, &games, &places, &stored, today)
            .iter()
            .map(|game| game.id)
            .collect();
        // Today's game (9) is not past; 2 is checked; 4 has no place of the team; 5 is gone; 6 is
        // not played; 7 is a game of the team. Oldest first.
        assert_eq!(ids, [3, 1]);
    }

    #[test]
    fn the_duty_games_not_checked_come_after_the_games_not_checked() {
        let season = Season::from_start_year(2026);
        let game = NotCheckedGame {
            season,
            id: 3,
            date: date(2026, 10, 3),
        };
        let duty = NotCheckedGame {
            season,
            id: 4,
            date: date(2026, 10, 3),
        };
        assert_eq!(
            open_items(
                &ReadingState::default(),
                0,
                Vec::new(),
                Vec::new(),
                &[],
                &[],
                NotCheckedGames {
                    games: &[game],
                    duty_games: &[duty]
                }
            ),
            [
                OpenItem::GamesNotChecked { games: vec![game] },
                OpenItem::DutyGamesNotChecked { games: vec![duty] },
            ]
        );
    }

    // How an event looks (the quieter start page, story 2).

    /// The places of a home game of the club with table 1 for the team and a player on it;
    /// `entered`: the schedule of the committee is entered.
    fn filled_home_places(entered: bool) -> GamePlaces {
        use crate::contribution::duties::{
            PlaceKind, RequiredDuties, StoredDuties, TeamRef, game_places,
        };
        let mut stored = StoredDuties {
            schedule_entered: entered,
            ..StoredDuties::default()
        };
        stored.teams.insert(
            (PlaceKind::Table, 1),
            TeamRef {
                number: TEAM,
                name: "M16 1".to_owned(),
            },
        );
        let required = RequiredDuties {
            schedule: true,
            drivers: false,
        };
        let filled = [(DutyKind::Table, 1)].into_iter().collect();
        game_places(
            required,
            Some(&stored),
            TEAM,
            &filled,
            10,
            GameMoment::played(true, false),
        )
        .expect("places")
    }

    fn game_look(
        scheduled: &ScheduledGame,
        places: Option<&GamePlaces>,
        checked: bool,
    ) -> EventLook {
        let event = Event {
            date: scheduled.game.date,
            start: scheduled.game.start_time,
            kind: EventKind::Game(scheduled),
        };
        event_look(&event, places, checked, scheduled.game.date)
    }

    fn training_look(row: &SeasonRow<'_>, state: TrainingState) -> EventLook {
        let event = Event {
            date: row.date,
            start: Some(row.start),
            kind: EventKind::Training(row, state),
        };
        event_look(&event, None, false, row.date)
    }

    #[test]
    fn a_checked_or_cancelled_training_is_lighter() {
        let on = date(2026, 11, 10);
        let mut checked = training(1, on);
        checked.checked = true;
        assert_eq!(
            training_look(&row(on, Some(&checked)), TrainingState::Absent(7)),
            EventLook::Lighter
        );
        let mut cancelled = training(2, on);
        cancelled.cancelled = true;
        assert_eq!(
            training_look(&row(on, Some(&cancelled)), TrainingState::Cancelled),
            EventLook::Lighter
        );
    }

    #[test]
    fn a_coming_training_and_a_training_day_are_normal() {
        let on = date(2026, 11, 17);
        let opened = training(1, on);
        assert_eq!(
            training_look(&row(on, Some(&opened)), TrainingState::AllThere),
            EventLook::Normal
        );
        assert_eq!(
            training_look(&row(on, None), TrainingState::NotOpened),
            EventLook::Normal
        );
    }

    #[test]
    fn a_game_of_the_team_is_lighter_only_when_checked_or_not_played() {
        let on = date(2026, 11, 21);
        let game = scheduled(1, on, ScheduleReason::TeamGame);
        // All driver places filled is not enough: the game is still to come.
        let drivers = driver_places(9, 3, true);
        assert_eq!(open_places(&drivers), 0);
        assert_eq!(game_look(&game, Some(&drivers), false), EventLook::Normal);
        assert_eq!(game_look(&game, Some(&drivers), true), EventLook::Lighter);
        let mut not_played = game.clone();
        not_played.not_played = true;
        assert_eq!(game_look(&not_played, None, false), EventLook::Lighter);
    }

    #[test]
    fn a_duty_game_is_lighter_when_filled_and_entered_or_not_played() {
        let on = date(2026, 11, 14);
        let game = scheduled(2, on, ScheduleReason::DutyGame);
        assert_eq!(
            game_look(&game, Some(&filled_home_places(true)), false),
            EventLook::Lighter
        );
        let mut not_played = game.clone();
        not_played.not_played = true;
        assert_eq!(
            game_look(
                &not_played,
                Some(&home_places(true, true, true, true, false)),
                false
            ),
            EventLook::Lighter
        );
    }

    #[test]
    fn a_duty_game_with_something_left_to_do_is_normal() {
        let on = date(2026, 11, 14);
        let game = scheduled(2, on, ScheduleReason::DutyGame);
        // An open place of the team.
        assert_eq!(
            game_look(
                &game,
                Some(&home_places(true, true, true, true, false)),
                false
            ),
            EventLook::Normal
        );
        // Filled, but the schedule of the committee is not entered.
        assert_eq!(
            game_look(&game, Some(&filled_home_places(false)), false),
            EventLook::Normal
        );
        // No longer in the game list: crossed out, not lighter.
        let mut gone = game.clone();
        gone.in_list = false;
        assert_eq!(
            game_look(&gone, Some(&filled_home_places(true)), false),
            EventLook::Normal
        );
    }

    #[test]
    fn a_past_duty_game_is_lighter_also_with_an_open_place_or_no_schedule() {
        let on = date(2026, 11, 14);
        let game = scheduled(2, on, ScheduleReason::DutyGame);
        let event = Event {
            date: on,
            start: game.game.start_time,
            kind: EventKind::Game(&game),
        };
        let monday = date(2026, 11, 16);
        let open = home_places(true, true, true, true, true);
        assert_eq!(open_places(&open), 1);
        assert_eq!(
            event_look(&event, Some(&open), false, monday),
            EventLook::Lighter
        );
        let not_entered = filled_home_places(false);
        assert_eq!(
            event_look(&event, Some(&not_entered), false, monday),
            EventLook::Lighter
        );
        // On its own date it still needs the coordinator.
        assert_eq!(
            event_look(&event, Some(&open), false, on),
            EventLook::Normal
        );
    }

    #[test]
    fn a_game_that_left_the_game_list_is_lighter_when_not_played_or_checked() {
        let on = date(2026, 11, 14);
        let mut duty = scheduled(2, on, ScheduleReason::DutyGame);
        duty.in_list = false;
        duty.not_played = true;
        assert_eq!(game_look(&duty, None, false), EventLook::Lighter);
        let mut team = scheduled(1, on, ScheduleReason::TeamGame);
        team.in_list = false;
        assert_eq!(game_look(&team, None, true), EventLook::Lighter);
        assert_eq!(game_look(&team, None, false), EventLook::Normal);
    }
}
