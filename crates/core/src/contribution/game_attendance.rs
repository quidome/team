//! Attendance at games (story A of attendance at games): who comes to a game of the team, who does
//! not, and who has not answered. A player without a stored answer is "no answer".

use chrono::NaiveDate;

use crate::contribution::duties::plays_games_on;
use crate::contribution::reasons::ReasonId;
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

#[cfg(test)]
mod tests {
    use super::*;
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
}
