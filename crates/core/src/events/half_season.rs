//! The half seasons of the club's games: for each half, the club teams that have a competition
//! whose first game is in it, and the club teams that still wait for their second half. A half
//! season is ready (the referee committee can make its schedule) when more than 2 club teams
//! count for it. Decision 0037.

use std::collections::{BTreeMap, BTreeSet};

use chrono::NaiveDate;

use crate::events::club::ClubNumber;
use crate::events::game::TeamNumber;
use crate::events::schedule::StoredGame;
use crate::shared::season::Half;

/// A half season is ready when more than this many club teams count for it.
const READY_ABOVE: usize = 2;

/// For one half season: the club teams with a competition whose first game is in that half
/// (`known`), and the club teams with games in the first half and none in the second (`waiting`;
/// only for the second half, 0 for the first).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HalfCount {
    pub known: usize,
    pub waiting: usize,
}

/// The counts of both half seasons of one season. The default has no club team: no half is
/// ready.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HalfSeasons {
    pub first: HalfCount,
    pub second: HalfCount,
}

impl HalfSeasons {
    pub fn of(&self, half: Half) -> HalfCount {
        match half {
            Half::First => self.first,
            Half::Second => self.second,
        }
    }

    /// More than 2 club teams count for `half`.
    pub fn ready(&self, half: Half) -> bool {
        self.of(half).known > READY_ABOVE
    }
}

