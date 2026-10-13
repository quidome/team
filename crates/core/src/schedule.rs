//! Which games of the club's game list go in the coordinator's schedule.

use std::cmp::Ordering;
use std::collections::BTreeSet;

use chrono::NaiveDate;

use crate::club::ClubNumber;
use crate::comparison::StoredGame;
use crate::game::{Game, TeamNumber};

/// Why a game is in the schedule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleReason {
    /// The chosen team plays (home or away).
    TeamGame,
    /// A home game of another club team: a possible duty game.
    DutyGame,
}

/// `TeamGame`: the chosen team plays (home or away). `DutyGame`: the home club is the chosen club
/// and the chosen team does not play. `None`: an away game of another club team, or a game of
/// other clubs. Without a team, every home game of the club is a `DutyGame`.
pub fn in_schedule(
    game: &Game,
    club: ClubNumber,
    team: Option<TeamNumber>,
) -> Option<ScheduleReason> {
    if team.is_some_and(|team| game.home.team == team || game.away.team == team) {
        Some(ScheduleReason::TeamGame)
    } else if game.home.club == club {
        Some(ScheduleReason::DutyGame)
    } else {
        None
    }
}

/// A game in the schedule, with why it is there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledGame {
    pub game: Game,
    pub reason: ScheduleReason,
    /// `false`: the game is no longer in the game list (shown crossed out).
    pub in_list: bool,
}

/// The schedule of one team, split on today's date.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Schedule {
    /// Today and later, in time order.
    pub coming: Vec<ScheduledGame>,
    /// Before today, newest date first, in time order within a date.
    pub past: Vec<ScheduledGame>,
}

/// Keeps the stored games that [`in_schedule`] accepts and splits them on `today` (a game today
/// is coming). A game no longer in the game list is kept only while `open` (the games with a
/// change not seen yet) has it: after "seen" it is hidden. Within a date, known start times come first in time order, then the games whose
/// start time is not known, then by source id.
pub fn schedule(
    stored: Vec<StoredGame>,
    club: ClubNumber,
    team: TeamNumber,
    today: NaiveDate,
    open: &BTreeSet<i64>,
) -> Schedule {
    let mut schedule = Schedule::default();
    for StoredGame { game, in_list } in stored {
        if !in_list && !open.contains(&game.source_id) {
            continue;
        }
        let Some(reason) = in_schedule(&game, club, Some(team)) else {
            continue;
        };
        let scheduled = ScheduledGame {
            game,
            reason,
            in_list,
        };
        if scheduled.game.date >= today {
            schedule.coming.push(scheduled);
        } else {
            schedule.past.push(scheduled);
        }
    }
    schedule.coming.sort_by(|a, b| {
        a.game
            .date
            .cmp(&b.game.date)
            .then_with(|| within_date(a, b))
    });
    schedule.past.sort_by(|a, b| {
        b.game
            .date
            .cmp(&a.game.date)
            .then_with(|| within_date(a, b))
    });
    schedule
}

/// The order of two games on the same date.
fn within_date(a: &ScheduledGame, b: &ScheduledGame) -> Ordering {
    let key = |game: &Game| (game.start_time.is_none(), game.start_time, game.source_id);
    key(&a.game).cmp(&key(&b.game))
}

#[cfg(test)]
pub(crate) mod tests {
    use chrono::{NaiveDate, NaiveTime};

    use super::*;
    use crate::game::{Place, PlaceNumber, Side};
    use crate::season::Season;

    pub(crate) const CLUB: ClubNumber = ClubNumber(37);
    pub(crate) const OTHER_CLUB: ClubNumber = ClubNumber(99);
    pub(crate) const TEAM: TeamNumber = TeamNumber(184);
    pub(crate) const CLUB_TEAM: TeamNumber = TeamNumber(185);
    pub(crate) const OPPONENT: TeamNumber = TeamNumber(900);

    fn side(team: TeamNumber) -> Side {
        let club = if team == OPPONENT || team.0 > 900 {
            OTHER_CLUB
        } else {
            CLUB
        };
        Side {
            team,
            team_name: format!("Team {}", team.0),
            club,
        }
    }

    /// A game on 1 November 2026 at 10:00 in place 1, court A.
    pub(crate) fn game(source_id: i64, home: TeamNumber, away: TeamNumber) -> Game {
        let date = NaiveDate::from_ymd_opt(2026, 11, 1).expect("valid date");
        Game {
            source_id,
            game_number: format!("N{source_id}"),
            season: Season::of(date),
            date,
            start_time: NaiveTime::from_hms_opt(10, 0, 0),
            home: side(home),
            away: side(away),
            place: Place {
                number: PlaceNumber(1),
                hall: "Hall".to_owned(),
                town: "Town".to_owned(),
                court: Some("A".to_owned()),
            },
            competition: "Competition".to_owned(),
        }
    }

    #[test]
    fn the_team_at_home_and_away_plays_a_team_game() {
        let reason = Some(ScheduleReason::TeamGame);
        assert_eq!(
            in_schedule(&game(1, TEAM, OPPONENT), CLUB, Some(TEAM)),
            reason
        );
        assert_eq!(
            in_schedule(&game(2, OPPONENT, TEAM), CLUB, Some(TEAM)),
            reason
        );
    }

