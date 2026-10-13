//! The coordinator's choice of club and team, for each season.

use std::collections::BTreeMap;

use crate::club::Club;
use crate::game::{Game, TeamNumber};
use crate::season::Season;

/// A team of the chosen club, with the short name to show (for example "M16 1").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Team {
    pub number: TeamNumber,
    pub name: String,
}

/// The club and team chosen for one season. `team` is `None` while the season has no games.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FollowedTeam {
    pub season: Season,
    pub club: Club,
    pub team: Option<Team>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeamNeededReason {
    /// The club is saved for the current season, which had no games when it was saved.
    NoGamesYet,
    /// A new season started and nothing is saved for it yet: the club of the earlier season
    /// stays, the team must be chosen.
    NewSeason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChoiceState {
    NotChosen,
    TeamNeeded {
        club: Club,
        reason: TeamNeededReason,
        /// The choice of the newest earlier season, if any.
        last: Option<FollowedTeam>,
    },
    Chosen {
        season: Season,
        club: Club,
        team: Team,
    },
}

/// Whose schedule to show: a season, its club and team.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleTarget {
    pub season: Season,
    pub club: Club,
    pub team: Team,
    /// True: a team is needed for the current season; this is the newest earlier choice.
    pub last_season: bool,
}

/// The schedule to show for a state of the choice: the chosen team of the current season; while a
/// team is needed, the team of the newest earlier choice (functional story 1: last season's games
/// until the team is chosen again). `None` when nothing is chosen, or the newest earlier choice
/// has no team.
pub fn schedule_target(state: ChoiceState) -> Option<ScheduleTarget> {
    match state {
        ChoiceState::Chosen { season, club, team } => Some(ScheduleTarget {
            season,
            club,
            team,
            last_season: false,
        }),
        ChoiceState::TeamNeeded { last, .. } => last.and_then(|last| {
            Some(ScheduleTarget {
                season: last.season,
                club: last.club,
                team: last.team?,
                last_season: true,
            })
        }),
        ChoiceState::NotChosen => None,
    }
}

/// The teams of `club` in `games` (opponents left out), each team once, sorted by name.
pub fn own_teams(club: &Club, games: &[Game]) -> Vec<Team> {
    // For each team number, the full name of its latest game.
    let mut latest = BTreeMap::new();
    for game in games {
        let when = (game.date, game.start_time);
        for side in [&game.home, &game.away] {
            if side.club != club.number {
                continue;
            }
            latest
                .entry(side.team)
                .and_modify(|(seen, name)| {
                    if when >= *seen {
                        *seen = when;
                        *name = &side.team_name;
                    }
                })
                .or_insert((when, &side.team_name));
        }
    }
    let mut teams: Vec<Team> = latest
        .into_iter()
        .map(|(number, (_, name))| Team {
            number,
            name: short_name(&club.name, name),
        })
        .collect();
    teams.sort_by(|a, b| a.name.cmp(&b.name).then(a.number.cmp(&b.number)));
    teams
}