/// The counts of the half seasons from the stored games of one season. Only the games still in
/// the game list count, home or away, also a game marked "Not played". A side is a club team when
/// its club is `club`; a game between two club teams counts for both. A competition is known by
/// its name within the season (the source's names and numbers match one to one); its first game
/// is its earliest game of that club team in the game list.
pub fn half_seasons(club: ClubNumber, games: &[StoredGame]) -> HalfSeasons {
    let mut first_games: BTreeMap<(TeamNumber, &str), NaiveDate> = BTreeMap::new();
    let mut halves: BTreeMap<TeamNumber, BTreeSet<Half>> = BTreeMap::new();
    for stored in games.iter().filter(|stored| stored.in_list) {
        let game = &stored.game;
        for side in [&game.home, &game.away] {
            if side.club != club {
                continue;
            }
            first_games
                .entry((side.team, game.competition.as_str()))
                .and_modify(|first| *first = (*first).min(game.date))
                .or_insert(game.date);
            halves
                .entry(side.team)
                .or_default()
                .insert(Half::of(game.date));
        }
    }
    let known = |half: Half| {
        first_games
            .iter()
            .filter(|&(_, &first)| Half::of(first) == half)
            .map(|(&(team, _), _)| team)
            .collect::<BTreeSet<_>>()
            .len()
    };
    let waiting = halves
        .values()
        .filter(|halves| halves.contains(&Half::First) && !halves.contains(&Half::Second))
        .count();
    HalfSeasons {
        first: HalfCount {
            known: known(Half::First),
            waiting: 0,
        },
        second: HalfCount {
            known: known(Half::Second),
            waiting,
        },
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;
    use crate::events::schedule::tests::{CLUB, CLUB_TEAM, OPPONENT, TEAM, game};

    const SENIORS: TeamNumber = TeamNumber(186);
    const FOURTH: TeamNumber = TeamNumber(187);

    /// A game of `home` against `away` on a date, in a competition, in the game list.
    fn on(
        id: i64,
        home: TeamNumber,
        away: TeamNumber,
        (year, month, day): (i32, u32, u32),
        competition: &str,
    ) -> StoredGame {
        let mut game = game(id, home, away);
        game.date = NaiveDate::from_ymd_opt(year, month, day).expect("valid date");
        game.competition = competition.to_owned();
        StoredGame {
            game,
            in_list: true,
            not_played_for: None,
        }
    }

    fn count(known: usize, waiting: usize) -> HalfCount {
        HalfCount { known, waiting }
    }

    #[test]
    fn a_senior_team_with_one_competition_counts_for_the_first_half_only() {
        let games = [
            on(1, SENIORS, OPPONENT, (2026, 9, 12), "Seniors"),
            on(2, OPPONENT, SENIORS, (2027, 1, 9), "Seniors"),
            on(3, SENIORS, OPPONENT, (2027, 4, 10), "Seniors"),
        ];
        let halves = half_seasons(CLUB, &games);
        assert_eq!(halves.first, count(1, 0));
        assert_eq!(halves.second, count(0, 0));
    }

    #[test]
    fn a_youth_team_waits_until_its_second_half_is_in_the_game_list() {
        let mut games = vec![
            on(1, TEAM, OPPONENT, (2026, 9, 12), "U16 first half"),
            on(2, OPPONENT, TEAM, (2026, 12, 12), "U16 first half"),
        ];
        let halves = half_seasons(CLUB, &games);
        assert_eq!(halves.first, count(1, 0));
        assert_eq!(halves.second, count(0, 1));

        games.push(on(3, TEAM, OPPONENT, (2027, 1, 16), "U16 second half"));
        let halves = half_seasons(CLUB, &games);
        assert_eq!(halves.first, count(1, 0));
        assert_eq!(halves.second, count(1, 0));
    }

    #[test]
    fn a_team_with_only_away_games_counts() {
        let games = [
            on(1, OPPONENT, TEAM, (2027, 1, 16), "U16 second half"),
            on(2, OPPONENT, TEAM, (2027, 1, 23), "U16 second half"),
        ];
        assert_eq!(half_seasons(CLUB, &games).second, count(1, 0));
    }

    #[test]
    fn a_game_between_two_club_teams_counts_for_both() {
        let games = [on(1, TEAM, CLUB_TEAM, (2027, 1, 16), "U16 second half")];
        assert_eq!(half_seasons(CLUB, &games).second, count(2, 0));
    }

    #[test]
    fn two_club_teams_in_one_competition_count_as_two() {
        let games = [
            on(1, TEAM, OPPONENT, (2026, 9, 12), "U14 first half"),
            on(2, CLUB_TEAM, OPPONENT, (2026, 9, 19), "U14 first half"),
        ];
        let halves = half_seasons(CLUB, &games);
        assert_eq!(halves.first, count(2, 0));
        assert_eq!(halves.second, count(0, 2));
    }

    #[test]
    fn a_game_not_played_counts_and_a_game_out_of_the_list_does_not() {
        let mut not_played = on(1, TEAM, OPPONENT, (2027, 1, 16), "U16 second half");
        not_played.not_played_for = Some(not_played.game.date);
        let mut gone = on(2, CLUB_TEAM, OPPONENT, (2027, 1, 16), "U14 second half");
        gone.in_list = false;
        assert_eq!(half_seasons(CLUB, &[not_played, gone]).second, count(1, 0));
    }

    #[test]
    fn when_the_first_game_leaves_the_list_the_next_game_decides_the_half() {
        // A competition whose first game in December left the game list; its games of January
        // remain, so its first game in the list is in the second half.
        let mut december = on(1, TEAM, OPPONENT, (2026, 12, 19), "U16 second half");
        december.in_list = false;
        let january = on(2, TEAM, OPPONENT, (2027, 1, 16), "U16 second half");
        let halves = half_seasons(CLUB, &[december, january]);
        assert_eq!(halves.first, count(0, 0));
        assert_eq!(halves.second, count(1, 0));
    }

    #[test]
    fn a_team_with_games_only_in_the_second_half_counts_and_does_not_wait() {
        let games = [on(1, TEAM, OPPONENT, (2027, 2, 6), "New team")];
        let halves = half_seasons(CLUB, &games);
        assert_eq!(halves.first, count(0, 0));
        assert_eq!(halves.second, count(1, 0));
    }

    #[test]
    fn the_games_of_other_clubs_do_not_count() {
        let games = [on(1, OPPONENT, TeamNumber(901), (2027, 1, 16), "Other")];
        assert_eq!(half_seasons(CLUB, &games), HalfSeasons::default());
    }

    #[test]
    fn a_half_is_ready_from_3_club_teams() {
        let mut games = vec![
            on(1, TEAM, OPPONENT, (2027, 1, 16), "U16 second half"),
            on(2, CLUB_TEAM, OPPONENT, (2027, 1, 16), "U14 second half"),
            on(3, SENIORS, OPPONENT, (2026, 9, 12), "Seniors"),
            on(4, SENIORS, OPPONENT, (2027, 1, 9), "Seniors"),
        ];
        let halves = half_seasons(CLUB, &games);
        assert!(!halves.ready(Half::Second), "{halves:?}");
        assert!(!halves.ready(Half::First), "{halves:?}");

        games.push(on(5, FOURTH, OPPONENT, (2027, 1, 23), "U18 second half"));
        let halves = half_seasons(CLUB, &games);
        assert!(halves.ready(Half::Second), "{halves:?}");
        assert_eq!(halves.second, count(3, 0));
    }
}
