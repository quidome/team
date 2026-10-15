//! Players and their roles. A player is the person and has no dates; a role ("plays games" or
//! "only trains") in the team of one season has a start and an end. Both roles come to training.

use chrono::NaiveDate;

use crate::season::Season;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlayerId(pub i64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    pub id: PlayerId,
    pub first_name: String,
    pub last_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    PlaysGames,
    OnlyTrains,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RoleId(pub i64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerRole {
    pub id: RoleId,
    pub player: PlayerId,
    pub season: Season,
    pub role: Role,
    pub from: NaiveDate,
    /// `None` while the role continues.
    pub until: Option<NaiveDate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PlayerError {
    #[error("a first name is needed")]
    FirstNameRequired,
    #[error("the end of a role is before its start")]
    UntilBeforeFrom,
    #[error("a date of the role is outside its season")]
    OutsideSeason,
    #[error("the player already has a role on that date")]
    RolesOverlap,
}

/// The names without outer spaces; an empty last name is no last name.
pub fn clean_name(
    first: &str,
    last: Option<&str>,
) -> Result<(String, Option<String>), PlayerError> {
    let first = first.trim();
    if first.is_empty() {
        return Err(PlayerError::FirstNameRequired);
    }
    let last = last.map(str::trim).filter(|last| !last.is_empty());
    Ok((first.to_owned(), last.map(str::to_owned)))
}

/// The role of `player` on `date`, if any. Roles of one player never overlap, so there is at most
/// one.
pub fn role_on(roles: &[PlayerRole], player: PlayerId, date: NaiveDate) -> Option<&PlayerRole> {
    roles
        .iter()
        .find(|role| role.player == player && covers(role.from, role.until, date))
}

/// Checks a role of `season` from `from` until `until` against `others`, the player's other roles
/// (without the role that is changed).
pub fn check_role(
    season: Season,
    from: NaiveDate,
    until: Option<NaiveDate>,
    others: &[PlayerRole],
) -> Result<(), PlayerError> {
    if until.is_some_and(|until| until < from) {
        return Err(PlayerError::UntilBeforeFrom);
    }
    if Season::of(from) != season || until.is_some_and(|until| Season::of(until) != season) {
        return Err(PlayerError::OutsideSeason);
    }
    let overlaps = |other: &PlayerRole| {
        other.until.is_none_or(|other_until| from <= other_until)
            && until.is_none_or(|until| other.from <= until)
    };
    if others.iter().any(overlaps) {
        return Err(PlayerError::RolesOverlap);
    }
    Ok(())
}

/// "Add all": the players to bring into `season`, each with the kind of role it had at the end of
/// the previous season. A player counts when its newest role of the previous season did not end
/// before 30 June, and it has no role in `season` yet. `roles` holds the roles of both seasons.
pub fn roles_to_carry(roles: &[PlayerRole], season: Season) -> Vec<(PlayerId, Role)> {
    let previous = Season::from_start_year(season.start_year() - 1);
    let season_end = NaiveDate::from_ymd_opt(season.start_year(), 6, 30);
    let mut carried: Vec<(PlayerId, Role)> = Vec::new();
    let mut players: Vec<PlayerId> = roles.iter().map(|role| role.player).collect();
    players.sort();
    players.dedup();
    for player in players {
        let of = |wanted: Season| {
            roles
                .iter()
                .filter(move |role| role.player == player && role.season == wanted)
        };
        if of(season).next().is_some() {
            continue;
        }
        let Some(newest) = of(previous).max_by_key(|role| role.from) else {
            continue;
        };
        let stayed = match (newest.until, season_end) {
            (None, _) => true,
            (Some(until), Some(end)) => until >= end,
            (Some(_), None) => false,
        };
        if stayed {
            carried.push((player, newest.role));
        }
    }
    carried
}

fn covers(from: NaiveDate, until: Option<NaiveDate>, date: NaiveDate) -> bool {
    from <= date && until.is_none_or(|until| date <= until)
}

#[cfg(test)]
mod tests {
    use super::*;

    const AVERY: PlayerId = PlayerId(1);

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    fn season() -> Season {
        Season::from_start_year(2026)
    }

    fn role(id: i64, from: NaiveDate, until: Option<NaiveDate>) -> PlayerRole {
        PlayerRole {
            id: RoleId(id),
            player: AVERY,
            season: season(),
            role: Role::PlaysGames,
            from,
            until,
        }
    }

    #[test]
    fn names_are_trimmed_and_an_empty_last_name_is_none() {
        assert_eq!(
            clean_name("  Avery ", Some(" Jansen ")),
            Ok(("Avery".to_owned(), Some("Jansen".to_owned())))
        );
        assert_eq!(
            clean_name("Casey", Some("  ")),
            Ok(("Casey".to_owned(), None))
        );
        assert_eq!(
            clean_name("Casey", Some("")),
            Ok(("Casey".to_owned(), None))
        );
        assert_eq!(clean_name("Casey", None), Ok(("Casey".to_owned(), None)));
    }

    #[test]
    fn a_first_name_is_needed() {
        assert_eq!(clean_name("", None), Err(PlayerError::FirstNameRequired));
        assert_eq!(
            clean_name("   ", Some("Jansen")),
            Err(PlayerError::FirstNameRequired)
        );
    }

    #[test]
    fn an_open_role_covers_every_day_from_its_start() {
        let roles = [role(1, date(2026, 9, 1), None)];
        assert!(role_on(&roles, AVERY, date(2026, 8, 31)).is_none());
        assert!(role_on(&roles, AVERY, date(2026, 9, 1)).is_some());
        assert!(role_on(&roles, AVERY, date(2026, 10, 12)).is_some());
        assert!(role_on(&roles, PlayerId(2), date(2026, 10, 12)).is_none());
    }

    #[test]
    fn a_role_covers_its_last_day() {
        let roles = [role(1, date(2026, 9, 1), Some(date(2026, 11, 15)))];
        assert!(role_on(&roles, AVERY, date(2026, 11, 15)).is_some());
        assert!(role_on(&roles, AVERY, date(2026, 11, 16)).is_none());
    }

    #[test]
    fn the_time_between_two_roles_has_no_role() {
        let roles = [
            role(1, date(2026, 9, 1), Some(date(2026, 11, 15))),
            role(2, date(2027, 1, 10), None),
        ];
        assert!(role_on(&roles, AVERY, date(2026, 12, 1)).is_none());
        assert_eq!(
            role_on(&roles, AVERY, date(2027, 1, 10)).map(|role| role.id),
            Some(RoleId(2))
        );
    }

    #[test]
    fn add_all_brings_the_players_who_stayed_until_the_end_of_last_season() {
        let last = Season::from_start_year(2025);
        let mut roles: Vec<PlayerRole> = (1..=14)
            .map(|n| PlayerRole {
                id: RoleId(n),
                player: PlayerId(n),
                season: last,
                role: if n <= 10 {
                    Role::PlaysGames
                } else {
                    Role::OnlyTrains
                },
                from: date(2025, 9, 2),
                until: None,
            })
            .collect();
        // One left in March; one stayed until the last day of the season.
        roles[2].until = Some(date(2026, 3, 10));
        roles[3].until = Some(date(2026, 6, 30));
        // One already has a role in the new season.
        roles.push(PlayerRole {
            id: RoleId(100),
            player: PlayerId(5),
            season: season(),
            role: Role::PlaysGames,
            from: date(2026, 9, 1),
            until: None,
        });
        let carried = roles_to_carry(&roles, season());
        assert_eq!(carried.len(), 12);
        assert!(!carried.iter().any(|(player, _)| *player == PlayerId(3)));
        assert!(!carried.iter().any(|(player, _)| *player == PlayerId(5)));
        assert!(carried.contains(&(PlayerId(4), Role::PlaysGames)));
        assert!(carried.contains(&(PlayerId(12), Role::OnlyTrains)));
    }

    #[test]
    fn add_all_uses_the_newest_role_of_last_season() {
        let last = Season::from_start_year(2025);
        let roles = [
            PlayerRole {
                id: RoleId(1),
                player: AVERY,
                season: last,
                role: Role::OnlyTrains,
                from: date(2025, 9, 2),
                until: Some(date(2026, 1, 31)),
            },
            PlayerRole {
                id: RoleId(2),
                player: AVERY,
                season: last,
                role: Role::PlaysGames,
                from: date(2026, 2, 1),
                until: None,
            },
        ];
        assert_eq!(
            roles_to_carry(&roles, season()),
            [(AVERY, Role::PlaysGames)]
        );
    }

    #[test]
    fn the_end_of_a_role_is_not_before_its_start() {
        assert_eq!(
            check_role(season(), date(2026, 9, 1), Some(date(2026, 8, 31)), &[]),
            Err(PlayerError::UntilBeforeFrom)
        );
        assert_eq!(
            check_role(season(), date(2026, 9, 1), Some(date(2026, 9, 1)), &[]),
            Ok(())
        );
    }

    #[test]
    fn a_role_stays_inside_its_season() {
        assert_eq!(
            check_role(season(), date(2026, 9, 1), Some(date(2027, 6, 30)), &[]),
            Ok(())
        );
        assert_eq!(
            check_role(season(), date(2026, 9, 1), Some(date(2027, 7, 1)), &[]),
            Err(PlayerError::OutsideSeason)
        );
        assert_eq!(
            check_role(season(), date(2026, 6, 30), None, &[]),
            Err(PlayerError::OutsideSeason)
        );
    }

    #[test]
    fn roles_of_one_player_never_overlap() {
        let first = [role(1, date(2026, 9, 1), Some(date(2026, 11, 15)))];
        assert_eq!(
            check_role(season(), date(2026, 11, 15), None, &first),
            Err(PlayerError::RolesOverlap)
        );
        assert_eq!(
            check_role(season(), date(2026, 11, 16), None, &first),
            Ok(())
        );

        let open = [role(1, date(2026, 9, 1), None)];
        assert_eq!(
            check_role(season(), date(2027, 1, 10), None, &open),
            Err(PlayerError::RolesOverlap)
        );
        assert_eq!(
            check_role(season(), date(2026, 8, 1), Some(date(2026, 8, 31)), &open),
            Ok(())
        );
    }
}
