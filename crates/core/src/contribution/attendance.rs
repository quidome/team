//! Attendance at training: who is at a training, and the absences with their reason. "There" is
//! not stored: a player is there when the training exists, the player has a role on its date, and
//! there is no absence (decision 0020).

use chrono::NaiveDate;

use crate::contribution::game_attendance::{Answer, CountedGame};
use crate::contribution::reasons::ReasonId;
use crate::events::training::{
    SeasonRow, Training, TrainingId, WeeklyTraining, not_opened, season_list,
};
use crate::shared::season::Season;
use crate::team::players::{Group, Player, PlayerId, PlayerRole, Role, place, role_on};

/// A player who was not at a training, with the reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Absence {
    pub training: TrainingId,
    pub player: PlayerId,
    pub reason: ReasonId,
}

/// A player at a training: the role on its date, and the reason when absent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtTraining<'a> {
    pub player: &'a Player,
    pub role: Role,
    pub absence: Option<ReasonId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AttendanceError {
    #[error("the player is not in the team on that date")]
    NotInTeam,
    #[error("the reason is hidden")]
    ReasonHidden,
}

/// The absences at `training` of players with a role on its date.
pub fn absent_count(training: &Training, absences: &[Absence], roles: &[PlayerRole]) -> usize {
    absences
        .iter()
        .filter(|absence| {
            absence.training == training.id
                && role_on(roles, absence.player, training.date).is_some()
        })
        .count()
}

/// What the list of a season shows about the attendance of a row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrainingState {
    /// A training day without a training.
    NotOpened,
    AllThere,
    /// The number of absences of players with a role on the date.
    Absent(usize),
    Cancelled,
}

/// The state of `row`: for a training that is not cancelled, the absences of players with a role
/// on its date (`roles`: at least the roles of the season).
pub fn training_state(
    row: &SeasonRow<'_>,
    absences: &[Absence],
    roles: &[PlayerRole],
) -> TrainingState {
    match row.training {
        None => TrainingState::NotOpened,
        Some(training) if training.cancelled => TrainingState::Cancelled,
        Some(training) => match absent_count(training, absences, roles) {
            0 => TrainingState::AllThere,
            n => TrainingState::Absent(n),
        },
    }
}

/// The players at `training`: those with a role on its date (also hidden players), sorted by first
/// name, then last name, each with its role and its absence, if any.
pub fn players_at<'a>(
    players: &'a [Player],
    roles: &[PlayerRole],
    absences: &[Absence],
    training: &Training,
) -> Vec<AtTraining<'a>> {
    let mut at: Vec<AtTraining<'a>> = players
        .iter()
        .filter_map(|player| {
            let role = role_on(roles, player.id, training.date)?;
            let absence = absences
                .iter()
                .find(|absence| absence.training == training.id && absence.player == player.id)
                .map(|absence| absence.reason);
            Some(AtTraining {
                player,
                role: role.role,
                absence,
            })
        })
        .collect();
    at.sort_by_cached_key(|entry| entry.player.sort_key());
    at
}

/// The trainings at which `player` now counts as there after a change of its roles from `before`
/// to `after`: covered by a role after and not before, not cancelled, and without an absence of the
/// player. Sorted by date.
pub fn now_there<'a>(
    player: PlayerId,
    before: &[PlayerRole],
    after: &[PlayerRole],
    trainings: &'a [Training],
    absences: &[Absence],
) -> Vec<&'a Training> {
    let covered = |roles: &[PlayerRole], date: NaiveDate| role_on(roles, player, date).is_some();
    let mut found: Vec<&Training> = trainings
        .iter()
        .filter(|training| {
            !training.cancelled
                && covered(after, training.date)
                && !covered(before, training.date)
                && !absences
                    .iter()
                    .any(|absence| absence.training == training.id && absence.player == player)
        })
        .collect();
    found.sort_by_key(|training| training.date);
    found
}

/// Checks a mark of `player` as absent at `training` with `reason`: the player needs a role on the
/// date, and a hidden reason is allowed only when the absence already has it (`current`).
pub fn check_mark(
    player: PlayerId,
    roles: &[PlayerRole],
    training: &Training,
    reason: ReasonId,
    reason_hidden: bool,
    current: Option<ReasonId>,
) -> Result<(), AttendanceError> {
    if role_on(roles, player, training.date).is_none() {
        return Err(AttendanceError::NotInTeam);
    }
    if reason_hidden && current != Some(reason) {
        return Err(AttendanceError::ReasonHidden);
    }
    Ok(())
}