/// The team name without the club name in front ("Blue Drakes M16 1" gives "M16 1"). When
/// nothing is left, the full name stays.
fn short_name(club_name: &str, team_name: &str) -> String {
    // A separator must follow the club name, so "Blue Drakesters" stays whole.
    let rest = team_name
        .strip_prefix(club_name)
        .and_then(|rest| rest.strip_prefix(" - ").or_else(|| rest.strip_prefix(' ')))
        .map(str::trim);
    match rest {
        Some(rest) if !rest.is_empty() => rest.to_owned(),
        _ => team_name.to_owned(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TeamChoiceError {
    #[error("the team does not play for the club")]
    NotAnOwnTeam,
    #[error("a team must be chosen while the club has teams")]
    TeamRequired,
}

/// The team to save from the club's own teams. A team may be left out only while the club has no
/// teams in the season (no games yet).
pub fn choose_team(
    own_teams: Vec<Team>,
    team: Option<TeamNumber>,
) -> Result<Option<Team>, TeamChoiceError> {
    match team {
        Some(number) => own_teams
            .into_iter()
            .find(|team| team.number == number)
            .map(Some)
            .ok_or(TeamChoiceError::NotAnOwnTeam),
        None if own_teams.is_empty() => Ok(None),
        None => Err(TeamChoiceError::TeamRequired),
    }
}

/// The state of the choice, from the choice saved for the current season (`this_season`) and the
/// choice of the newest earlier season (`earlier`). Choices of later seasons (a clock that went
/// back) are not given here, so they cannot hide the current season.
///
/// `last` is the earlier choice whenever no team is chosen for the current season, also after
/// the club was saved for an empty new season: the link to last season's games stays.
pub fn choice_state(
    this_season: Option<FollowedTeam>,
    earlier: Option<FollowedTeam>,
) -> ChoiceState {
    match (this_season, earlier) {
        (
            Some(FollowedTeam {
                season,
                club,
                team: Some(team),
            }),
            _,
        ) => ChoiceState::Chosen { season, club, team },
        (
            Some(FollowedTeam {
                club, team: None, ..
            }),
            earlier,
        ) => ChoiceState::TeamNeeded {
            club,
            reason: TeamNeededReason::NoGamesYet,
            last: earlier,
        },
        (None, Some(earlier)) => ChoiceState::TeamNeeded {
            club: earlier.club.clone(),
            reason: TeamNeededReason::NewSeason,
            last: Some(earlier),
        },
        (None, None) => ChoiceState::NotChosen,
    }
}

/// Whether a new choice keeps the open changes: the season already had a choice with the same
/// club and the same team. Another club or team, or the first choice of a season, starts with no
/// changes.
pub fn same_choice(before: Option<&FollowedTeam>, now: &FollowedTeam) -> bool {
    let team = |choice: &FollowedTeam| choice.team.as_ref().map(|team| team.number);
    before.is_some_and(|before| {
        before.season == now.season
            && before.club.number == now.club.number
            && team(before) == team(now)
    })
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, NaiveTime};

    use super::*;
    use crate::club::ClubNumber;
    use crate::game::{Place, PlaceNumber, Side};

    const DRAKES: i64 = 37;
    const GIANTS: i64 = 21;

    fn club(number: i64, name: &str) -> Club {
        Club {
            number: ClubNumber(number),
            name: name.to_owned(),
            town: None,
        }
    }

    fn side(club: i64, team: i64, name: &str) -> Side {
        Side {
            team: TeamNumber(team),
            team_name: name.to_owned(),
            club: ClubNumber(club),
        }
    }

    fn game(day: u32, home: Side, away: Side) -> Game {
        let date = NaiveDate::from_ymd_opt(2026, 10, day).expect("valid date");
        Game {
            source_id: i64::from(day),
            game_number: day.to_string(),
            season: Season::of(date),
            date,
            start_time: NaiveTime::from_hms_opt(10, 0, 0),
            home,
            away,
            place: Place {
                number: PlaceNumber(1),
                hall: "Bulwijk".to_owned(),
                town: "Woerden".to_owned(),
                court: None,
            },
            competition: "M16".to_owned(),
        }
    }

    fn names(teams: &[Team]) -> Vec<&str> {
        teams.iter().map(|team| team.name.as_str()).collect()
    }

    fn season(text: &str) -> Season {
        text.parse().expect("valid season")
    }

    #[test]
    fn leaves_out_opponents_and_lists_each_team_once() {
        let games = [
            game(
                3,
                side(DRAKES, 184, "Blue Drakes M16 1"),
                side(GIANTS, 1688, "Green Giants M16 1"),
            ),
            game(
                10,
                side(GIANTS, 1688, "Green Giants M16 1"),
                side(DRAKES, 184, "Blue Drakes M16 1"),
            ),
            game(
                17,
                side(GIANTS, 1700, "Green Giants M18 1"),
                side(DRAKES, 154, "Blue Drakes M18 1"),
            ),
        ];
        let teams = own_teams(&club(DRAKES, "Blue Drakes"), &games);
        assert_eq!(
            teams,
            vec![
                Team {
                    number: TeamNumber(184),
                    name: "M16 1".to_owned()
                },
                Team {
                    number: TeamNumber(154),
                    name: "M18 1".to_owned()
                },
            ]
        );
    }

    #[test]
    fn removes_the_club_name_and_a_dash_in_front() {
        let games = [game(
            3,
            side(24, 9, "Utrecht Basketball - MSE-3"),
            side(DRAKES, 1379, "Blue Drakes MSE 1"),
        )];
        let teams = own_teams(&club(24, "Utrecht Basketball"), &games);
        assert_eq!(names(&teams), ["MSE-3"]);
    }

    #[test]
    fn keeps_the_full_name_when_nothing_is_left() {
        let games = [game(
            3,
            side(DRAKES, 1, "Blue Drakes"),
            side(GIANTS, 2, "Green Giants"),
        )];
        assert_eq!(
            names(&own_teams(&club(DRAKES, "Blue Drakes"), &games)),
            ["Blue Drakes"]
        );
    }

    #[test]
    fn keeps_the_full_name_when_the_club_name_is_only_the_start_of_a_word() {
        let games = [game(
            3,
            side(DRAKES, 1, "Blue Drakesters U14"),
            side(GIANTS, 2, "Green Giants U14"),
        )];
        assert_eq!(
            names(&own_teams(&club(DRAKES, "Blue Drakes"), &games)),
            ["Blue Drakesters U14"]
        );
    }

    #[test]
    fn keeps_the_full_name_when_another_club_name_is_in_front() {
        let games = [game(
            3,
            side(DRAKES, 1, "BD Woerden M16 1"),
            side(GIANTS, 2, "Green Giants M16 1"),
        )];
        assert_eq!(
            names(&own_teams(&club(DRAKES, "Blue Drakes"), &games)),
            ["BD Woerden M16 1"]
        );
    }

    #[test]
    fn a_team_with_two_names_takes_the_name_of_its_latest_game() {
        let games = [
            game(
                17,
                side(DRAKES, 184, "Blue Drakes M16 1a"),
                side(GIANTS, 1688, "Green Giants M16 1"),
            ),
            game(
                3,
                side(DRAKES, 184, "Blue Drakes M16 1"),
                side(GIANTS, 1688, "Green Giants M16 1"),
            ),
        ];
        assert_eq!(
            names(&own_teams(&club(DRAKES, "Blue Drakes"), &games)),
            ["M16 1a"]
        );
    }

    #[test]
    fn no_games_give_no_teams() {
        assert!(own_teams(&club(DRAKES, "Blue Drakes"), &[]).is_empty());
    }

    fn followed(season_text: &str, team: Option<(i64, &str)>) -> FollowedTeam {
        FollowedTeam {
            season: season(season_text),
            club: club(DRAKES, "Blue Drakes"),
            team: team.map(|(number, name)| Team {
                number: TeamNumber(number),
                name: name.to_owned(),
            }),
        }
    }

    #[test]
    fn the_schedule_is_of_the_chosen_team() {
        let state = choice_state(Some(followed("2026-2027", Some((184, "M16 1")))), None);
        let target = schedule_target(state).expect("a target");
        assert_eq!(target.season, season("2026-2027"));
        assert_eq!(target.team.number, TeamNumber(184));
        assert!(!target.last_season);
    }

    #[test]
    fn while_a_team_is_needed_the_schedule_is_of_last_season() {
        let earlier = followed("2026-2027", Some((184, "M16 1")));
        for this_season in [None, Some(followed("2027-2028", None))] {
            let state = choice_state(this_season, Some(earlier.clone()));
            let target = schedule_target(state).expect("a target");
            assert_eq!(target.season, season("2026-2027"));
            assert_eq!(target.team.number, TeamNumber(184));
            assert!(target.last_season);
        }
    }

    #[test]
    fn without_a_team_in_this_or_the_last_season_there_is_no_schedule() {
        assert_eq!(schedule_target(choice_state(None, None)), None);
        let club_only = choice_state(Some(followed("2026-2027", None)), None);
        assert_eq!(schedule_target(club_only), None);
        let earlier_club_only = choice_state(None, Some(followed("2026-2027", None)));
        assert_eq!(schedule_target(earlier_club_only), None);
    }

    #[test]
    fn nothing_saved_is_not_chosen() {
        assert_eq!(choice_state(None, None), ChoiceState::NotChosen);
    }

    #[test]
    fn a_team_of_the_current_season_is_chosen() {
        let earlier = followed("2025-2026", Some((184, "M14 1")));
        assert_eq!(
            choice_state(
                Some(followed("2026-2027", Some((184, "M16 1")))),
                Some(earlier)
            ),
            ChoiceState::Chosen {
                season: season("2026-2027"),
                club: club(DRAKES, "Blue Drakes"),
                team: Team {
                    number: TeamNumber(184),
                    name: "M16 1".to_owned()
                },
            }
        );
    }

    #[test]
    fn a_club_without_a_team_needs_a_team_when_games_appear() {
        assert_eq!(
            choice_state(Some(followed("2026-2027", None)), None),
            ChoiceState::TeamNeeded {
                club: club(DRAKES, "Blue Drakes"),
                reason: TeamNeededReason::NoGamesYet,
                last: None,
            }
        );
    }

    #[test]
    fn a_new_season_keeps_the_club_and_asks_for_the_team_again() {
        let earlier = followed("2026-2027", Some((184, "M16 1")));
        assert_eq!(
            choice_state(None, Some(earlier.clone())),
            ChoiceState::TeamNeeded {
                club: club(DRAKES, "Blue Drakes"),
                reason: TeamNeededReason::NewSeason,
                last: Some(earlier),
            }
        );
    }

    #[test]
    fn saving_the_club_for_an_empty_new_season_keeps_last_season() {
        let earlier = followed("2026-2027", Some((184, "M16 1")));
        assert_eq!(
            choice_state(Some(followed("2027-2028", None)), Some(earlier.clone())),
            ChoiceState::TeamNeeded {
                club: club(DRAKES, "Blue Drakes"),
                reason: TeamNeededReason::NoGamesYet,
                last: Some(earlier),
            }
        );
    }

    fn teams() -> Vec<Team> {
        vec![
            Team {
                number: TeamNumber(184),
                name: "M16 1".to_owned(),
            },
            Team {
                number: TeamNumber(1404),
                name: "M16 2".to_owned(),
            },
        ]
    }

    #[test]
    fn chooses_an_own_team() {
        assert_eq!(
            choose_team(teams(), Some(TeamNumber(1404))),
            Ok(Some(Team {
                number: TeamNumber(1404),
                name: "M16 2".to_owned()
            }))
        );
    }

    #[test]
    fn refuses_a_team_of_another_club() {
        assert_eq!(
            choose_team(teams(), Some(TeamNumber(1688))),
            Err(TeamChoiceError::NotAnOwnTeam)
        );
        assert_eq!(
            choose_team(Vec::new(), Some(TeamNumber(1688))),
            Err(TeamChoiceError::NotAnOwnTeam)
        );
    }

    #[test]
    fn a_team_may_be_left_out_only_while_the_club_has_no_teams() {
        assert_eq!(choose_team(Vec::new(), None), Ok(None));
        assert_eq!(
            choose_team(teams(), None),
            Err(TeamChoiceError::TeamRequired)
        );
    }

    fn choice(club_number: i64, team: Option<i64>) -> FollowedTeam {
        FollowedTeam {
            season: Season::of(NaiveDate::from_ymd_opt(2026, 10, 12).expect("valid date")),
            club: club(club_number, "Club"),
            team: team.map(|number| Team {
                number: TeamNumber(number),
                name: "M16 1".to_owned(),
            }),
        }
    }

    #[test]
    fn the_same_club_and_team_again_is_the_same_choice() {
        let now = choice(DRAKES, Some(184));
        assert!(same_choice(Some(&choice(DRAKES, Some(184))), &now));
    }

    #[test]
    fn another_team_or_club_or_a_first_choice_is_not_the_same_choice() {
        let now = choice(DRAKES, Some(184));
        assert!(!same_choice(Some(&choice(DRAKES, Some(1404))), &now));
        assert!(!same_choice(Some(&choice(GIANTS, Some(184))), &now));
        assert!(!same_choice(Some(&choice(DRAKES, None)), &now));
        assert!(!same_choice(None, &now));
    }
}
