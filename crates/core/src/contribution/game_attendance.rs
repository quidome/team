//! Attendance at games (story A of attendance at games): who comes to a game of the team, who does
//! not, and who has not answered. A player without a stored answer is "no answer". The warnings in
//! time (story B): too few players, a driver who does not come, and answers from before a move.

use chrono::{Days, NaiveDate, NaiveDateTime, NaiveTime};

use crate::contribution::duties::{Assignment, DutyKind, plays_games_on};
use crate::contribution::reasons::ReasonId;
use crate::events::game::Game;
use crate::events::schedule::ScheduledGame;
use crate::team::players::{Player, PlayerId, PlayerRole};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    Coming,
    NotComing,
}

/// The highest number of extra players at a game.
pub const MAX_EXTRA_PLAYERS: u8 = 10;

/// Why an answer is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AnswerError {
    #[error("the player is not on the list of the game")]
    NotOnList,
    #[error("the reason is hidden")]
    ReasonHidden,
}

/// The stored answer of a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoredAnswer {
    pub player: PlayerId,
    pub answer: Answer,
    pub reason: Option<ReasonId>,
}

/// What is stored for a game beside the answers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GameAttendance {
    pub extra_players: u8,
    /// The date of the game when its answers were given or confirmed.
    pub answers_for: Option<NaiveDate>,
}

/// A player on the list of a game; `answer`: `None` is "no answer".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListRow {
    pub player: PlayerId,
    pub answer: Option<Answer>,
    pub reason: Option<ReasonId>,
}

/// `coming` is the players coming plus the extra players.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GameCount {
    pub coming: usize,
    pub not_coming: usize,
    pub no_answer: usize,
}

fn group(answer: Option<Answer>) -> u8 {
    match answer {
        None => 0,
        Some(Answer::Coming) => 1,
        Some(Answer::NotComing) => 2,
    }
}

/// Checks a tap on the row of `player` at a game on `date`: the player must be on the list, and a
/// hidden reason (`reason` gives the reason and whether it is hidden) is allowed only when the
/// answer already has it (`current`).
pub fn check_answer(
    player: PlayerId,
    roles: &[PlayerRole],
    date: NaiveDate,
    reason: Option<(ReasonId, bool)>,
    current: Option<ReasonId>,
) -> Result<(), AnswerError> {
    if !plays_games_on(roles, player, date) {
        return Err(AnswerError::NotOnList);
    }
    match reason {
        Some((reason, true)) if current != Some(reason) => Err(AnswerError::ReasonHidden),
        _ => Ok(()),
    }
}

/// `number` as a number of extra players, when it is 0 to `MAX_EXTRA_PLAYERS`.
pub fn extra_players(number: i64) -> Option<u8> {
    u8::try_from(number)
        .ok()
        .filter(|number| *number <= MAX_EXTRA_PLAYERS)
}

/// The count on the row of a coming game of the team, from its `list`. A game that left the game
/// list has none; `without_answers`: also a game with no answer and no extra player.
pub fn row_count(
    list: &[ListRow],
    extra_players: u8,
    in_list: bool,
    without_answers: bool,
) -> Option<GameCount> {
    let answered = extra_players > 0 || list.iter().any(|row| row.answer.is_some());
    (in_list && (answered || without_answers)).then(|| game_count(list, extra_players))
}

/// The list of a game on `date`: the players with the role "plays games" on that date, also hidden
/// players, first "no answer", then "coming", then "not coming", each sorted by first name. An
/// answer of a player who is not on the list is left out.
pub fn game_list(
    players: &[Player],
    roles: &[PlayerRole],
    answers: &[StoredAnswer],
    date: NaiveDate,
) -> Vec<ListRow> {
    let mut list: Vec<(&Player, ListRow)> = players
        .iter()
        .filter(|player| plays_games_on(roles, player.id, date))
        .map(|player| {
            let stored = answers.iter().find(|answer| answer.player == player.id);
            let row = ListRow {
                player: player.id,
                answer: stored.map(|stored| stored.answer),
                reason: stored.and_then(|stored| stored.reason),
            };
            (player, row)
        })
        .collect();
    list.sort_by_cached_key(|(player, row)| (group(row.answer), player.sort_key()));
    list.into_iter().map(|(_, row)| row).collect()
}