/// What a player missed: a training or a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissedKind {
    Training,
    Game,
}

/// A training or a game that a player missed and that counts; only a game can have no reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Missed {
    pub date: NaiveDate,
    pub kind: MissedKind,
    pub reason: Option<ReasonId>,
}

/// The games of one player in a season: "of N games: there · absent, valid · absent, not valid".
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GameTotals {
    /// The games that count for this player.
    pub games: usize,
    pub there: usize,
    pub absent_valid: usize,
    pub absent_not_valid: usize,
}

/// The totals of one player in a season.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerTotals<'a> {
    pub player: &'a Player,
    pub there: usize,
    pub absent_valid: usize,
    pub absent_not_valid: usize,
    /// "of N trainings": the trainings that count for this player.
    pub trainings: usize,
    /// The checked games that were played and count for this player.
    pub games: GameTotals,
    /// Newest first; on one date the training before the game.
    pub missed: Vec<Missed>,
}

/// The overview of a season on the page "Attendance".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeasonOverview<'a> {
    /// The trainings of the season up to today that are not cancelled. With `games` 0 too: "No
    /// trainings or games yet this season."
    pub trainings: usize,
    /// The games of the season that count (`counted_games`). With `trainings` 0 too: "No trainings
    /// or games yet this season."
    pub games: usize,
    /// A role on the day of the overview or later in the season.
    pub team: Vec<PlayerTotals<'a>>,
    /// Roles earlier in the season, none on the day of the overview or later.
    pub left: Vec<PlayerTotals<'a>>,
    /// Training days before today without a training.
    pub not_opened: usize,
    /// The oldest of them.
    pub first_not_opened: Option<NaiveDate>,
}

/// The overview of `season` on `today` (decision 0020). A training counts for a player when it is
/// not cancelled, its date is today or earlier, and the player has a role on that date; "there" is
/// such a training without an absence of the player. Absences elsewhere are kept and do not count.
/// A player gets a row only when at least one training counts for it; hidden players too. The
/// groups follow the page "Team" (`place`) on today, or on the season's last day when the season
/// is over. `roles` holds at least the roles of `season`, `weekly` its weekly trainings; `valid`
/// is the setting of each reason (a reason not in it counts as not valid). `games` are the games
/// that count (`counted_games`): a game counts for a player on its list with an answer
/// ("coming": there; "not coming": absent, not valid without a reason); "no answer" does not
/// count. A player gets a row when a training or a game counts for it.
#[allow(clippy::too_many_arguments)]
pub fn season_overview<'a>(
    season: Season,
    players: &'a [Player],
    roles: &[PlayerRole],
    weekly: &[WeeklyTraining],
    trainings: &[Training],
    absences: &[Absence],
    valid: &[(ReasonId, bool)],
    games: &[CountedGame],
    today: NaiveDate,
) -> SeasonOverview<'a> {
    let counted: Vec<&Training> = trainings
        .iter()
        .filter(|training| {
            Season::of(training.date) == season && training.date <= today && !training.cancelled
        })
        .collect();
    let is_valid = |reason: ReasonId| valid.iter().any(|&(id, is_valid)| id == reason && is_valid);
    let day = today.min(season.last_day());

    let mut sorted: Vec<&Player> = players.iter().collect();
    sorted.sort_by_cached_key(|player| player.sort_key());
    let mut overview = SeasonOverview {
        trainings: counted.len(),
        games: games.len(),
        team: Vec::new(),
        left: Vec::new(),
        not_opened: 0,
        first_not_opened: None,
    };
    for player in sorted {
        let mut totals = PlayerTotals {
            player,
            there: 0,
            absent_valid: 0,
            absent_not_valid: 0,
            trainings: 0,
            games: GameTotals::default(),
            missed: Vec::new(),
        };
        for training in counted
            .iter()
            .filter(|training| role_on(roles, player.id, training.date).is_some())
        {
            totals.trainings += 1;
            let absence = absences
                .iter()
                .find(|absence| absence.training == training.id && absence.player == player.id);
            match absence {
                None => totals.there += 1,
                Some(absence) => {
                    if is_valid(absence.reason) {
                        totals.absent_valid += 1;
                    } else {
                        totals.absent_not_valid += 1;
                    }
                    totals.missed.push(Missed {
                        date: training.date,
                        kind: MissedKind::Training,
                        reason: Some(absence.reason),
                    });
                }
            }
        }
        for game in games {
            let Some((answer, reason)) = game
                .list
                .iter()
                .find(|row| row.player == player.id)
                .and_then(|row| row.answer.map(|answer| (answer, row.reason)))
            else {
                continue;
            };
            totals.games.games += 1;
            match answer {
                Answer::Coming => totals.games.there += 1,
                Answer::NotComing => {
                    if reason.is_some_and(is_valid) {
                        totals.games.absent_valid += 1;
                    } else {
                        totals.games.absent_not_valid += 1;
                    }
                    totals.missed.push(Missed {
                        date: game.date,
                        kind: MissedKind::Game,
                        reason,
                    });
                }
            }
        }
        if totals.trainings == 0 && totals.games.games == 0 {
            continue;
        }
        // Newest first; on one date the training before the game.
        totals.missed.sort_by_key(|missed| {
            (
                std::cmp::Reverse(missed.date),
                missed.kind == MissedKind::Game,
            )
        });
        match place(roles, player.id, season, day).group {
            Group::Team => overview.team.push(totals),
            Group::Left => overview.left.push(totals),
            // No role in the season, so nothing counted: not reached.
            Group::LastSeason | Group::Other => {}
        }
    }

    let list = season_list(season, weekly, trainings, today);
    let not_opened = not_opened(&list);
    overview.not_opened = not_opened.len();
    overview.first_not_opened = not_opened.first().copied();
    overview
}

