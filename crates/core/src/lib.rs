//! Product rules of the team coordinator tool. No I/O: see decision 0001.

pub mod access;
pub mod club;
pub mod comparison;
pub mod game;
pub mod guard;
pub mod schedule;
pub mod season;
pub mod team_choice;

pub use access::{Identity, may_access};
pub use club::{Club, ClubNumber, search_clubs};
pub use comparison::{
    Change, Counts, Difference, GameChange, StoredGame, compare, count, no_longer_in_list,
    same_number_pairs,
};
pub use game::{Game, Place, PlaceNumber, Side, TeamNumber, start_time_from_source};
pub use guard::{Guard, HoldReason, guard, guard_after_confirm, is_first_reading};
pub use schedule::{Schedule, ScheduleReason, ScheduledGame, in_schedule, schedule};
pub use season::{Season, SeasonError};
pub use team_choice::{
    ChoiceState, FollowedTeam, ScheduleTarget, Team, TeamChoiceError, TeamNeededReason,
    choice_state, choose_team, own_teams, schedule_target,
};