    #[test]
    fn a_home_game_of_another_club_team_is_a_duty_game() {
        assert_eq!(
            in_schedule(&game(1, CLUB_TEAM, OPPONENT), CLUB, Some(TEAM)),
            Some(ScheduleReason::DutyGame)
        );
    }

    #[test]
    fn an_away_game_of_another_club_team_is_not_in_the_schedule() {
        assert_eq!(
            in_schedule(&game(1, OPPONENT, CLUB_TEAM), CLUB, Some(TEAM)),
            None
        );
    }

    #[test]
    fn a_game_of_two_other_clubs_is_not_in_the_schedule() {
        assert_eq!(
            in_schedule(&game(1, OPPONENT, TeamNumber(901)), CLUB, Some(TEAM)),
            None
        );
    }

    #[test]
    fn without_a_team_every_club_home_game_is_a_duty_game() {
        let duty = Some(ScheduleReason::DutyGame);
        assert_eq!(in_schedule(&game(1, TEAM, OPPONENT), CLUB, None), duty);
        assert_eq!(in_schedule(&game(2, CLUB_TEAM, OPPONENT), CLUB, None), duty);
        assert_eq!(in_schedule(&game(3, OPPONENT, TEAM), CLUB, None), None);
    }

    fn day(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, day).expect("valid date")
    }

    /// A stored team game on `day` November 2026 at `time` ("HH:MM"; `None` = not known).
    fn stored(source_id: i64, date: u32, time: Option<(u32, u32)>) -> StoredGame {
        let mut game = game(source_id, TEAM, OPPONENT);
        game.date = day(date);
        game.start_time = time.and_then(|(hour, minute)| NaiveTime::from_hms_opt(hour, minute, 0));
        StoredGame {
            game,
            in_list: true,
        }
    }

    fn ids(games: &[ScheduledGame]) -> Vec<i64> {
        games
            .iter()
            .map(|scheduled| scheduled.game.source_id)
            .collect()
    }

    #[test]
    fn the_schedule_has_no_away_games_of_other_club_teams() {
        let games = vec![
            StoredGame {
                game: game(1, CLUB_TEAM, OPPONENT),
                in_list: true,
            },
            StoredGame {
                game: game(2, OPPONENT, CLUB_TEAM),
                in_list: true,
            },
            StoredGame {
                game: game(3, OPPONENT, TEAM),
                in_list: true,
            },
        ];
        let schedule = schedule(games, CLUB, TEAM, day(1), &BTreeSet::new());
        assert_eq!(ids(&schedule.coming), [1, 3]);
        assert_eq!(
            schedule
                .coming
                .iter()
                .map(|scheduled| scheduled.reason)
                .collect::<Vec<_>>(),
            [ScheduleReason::DutyGame, ScheduleReason::TeamGame]
        );
        assert!(schedule.past.is_empty());
    }

    #[test]
    fn a_game_today_is_coming_and_a_game_yesterday_is_past() {
        let games = vec![stored(1, 9, Some((10, 0))), stored(2, 10, Some((10, 0)))];
        let schedule = schedule(games, CLUB, TEAM, day(10), &BTreeSet::new());
        assert_eq!(ids(&schedule.coming), [2]);
        assert_eq!(ids(&schedule.past), [1]);
    }

    #[test]
    fn coming_games_are_in_time_order_with_unknown_times_last_in_their_date() {
        let games = vec![
            stored(1, 21, Some((9, 0))),
            stored(2, 14, None),
            stored(3, 14, Some((14, 0))),
            stored(4, 14, Some((10, 0))),
            stored(6, 14, Some((10, 0))),
            stored(5, 14, Some((10, 0))),
        ];
        let schedule = schedule(games, CLUB, TEAM, day(1), &BTreeSet::new());
        assert_eq!(ids(&schedule.coming), [4, 5, 6, 3, 2, 1]);
    }

    #[test]
    fn past_games_show_the_newest_date_first_in_time_order_within_a_date() {
        let games = vec![
            stored(1, 7, Some((9, 0))),
            stored(2, 14, None),
            stored(3, 14, Some((14, 0))),
            stored(4, 14, Some((10, 0))),
        ];
        let schedule = schedule(games, CLUB, TEAM, day(20), &BTreeSet::new());
        assert!(schedule.coming.is_empty());
        assert_eq!(ids(&schedule.past), [4, 3, 2, 1]);
    }

    #[test]
    fn a_game_no_longer_in_the_game_list_stays_while_its_change_is_not_seen() {
        let mut gone = stored(1, 14, Some((10, 0)));
        gone.in_list = false;
        let games = vec![gone, stored(2, 14, Some((11, 0)))];
        let schedule = schedule(games, CLUB, TEAM, day(1), &BTreeSet::from([1]));
        assert_eq!(
            schedule
                .coming
                .iter()
                .map(|scheduled| (scheduled.game.source_id, scheduled.in_list))
                .collect::<Vec<_>>(),
            [(1, false), (2, true)]
        );
    }

    #[test]
    fn a_game_no_longer_in_the_game_list_is_hidden_after_its_change_is_seen() {
        let mut gone = stored(1, 14, Some((10, 0)));
        gone.in_list = false;
        let games = vec![gone, stored(2, 14, Some((11, 0)))];
        let schedule = schedule(games, CLUB, TEAM, day(1), &BTreeSet::new());
        assert_eq!(ids(&schedule.coming), [2]);
    }
}
