//! The guard: a reading with no games, or with far fewer games than the last good reading, is held
//! back so that games are never lost on a bad answer of the source.

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
}