#[cfg(test)]
mod tests {
    use chrono::NaiveTime;

    use super::*;
    use crate::contribution::game_attendance::{StoredAnswer, game_list};
    use crate::shared::season::Season;
    use crate::team::players::RoleId;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    fn player(id: i64, first_name: &str, hidden: bool) -> Player {
        Player {
            id: PlayerId(id),
            first_name: first_name.to_owned(),
            last_name: None,
            hidden,
        }
    }

    fn role(id: i64, player: i64, from: NaiveDate, until: Option<NaiveDate>) -> PlayerRole {
        PlayerRole {
            id: RoleId(id),
            player: PlayerId(player),
            season: Season::from_start_year(2026),
            role: Role::PlaysGames,
            from,
            until,
        }
    }

    fn training(id: i64, on: NaiveDate) -> Training {
        Training {
            id: TrainingId(id),
            date: on,
            start: NaiveTime::from_hms_opt(19, 45, 0).expect("valid time"),
            end: NaiveTime::from_hms_opt(21, 0, 0).expect("valid time"),
            cancelled: false,
            checked: false,
        }
    }

    fn absence(training: i64, player: i64, reason: i64) -> Absence {
        Absence {
            training: TrainingId(training),
            player: PlayerId(player),
            reason: ReasonId(reason),
        }
    }

    #[test]
    fn the_players_at_a_training_are_those_with_a_role_on_its_date() {
        let players = [
            player(1, "dana", false),
            player(2, "Blake", true),
            player(3, "Avery", false),
            player(4, "Casey", false),
        ];
        let roles = [
            role(1, 1, date(2026, 9, 1), None),
            // A hidden player in the team is at the training.
            PlayerRole {
                role: Role::OnlyTrains,
                ..role(2, 2, date(2026, 9, 1), None)
            },
            role(3, 3, date(2026, 9, 1), None),
            // Casey left before the training.
            role(4, 4, date(2026, 9, 1), Some(date(2026, 9, 30))),
        ];
        let tuesday = training(1, date(2026, 10, 6));
        let absences = [absence(1, 3, 7), absence(2, 1, 7)];
        let at = players_at(&players, &roles, &absences, &tuesday);
        let shown: Vec<(&str, Role, Option<ReasonId>)> = at
            .iter()
            .map(|entry| (entry.player.first_name.as_str(), entry.role, entry.absence))
            .collect();
        assert_eq!(
            shown,
            [
                ("Avery", Role::PlaysGames, Some(ReasonId(7))),
                ("Blake", Role::OnlyTrains, None),
                ("dana", Role::PlaysGames, None),
            ]
        );
    }

