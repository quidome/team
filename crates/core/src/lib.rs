//! Product rules of the team coordinator tool. No I/O: see decision 0001.

pub mod access;
pub mod club;
pub mod game;
pub mod season;
pub mod team_choice;

pub use access::{Identity, may_access};
pub use club::{Club, ClubNumber, search_clubs};
pub use game::{Game, Place, PlaceNumber, Side, TeamNumber, start_time_from_source};
pub use season::{Season, SeasonError};
pub use team_choice::{
    ChoiceState, FollowedTeam, Team, TeamChoiceError, TeamNeededReason, choice_state, choose_team,
    own_teams,
};
