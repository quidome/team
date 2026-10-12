//! The comparison of the stored games with a new reading of the game list. The result line of a
//! reading and the list of changes (story 4) both come from it.

use std::collections::{BTreeMap, BTreeSet};

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
/// each game, sorted by source id.
pub fn compare(
    stored: &[StoredGame],
    read: &[Game],
    club: ClubNumber,
    team: Option<TeamNumber>,
) -> Vec<GameChange> {
    let schedule = |game: &Game| in_schedule(game, club, team);
    let stored_by_id: BTreeMap<i64, &StoredGame> = stored
        .iter()
        .map(|stored| (stored.game.source_id, stored))
        .collect();
    let read_ids: BTreeSet<i64> = read.iter().map(|game| game.source_id).collect();

    let mut changes = Vec::new();
    for after in read {
        let change = match stored_by_id.get(&after.source_id) {
            None => schedule(after).map(|_| Change::New),
            Some(stored) if !stored.in_list => schedule(after).map(|_| Change::CameBack),
            Some(stored) => match (schedule(&stored.game), schedule(after)) {
                (Some(_), Some(_)) => {
                    let differences = differences(&stored.game, after);
                    (!differences.is_empty()).then_some(Change::Changed(differences))
                }
                (None, Some(reason)) => Some(Change::IntoSchedule(reason)),
                (Some(_), None) => Some(Change::OutOfSchedule),
                (None, None) => None,
            },
        };
        if let Some(change) = change {
            changes.push(GameChange {
                source_id: after.source_id,
                change,
                before: stored_by_id.get(&after.source_id).map(|s| s.game.clone()),
                after: Some(after.clone()),
            });
        }
    }
    for stored in stored {
        let gone = stored.in_list
            && !read_ids.contains(&stored.game.source_id)
            && schedule(&stored.game).is_some();
        if gone {
            changes.push(GameChange {
                source_id: stored.game.source_id,
                change: Change::Gone,
                before: Some(stored.game.clone()),
                after: None,
            });
        }
    }
    changes.sort_by_key(|change| change.source_id);
    changes
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

/// Counts each change once. A `Changed` counts by its first difference in this order: date, start
/// time, place ("moved"), start time known, home and away, team.
pub fn count(changes: &[GameChange]) -> Counts {
    let mut counts = Counts::default();
    for change in changes {
        let counter = match &change.change {
            Change::New => &mut counts.new,
            Change::Gone => &mut counts.gone,
            Change::CameBack => &mut counts.came_back,
            Change::IntoSchedule(_) => &mut counts.into_schedule,
            Change::OutOfSchedule => &mut counts.out_of_schedule,
            Change::Changed(differences) => {
                let has = |difference| differences.contains(&difference);
                if has(Difference::Date) || has(Difference::StartTime) || has(Difference::Place) {
                    &mut counts.moved
                } else if has(Difference::StartTimeKnown) {
                    &mut counts.start_time_known
                } else if has(Difference::HomeAndAway) {
                    &mut counts.home_and_away
                } else if has(Difference::Team) {
                    &mut counts.team_changed
                } else {
                    continue;
                }
            }
        };
        *counter += 1;
    }
    counts
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
        compare(stored, read, CLUB, Some(TEAM))
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
        let changes = compare(&stored(&[team_game()]), &[not_known], CLUB, Some(TEAM));
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
        let changes = compare(&stored(&[team_game()]), &[moved], CLUB, Some(TEAM));
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
        let changes = compare(&stored(&[team_game()]), &[moved.clone()], CLUB, Some(TEAM));
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
            count(&compare(&before, &read, CLUB, Some(TEAM))),
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
    fn a_gone_and_a_new_game_with_the_same_number_are_a_pair() {
        let mut renamed = game(2, TEAM, OPPONENT);
        renamed.game_number = team_game().game_number;
        let other_new = game(3, TEAM, OPPONENT);
        let changes = compare(
            &stored(&[team_game()]),
            &[renamed, other_new],
            CLUB,
            Some(TEAM),
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
        );
        assert_eq!(same_number_pairs(&changes), []);
    }
}