/// The count of `list`: the only count, for the page of a game and the rows.
pub fn game_count(list: &[ListRow], extra_players: u8) -> GameCount {
    let count = |answer| list.iter().filter(|row| row.answer == answer).count();
    GameCount {
        coming: count(Some(Answer::Coming)) + usize::from(extra_players),
        not_coming: count(Some(Answer::NotComing)),
        no_answer: count(None),
    }
}

/// Fewer coming than this is too few.
pub const ENOUGH_PLAYERS: usize = 7;

/// The open items "too few" and "moved" start this many days before the date of the game, at 00:00.
pub const WARNING_DAYS: u64 = 3;

/// What a game of the team warns about at a moment.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GameWarnings {
    /// The open item: in the warning period and fewer than `ENOUGH_PLAYERS` coming: the count.
    pub too_few: Option<GameCount>,
    /// The line on the page of the game: fewer than `ENOUGH_PLAYERS` coming, before the start,
    /// and in the warning period or once the game has an answer.
    pub too_few_line: bool,
    /// Before the start: the planned players on driver places with the answer "not coming".
    pub drivers_not_coming: Vec<PlayerId>,
    /// In the warning period: answers from before a move (`answers_moved`).
    pub moved: bool,
}

/// The start of `game` in Dutch local time; without a start time, the end of its date.
fn start_of(game: &Game) -> NaiveDateTime {
    match game.start_time {
        Some(time) => game.date.and_time(time),
        None => game
            .date
            .checked_add_days(Days::new(1))
            .unwrap_or(game.date)
            .and_time(NaiveTime::MIN),
    }
}

/// Before the start time of `game` (without a start time: before the end of its date). `now`:
/// Dutch local time.
pub fn before_start(game: &Game, now: NaiveDateTime) -> bool {
    now < start_of(game)
}

/// From 00:00 `WARNING_DAYS` days before the date of `game` until its start time (not included);
/// without a start time until the end of the date. `now`: Dutch local time.
pub fn in_warning_period(game: &Game, now: NaiveDateTime) -> bool {
    let from = game
        .date
        .checked_sub_days(Days::new(WARNING_DAYS))
        .unwrap_or(game.date)
        .and_time(NaiveTime::MIN);
    from <= now && before_start(game, now)
}

/// The planned players on the driver places of `assignments` whose answer on `list` is "not
/// coming", in the order of the places. A player who is not on the list gives nothing.
pub fn drivers_not_coming(list: &[ListRow], assignments: &[Assignment]) -> Vec<PlayerId> {
    let mut drivers: Vec<&Assignment> = assignments
        .iter()
        .filter(|assignment| assignment.kind == DutyKind::Drive)
        .collect();
    drivers.sort_by_key(|assignment| assignment.number);
    drivers
        .into_iter()
        .map(|assignment| assignment.planned)
        .filter(|player| {
            list.iter()
                .any(|row| row.player == *player && row.answer == Some(Answer::NotComing))
        })
        .collect()
}

/// The game has answers (`has_answers`: at least one stored answer, also of a player who is not on
/// the list), and they were given for another date than `date`, the date of the game now.
pub fn answers_moved(attendance: &GameAttendance, has_answers: bool, date: NaiveDate) -> bool {
    has_answers && attendance.answers_for.is_some_and(|given| given != date)
}

/// The note "Answers from before the move to …" on the page of the game `scheduled`: its answers
/// are from before a move (`answers_moved`, with no period), and it is not "not played".
pub fn move_note(
    scheduled: &ScheduledGame,
    attendance: &GameAttendance,
    has_answers: bool,
) -> bool {
    !scheduled.not_played && answers_moved(attendance, has_answers, scheduled.game.date)
}