    #[test]
    fn a_role_moved_earlier_counts_at_the_trainings_it_now_covers() {
        let before = [role(1, 1, date(2026, 10, 12), None)];
        let after = [role(1, 1, date(2026, 9, 1), None)];
        let mut cancelled = training(4, date(2026, 9, 22));
        cancelled.cancelled = true;
        let trainings = [
            training(3, date(2026, 9, 15)),
            training(1, date(2026, 9, 1)),
            training(2, date(2026, 9, 8)),
            cancelled,
            training(5, date(2026, 9, 29)),
            training(6, date(2026, 10, 13)),
        ];
        // An absence on 29 Sep: that training does not count as "there".
        let absences = [absence(5, 1, 1)];
        let dates: Vec<NaiveDate> = now_there(PlayerId(1), &before, &after, &trainings, &absences)
            .iter()
            .map(|training| training.date)
            .collect();
        assert_eq!(
            dates,
            [date(2026, 9, 1), date(2026, 9, 8), date(2026, 9, 15)]
        );
        // The same coverage: nothing new.
        assert!(now_there(PlayerId(1), &after, &after, &trainings, &absences).is_empty());
        // Another player's roles do not count.
        assert!(now_there(PlayerId(2), &before, &after, &trainings, &absences).is_empty());
    }

    #[test]
    fn a_mark_is_checked() {
        let roles = [role(1, 1, date(2026, 9, 1), Some(date(2026, 9, 30)))];
        let check = |on, hidden, current| {
            check_mark(
                PlayerId(1),
                &roles,
                &training(1, on),
                ReasonId(2),
                hidden,
                current,
            )
        };
        assert_eq!(check(date(2026, 9, 15), false, None), Ok(()));
        assert_eq!(
            check(date(2026, 10, 6), false, None),
            Err(AttendanceError::NotInTeam)
        );
        assert_eq!(
            check(date(2026, 9, 15), true, None),
            Err(AttendanceError::ReasonHidden)
        );
        assert_eq!(
            check(date(2026, 9, 15), true, Some(ReasonId(3))),
            Err(AttendanceError::ReasonHidden)
        );
        // The absence already has the hidden reason.
        assert_eq!(check(date(2026, 9, 15), true, Some(ReasonId(2))), Ok(()));
    }

    // The overview: season 2026-2027, a Tuesday weekly training Tue 1 Sep 2026 – Tue 25 May 2027;
    // reason 1 is School (valid), reason 2 Holiday (not valid).
    const SCHOOL: i64 = 1;
    const HOLIDAY: i64 = 2;

    fn tuesdays() -> Vec<WeeklyTraining> {
        vec![WeeklyTraining {
            id: crate::events::training::WeeklyTrainingId(1),
            season: Season::from_start_year(2026),
            weekday: chrono::Weekday::Tue,
            start: NaiveTime::from_hms_opt(19, 45, 0).expect("valid time"),
            end: NaiveTime::from_hms_opt(21, 0, 0).expect("valid time"),
            from: date(2026, 9, 1),
            until: date(2027, 5, 25),
        }]
    }

    fn reasons(school_valid: bool) -> Vec<(ReasonId, bool)> {
        vec![(ReasonId(SCHOOL), school_valid), (ReasonId(HOLIDAY), false)]
    }

    /// Trainings with ids 1, 2, … on these dates.
    fn trainings_on(dates: &[NaiveDate]) -> Vec<Training> {
        dates
            .iter()
            .zip(1..)
            .map(|(&on, id)| training(id, on))
            .collect()
    }

