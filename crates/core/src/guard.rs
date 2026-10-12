//! The guard: a reading with no games, or with far fewer games than the last good reading, is held
//! back so that games are never lost on a bad answer of the source. Also: which reading is the
//! first one after a choice.

use crate::game::TeamNumber;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Guard {
    Accept,
    HoldBack(HoldReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldReason {
    /// No games, while the last good reading had `before` games.
    Empty { before: u32 },
    /// Fewer than half the games of the last good reading.
    FarFewer { before: u32, now: u32 },
}

/// `last_good`: the number of games of the last good reading of the same club and season.
/// `read`: the number of games of this reading.
pub fn guard(last_good: Option<u32>, read: u32) -> Guard {
    let Some(before) = last_good else {
        return Guard::Accept;
    };
    if read == 0 && before > 0 {
        Guard::HoldBack(HoldReason::Empty { before })
    } else if u64::from(read) * 2 < u64::from(before) {
        Guard::HoldBack(HoldReason::FarFewer { before, now: read })
    } else {
        Guard::Accept
    }
}

/// The guard after the coordinator confirmed that the games of a held-back reading with
/// `confirmed` games are really gone: a reading with at least that many games is accepted; with
/// fewer, the normal guard applies again, so no more games go than the coordinator confirmed.
pub fn guard_after_confirm(last_good: Option<u32>, confirmed: u32, read: u32) -> Guard {
    if read >= confirmed {
        Guard::Accept
    } else {
        guard(last_good, read)
    }
}

/// Whether a reading is the first one after a choice of club or team, so it counts nothing.
/// `last_good_team`: the team of the last good reading of the same club and season (`None` when
/// there is no such reading; `Some(None)` when it was read without a team).
pub fn is_first_reading(
    last_good_team: Option<Option<TeamNumber>>,
    team: Option<TeamNumber>,
) -> bool {
    last_good_team != Some(team)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_earlier_good_reading_is_accepted() {
        assert_eq!(guard(None, 0), Guard::Accept);
    }

    #[test]
    fn an_empty_reading_after_an_empty_reading_is_accepted() {
        assert_eq!(guard(Some(0), 0), Guard::Accept);
    }

    #[test]
    fn an_empty_reading_after_games_is_held_back() {
        assert_eq!(
            guard(Some(71), 0),
            Guard::HoldBack(HoldReason::Empty { before: 71 })
        );
    }

    #[test]
    fn fewer_than_half_is_held_back() {
        assert_eq!(
            guard(Some(71), 35),
            Guard::HoldBack(HoldReason::FarFewer {
                before: 71,
                now: 35
            })
        );
    }

    #[test]
    fn half_or_more_is_accepted() {
        assert_eq!(guard(Some(71), 36), Guard::Accept);
        assert_eq!(guard(Some(10), 5), Guard::Accept);
    }

    #[test]
    fn more_games_are_accepted() {
        assert_eq!(guard(Some(71), 80), Guard::Accept);
    }

    #[test]
    fn after_a_confirmation_the_confirmed_number_or_more_is_accepted() {
        assert_eq!(guard_after_confirm(Some(71), 30, 30), Guard::Accept);
        assert_eq!(guard_after_confirm(Some(71), 30, 50), Guard::Accept);
        assert_eq!(guard_after_confirm(Some(71), 0, 0), Guard::Accept);
    }

    #[test]
    fn after_a_confirmation_fewer_games_are_held_back_again() {
        assert_eq!(
            guard_after_confirm(Some(71), 30, 10),
            Guard::HoldBack(HoldReason::FarFewer {
                before: 71,
                now: 10
            })
        );
        assert_eq!(
            guard_after_confirm(Some(71), 30, 0),
            Guard::HoldBack(HoldReason::Empty { before: 71 })
        );
    }

    #[test]
    fn the_first_reading_is_the_one_without_a_good_reading_of_the_same_team() {
        let team = Some(TeamNumber(184));
        assert!(is_first_reading(None, team));
        assert!(is_first_reading(Some(Some(TeamNumber(1404))), team));
        assert!(is_first_reading(Some(None), team));
        assert!(!is_first_reading(Some(team), team));
        assert!(!is_first_reading(Some(None), None));
    }
}
