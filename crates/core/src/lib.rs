//! Product rules of the team coordinator tool. No I/O: see decision 0001.

pub mod access;
pub mod attendance;
pub mod club;
pub mod comparison;
pub mod duties;
pub mod game;
pub mod guard;
pub mod messages;
pub mod nightly;
pub mod players;
pub mod reasons;
pub mod schedule;
pub mod season;
pub mod season_view;
pub mod start_page;
pub mod team_choice;
pub mod training;

pub use access::{Identity, may_access};
pub use attendance::{
    Absence, AtTraining, AttendanceError, Missed, PlayerTotals, SeasonOverview, check_mark,
    now_there, players_at, season_overview,
};
pub use club::{Club, ClubNumber, search_clubs};
pub use comparison::{
    Change, ChangeKind, ChangeUpdate, Counts, Difference, GameChange, ListedChange, OpenChange,
    StoredGame, change_between, compare, count, list_changes, no_longer_in_list, same_number_pairs,
    update_open_changes,
};
pub use duties::{
    Assignment, Candidate, ChoicePlayer, Clash, ClashLevel, ClashWhat, Counted, DriverPart,
    DutyError, DutyKind, DutyPlace, GameDuties, GamePlaces, KindTotals, NotOffered, Offer, Outcome,
    PlaceKind, RequiredDuties, SCHEDULE_PLACES, SchedulePart, SchedulePlace, StoredDuties, TeamRef,
    Total, check_assignment, check_drivers, check_entered, check_outcome, check_team,
    choice_players, clash_level, clashes, club_teams, display_names, driver_proposal, duty_rows,
    duty_totals, game_places, has_team_places, highest_driver, kept_games, not_entered, offers,
    open_by_kind, open_places, pick_team, players_playing_on, plays_games_on, required_duties,
    who_did, worst_clash,
};
pub use game::{Game, Place, PlaceNumber, Side, TeamNumber, start_time_from_source};
pub use guard::{Guard, HoldReason, guard, guard_after_confirm, is_first_reading};
pub use messages::{MessageGame, coming_week, next_game_message};
pub use nightly::{NIGHTLY_AT, OUT_OF_DATE_NIGHTS, last_run_moment, nightly_due, out_of_date};
pub use players::{
    Group, Member, Placement, Player, PlayerError, PlayerId, PlayerRole, Removal, Role, RoleChange,
    RoleDates, RoleId, TeamPage, change_role, check_change, check_role, clean_name, place,
    proposed_start, remove_from_team, role_on, same_name, team, undo_removal,
};
pub use reasons::{ReasonError, ReasonId, clean_reason_name};
pub use schedule::{Schedule, ScheduleReason, ScheduledGame, in_schedule, schedule};
pub use season::{Season, SeasonError};
pub use season_view::{Event, EventKind, SeasonEvents, season_events};
pub use start_page::{
    OpenGame, OpenItem, PlacesOf, ReadingState, last_games, last_training, next_events,
    open_item_seasons, open_items, open_places_games, schedule_not_entered,
};
pub use team_choice::{
    ChoiceState, FollowedTeam, Team, TeamChoiceError, TeamNeededReason, choice_state, choose_team,
    own_teams, same_choice,
};
pub use training::{
    DayStatus, SeasonList, SeasonRow, Training, TrainingError, TrainingId, WeeklyProposal,
    WeeklyTraining, WeeklyTrainingId, absent_count, check_checking, check_training,
    check_weekly_training, day_times, not_checked, not_opened, proposals_from, proposed_times,
    proposed_window, season_list, training_days,
};
