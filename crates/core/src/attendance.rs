//! Attendance at training: who is at a training, and the absences with their reason. "There" is
//! not stored: a player is there when the training exists, the player has a role on its date, and
//! there is no absence (decision 0020).

use chrono::NaiveDate;

use crate::players::{Player, PlayerId, PlayerRole, Role, role_on};
use crate::reasons::ReasonId;
use crate::training::{Training, TrainingId};

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
    at.sort_by_key(|entry| {
        (
            entry.player.first_name.to_lowercase(),
            entry.player.last_name.as_deref().map(str::to_lowercase),
            entry.player.id,
        )
    });
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

#[cfg(test)]
mod tests {
    use chrono::NaiveTime;

    use super::*;
    use crate::players::RoleId;
    use crate::season::Season;

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
}
