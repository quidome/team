//! The comparison of the stored games with a new reading of the game list. The result line of a
//! reading and the list of changes (story 4) both come from it.

use std::collections::{BTreeMap, BTreeSet};

use chrono::NaiveDate;

use crate::club::ClubNumber;
use crate::game::{Game, TeamNumber};
use crate::schedule::{ScheduleReason, in_schedule};

/// A game as stored by the tool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredGame {
    pub game: Game,
    /// `false`: the game is no longer in the game list.
    pub in_list: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    New,
    Gone,
    CameBack,
    IntoSchedule(ScheduleReason),
    OutOfSchedule,
    /// Every difference, in the order of `Difference`.
    Changed(Vec<Difference>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Difference {
    Date,
    /// Another known start time, or a known start time that is no longer known.
    StartTime,
    /// The start time was not known and is known now.
    StartTimeKnown,
    /// Another place number or court.
    Place,
    /// The home team and the away team changed places.
    HomeAndAway,
    /// A team number on one side changed, and it is not a swap.
    Team,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameChange {
    pub source_id: i64,
    pub change: Change,
    /// The stored game, if any.
    pub before: Option<Game>,
    /// The read game, if any.
    pub after: Option<Game>,
}

/// The changes of the games that are in the schedule before or after the reading, one change for
/// each game, sorted by source id. Each change is [`change_between`] the stored game and the game
/// after the reading, so the result line and the list of changes use one rule.
pub fn compare(
    stored: &[StoredGame],
    read: &[Game],
    club: ClubNumber,
    team: Option<TeamNumber>,
    today: NaiveDate,
) -> Vec<GameChange> {
    let stored_by_id = by_id(stored);
    let mut changes = Vec::new();
    for after in read {
        let before = stored_by_id.get(&after.source_id).copied();
        let now = StoredGame {
            game: after.clone(),
            in_list: true,
        };
        if let Some(change) = change_between(before, &now, club, team, today) {
            changes.push(GameChange {
                source_id: after.source_id,
                change,
                before: before.map(|stored| stored.game.clone()),
                after: Some(after.clone()),
            });
        }
    }
    for stored in no_longer_in_list(stored, read) {
        let now = StoredGame {
            game: stored.game.clone(),
            in_list: false,
        };
        if let Some(change) = change_between(Some(stored), &now, club, team, today) {
            changes.push(GameChange {
                source_id: stored.game.source_id,
                change,
                before: Some(stored.game.clone()),
                after: None,
            });
        }
    }
    changes.sort_by_key(|change| change.source_id);
    changes
}

/// The change from the game as the coordinator last saw it to the game now; `None`: nothing to
/// show. `last_seen: None`: the game was not in the tool before. Only games of `today` or later
/// (as last seen or now) give a change: a change to a past game needs no action.
pub fn change_between(
    last_seen: Option<&StoredGame>,
    now: &StoredGame,
    club: ClubNumber,
    team: Option<TeamNumber>,
    today: NaiveDate,
) -> Option<Change> {
    let past = |stored: &StoredGame| stored.game.date < today;
    if past(now) && last_seen.is_none_or(past) {
        return None;
    }
    let visible = |stored: &StoredGame| {
        stored
            .in_list
            .then(|| in_schedule(&stored.game, club, team))
            .flatten()
    };
    let Some(last_seen) = last_seen else {
        return visible(now).map(|_| Change::New);
    };
    match (visible(last_seen), visible(now)) {
        (Some(_), Some(_)) => {
            let differences = differences(&last_seen.game, &now.game);
            (!differences.is_empty()).then_some(Change::Changed(differences))
        }
        (Some(_), None) if !now.in_list => Some(Change::Gone),
        (Some(_), None) => Some(Change::OutOfSchedule),
        // It left the game list while in the schedule.
        (None, Some(_))
            if !last_seen.in_list && in_schedule(&last_seen.game, club, team).is_some() =>
        {
            Some(Change::CameBack)
        }
        // It was outside the schedule, also when it left the game list from there.
        (None, Some(reason)) => Some(Change::IntoSchedule(reason)),
        (None, None) => None,
    }
}

/// A change the coordinator has not seen yet: the game as last seen (`None`: a new game).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenChange {
    pub source_id: i64,
    pub last_seen: Option<StoredGame>,
}

/// What a reading does with the open change of one game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeUpdate {
    /// A new open change; `last_seen` is the stored game before this reading.
    Insert {
        source_id: i64,
        last_seen: Option<Box<StoredGame>>,
    },
    /// The game changed in this reading and still differs from last seen: `last_seen` stays,
    /// the version goes up.
    Touch { source_id: i64 },
    /// Nothing to show any more: the game is as last seen again, or it is a past game.
    Remove { source_id: i64 },
}