    fn overview<'a>(
        players: &'a [Player],
        roles: &[PlayerRole],
        trainings: &[Training],
        absences: &[Absence],
        today: NaiveDate,
    ) -> SeasonOverview<'a> {
        season_overview(
            Season::of(today.min(date(2027, 6, 30))),
            players,
            roles,
            &tuesdays(),
            trainings,
            absences,
            &reasons(true),
            &[],
            today,
        )
    }

    /// (first name, there, valid, not valid, of N) of each row.
    fn counts<'a>(rows: &'a [PlayerTotals]) -> Vec<(&'a str, usize, usize, usize, usize)> {
        rows.iter()
            .map(|row| {
                (
                    row.player.first_name.as_str(),
                    row.there,
                    row.absent_valid,
                    row.absent_not_valid,
                    row.trainings,
                )
            })
            .collect()
    }

    fn september() -> Vec<Training> {
        trainings_on(&[
            date(2026, 9, 1),
            date(2026, 9, 8),
            date(2026, 9, 15),
            date(2026, 9, 22),
        ])
    }

    #[test]
    fn the_totals_count_the_trainings_and_the_absences() {
        let players = [player(1, "Avery", false)];
        let roles = [role(1, 1, date(2026, 9, 1), None)];
        let absences = [absence(3, 1, SCHOOL)];
        let result = overview(
            &players,
            &roles,
            &september(),
            &absences,
            date(2026, 10, 13),
        );
        assert_eq!(counts(&result.team), [("Avery", 3, 1, 0, 4)]);
        assert_eq!(
            result.team[0].missed,
            [Missed {
                date: date(2026, 9, 15),
                kind: MissedKind::Training,
                reason: Some(ReasonId(SCHOOL))
            }]
        );
        assert_eq!(result.trainings, 4);
        assert!(result.left.is_empty());
    }

    #[test]
    fn a_player_who_joined_later_has_no_missed_trainings_before() {
        let players = [player(1, "Avery", false)];
        let roles = [role(1, 1, date(2026, 11, 3), None)];
        let trainings = trainings_on(&[
            date(2026, 10, 20),
            date(2026, 10, 27),
            date(2026, 11, 3),
            date(2026, 11, 10),
        ]);
        let result = overview(&players, &roles, &trainings, &[], date(2026, 11, 10));
        assert_eq!(counts(&result.team), [("Avery", 2, 0, 0, 2)]);
        assert!(result.team[0].missed.is_empty());
    }

    #[test]
    fn a_cancelled_training_does_not_count_and_counts_again_after_undo() {
        let players = [player(1, "Avery", false)];
        let roles = [role(1, 1, date(2026, 9, 1), None)];
        let absences = [absence(2, 1, HOLIDAY)];
        let mut trainings = september();
        trainings[1].cancelled = true;
        let result = overview(&players, &roles, &trainings, &absences, date(2026, 10, 13));
        assert_eq!(counts(&result.team), [("Avery", 3, 0, 0, 3)]);
        assert!(result.team[0].missed.is_empty());
        assert_eq!(result.trainings, 3);
        // "Undo cancel".
        trainings[1].cancelled = false;
        let result = overview(&players, &roles, &trainings, &absences, date(2026, 10, 13));
        assert_eq!(counts(&result.team), [("Avery", 3, 0, 1, 4)]);
    }

    #[test]
    fn an_absence_noted_in_advance_does_not_count_yet() {
        let players = [player(1, "Avery", false)];
        let roles = [role(1, 1, date(2026, 9, 1), None)];
        let trainings = trainings_on(&[date(2026, 10, 6), date(2026, 10, 20)]);
        let absences = [absence(2, 1, HOLIDAY)];
        let result = overview(&players, &roles, &trainings, &absences, date(2026, 10, 13));
        assert_eq!(counts(&result.team), [("Avery", 1, 0, 0, 1)]);
        assert!(result.team[0].missed.is_empty());
    }

    #[test]
    fn a_training_today_counts() {
        let players = [player(1, "Avery", false)];
        let roles = [role(1, 1, date(2026, 9, 1), None)];
        let trainings = trainings_on(&[date(2026, 10, 13)]);
        let result = overview(&players, &roles, &trainings, &[], date(2026, 10, 13));
        assert_eq!(counts(&result.team), [("Avery", 1, 0, 0, 1)]);
        assert_eq!(result.trainings, 1);
    }

    #[test]
    fn a_reason_set_to_not_valid_moves_the_count() {
        let players = [player(1, "Avery", false)];
        let roles = [role(1, 1, date(2026, 9, 1), None)];
        let trainings = september();
        let absences = [absence(3, 1, SCHOOL)];
        let result = season_overview(
            Season::from_start_year(2026),
            &players,
            &roles,
            &tuesdays(),
            &trainings,
            &absences,
            &reasons(false),
            &[],
            date(2026, 10, 13),
        );
        assert_eq!(counts(&result.team), [("Avery", 3, 0, 1, 4)]);
    }

    #[test]
    fn an_absence_outside_the_role_is_kept_but_does_not_count() {
        let players = [player(1, "Avery", false)];
        // The role ends on 14 Sep; the absences on 15 and 22 Sep stay in the data.
        let roles = [role(1, 1, date(2026, 9, 1), Some(date(2026, 9, 14)))];
        let absences = [absence(3, 1, SCHOOL), absence(4, 1, HOLIDAY)];
        let result = overview(
            &players,
            &roles,
            &september(),
            &absences,
            date(2026, 10, 13),
        );
        assert_eq!(counts(&result.left), [("Avery", 2, 0, 0, 2)]);
        assert!(result.left[0].missed.is_empty());
    }

    #[test]
    fn a_player_after_change_role_is_counted_once_in_the_team() {
        let players = [player(1, "Avery", false)];
        let roles = [
            PlayerRole {
                role: Role::OnlyTrains,
                ..role(1, 1, date(2026, 9, 1), Some(date(2026, 10, 31)))
            },
            role(2, 1, date(2026, 11, 1), None),
        ];
        let trainings = trainings_on(&[date(2026, 10, 27), date(2026, 11, 3), date(2026, 11, 10)]);
        let absences = [absence(1, 1, SCHOOL), absence(3, 1, HOLIDAY)];
        let result = overview(&players, &roles, &trainings, &absences, date(2026, 11, 10));
        assert_eq!(counts(&result.team), [("Avery", 1, 1, 1, 3)]);
        assert!(result.left.is_empty());
        // Newest first.
        let dates: Vec<NaiveDate> = result.team[0].missed.iter().map(|m| m.date).collect();
        assert_eq!(dates, [date(2026, 11, 10), date(2026, 10, 27)]);
    }

    #[test]
    fn a_player_who_left_is_in_left_the_team_and_one_who_came_back_is_in_the_team() {
        let players = [player(1, "Avery", false), player(2, "Blake", false)];
        let roles = [
            role(1, 1, date(2026, 9, 1), Some(date(2026, 9, 30))),
            role(2, 2, date(2026, 9, 1), Some(date(2026, 9, 30))),
            role(3, 2, date(2026, 11, 1), None),
        ];
        let trainings = trainings_on(&[
            date(2026, 9, 1),
            date(2026, 9, 8),
            date(2026, 10, 6),
            date(2026, 10, 13),
            date(2026, 11, 3),
            date(2026, 11, 10),
        ]);
        let result = overview(&players, &roles, &trainings, &[], date(2026, 11, 10));
        assert_eq!(counts(&result.left), [("Avery", 2, 0, 0, 2)]);
        // October does not count for Blake.
        assert_eq!(counts(&result.team), [("Blake", 4, 0, 0, 4)]);
    }

    #[test]
    fn a_row_needs_a_training_that_counts_and_hidden_players_have_rows() {
        let players = [player(1, "Avery", false), player(2, "Blake", true)];
        let roles = [
            // From next week.
            role(1, 1, date(2026, 10, 20), None),
            role(2, 2, date(2026, 9, 1), None),
        ];
        let result = overview(&players, &roles, &september(), &[], date(2026, 10, 13));
        assert_eq!(counts(&result.team), [("Blake", 4, 0, 0, 4)]);
    }

    #[test]
    fn an_earlier_season_groups_on_its_last_day() {
        let players = [player(1, "Avery", false), player(2, "Blake", false)];
        let roles = [
            role(1, 1, date(2026, 9, 1), None),
            role(2, 2, date(2026, 9, 1), Some(date(2027, 3, 31))),
        ];
        let result = overview(&players, &roles, &september(), &[], date(2027, 7, 15));
        assert_eq!(result.trainings, 4);
        assert_eq!(counts(&result.team), [("Avery", 4, 0, 0, 4)]);
        assert_eq!(counts(&result.left), [("Blake", 4, 0, 0, 4)]);
    }

    #[test]
    fn an_opened_training_shows_its_absent_players_with_a_role() {
        let mut cancelled = training(3, date(2026, 9, 22));
        cancelled.cancelled = true;
        let trainings = [
            training(1, date(2026, 10, 6)),
            training(2, date(2026, 9, 29)),
            cancelled,
            training(4, date(2026, 10, 13)),
        ];
        let roles = [
            role(1, 1, date(2026, 9, 1), None),
            role(2, 2, date(2026, 9, 1), None),
        ];
        // Player 3 has no role: its absence is kept but not counted.
        let absences = [
            absence(1, 1, SCHOOL),
            absence(1, 2, SCHOOL),
            absence(1, 3, SCHOOL),
            absence(3, 1, SCHOOL),
            absence(4, 2, HOLIDAY),
        ];
        let weekly = tuesdays();
        let list = season_list(
            Season::from_start_year(2026),
            &weekly,
            &trainings,
            date(2026, 10, 12),
        );
        let status = |on| {
            list.past
                .iter()
                .chain(&list.coming)
                .find(|row| row.date == on)
                .map(|row| training_state(row, &absences, &roles))
        };
        assert_eq!(status(date(2026, 10, 6)), Some(TrainingState::Absent(2)));
        assert_eq!(status(date(2026, 9, 29)), Some(TrainingState::AllThere));
        assert_eq!(status(date(2026, 9, 22)), Some(TrainingState::Cancelled));
        assert_eq!(status(date(2026, 9, 15)), Some(TrainingState::NotOpened));
        // A coming training is counted too.
        assert_eq!(status(date(2026, 10, 13)), Some(TrainingState::Absent(1)));
    }

    #[test]
    fn the_training_days_not_opened_are_those_before_today() {
        let players = [player(1, "Avery", false)];
        let roles = [role(1, 1, date(2026, 9, 1), None)];
        let mut trainings = trainings_on(&[date(2026, 9, 1), date(2026, 9, 15), date(2026, 9, 29)]);
        // A cancelled training day is not "not opened".
        trainings[2].cancelled = true;
        let result = overview(&players, &roles, &trainings, &[], date(2026, 10, 13));
        // 8 Sep, 22 Sep and 6 Oct; today's 13 Oct is coming.
        assert_eq!(result.not_opened, 3);
        assert_eq!(result.first_not_opened, Some(date(2026, 9, 8)));
        let all = trainings_on(&[
            date(2026, 9, 1),
            date(2026, 9, 8),
            date(2026, 9, 15),
            date(2026, 9, 22),
            date(2026, 9, 29),
            date(2026, 10, 6),
        ]);
        let result = overview(&players, &roles, &all, &[], date(2026, 10, 13));
        assert_eq!(result.not_opened, 0);
        assert_eq!(result.first_not_opened, None);
    }

    #[test]
    fn no_training_counts_yet() {
        let players = [player(1, "Avery", false)];
        let roles = [role(1, 1, date(2026, 9, 1), None)];
        let today = date(2026, 10, 13);
        let mut cancelled = trainings_on(&[date(2026, 10, 6)]);
        cancelled[0].cancelled = true;
        let next_week = trainings_on(&[date(2026, 10, 20)]);
        for trainings in [Vec::new(), cancelled, next_week] {
            let result = overview(&players, &roles, &trainings, &[], today);
            assert_eq!(result.trainings, 0);
            assert!(result.team.is_empty());
            assert!(result.left.is_empty());
        }
    }

    #[test]
    fn the_rows_are_sorted_by_first_name_then_last_name() {
        let players = [
            player(1, "Bob", false),
            Player {
                last_name: Some("de Vries".to_owned()),
                ..player(2, "Anna", false)
            },
            player(3, "anna", false),
        ];
        let roles = [
            role(1, 1, date(2026, 9, 1), None),
            role(2, 2, date(2026, 9, 1), None),
            role(3, 3, date(2026, 9, 1), None),
        ];
        let result = overview(&players, &roles, &september(), &[], date(2026, 10, 13));
        let ids: Vec<i64> = result.team.iter().map(|row| row.player.id.0).collect();
        assert_eq!(ids, [3, 2, 1]);
    }

    #[test]
    fn a_reason_without_a_setting_counts_as_not_valid() {
        let players = [player(1, "Avery", false)];
        let roles = [role(1, 1, date(2026, 9, 1), None)];
        let absences = [absence(1, 1, 99)];
        let result = overview(
            &players,
            &roles,
            &september(),
            &absences,
            date(2026, 10, 13),
        );
        assert_eq!(counts(&result.team), [("Avery", 3, 0, 1, 4)]);
    }

    // The games in the totals (story D of attendance at games).

    fn said(player: i64, answer: Answer, reason: Option<i64>) -> StoredAnswer {
        StoredAnswer {
            player: PlayerId(player),
            answer,
            reason: reason.map(ReasonId),
            settled: false,
        }
    }

    /// A counted game on `on` with the list of its page (`game_list`).
    fn game_on(
        players: &[Player],
        roles: &[PlayerRole],
        on: NaiveDate,
        answers: Vec<StoredAnswer>,
    ) -> CountedGame {
        CountedGame {
            date: on,
            list: game_list(players, roles, &answers, on),
        }
    }

    fn with_games<'a>(
        players: &'a [Player],
        roles: &[PlayerRole],
        trainings: &[Training],
        games: &[CountedGame],
    ) -> SeasonOverview<'a> {
        season_overview(
            Season::from_start_year(2026),
            players,
            roles,
            &tuesdays(),
            trainings,
            &[],
            &reasons(true),
            games,
            date(2026, 10, 13),
        )
    }

    #[test]
    fn each_answer_at_a_counted_game_counts_by_its_reason() {
        let players = [
            player(1, "Avery", false),
            player(2, "Blake", false),
            player(3, "Casey", false),
            player(4, "Dana", false),
            player(5, "Emery", false),
        ];
        let roles: Vec<PlayerRole> = (1..=5)
            .map(|id| role(id, id, date(2026, 9, 1), None))
            .collect();
        let games = [game_on(
            &players,
            &roles,
            date(2026, 10, 10),
            vec![
                said(1, Answer::Coming, None),
                said(2, Answer::NotComing, Some(SCHOOL)),
                said(3, Answer::NotComing, Some(HOLIDAY)),
                // Without a reason, as "Checked" makes it: not valid.
                StoredAnswer {
                    settled: true,
                    ..said(4, Answer::NotComing, None)
                },
            ],
        )];
        let result = with_games(&players, &roles, &[], &games);
        let totals = |at: usize| {
            (
                result.team[at].player.first_name.as_str(),
                result.team[at].games,
            )
        };
        let of = |there, absent_valid, absent_not_valid| GameTotals {
            games: 1,
            there,
            absent_valid,
            absent_not_valid,
        };
        // Emery has no answer: no game counts, and no training either, so no row.
        assert_eq!(
            (0..result.team.len()).map(totals).collect::<Vec<_>>(),
            [
                ("Avery", of(1, 0, 0)),
                ("Blake", of(0, 1, 0)),
                ("Casey", of(0, 0, 1)),
                ("Dana", of(0, 0, 1)),
            ]
        );
        // A player with games and no trainings has a row.
        assert_eq!(result.team[0].trainings, 0);
        assert_eq!((result.trainings, result.games), (0, 1));
        assert_eq!(
            result.team[3].missed,
            [Missed {
                date: date(2026, 10, 10),
                kind: MissedKind::Game,
                reason: None
            }]
        );
    }

    #[test]
    fn a_game_counts_only_for_a_player_on_its_list() {
        let players = [
            player(1, "Avery", false),
            player(2, "Blake", false),
            player(3, "Casey", false),
        ];
        let roles = [
            // Plays games from 11 October: not on the list of 10 October.
            role(1, 1, date(2026, 10, 11), None),
            // Only trains.
            PlayerRole {
                role: Role::OnlyTrains,
                ..role(2, 2, date(2026, 9, 1), None)
            },
            role(3, 3, date(2026, 9, 1), None),
        ];
        let games = [game_on(
            &players,
            &roles,
            date(2026, 10, 10),
            vec![
                said(1, Answer::Coming, None),
                said(2, Answer::Coming, None),
                said(3, Answer::Coming, None),
            ],
        )];
        let result = with_games(&players, &roles, &[], &games);
        let names: Vec<&str> = result
            .team
            .iter()
            .map(|row| row.player.first_name.as_str())
            .collect();
        assert_eq!(names, ["Casey"]);
    }

    #[test]
    fn the_missed_list_has_trainings_and_games_newest_first() {
        let players = [player(1, "Avery", false)];
        // The role ends on 11 October: Avery is under "Left the team", with the games.
        let roles = [role(1, 1, date(2026, 9, 1), Some(date(2026, 10, 11)))];
        let trainings = trainings_on(&[date(2026, 9, 29), date(2026, 10, 6)]);
        let absences = [absence(2, 1, SCHOOL)];
        let games = [
            game_on(
                &players,
                &roles,
                date(2026, 10, 3),
                vec![said(1, Answer::NotComing, Some(HOLIDAY))],
            ),
            game_on(
                &players,
                &roles,
                date(2026, 10, 6),
                vec![said(1, Answer::NotComing, None)],
            ),
            game_on(
                &players,
                &roles,
                date(2026, 10, 10),
                vec![said(1, Answer::Coming, None)],
            ),
        ];
        let result = season_overview(
            Season::from_start_year(2026),
            &players,
            &roles,
            &tuesdays(),
            &trainings,
            &absences,
            &reasons(true),
            &games,
            date(2026, 10, 13),
        );
        assert!(result.team.is_empty());
        let left = &result.left[0];
        assert_eq!(
            left.games,
            GameTotals {
                games: 3,
                there: 1,
                absent_valid: 0,
                absent_not_valid: 2
            }
        );
        let missed: Vec<(NaiveDate, MissedKind)> =
            left.missed.iter().map(|m| (m.date, m.kind)).collect();
        assert_eq!(
            missed,
            [
                (date(2026, 10, 6), MissedKind::Training),
                (date(2026, 10, 6), MissedKind::Game),
                (date(2026, 10, 3), MissedKind::Game),
            ]
        );
    }
}
