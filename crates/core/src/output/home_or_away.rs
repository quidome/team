//! Home game or away game of the team (`docs/language.md`), and its opponent: one rule for the
//! message of the week and the start page.

use crate::events::club::ClubNumber;
use crate::events::game::{Game, TeamNumber};

/// Where the team plays a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomeOrAway {
    /// In the club's own hall, also with the team as the away side against another club team.
    Home,
    /// At another club.
    Away,
}

/// Home or away, and the name of the opponent, for a game of `team` of `club`.
pub fn team_side(game: &Game, club: ClubNumber, team: TeamNumber) -> (HomeOrAway, &str) {
    let side = if game.home.club == club {
        HomeOrAway::Home
    } else {
        HomeOrAway::Away
    };
    (side, opponent(game, team))
}

/// The name of the other side of a game of `team`.
pub(crate) fn opponent(game: &Game, team: TeamNumber) -> &str {
    if game.home.team == team {
        &game.away.team_name
    } else {
        &game.home.team_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::schedule::tests::{CLUB, CLUB_TEAM, OPPONENT, TEAM, game};

    #[test]
    fn a_game_in_the_clubs_hall_is_a_home_game() {
        assert_eq!(
            team_side(&game(1, TEAM, OPPONENT), CLUB, TEAM),
            (HomeOrAway::Home, "Team 900")
        );
    }

    #[test]
    fn a_game_at_another_club_is_an_away_game() {
        assert_eq!(
            team_side(&game(2, OPPONENT, TEAM), CLUB, TEAM),
            (HomeOrAway::Away, "Team 900")
        );
    }

    #[test]
    fn the_away_side_against_another_club_team_plays_at_home() {
        assert_eq!(
            team_side(&game(3, CLUB_TEAM, TEAM), CLUB, TEAM),
            (HomeOrAway::Home, "Team 185")
        );
    }
}
