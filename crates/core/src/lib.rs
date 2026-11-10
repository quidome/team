//! Product rules of the team coordinator tool. No I/O: see decision 0001.
//! The modules are grouped in six domains with a fixed direction of use (decision 0027).

pub mod contribution;
pub mod events;
pub mod game_list;
pub mod output;
pub mod shared;
pub mod team;

pub use contribution::attendance::{
    Absence, AtTraining, AttendanceError, GameTotals, Missed, MissedKind, PlayerTotals,
    SeasonOverview, TrainingState, absent_count, check_mark, now_there, players_at,
    season_overview, training_state,
};
pub use contribution::duties::{
    Assignment, Candidate, ChoicePlayer, Clash, ClashLevel, ClashWhat, Counted, DriverPart,
    DutyError, DutyKind, DutyPlace, GameDuties, GameMoment, GamePlaces, KindTotals, NotOffered,
    Offer, Outcome, PlaceKind, RequiredDuties, SCHEDULE_PLACES, SchedulePart, SchedulePlace,
    StoredDuties, TeamRef, Total, check_assignment, check_drivers, check_entered, check_outcome,
    check_team, choice_players, clash_level, clashes, club_teams, display_names, driver_proposal,
    duty_rows, duty_totals, game_places, has_duty_row, has_team_places, highest_driver, kept_games,
    not_entered, offers, open_by_kind, open_places, pick_team, players_playing_on, plays_games_on,
    required_duties, team_games, who_did, worst_clash,
};
pub use contribution::game_attendance::{
    Answer, AnswerError, CheckError, CountedGame, ENOUGH_PLAYERS, GameAttendance, GameCount,
    GameWarnings, ListRow, MAX_EXTRA_PLAYERS, StoredAnswer, WARNING_DAYS, answers_in_effect,
    answers_moved, before_start, check_answer, check_game_checking, checked, counted_games,
    drivers_not_coming, extra_players, game_count, game_list, game_warnings, in_warning_period,
    move_note, now_no_answer, row_count, to_settle,
};
pub use contribution::reasons::{ReasonError, ReasonId, clean_reason_name};
pub use events::club::{Club, ClubNumber, search_clubs};
pub use events::game::{
    Game, Place, PlaceNumber, Side, TeamNumber, start_time_from_source, started,
};
pub use events::schedule::{
    Schedule, ScheduleReason, ScheduledGame, StoredGame, in_schedule, schedule,
};
pub use events::training::{
    DayStatus, SeasonList, SeasonRow, Training, TrainingError, TrainingId, WeeklyProposal,
    WeeklyTraining, WeeklyTrainingId, check_checking, check_training, check_weekly_training,
    day_times, not_checked, not_opened, proposals_from, proposed_times, proposed_window,
    season_list, training_days,
};
pub use game_list::comparison::{
    Change, ChangeKind, ChangeUpdate, Counts, Difference, GameChange, ListedChange, OpenChange,
    change_between, compare, count, list_changes, no_longer_in_list, same_number_pairs,
    update_open_changes,
};
pub use game_list::guard::{Guard, HoldReason, guard, guard_after_confirm, is_first_reading};
pub use game_list::nightly::{
    NIGHTLY_AT, OUT_OF_DATE_NIGHTS, last_run_moment, nightly_due, out_of_date,
};
pub use output::home_or_away::{HomeOrAway, team_side};
pub use output::messages::{
    MessageGame, committee_message, driving_overview, full_names, table_referee_overview,
    week_has_duty_places, week_message, week_of,
};
pub use output::report::{ReportColumn, ReportRow, Share, percent, report_copy, report_rows};
pub use output::season_view::{Event, EventKind, SeasonEvents, season_events};
pub use output::start_page::{
    EventLook, NotCheckedGame, OpenGame, OpenItem, PlacesOf, ReadingState, event_look,
    games_not_checked, last_games, last_training, next_events, open_item_seasons, open_items,
    open_places_games, schedule_not_entered,
};
pub use shared::access::{Identity, may_access};
pub use shared::season::{Season, SeasonError};
pub use team::players::{
    Group, Member, Placement, Player, PlayerError, PlayerId, PlayerRole, Removal, Role, RoleChange,
    RoleDates, RoleId, TeamPage, change_role, check_change, check_role, clean_name, place,
    proposed_start, remove_from_team, role_on, same_name, team, undo_removal,
};
pub use team::team_choice::{
    ChoiceState, FollowedTeam, Team, TeamChoiceError, TeamNeededReason, choice_state, choose_team,
    own_teams, same_choice,
};