/// The updates of the open changes after a reading, sorted by source id. For each game of the club
/// and season: last seen is the copy in its open change, or else the stored game before this
/// reading; now is the game after this reading.
pub fn update_open_changes(
    open: &[OpenChange],
    stored: &[StoredGame],
    read: &[Game],
    club: ClubNumber,
    team: TeamNumber,
    today: NaiveDate,
) -> Vec<ChangeUpdate> {
    let team = Some(team);
    let open_by_id: BTreeMap<i64, &OpenChange> =
        open.iter().map(|open| (open.source_id, open)).collect();
    let stored_by_id = by_id(stored);
    // After the reading: every stored game not read is no longer in the game list.
    let mut after: BTreeMap<i64, StoredGame> = stored
        .iter()
        .map(|stored| {
            let game = StoredGame {
                game: stored.game.clone(),
                in_list: false,
            };
            (stored.game.source_id, game)
        })
        .collect();
    for game in read {
        let now = StoredGame {
            game: game.clone(),
            in_list: true,
        };
        after.insert(game.source_id, now);
    }

    let mut updates = Vec::new();
    for (source_id, now) in &after {
        let source_id = *source_id;
        let before = stored_by_id.get(&source_id).copied();
        match open_by_id.get(&source_id) {
            Some(open) => {
                let change = change_between(open.last_seen.as_ref(), now, club, team, today);
                if change.is_none() {
                    updates.push(ChangeUpdate::Remove { source_id });
                } else if before != Some(now) {
                    updates.push(ChangeUpdate::Touch { source_id });
                }
            }
            None => {
                if change_between(before, now, club, team, today).is_some() {
                    updates.push(ChangeUpdate::Insert {
                        source_id,
                        last_seen: before.cloned().map(Box::new),
                    });
                }
            }
        }
    }
    updates
}

fn by_id(stored: &[StoredGame]) -> BTreeMap<i64, &StoredGame> {
    stored
        .iter()
        .map(|stored| (stored.game.source_id, stored))
        .collect()
}

/// The stored games that were in the game list and are not in this reading. All of them are
/// stored as no longer in the game list (the game list is the truth); `compare` counts only the
/// ones in the schedule as `Gone`.
pub fn no_longer_in_list<'a>(stored: &'a [StoredGame], read: &[Game]) -> Vec<&'a StoredGame> {
    let read_ids: BTreeSet<i64> = read.iter().map(|game| game.source_id).collect();
    stored
        .iter()
        .filter(|stored| stored.in_list && !read_ids.contains(&stored.game.source_id))
        .collect()
}

fn differences(before: &Game, after: &Game) -> Vec<Difference> {
    let mut differences = Vec::new();
    if before.date != after.date {
        differences.push(Difference::Date);
    }
    match (before.start_time, after.start_time) {
        (None, Some(_)) => differences.push(Difference::StartTimeKnown),
        (Some(_), None) => differences.push(Difference::StartTime),
        (Some(old), Some(new)) if old != new => differences.push(Difference::StartTime),
        _ => {}
    }
    if before.place.number != after.place.number || before.place.court != after.place.court {
        differences.push(Difference::Place);
    }
    let teams = |game: &Game| (game.home.team, game.away.team);
    let (home, away) = teams(before);
    if teams(after) == (away, home) && home != away {
        differences.push(Difference::HomeAndAway);
    } else if teams(after) != (home, away) {
        differences.push(Difference::Team);
    }
    differences.sort();
    differences
}

