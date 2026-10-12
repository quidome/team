//! Which games of the club's game list go in the coordinator's schedule.

use crate::club::ClubNumber;
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
}
