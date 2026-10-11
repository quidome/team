//! A game as the tool understands it. Dates and start times are Dutch local time.

use chrono::{NaiveDate, NaiveTime};

use crate::club::ClubNumber;
use crate::season::Season;

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
}
