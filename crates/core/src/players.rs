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

impl PlayerRole {
    /// The last day of the role: `until`, or the last day of its season while it continues. A role
    /// never runs into the next season.
    pub fn last_day(&self) -> NaiveDate {
        self.until.unwrap_or_else(|| self.season.last_day())
    }

    fn covers(&self, date: NaiveDate) -> bool {
        self.from <= date && date <= self.last_day()
    }
}

/// The role of `player` on `date`, if any. Roles of one player never overlap, so there is at most
/// one.
pub fn role_on(roles: &[PlayerRole], player: PlayerId, date: NaiveDate) -> Option<&PlayerRole> {
    roles
        .iter()
        .find(|role| role.player == player && role.covers(date))
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
    let last_day = until.unwrap_or_else(|| season.last_day());
    if others
        .iter()
        .any(|other| other.from <= last_day && from <= other.last_day())
    {
        return Err(PlayerError::RolesOverlap);
    }
    Ok(())
}

/// Checks a change of `changed`, a stored role, to a role of `season` from `from` until `until`:
/// only a role of `season` is changed, then as `check_role`.
pub fn check_change(
    season: Season,
    changed: &PlayerRole,
    from: NaiveDate,
    until: Option<NaiveDate>,
    others: &[PlayerRole],
) -> Result<(), PlayerError> {
    if changed.season != season {
        return Err(PlayerError::OutsideSeason);
    }
    check_role(season, from, until, others)
}

/// The proposed start of a new role: the first training date of the season, or today when
/// trainings have started, or today when the season has no training settings.
pub fn proposed_start(first_training: Option<NaiveDate>, today: NaiveDate) -> NaiveDate {
    first_training.map_or(today, |first| first.max(today))
}

/// Where the players page lists a player in a season.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    /// A role today, or one that starts later in the season (for example from the first training
    /// date, after "Add all" in summer).
    Team,
    /// Roles earlier in the season, none today or later.
    Left,
    /// No role in the season, and in the team until the end of last season: "Add all" brings it
    /// back.
    LastSeason,
    /// None of the above.
    Other,
}

/// A player's place on the players page: the group, the role shown with the name, and the roles
/// listed (of the season, or of last season for `LastSeason`), in order of start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placement<'a> {
    pub group: Group,
    pub role: Option<Role>,
    pub roles: Vec<&'a PlayerRole>,
}

/// Places `player` for `season` on `today`. `roles` holds at least the player's roles of `season`
/// and of the season before; roles of other players are ignored.
pub fn place(
    roles: &[PlayerRole],
    player: PlayerId,
    season: Season,
    today: NaiveDate,
) -> Placement<'_> {
    let this = roles_in(roles, player, season);
    let current = this
        .iter()
        .find(|role| role.covers(today))
        .or_else(|| this.iter().find(|role| role.from > today))
        .map(|role| role.role);
    if let Some(role) = current {
        return Placement {
            group: Group::Team,
            role: Some(role),
            roles: this,
        };
    }
    if let Some(newest) = this.last() {
        return Placement {
            group: Group::Left,
            role: Some(newest.role),
            roles: this,
        };
    }
    if let Some(role) = stayed_until_end(roles, player, season.previous()) {
        return Placement {
            group: Group::LastSeason,
            role: Some(role),
            roles: roles_in(roles, player, season.previous()),
        };
    }
    Placement {
        group: Group::Other,
        role: None,
        roles: Vec::new(),
    }
}

/// "Add all": the players to bring into `season`, each with the kind of role it had at the end of
/// the previous season. A player counts when its newest role of the previous season lasted until
/// the season's last day, and it has no role in `season` yet. `roles` holds the roles of both
/// seasons.
pub fn roles_to_carry(roles: &[PlayerRole], season: Season) -> Vec<(PlayerId, Role)> {
    let mut players: Vec<PlayerId> = roles.iter().map(|role| role.player).collect();
    players.sort();
    players.dedup();
    players
        .into_iter()
        .filter(|player| roles_in(roles, *player, season).is_empty())
        .filter_map(|player| {
            stayed_until_end(roles, player, season.previous()).map(|role| (player, role))
        })
        .collect()
}

/// The roles of `player` in `season`, in order of start.
fn roles_in(roles: &[PlayerRole], player: PlayerId, season: Season) -> Vec<&PlayerRole> {
    let mut found: Vec<&PlayerRole> = roles
        .iter()
        .filter(|role| role.player == player && role.season == season)
        .collect();
    found.sort_by_key(|role| role.from);
    found
}