/// How many changes of each kind a reading found.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    pub new: u32,
    /// Another date, start time or place.
    pub moved: u32,
    pub start_time_known: u32,
    pub home_and_away: u32,
    pub team_changed: u32,
    pub gone: u32,
    pub came_back: u32,
    pub into_schedule: u32,
    pub out_of_schedule: u32,
}

/// The kind of a change, as the result line counts it and the list of changes names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    New,
    /// Another date, start time or place.
    Moved,
    StartTimeKnown,
    HomeAndAway,
    TeamChanged,
    Gone,
    CameBack,
    IntoSchedule,
    OutOfSchedule,
}

impl Change {
    /// A `Changed` is named by its first difference in this order: date, start time, place
    /// ("moved"), start time known, home and away, team. `None`: a `Changed` with no difference.
    pub fn kind(&self) -> Option<ChangeKind> {
        Some(match self {
            Change::New => ChangeKind::New,
            Change::Gone => ChangeKind::Gone,
            Change::CameBack => ChangeKind::CameBack,
            Change::IntoSchedule(_) => ChangeKind::IntoSchedule,
            Change::OutOfSchedule => ChangeKind::OutOfSchedule,
            Change::Changed(differences) => {
                let has = |difference| differences.contains(&difference);
                if has(Difference::Date) || has(Difference::StartTime) || has(Difference::Place) {
                    ChangeKind::Moved
                } else if has(Difference::StartTimeKnown) {
                    ChangeKind::StartTimeKnown
                } else if has(Difference::HomeAndAway) {
                    ChangeKind::HomeAndAway
                } else if has(Difference::Team) {
                    ChangeKind::TeamChanged
                } else {
                    return None;
                }
            }
        })
    }
}

/// Counts each change once, by its [`Change::kind`].
pub fn count(changes: &[GameChange]) -> Counts {
    let mut counts = Counts::default();
    for change in changes {
        let Some(kind) = change.change.kind() else {
            continue;
        };
        let counter = match kind {
            ChangeKind::New => &mut counts.new,
            ChangeKind::Moved => &mut counts.moved,
            ChangeKind::StartTimeKnown => &mut counts.start_time_known,
            ChangeKind::HomeAndAway => &mut counts.home_and_away,
            ChangeKind::TeamChanged => &mut counts.team_changed,
            ChangeKind::Gone => &mut counts.gone,
            ChangeKind::CameBack => &mut counts.came_back,
            ChangeKind::IntoSchedule => &mut counts.into_schedule,
            ChangeKind::OutOfSchedule => &mut counts.out_of_schedule,
        };
        *counter += 1;
    }
    counts
}

/// An open change as the list of changes shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListedChange {
    pub source_id: i64,
    pub change: Change,
    /// The reason the game is in the schedule now, or, when it is no longer there, was when last
    /// seen. It decides the filter and the team look.
    pub reason: ScheduleReason,
    /// `None`: a new game.
    pub last_seen: Option<StoredGame>,
    pub now: StoredGame,
}

/// The open changes to show: each one is [`change_between`] its last-seen game and the stored
/// game now; changes that give nothing (for example of past games) are left out. In the order of
/// the game's date and start time (now, or as last seen for a game no longer in the game list);
/// not known start times last within a date; then by source id.
pub fn list_changes(
    open: Vec<OpenChange>,
    stored: &[StoredGame],
    club: ClubNumber,
    team: TeamNumber,
    today: NaiveDate,
) -> Vec<ListedChange> {
    let team = Some(team);
    let stored_by_id = by_id(stored);
    let mut listed: Vec<ListedChange> = open
        .into_iter()
        .filter_map(|open| {
            let now = (*stored_by_id.get(&open.source_id)?).clone();
            let change = change_between(open.last_seen.as_ref(), &now, club, team, today)?;
            let reason = now
                .in_list
                .then(|| in_schedule(&now.game, club, team))
                .flatten()
                .or_else(|| {
                    let last_seen = open.last_seen.as_ref()?;
                    in_schedule(&last_seen.game, club, team)
                })?;
            Some(ListedChange {
                source_id: open.source_id,
                change,
                reason,
                last_seen: open.last_seen,
                now,
            })
        })
        .collect();
    let key = |listed: &ListedChange| {
        let game = match (&listed.last_seen, listed.now.in_list) {
            (Some(last_seen), false) => &last_seen.game,
            _ => &listed.now.game,
        };
        (
            game.date,
            game.start_time.is_none(),
            game.start_time,
            listed.source_id,
        )
    };
    listed.sort_by_key(key);
    listed
}

