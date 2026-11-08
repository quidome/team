//! A game as the tool understands it. Dates and start times are Dutch local time.

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use crate::events::club::ClubNumber;
use crate::shared::season::Season;

/// The source's own team number. It stays the same between seasons, so a team is always used
/// together with its season.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TeamNumber(pub i64);

/// The source's own number of a place (a hall).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlaceNumber(pub i64);

/// One side of a game: the home team or the away team.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Side {
    pub team: TeamNumber,
    /// The full name from the source, with the club name in front.
    pub team_name: String,
    pub club: ClubNumber,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    pub number: PlaceNumber,
    pub hall: String,
    pub town: String,
    pub court: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    /// The source's own game id.
    pub source_id: i64,
    /// The game number (probably the federation's).
    pub game_number: String,
    pub season: Season,
    pub date: NaiveDate,
    /// `None`: the start time is not known yet.
    pub start_time: Option<NaiveTime>,
    pub home: Side,
    pub away: Side,
    pub place: Place,
    pub competition: String,
}

/// `game` has started at `now` (Dutch local time): its start time has passed; without a start
/// time, its date has begun.
pub fn started(game: &Game, now: NaiveDateTime) -> bool {
    match game.start_time {
        Some(time) => now >= game.date.and_time(time),
        None => now.date() >= game.date,
    }
}

/// The source writes 00:00 when the start time is not known yet.
pub fn start_time_from_source(time: NaiveTime) -> Option<NaiveTime> {
    (time != NaiveTime::MIN).then_some(time)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn time(hour: u32, minute: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(hour, minute, 0).expect("valid time")
    }

    #[test]
    fn midnight_means_not_known() {
        assert_eq!(start_time_from_source(time(0, 0)), None);
    }

    #[test]
    fn other_times_stay() {
        assert_eq!(start_time_from_source(time(11, 15)), Some(time(11, 15)));
        assert_eq!(start_time_from_source(time(0, 30)), Some(time(0, 30)));
    }

    #[test]
    fn a_game_starts_at_its_start_time_or_else_when_its_date_begins() {
        let date = NaiveDate::from_ymd_opt(2026, 11, 7).expect("valid date");
        let at = |day: u32, hour: u32, minute: u32| {
            NaiveDate::from_ymd_opt(2026, 11, day)
                .expect("valid date")
                .and_hms_opt(hour, minute, 0)
                .expect("valid time")
        };
        let mut game = crate::events::schedule::tests::game(1, TeamNumber(1), TeamNumber(2));
        game.date = date;
        game.start_time = Some(time(11, 15));
        assert!(!started(&game, at(7, 11, 14)));
        assert!(started(&game, at(7, 11, 15)));
        game.start_time = None;
        assert!(!started(&game, at(6, 23, 59)));
        assert!(started(&game, at(7, 0, 0)));
    }
}