/// The kind of the player's newest role in `season`, when that role lasted until the season's
/// last day.
fn stayed_until_end(roles: &[PlayerRole], player: PlayerId, season: Season) -> Option<Role> {
    roles_in(roles, player, season)
        .last()
        .filter(|newest| newest.last_day() >= season.last_day())
        .map(|newest| newest.role)
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

    fn last_season_role(from: NaiveDate, until: Option<NaiveDate>) -> PlayerRole {
        PlayerRole {
            id: RoleId(50),
            season: Season::from_start_year(2025),
            ..role(50, from, until)
        }
    }

    #[test]
    fn an_open_role_ends_on_the_last_day_of_its_season() {
        let roles = [last_season_role(date(2025, 9, 2), None)];
        assert!(role_on(&roles, AVERY, date(2026, 6, 30)).is_some());
        assert!(role_on(&roles, AVERY, date(2026, 7, 1)).is_none());
        assert!(role_on(&roles, AVERY, date(2026, 10, 12)).is_none());
        assert_eq!(roles[0].last_day(), date(2026, 6, 30));
    }

    #[test]
    fn an_open_role_of_last_season_does_not_overlap_a_role_in_this_season() {
        let last = [last_season_role(date(2025, 9, 2), None)];
        assert_eq!(
            check_role(season(), date(2026, 10, 12), None, &last),
            Ok(())
        );
        assert_eq!(check_role(season(), date(2026, 7, 1), None, &last), Ok(()));
        // "Left the team" on the new role: the other roles are only last season's.
        assert_eq!(
            check_role(season(), date(2026, 10, 12), Some(date(2026, 11, 1)), &last),
            Ok(())
        );
    }

    #[test]
    fn the_proposed_start_is_the_first_training_date_or_today() {
        let first = Some(date(2026, 9, 1));
        assert_eq!(proposed_start(first, date(2026, 8, 15)), date(2026, 9, 1));
        assert_eq!(
            proposed_start(first, date(2026, 10, 12)),
            date(2026, 10, 12)
        );
        assert_eq!(proposed_start(None, date(2026, 8, 15)), date(2026, 8, 15));
    }

    #[test]
    fn a_player_with_a_role_today_or_later_in_the_season_is_in_the_team() {
        // After "Add all" on 15 August, from the first training date.
        let roles = [
            last_season_role(date(2025, 9, 2), None),
            PlayerRole {
                role: Role::OnlyTrains,
                ..role(2, date(2026, 9, 1), None)
            },
        ];
        for today in [
            date(2026, 8, 15),
            date(2026, 9, 1),
            date(2026, 9, 2),
            date(2027, 6, 30),
        ] {
            let placement = place(&roles, AVERY, season(), today);
            assert_eq!(placement.group, Group::Team, "on {today}");
            assert_eq!(placement.role, Some(Role::OnlyTrains));
            assert_eq!(placement.roles, [&roles[1]]);
        }
    }

    #[test]
    fn a_player_added_to_this_season_is_in_the_team_and_not_left() {
        let roles = [
            last_season_role(date(2025, 9, 2), None),
            role(2, date(2026, 10, 12), None),
        ];
        let placement = place(&roles, AVERY, season(), date(2026, 10, 12));
        assert_eq!(placement.group, Group::Team);
        assert_eq!(placement.roles, [&roles[1]]);
    }

    #[test]
    fn a_player_whose_roles_ended_is_left_with_the_newest_role() {
        let roles = [
            PlayerRole {
                role: Role::OnlyTrains,
                ..role(1, date(2026, 9, 1), Some(date(2026, 9, 30)))
            },
            role(2, date(2026, 10, 1), Some(date(2026, 10, 15))),
        ];
        let placement = place(&roles, AVERY, season(), date(2026, 10, 20));
        assert_eq!(placement.group, Group::Left);
        assert_eq!(placement.role, Some(Role::PlaysGames));
        assert_eq!(placement.roles.len(), 2);
    }

    #[test]
    fn a_player_who_stayed_last_season_waits_under_last_season() {
        let roles = [PlayerRole {
            role: Role::OnlyTrains,
            ..last_season_role(date(2025, 9, 2), None)
        }];
        let placement = place(&roles, AVERY, season(), date(2026, 8, 15));
        assert_eq!(placement.group, Group::LastSeason);
        assert_eq!(placement.role, Some(Role::OnlyTrains));
        assert_eq!(placement.roles, [&roles[0]]);
    }

    #[test]
    fn a_player_who_left_early_last_season_or_has_no_role_is_other() {
        let roles = [last_season_role(date(2025, 9, 2), Some(date(2026, 3, 10)))];
        let placement = place(&roles, AVERY, season(), date(2026, 8, 15));
        assert_eq!(placement.group, Group::Other);
        assert_eq!(placement.role, None);
        assert!(placement.roles.is_empty());
        assert_eq!(
            place(&[], AVERY, season(), date(2026, 8, 15)).group,
            Group::Other
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
    fn only_a_role_of_the_season_is_changed() {
        let last = last_season_role(date(2025, 9, 2), None);
        assert_eq!(
            check_change(season(), &last, date(2026, 9, 1), None, &[]),
            Err(PlayerError::OutsideSeason)
        );
        let this = role(1, date(2026, 9, 1), None);
        assert_eq!(
            check_change(
                season(),
                &this,
                date(2026, 9, 1),
                Some(date(2026, 10, 1)),
                &[last]
            ),
            Ok(())
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
