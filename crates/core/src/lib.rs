//! Product rules of the team coordinator tool. No I/O: see decision 0001.

pub mod access;
pub mod club;
pub mod comparison;
pub mod game;
pub mod guard;
pub mod nightly;
pub mod players;
pub mod reasons;
pub mod schedule;
pub mod season;
pub mod team_choice;
pub mod training;

pub use access::{Identity, may_access};
pub use club::{Club, ClubNumber, search_clubs};
pub use comparison::{
    Change, ChangeKind, ChangeUpdate, Counts, Difference, GameChange, ListedChange, OpenChange,
    StoredGame, change_between, compare, count, list_changes, no_longer_in_list, same_number_pairs,
    update_open_changes,
};
pub use game::{Game, Place, PlaceNumber, Side, TeamNumber, start_time_from_source};
pub use guard::{Guard, HoldReason, guard, guard_after_confirm, is_first_reading};
pub use nightly::{NIGHTLY_AT, OUT_OF_DATE_NIGHTS, last_run_moment, nightly_due, out_of_date};
pub use players::{
    Group, Placement, Player, PlayerError, PlayerId, PlayerRole, Role, RoleId, check_role,
    clean_name, place, proposed_start, role_on, roles_to_carry,
};
pub use reasons::{ReasonError, clean_reason_name};
pub use schedule::{Schedule, ScheduleReason, ScheduledGame, in_schedule, schedule};
pub use season::{Season, SeasonError};
pub use team_choice::{
    ChoiceState, FollowedTeam, ScheduleTarget, Team, TeamChoiceError, TeamNeededReason,
    choice_state, choose_team, own_teams, same_choice, schedule_target,
};
pub use training::{
    TrainingDay, TrainingDaySet, TrainingError, TrainingSeason, proposed_dates, set_on,
    settings_change, training_dates,
};