/// Pairs (gone source id, new source id) with the same game number: a sign that the source's id
/// changed for one game. The server logs them.
pub fn same_number_pairs(changes: &[GameChange]) -> Vec<(i64, i64)> {
    let numbers = |kind: Change| -> Vec<(&str, i64)> {
        changes
            .iter()
            .filter(|change| change.change == kind)
            .filter_map(|change| {
                let game = change.before.as_ref().or(change.after.as_ref())?;
                Some((game.game_number.as_str(), change.source_id))
            })
            .collect()
    };
    let new = numbers(Change::New);
    numbers(Change::Gone)
        .into_iter()
        .flat_map(|(number, gone_id)| {
            new.iter()
                .filter(move |(new_number, _)| *new_number == number)
                .map(move |(_, new_id)| (gone_id, *new_id))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, NaiveTime};

    use super::*;
    use crate::game::PlaceNumber;
    use crate::schedule::tests::{CLUB, CLUB_TEAM, OPPONENT, TEAM, game};

    /// The test games are on 1 November 2026.
    const TODAY: NaiveDate = NaiveDate::from_ymd_opt(2026, 10, 13).expect("valid date");

    fn stored(games: &[Game]) -> Vec<StoredGame> {
        games
            .iter()
            .map(|game| StoredGame {
                game: game.clone(),
                in_list: true,
            })
            .collect()
    }

    fn kinds(stored: &[StoredGame], read: &[Game]) -> Vec<(i64, Change)> {
        compare(stored, read, CLUB, Some(TEAM), TODAY)
            .into_iter()
            .map(|change| (change.source_id, change.change))
            .collect()
    }

    fn team_game() -> Game {
        game(1, TEAM, OPPONENT)
    }

    fn time(hour: u32) -> Option<NaiveTime> {
        NaiveTime::from_hms_opt(hour, 0, 0)
    }

    fn changed(differences: &[Difference]) -> Change {
        Change::Changed(differences.to_vec())
    }

    #[test]
    fn the_same_games_twice_give_no_changes() {
        let games = [team_game(), game(2, CLUB_TEAM, OPPONENT)];
        assert_eq!(kinds(&stored(&games), &games), []);
    }

    #[test]
    fn a_new_game_in_the_schedule_is_new() {
        assert_eq!(kinds(&[], &[team_game()]), [(1, Change::New)]);
    }

    #[test]
    fn a_new_away_game_of_another_club_team_gives_nothing() {
        assert_eq!(kinds(&[], &[game(1, OPPONENT, CLUB_TEAM)]), []);
    }

    #[test]
    fn a_new_date_start_time_or_place_is_a_move() {
        let mut date = team_game();
        date.date = NaiveDate::from_ymd_opt(2026, 11, 8).expect("valid date");
        let mut start = team_game();
        start.start_time = time(11);
        let mut place = team_game();
        place.place.number = PlaceNumber(2);
        let mut court = team_game();
        court.place.court = Some("B".to_owned());

        let before = stored(&[team_game()]);
        assert_eq!(kinds(&before, &[date]), [(1, changed(&[Difference::Date]))]);
        assert_eq!(
            kinds(&before, &[start]),
            [(1, changed(&[Difference::StartTime]))]
        );
        assert_eq!(
            kinds(&before, &[place]),
            [(1, changed(&[Difference::Place]))]
        );
        assert_eq!(
            kinds(&before, &[court]),
            [(1, changed(&[Difference::Place]))]
        );
    }

    #[test]
    fn a_start_time_that_becomes_known_is_start_time_known() {
        let mut not_known = team_game();
        not_known.start_time = None;
        assert_eq!(
            kinds(&stored(&[not_known]), &[team_game()]),
            [(1, changed(&[Difference::StartTimeKnown]))]
        );
    }

    #[test]
    fn a_start_time_that_is_no_longer_known_is_a_move() {
        let mut not_known = team_game();
        not_known.start_time = None;
        let changes = compare(
            &stored(&[team_game()]),
            &[not_known],
            CLUB,
            Some(TEAM),
            TODAY,
        );
        assert_eq!(changes[0].change, changed(&[Difference::StartTime]));
        assert_eq!(count(&changes).moved, 1);
    }

    #[test]
    fn home_and_away_that_change_places_are_a_swap() {
        assert_eq!(
            kinds(&stored(&[team_game()]), &[game(1, OPPONENT, TEAM)]),
            [(1, changed(&[Difference::HomeAndAway]))]
        );
    }

    #[test]
    fn another_team_on_one_side_is_a_team_change() {
        assert_eq!(
            kinds(&stored(&[team_game()]), &[game(1, TEAM, TeamNumber(901))]),
            [(1, changed(&[Difference::Team]))]
        );
    }

    #[test]
    fn a_missing_game_is_gone_and_a_game_read_again_came_back() {
        assert_eq!(kinds(&stored(&[team_game()]), &[]), [(1, Change::Gone)]);

        let gone = [StoredGame {
            game: team_game(),
            in_list: false,
        }];
        assert_eq!(kinds(&gone, &[team_game()]), [(1, Change::CameBack)]);
        assert_eq!(kinds(&gone, &[]), []);
    }

    #[test]
    fn a_game_that_came_back_changed_is_only_came_back() {
        let gone = [StoredGame {
            game: team_game(),
            in_list: false,
        }];
        let mut moved = team_game();
        moved.start_time = time(11);
        assert_eq!(kinds(&gone, &[moved]), [(1, Change::CameBack)]);
    }

    #[test]
    fn a_game_that_came_back_out_of_the_schedule_gives_nothing() {
        let gone = [StoredGame {
            game: game(1, CLUB_TEAM, OPPONENT),
            in_list: false,
        }];
        assert_eq!(kinds(&gone, &[game(1, OPPONENT, CLUB_TEAM)]), []);
    }

    #[test]
    fn an_away_game_that_becomes_a_home_game_comes_into_the_schedule() {
        let away = game(1, OPPONENT, CLUB_TEAM);
        let home = game(1, CLUB_TEAM, OPPONENT);
        assert_eq!(
            kinds(&stored(&[away.clone()]), &[home.clone()]),
            [(1, Change::IntoSchedule(ScheduleReason::DutyGame))]
        );
        assert_eq!(
            kinds(&stored(&[home]), &[away]),
            [(1, Change::OutOfSchedule)]
        );
    }

    #[test]
    fn changes_outside_the_schedule_give_nothing() {
        let away = game(1, OPPONENT, CLUB_TEAM);
        let mut moved = away.clone();
        moved.start_time = time(11);
        assert_eq!(kinds(&stored(&[away.clone()]), &[moved]), []);
        assert_eq!(kinds(&stored(&[away]), &[]), []);
    }

    #[test]
    fn a_new_date_and_place_count_once_as_moved() {
        let mut moved = team_game();
        moved.date = NaiveDate::from_ymd_opt(2026, 11, 8).expect("valid date");
        moved.place.number = PlaceNumber(2);
        let changes = compare(&stored(&[team_game()]), &[moved], CLUB, Some(TEAM), TODAY);
        assert_eq!(
            changes[0].change,
            changed(&[Difference::Date, Difference::Place])
        );
        assert_eq!(
            count(&changes),
            Counts {
                moved: 1,
                ..Counts::default()
            }
        );
    }

    #[test]
    fn a_change_keeps_the_stored_and_the_read_game() {
        let mut moved = team_game();
        moved.start_time = time(11);
        let changes = compare(
            &stored(&[team_game()]),
            &[moved.clone()],
            CLUB,
            Some(TEAM),
            TODAY,
        );
        assert_eq!(changes[0].before, Some(team_game()));
        assert_eq!(changes[0].after, Some(moved));
    }

    #[test]
    fn count_adds_up_each_kind() {
        let mut moved = game(2, TEAM, OPPONENT);
        moved.start_time = time(11);
        let mut not_known = game(3, TEAM, OPPONENT);
        not_known.start_time = None;
        let before = [
            StoredGame {
                game: game(1, CLUB_TEAM, OPPONENT),
                in_list: true,
            },
            StoredGame {
                game: game(2, TEAM, OPPONENT),
                in_list: true,
            },
            StoredGame {
                game: not_known,
                in_list: true,
            },
            StoredGame {
                game: game(4, TEAM, OPPONENT),
                in_list: true,
            },
            StoredGame {
                game: game(5, TEAM, OPPONENT),
                in_list: true,
            },
            StoredGame {
                game: game(6, OPPONENT, CLUB_TEAM),
                in_list: true,
            },
            StoredGame {
                game: game(7, CLUB_TEAM, OPPONENT),
                in_list: true,
            },
            StoredGame {
                game: game(8, TEAM, OPPONENT),
                in_list: false,
            },
        ];
        let read = [
            moved,
            game(3, TEAM, OPPONENT),
            game(4, OPPONENT, TEAM),
            game(5, TEAM, TeamNumber(901)),
            game(6, CLUB_TEAM, OPPONENT),
            game(7, OPPONENT, CLUB_TEAM),
            game(8, TEAM, OPPONENT),
            game(10, TEAM, OPPONENT),
        ];
        assert_eq!(
            count(&compare(&before, &read, CLUB, Some(TEAM), TODAY)),
            Counts {
                new: 1,
                moved: 1,
                start_time_known: 1,
                home_and_away: 1,
                team_changed: 1,
                gone: 1,
                came_back: 1,
                into_schedule: 1,
                out_of_schedule: 1,
            }
        );
    }

    #[test]
    fn every_missing_game_in_the_list_is_no_longer_in_the_list() {
        let before = [
            StoredGame {
                game: team_game(),
                in_list: true,
            },
            StoredGame {
                game: game(2, OPPONENT, CLUB_TEAM),
                in_list: true,
            },
            StoredGame {
                game: game(3, TEAM, OPPONENT),
                in_list: false,
            },
            StoredGame {
                game: game(4, TEAM, OPPONENT),
                in_list: true,
            },
        ];
        let read = [game(4, TEAM, OPPONENT)];
        let ids: Vec<i64> = no_longer_in_list(&before, &read)
            .iter()
            .map(|stored| stored.game.source_id)
            .collect();
        assert_eq!(ids, [1, 2]);
        // Only the one in the schedule counts.
        assert_eq!(kinds(&before, &read), [(1, Change::Gone)]);
    }

    #[test]
    fn a_gone_and_a_new_game_with_the_same_number_are_a_pair() {
        let mut renamed = game(2, TEAM, OPPONENT);
        renamed.game_number = team_game().game_number;
        let other_new = game(3, TEAM, OPPONENT);
        let changes = compare(
            &stored(&[team_game()]),
            &[renamed, other_new],
            CLUB,
            Some(TEAM),
            TODAY,
        );
        assert_eq!(same_number_pairs(&changes), [(1, 2)]);
    }

    #[test]
    fn no_pairs_without_the_same_number() {
        let changes = compare(
            &stored(&[team_game()]),
            &[game(2, TEAM, OPPONENT)],
            CLUB,
            Some(TEAM),
            TODAY,
        );
        assert_eq!(same_number_pairs(&changes), []);
    }

    fn day(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, day).expect("valid date")
    }

    fn on(mut game: Game, date: NaiveDate) -> Game {
        game.date = date;
        game
    }

    #[test]
    fn a_game_that_left_outside_the_schedule_and_comes_back_into_it_is_now_in_the_schedule() {
        let gone = [StoredGame {
            game: game(1, OPPONENT, CLUB_TEAM),
            in_list: false,
        }];
        assert_eq!(
            kinds(&gone, &[game(1, CLUB_TEAM, OPPONENT)]),
            [(1, Change::IntoSchedule(ScheduleReason::DutyGame))]
        );
    }

    #[test]
    fn a_change_to_a_past_game_gives_nothing() {
        let yesterday = on(team_game(), day(12));
        let mut moved = yesterday.clone();
        moved.start_time = time(11);
        assert_eq!(kinds(&stored(&[yesterday.clone()]), &[moved]), []);
        assert_eq!(kinds(&stored(&[yesterday]), &[]), []);
        let changes = compare(&[], &[on(team_game(), day(12))], CLUB, Some(TEAM), TODAY);
        assert_eq!(changes, []);
        assert_eq!(count(&changes), Counts::default());
    }

    #[test]
    fn a_game_moved_from_the_past_to_the_future_or_back_is_a_change() {
        let yesterday = on(team_game(), day(12));
        let next_week = on(team_game(), day(20));
        let date = [(1, changed(&[Difference::Date]))];
        assert_eq!(
            kinds(&stored(&[yesterday.clone()]), &[next_week.clone()]),
            date
        );
        assert_eq!(kinds(&stored(&[next_week]), &[yesterday]), date);
    }

    #[test]
    fn a_change_to_a_game_of_today_is_a_change() {
        assert_eq!(kinds(&[], &[on(team_game(), TODAY)]), [(1, Change::New)]);
    }

    fn open(source_id: i64, last_seen: Option<Game>) -> OpenChange {
        OpenChange {
            source_id,
            last_seen: last_seen.map(|game| StoredGame {
                game,
                in_list: true,
            }),
        }
    }

    fn updates(open: &[OpenChange], stored: &[StoredGame], read: &[Game]) -> Vec<ChangeUpdate> {
        update_open_changes(open, stored, read, CLUB, TEAM, TODAY)
    }

    fn at(hour: u32) -> Game {
        let mut game = team_game();
        game.start_time = time(hour);
        game
    }

    #[test]
    fn a_first_change_is_inserted_with_the_stored_game_as_last_seen() {
        assert_eq!(
            updates(&[], &stored(&[at(10)]), &[at(11)]),
            [ChangeUpdate::Insert {
                source_id: 1,
                last_seen: Some(Box::new(StoredGame {
                    game: at(10),
                    in_list: true,
                })),
            }]
        );
        assert_eq!(
            updates(&[], &[], &[at(10)]),
            [ChangeUpdate::Insert {
                source_id: 1,
                last_seen: None,
            }]
        );
    }

    #[test]
    fn a_game_that_moves_twice_keeps_its_first_old_value() {
        let open = [open(1, Some(at(10)))];
        assert_eq!(
            updates(&open, &stored(&[at(11)]), &[at(12)]),
            [ChangeUpdate::Touch { source_id: 1 }]
        );
    }

    #[test]
    fn a_game_that_moves_back_has_no_change() {
        let open = [open(1, Some(at(10)))];
        assert_eq!(
            updates(&open, &stored(&[at(11)]), &[at(10)]),
            [ChangeUpdate::Remove { source_id: 1 }]
        );
    }

    #[test]
    fn a_new_game_that_is_gone_before_it_is_seen_has_no_change() {
        let open = [open(1, None)];
        assert_eq!(
            updates(&open, &stored(&[at(10)]), &[]),
            [ChangeUpdate::Remove { source_id: 1 }]
        );
    }

    #[test]
    fn a_game_the_reading_does_not_change_keeps_its_change_and_version() {
        let open = [open(1, Some(at(10)))];
        assert_eq!(updates(&open, &stored(&[at(11)]), &[at(11)]), []);
        assert_eq!(updates(&[], &stored(&[at(11)]), &[at(11)]), []);
    }

    #[test]
    fn a_change_whose_game_is_now_past_on_both_dates_is_removed() {
        let open = [open(1, Some(on(team_game(), day(11))))];
        let yesterday = on(at(11), day(12));
        assert_eq!(
            updates(&open, &stored(&[yesterday.clone()]), &[yesterday]),
            [ChangeUpdate::Remove { source_id: 1 }]
        );
    }

    #[test]
    fn a_game_moved_from_next_week_to_yesterday_keeps_its_change() {
        let next_week = on(team_game(), day(20));
        let open = [open(1, Some(next_week.clone()))];
        let yesterday = on(team_game(), day(12));
        assert_eq!(
            updates(&open, &stored(&[next_week.clone()]), &[yesterday.clone()]),
            [ChangeUpdate::Touch { source_id: 1 }]
        );
        assert_eq!(
            updates(&[], &stored(&[next_week]), &[yesterday]),
            [ChangeUpdate::Insert {
                source_id: 1,
                last_seen: Some(Box::new(StoredGame {
                    game: on(team_game(), day(20)),
                    in_list: true,
                })),
            }]
        );
    }

    #[test]
    fn a_gone_game_that_was_seen_and_comes_back_came_back() {
        let seen_gone = StoredGame {
            game: team_game(),
            in_list: false,
        };
        assert_eq!(
            updates(&[], &[seen_gone.clone()], &[team_game()]),
            [ChangeUpdate::Insert {
                source_id: 1,
                last_seen: Some(Box::new(seen_gone.clone())),
            }]
        );
        assert_eq!(
            change_between(
                Some(&seen_gone),
                &StoredGame {
                    game: team_game(),
                    in_list: true
                },
                CLUB,
                Some(TEAM),
                TODAY
            ),
            Some(Change::CameBack)
        );
    }

    fn open_seen(game: Game) -> OpenChange {
        OpenChange {
            source_id: game.source_id,
            last_seen: Some(StoredGame {
                game,
                in_list: true,
            }),
        }
    }

    #[test]
    fn the_list_has_the_reason_now_or_as_last_seen() {
        // 1: a duty game that became an away game of another club team; 2: a team game gone.
        let open = vec![
            open_seen(game(1, CLUB_TEAM, OPPONENT)),
            open_seen(game(2, TEAM, OPPONENT)),
        ];
        let stored = [
            StoredGame {
                game: game(1, OPPONENT, CLUB_TEAM),
                in_list: true,
            },
            StoredGame {
                game: game(2, TEAM, OPPONENT),
                in_list: false,
            },
        ];
        let listed = list_changes(open, &stored, CLUB, TEAM, TODAY);
        assert_eq!(
            listed
                .iter()
                .map(|listed| (listed.source_id, listed.change.clone(), listed.reason))
                .collect::<Vec<_>>(),
            [
                (1, Change::OutOfSchedule, ScheduleReason::DutyGame),
                (2, Change::Gone, ScheduleReason::TeamGame)
            ]
        );
    }

    #[test]
    fn the_list_leaves_out_changes_that_give_nothing_and_is_in_date_order() {
        let mut later = at(11);
        later.source_id = 2;
        later.date = day(20);
        let mut not_known = team_game();
        not_known.source_id = 3;
        not_known.date = day(20);
        not_known.start_time = None;
        let mut past = team_game();
        past.source_id = 4;
        past.date = day(12);
        let open = vec![
            open(1, None),
            open(2, None),
            open(3, None),
            open(4, None),
            // 5: as last seen again.
            open_seen(game(5, TEAM, OPPONENT)),
        ];
        let stored = stored(&[team_game(), later, not_known, past, game(5, TEAM, OPPONENT)]);
        let ids: Vec<i64> = list_changes(open, &stored, CLUB, TEAM, TODAY)
            .iter()
            .map(|listed| listed.source_id)
            .collect();
        assert_eq!(ids, [2, 3, 1]);
    }

    #[test]
    fn a_change_is_named_by_its_first_difference() {
        let kind = |differences: &[Difference]| changed(differences).kind();
        assert_eq!(
            kind(&[Difference::StartTimeKnown, Difference::Place]),
            Some(ChangeKind::Moved)
        );
        assert_eq!(
            kind(&[Difference::StartTimeKnown, Difference::Team]),
            Some(ChangeKind::StartTimeKnown)
        );
        assert_eq!(kind(&[Difference::Team]), Some(ChangeKind::TeamChanged));
        assert_eq!(kind(&[]), None);
        assert_eq!(
            Change::IntoSchedule(ScheduleReason::DutyGame).kind(),
            Some(ChangeKind::IntoSchedule)
        );
    }
}