/// All warnings of a game of the team `scheduled` at `now` (Dutch local time), from its `list` and
/// the players on its places; none for a game that is not in the game list or is "not played". The
/// page of the game and the open items both use it (decision 0022).
pub fn game_warnings(
    scheduled: &ScheduledGame,
    list: &[ListRow],
    attendance: &GameAttendance,
    has_answers: bool,
    assignments: &[Assignment],
    now: NaiveDateTime,
) -> GameWarnings {
    let game = &scheduled.game;
    if !scheduled.in_list || scheduled.not_played || !before_start(game, now) {
        return GameWarnings::default();
    }
    let count = game_count(list, attendance.extra_players);
    let short = count.coming < ENOUGH_PLAYERS;
    let in_period = in_warning_period(game, now);
    GameWarnings {
        too_few: (short && in_period).then_some(count),
        too_few_line: short && (in_period || has_answers),
        drivers_not_coming: drivers_not_coming(list, assignments),
        moved: in_period && answers_moved(attendance, has_answers, game.date),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::schedule::ScheduleReason;
    use crate::shared::season::Season;
    use crate::team::players::{Role, RoleId};

    fn date(month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, month, day).expect("valid date")
    }

    fn player(id: i64, first_name: &str) -> Player {
        Player {
            id: PlayerId(id),
            first_name: first_name.to_owned(),
            last_name: None,
            hidden: false,
        }
    }

    fn role(id: i64, role: Role, from: NaiveDate, until: Option<NaiveDate>) -> PlayerRole {
        PlayerRole {
            id: RoleId(id),
            player: PlayerId(id),
            season: Season::of(from),
            role,
            from,
            until,
        }
    }

    fn plays(id: i64) -> PlayerRole {
        role(id, Role::PlaysGames, date(9, 1), None)
    }

    fn answer(id: i64, answer: Answer) -> StoredAnswer {
        StoredAnswer {
            player: PlayerId(id),
            answer,
            reason: None,
        }
    }

    fn ids(list: &[ListRow]) -> Vec<i64> {
        list.iter().map(|row| row.player.0).collect()
    }

    #[test]
    fn the_list_has_no_answer_first_and_each_group_by_first_name() {
        let players = [
            player(1, "Cas"),
            player(2, "Anna"),
            player(3, "Bram"),
            player(4, "Eva"),
            player(5, "Dirk"),
        ];
        let roles = [plays(1), plays(2), plays(3), plays(4), plays(5)];
        let answers = [
            answer(4, Answer::NotComing),
            answer(1, Answer::Coming),
            answer(3, Answer::Coming),
        ];
        let list = game_list(&players, &roles, &answers, date(10, 31));
        assert_eq!(ids(&list), [2, 5, 3, 1, 4]);
    }

    #[test]
    fn a_hidden_player_is_on_the_list() {
        let mut hidden = player(1, "Anna");
        hidden.hidden = true;
        let list = game_list(&[hidden], &[plays(1)], &[], date(10, 31));
        assert_eq!(ids(&list), [1]);
    }

    #[test]
    fn only_the_role_plays_games_on_the_date_counts() {
        let players = [
            player(1, "Anna"),
            player(2, "Bram"),
            player(3, "Cas"),
            player(4, "Dirk"),
            player(5, "Eva"),
        ];
        let roles = [
            plays(1),
            role(2, Role::OnlyTrains, date(9, 1), None),
            role(3, Role::PlaysGames, date(9, 1), Some(date(10, 30))),
            role(4, Role::PlaysGames, date(11, 1), None),
            role(5, Role::PlaysGames, date(9, 1), Some(date(10, 31))),
        ];
        let list = game_list(&players, &roles, &[], date(10, 31));
        assert_eq!(ids(&list), [1, 5]);
    }

    #[test]
    fn an_answer_of_a_player_off_the_list_is_left_out() {
        let players = [player(1, "Anna"), player(2, "Bram")];
        let answers = [answer(2, Answer::Coming)];
        let list = game_list(&players, &[plays(1)], &answers, date(10, 31));
        assert_eq!(ids(&list), [1]);
        assert_eq!(game_count(&list, 0).coming, 0);
    }

    #[test]
    fn a_row_keeps_its_reason() {
        let stored = StoredAnswer {
            reason: Some(ReasonId(9)),
            ..answer(1, Answer::NotComing)
        };
        let list = game_list(&[player(1, "Anna")], &[plays(1)], &[stored], date(10, 31));
        assert_eq!(list[0].reason, Some(ReasonId(9)));
    }

    #[test]
    fn the_count_adds_the_extra_players_to_coming() {
        let players = [
            player(1, "Anna"),
            player(2, "Bram"),
            player(3, "Cas"),
            player(4, "Dirk"),
        ];
        let roles = [plays(1), plays(2), plays(3), plays(4)];
        let answers = [
            answer(1, Answer::Coming),
            answer(2, Answer::Coming),
            answer(3, Answer::NotComing),
        ];
        let list = game_list(&players, &roles, &answers, date(10, 31));
        assert_eq!(
            game_count(&list, 2),
            GameCount {
                coming: 4,
                not_coming: 1,
                no_answer: 1
            }
        );
    }

    #[test]
    fn without_answers_all_are_no_answer() {
        let players = [player(1, "Anna"), player(2, "Bram")];
        let list = game_list(&players, &[plays(1), plays(2)], &[], date(10, 31));
        assert_eq!(
            game_count(&list, 0),
            GameCount {
                coming: 0,
                not_coming: 0,
                no_answer: 2
            }
        );
    }

    #[test]
    fn an_answer_needs_a_player_on_the_list() {
        let roles = [plays(1)];
        let date = date(10, 17);
        assert_eq!(check_answer(PlayerId(1), &roles, date, None, None), Ok(()));
        assert_eq!(
            check_answer(PlayerId(2), &roles, date, None, None),
            Err(AnswerError::NotOnList)
        );
    }

    #[test]
    fn a_hidden_reason_is_allowed_only_when_the_answer_has_it() {
        let roles = [plays(1)];
        let date = date(10, 17);
        let check = |reason, current| check_answer(PlayerId(1), &roles, date, reason, current);
        assert_eq!(check(Some((ReasonId(5), false)), None), Ok(()));
        assert_eq!(
            check(Some((ReasonId(5), true)), None),
            Err(AnswerError::ReasonHidden)
        );
        assert_eq!(
            check(Some((ReasonId(5), true)), Some(ReasonId(6))),
            Err(AnswerError::ReasonHidden)
        );
        assert_eq!(check(Some((ReasonId(5), true)), Some(ReasonId(5))), Ok(()));
    }

    #[test]
    fn extra_players_are_0_to_10() {
        assert_eq!(extra_players(0), Some(0));
        assert_eq!(extra_players(10), Some(10));
        assert_eq!(extra_players(11), None);
        assert_eq!(extra_players(-1), None);
        assert_eq!(extra_players(300), None);
    }

    #[test]
    fn the_list_uses_the_order_of_players() {
        let mut anna_b = player(1, "anna");
        anna_b.last_name = Some("Bos".to_owned());
        let mut anna_a = player(2, "Anna");
        anna_a.last_name = Some("Akker".to_owned());
        let players = [player(3, "bram"), player(4, "Cas"), anna_b, anna_a];
        let roles = [plays(1), plays(2), plays(3), plays(4)];
        let list = game_list(&players, &roles, &[], date(10, 31));
        assert_eq!(ids(&list), [2, 1, 3, 4]);
    }

    #[test]
    fn a_row_has_a_count_only_when_it_says_something() {
        let players = [player(1, "Anna"), player(2, "Bram")];
        let roles = [plays(1), plays(2)];
        let none = game_list(&players, &roles, &[], date(10, 31));
        // An answer of a player off the list does not count.
        let off = game_list(&players, &roles, &[answer(9, Answer::Coming)], date(10, 31));
        let on = game_list(&players, &roles, &[answer(1, Answer::Coming)], date(10, 31));
        assert_eq!(row_count(&none, 0, true, false), None);
        assert_eq!(row_count(&off, 0, true, false), None);
        assert!(row_count(&none, 0, true, true).is_some());
        assert!(row_count(&none, 1, true, false).is_some());
        assert!(row_count(&on, 0, true, false).is_some());
        assert_eq!(row_count(&on, 0, false, true), None);
    }

    // The warnings in time (story B).

    use crate::contribution::duties::Outcome;
    use crate::events::schedule::tests::{OPPONENT, TEAM, game};

    /// A game on Sat 31 Oct 2026 at 14:30, or with no start time.
    fn saturday(start: Option<(u32, u32)>) -> Game {
        let mut game = game(1, OPPONENT, TEAM);
        game.date = date(10, 31);
        game.start_time = start.and_then(|(hour, minute)| NaiveTime::from_hms_opt(hour, minute, 0));
        game
    }

    fn at(month: u32, day: u32, hour: u32, minute: u32) -> NaiveDateTime {
        date(month, day)
            .and_hms_opt(hour, minute, 0)
            .expect("valid time")
    }

    fn row(id: i64, answer: Option<Answer>) -> ListRow {
        ListRow {
            player: PlayerId(id),
            answer,
            reason: None,
        }
    }

    fn place(kind: DutyKind, number: u8, player: i64) -> Assignment {
        Assignment {
            kind,
            number,
            planned: PlayerId(player),
            outcome: Outcome::AsPlanned,
        }
    }

    /// `coming` players coming and `open` players without an answer.
    fn list_of(coming: i64, open: i64) -> Vec<ListRow> {
        (1..=coming)
            .map(|id| row(id, Some(Answer::Coming)))
            .chain((coming + 1..=coming + open).map(|id| row(id, None)))
            .collect()
    }

    fn answered_for(on: NaiveDate) -> GameAttendance {
        GameAttendance {
            extra_players: 0,
            answers_for: Some(on),
        }
    }

    fn warnings(game: &Game, list: &[ListRow], extra: u8, now: NaiveDateTime) -> GameWarnings {
        let attendance = GameAttendance {
            extra_players: extra,
            answers_for: Some(game.date),
        };
        let has_answers = list.iter().any(|row| row.answer.is_some());
        game_warnings(
            &scheduled(game, true, false),
            list,
            &attendance,
            has_answers,
            &[],
            now,
        )
    }

    fn scheduled(game: &Game, in_list: bool, not_played: bool) -> ScheduledGame {
        ScheduledGame {
            game: game.clone(),
            reason: ScheduleReason::TeamGame,
            in_list,
            not_played,
        }
    }

    #[test]
    fn the_warning_period_is_3_days_from_00_00_until_the_start_time() {
        let game = saturday(Some((14, 30)));
        assert!(!in_warning_period(&game, at(10, 27, 23, 59)));
        assert!(in_warning_period(&game, at(10, 28, 0, 0)));
        assert!(in_warning_period(&game, at(10, 31, 14, 29)));
        assert!(!in_warning_period(&game, at(10, 31, 14, 30)));
    }

    #[test]
    fn without_a_start_time_the_period_ends_with_the_date() {
        let game = saturday(None);
        assert!(in_warning_period(&game, at(10, 31, 23, 59)));
        assert!(!in_warning_period(&game, at(11, 1, 0, 0)));
        assert!(before_start(&game, at(10, 31, 23, 59)));
        assert!(!before_start(&game, at(11, 1, 0, 0)));
    }

    #[test]
    fn before_the_start_is_any_moment_before_the_start_time() {
        let game = saturday(Some((14, 30)));
        assert!(before_start(&game, at(10, 21, 9, 0)));
        assert!(before_start(&game, at(10, 31, 14, 29)));
        assert!(!before_start(&game, at(10, 31, 14, 30)));
    }

    #[test]
    fn too_few_is_fewer_than_7_coming_with_the_extra_players() {
        let game = saturday(Some((14, 30)));
        let wednesday = at(10, 28, 9, 0);
        let six = list_of(6, 3);
        assert_eq!(
            warnings(&game, &six, 0, wednesday).too_few,
            Some(GameCount {
                coming: 6,
                not_coming: 0,
                no_answer: 3,
            })
        );
        assert_eq!(warnings(&game, &list_of(7, 2), 0, wednesday).too_few, None);
        // 5 players and 2 extra players are 7.
        assert_eq!(warnings(&game, &list_of(5, 4), 2, wednesday).too_few, None);
        // Before the period, no open item.
        assert_eq!(warnings(&game, &six, 0, at(10, 27, 23, 59)).too_few, None);
    }

    #[test]
    fn the_line_on_the_page_starts_earlier_once_the_game_has_an_answer() {
        let game = saturday(Some((14, 30)));
        let monday = at(10, 26, 9, 0);
        // 5 days before, with an answer: the line, not the open item.
        let early = warnings(&game, &list_of(5, 4), 0, monday);
        assert!(early.too_few_line);
        assert_eq!(early.too_few, None);
        // 5 days before without an answer: nothing.
        assert!(!warnings(&game, &list_of(0, 9), 0, monday).too_few_line);
        // In the period without an answer: the line.
        assert!(warnings(&game, &list_of(0, 9), 0, at(10, 28, 9, 0)).too_few_line);
        // At the start time, and with 7 coming: no line.
        assert!(!warnings(&game, &list_of(5, 4), 0, at(10, 31, 14, 30)).too_few_line);
        assert!(!warnings(&game, &list_of(7, 2), 0, monday).too_few_line);
    }

    #[test]
    fn a_driver_who_is_not_coming_is_found() {
        let list = [
            row(1, Some(Answer::NotComing)),
            row(2, Some(Answer::Coming)),
            row(3, None),
            row(4, Some(Answer::NotComing)),
        ];
        let places = [
            place(DutyKind::Drive, 1, 1),
            place(DutyKind::Drive, 2, 2),
            place(DutyKind::Drive, 3, 3),
        ];
        assert_eq!(drivers_not_coming(&list, &places), [PlayerId(1)]);
        // A table or referee place gives nothing.
        let other = [place(DutyKind::Table, 1, 4), place(DutyKind::Referee, 1, 1)];
        assert!(drivers_not_coming(&list, &other).is_empty());
        // A stored "not coming" of a player off the list (`game_list` leaves it out) gives nothing.
        assert!(drivers_not_coming(&list, &[place(DutyKind::Drive, 1, 9)]).is_empty());
    }

    #[test]
    fn two_drivers_who_are_not_coming_are_in_the_order_of_the_places() {
        let list = [
            row(1, Some(Answer::NotComing)),
            row(2, Some(Answer::NotComing)),
        ];
        let places = [place(DutyKind::Drive, 2, 1), place(DutyKind::Drive, 1, 2)];
        assert_eq!(
            drivers_not_coming(&list, &places),
            [PlayerId(2), PlayerId(1)]
        );
    }

    #[test]
    fn answers_moved_when_they_were_given_for_another_date() {
        let now = date(11, 4);
        assert!(!answers_moved(&GameAttendance::default(), false, now));
        assert!(!answers_moved(&answered_for(now), true, now));
        // Also when all players who answered are off the list: `has_answers` counts the stored
        // answers, not the rows of the list.
        assert!(answers_moved(&answered_for(date(10, 31)), true, now));
        // No answer stored any more: nothing to check.
        assert!(!answers_moved(&answered_for(date(10, 31)), false, now));
    }

    #[test]
    fn the_warnings_of_a_game_follow_their_periods() {
        let game = saturday(Some((14, 30)));
        let list = [row(1, Some(Answer::NotComing))];
        let places = [place(DutyKind::Drive, 1, 1)];
        let moved = answered_for(date(10, 24));
        let of_game = |in_list, not_played, now| {
            game_warnings(
                &scheduled(&game, in_list, not_played),
                &list,
                &moved,
                true,
                &places,
                now,
            )
        };
        let of = |in_list, now| of_game(in_list, false, now);
        // 10 days before: only the driver (and the line, as the game has an answer).
        let early = of(true, at(10, 21, 9, 0));
        assert_eq!(early.drivers_not_coming, [PlayerId(1)]);
        assert_eq!(early.too_few, None);
        assert!(!early.moved);
        // In the period: all.
        let late = of(true, at(10, 31, 14, 29));
        assert!(late.too_few.is_some() && late.moved && late.too_few_line);
        assert_eq!(late.drivers_not_coming, [PlayerId(1)]);
        // At the start time: none.
        assert_eq!(of(true, at(10, 31, 14, 30)), GameWarnings::default());
        // A game that is not in the game list: none.
        assert_eq!(of(false, at(10, 31, 9, 0)), GameWarnings::default());
        // A not played game, in the period with too few players and a driver not coming: none.
        assert_eq!(
            of_game(true, true, at(10, 31, 9, 0)),
            GameWarnings::default()
        );
        assert_eq!(
            of_game(true, true, at(10, 21, 9, 0)),
            GameWarnings::default()
        );
    }
}
