//! Duty places (story A of duties). A game requires duties: at each home game of the club the 4
//! places of the referee committee (referee 1 and 2, table 1 and 2), each given to a club team or
//! empty; at an away game of the team at another club its drivers. `game_places` is the only rule
//! for the places of a game: "Season", the page of a game, the start page and the list of changes
//! all use it (decision 0022).

use std::collections::{BTreeMap, BTreeSet};

use chrono::NaiveDate;

use crate::club::{Club, ClubNumber};
use crate::game::{Game, TeamNumber};
use crate::players::{PlayerRole, Role};
use crate::team_choice::own_teams;

/// A place of the referee committee at a home game of the club.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PlaceKind {
    Referee,
    Table,
}

/// What a player does on a duty place.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DutyKind {
    Drive,
    Table,
    Referee,
}

impl From<PlaceKind> for DutyKind {
    fn from(kind: PlaceKind) -> Self {
        match kind {
            PlaceKind::Referee => Self::Referee,
            PlaceKind::Table => Self::Table,
        }
    }
}

/// A club team on a place: its number, and the short name shown ("M16 1").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamRef {
    pub number: TeamNumber,
    pub name: String,
}

/// The duties a game requires now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequiredDuties {
    /// A home game of the club (of any club team, also of the team): the 4 places.
    pub schedule: bool,
    /// An away game of the team at another club: drivers.
    pub drivers: bool,
}

/// The duties `game` requires, for `team` of `club`.
pub fn required_duties(game: &Game, club: ClubNumber, team: TeamNumber) -> RequiredDuties {
    RequiredDuties {
        schedule: game.home.club == club,
        drivers: game.away.team == team && game.home.club != club,
    }
}

/// The stored duties of a game.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StoredDuties {
    /// "Schedule entered" was tapped.
    pub schedule_entered: bool,
    /// The number of drivers the coordinator set; `None`: the proposal.
    pub drivers: Option<u8>,
    /// The places that name a team; an empty place is not in it.
    pub teams: BTreeMap<(PlaceKind, u8), TeamRef>,
}

/// The 4 places of a home game of the club, in the order shown.
pub const SCHEDULE_PLACES: [(PlaceKind, u8); 4] = [
    (PlaceKind::Referee, 1),
    (PlaceKind::Referee, 2),
    (PlaceKind::Table, 1),
    (PlaceKind::Table, 2),
];

/// The most drivers that can be set.
const MAX_DRIVERS: u8 = 10;

/// One driver for every 3 players, rounded up; at most 10 (the most that can be set).
pub fn driver_proposal(players: usize) -> u8 {
    u8::try_from(players.div_ceil(3))
        .unwrap_or(MAX_DRIVERS)
        .min(MAX_DRIVERS)
}

/// The number of players with the role "plays games" on `date` (the input of the proposal).
pub fn players_playing_on(roles: &[PlayerRole], date: NaiveDate) -> usize {
    roles
        .iter()
        .filter(|role| role.role == Role::PlaysGames && role.covers(date))
        .map(|role| role.player)
        .collect::<BTreeSet<_>>()
        .len()
}

/// The teams of a pull-down list: the club's own teams in `games` (the stored games of a season,
/// also those no longer in the game list), plus the teams that places of that season still name,
/// with their stored names. Sorted by name.
pub fn club_teams(
    club: &Club,
    games: &[Game],
    stored: &BTreeMap<i64, StoredDuties>,
) -> Vec<TeamRef> {
    let mut teams: Vec<TeamRef> = own_teams(club, games)
        .into_iter()
        .map(|team| TeamRef {
            number: team.number,
            name: team.name,
        })
        .collect();
    for named in stored.values().flat_map(|duties| duties.teams.values()) {
        if !teams.iter().any(|team| team.number == named.number) {
            teams.push(named.clone());
        }
    }
    teams.sort_by(|a, b| a.name.cmp(&b.name).then(a.number.cmp(&b.number)));
    teams
}

/// The games that stay on "Season" for their duties, also when they left the game list or are no
/// longer in the schedule: a place names a team, or drivers are set above 0.
pub fn kept_games(stored: &BTreeMap<i64, StoredDuties>) -> BTreeSet<i64> {
    stored
        .iter()
        .filter(|(_, duties)| !duties.teams.is_empty() || duties.drivers.is_some_and(|d| d > 0))
        .map(|(&id, _)| id)
        .collect()
}

/// One place of the referee committee, with its team.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulePlace {
    pub kind: PlaceKind,
    pub number: u8,
    pub team: Option<TeamRef>,
}

/// A duty place of the team.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DutyPlace {
    pub kind: DutyKind,
    /// Referee and table: the place (1 or 2). Drivers: 1, 2, ….
    pub number: u8,
    /// A player is on the place (story B).
    pub filled: bool,
}

/// The places of the referee committee at a game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulePart {
    /// Always 4: referee 1, referee 2, table 1, table 2.
    pub places: Vec<SchedulePlace>,
    pub entered: bool,
    /// The places that name the team, in the order of `places`.
    pub of_team: Vec<DutyPlace>,
    /// False: the game is no longer a home game of the club; the places stay until cleared.
    pub required: bool,
    /// The places count: the game is past, or in the game list and still requires them.
    pub counts: bool,
}

/// The drivers of a game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriverPart {
    pub places: Vec<DutyPlace>,
    pub proposal: u8,
    /// The number is the proposal (none was set).
    pub proposed: bool,
    /// The proposal dropped below the filled driver places, so the number stays at those.
    pub more_than_proposed: bool,
    /// False: the game no longer needs drivers of the team; a set number stays until cleared.
    pub required: bool,
    pub counts: bool,
}

/// The duties of one game, in two parts that count on their own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GamePlaces {
    /// `None`: not a home game of the club, and no place names a team.
    pub schedule: Option<SchedulePart>,
    /// `None`: no drivers needed, and no number above 0 set.
    pub drivers: Option<DriverPart>,
}

/// The duties of a game. `required` is `required_duties` of the game for `team`; `stored` its
/// stored duties, if any; `filled` the places with a player (empty in story A); `players_on_date`
/// the players with "plays games" on its date; `in_list` whether it is still in the game list;
/// `past` whether its date is before today. `None`: nothing required and nothing kept.
pub fn game_places(
    required: RequiredDuties,
    stored: Option<&StoredDuties>,
    team: TeamNumber,
    filled: &BTreeSet<(DutyKind, u8)>,
    players_on_date: usize,
    in_list: bool,
    past: bool,
) -> Option<GamePlaces> {
    let empty = StoredDuties::default();
    let stored = stored.unwrap_or(&empty);
    let counts = |required: bool| past || (in_list && required);

    let schedule = (required.schedule || !stored.teams.is_empty()).then(|| {
        let places: Vec<SchedulePlace> = SCHEDULE_PLACES
            .iter()
            .map(|&(kind, number)| SchedulePlace {
                kind,
                number,
                team: stored.teams.get(&(kind, number)).cloned(),
            })
            .collect();
        let of_team = places
            .iter()
            .filter(|place| place.team.as_ref().is_some_and(|t| t.number == team))
            .map(|place| {
                let kind = DutyKind::from(place.kind);
                DutyPlace {
                    kind,
                    number: place.number,
                    filled: filled.contains(&(kind, place.number)),
                }
            })
            .collect();
        SchedulePart {
            places,
            entered: stored.schedule_entered,
            of_team,
            required: required.schedule,
            counts: counts(required.schedule),
        }
    });

    let set = stored.drivers;
    let drivers = (required.drivers || set.is_some_and(|d| d > 0)).then(|| {
        let proposal = driver_proposal(players_on_date);
        let full = filled
            .iter()
            .filter(|(kind, _)| *kind == DutyKind::Drive)
            .map(|&(_, number)| number)
            .max()
            .unwrap_or(0);
        let proposed = required.drivers && set.is_none();
        let number = match set {
            Some(set) => set.max(full),
            None if required.drivers => proposal.max(full),
            None => full,
        };
        DriverPart {
            places: (1..=number)
                .map(|number| DutyPlace {
                    kind: DutyKind::Drive,
                    number,
                    filled: filled.contains(&(DutyKind::Drive, number)),
                })
                .collect(),
            proposal,
            proposed,
            more_than_proposed: proposed && full > proposal,
            required: required.drivers,
            counts: counts(required.drivers),
        }
    });

    (schedule.is_some() || drivers.is_some()).then_some(GamePlaces { schedule, drivers })
}

/// A home game of the club whose schedule is not entered, and whose places count.
pub fn not_entered(places: &GamePlaces) -> bool {
    places
        .schedule
        .as_ref()
        .is_some_and(|part| part.required && part.counts && !part.entered)
}

/// The places of the team that count and have no player.
pub fn open_places(places: &GamePlaces) -> usize {
    let schedule = places
        .schedule
        .iter()
        .filter(|part| part.counts)
        .flat_map(|part| &part.of_team);
    let drivers = places
        .drivers
        .iter()
        .filter(|part| part.counts)
        .flat_map(|part| &part.places);
    schedule
        .chain(drivers)
        .filter(|place| !place.filled)
        .count()
}

/// The game has a place of the team in a part that counts.
pub fn has_team_places(places: &GamePlaces) -> bool {
    places
        .schedule
        .as_ref()
        .is_some_and(|part| part.counts && !part.of_team.is_empty())
        || places
            .drivers
            .as_ref()
            .is_some_and(|part| part.counts && !part.places.is_empty())
}

/// Why a save of duties is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum DutyError {
    #[error("the game is not a home game of the club")]
    NotAHomeGame,
    #[error("the game needs no drivers of the team")]
    NoDriversNeeded,
    #[error("more drivers than can be set")]
    OutOfRange,
    #[error("the team is not a team of the club in this season")]
    UnknownTeam,
    #[error("{filled} places are filled")]
    BelowFilled { filled: u8 },
    #[error("a player is on the place")]
    PlayerOnPlace,
}

/// The team `number` from the pull-down list `list` (`club_teams`).
pub fn pick_team(list: &[TeamRef], number: TeamNumber) -> Result<TeamRef, DutyError> {
    list.iter()
        .find(|team| team.number == number)
        .cloned()
        .ok_or(DutyError::UnknownTeam)
}

/// A change of a place from `current` to `new` (`None`: "—"). The same team again is no change.
/// A team only at a home game of the club; "—" always. A place with a player (story B) keeps its
/// team.
pub fn check_team(
    required: RequiredDuties,
    current: Option<&TeamRef>,
    new: Option<&TeamRef>,
    place_filled: bool,
) -> Result<(), DutyError> {
    if current.map(|team| team.number) == new.map(|team| team.number) {
        return Ok(());
    }
    if new.is_some() && !required.schedule {
        return Err(DutyError::NotAHomeGame);
    }
    if place_filled {
        return Err(DutyError::PlayerOnPlace);
    }
    Ok(())
}

/// "Schedule entered" on only at a home game of the club; off always.
pub fn check_entered(required: RequiredDuties, on: bool) -> Result<(), DutyError> {
    if on && !required.schedule {
        return Err(DutyError::NotAHomeGame);
    }
    Ok(())
}

/// A number of drivers (`None`: the proposal), with `filled` the number of filled driver places:
/// at most 10; above 0 only where drivers are needed; never below the filled places. 0, and the
/// proposal where no drivers are needed, clear a number that is no longer needed.
pub fn check_drivers(
    required: RequiredDuties,
    new: Option<u8>,
    filled: u8,
) -> Result<(), DutyError> {
    let Some(number) = new else {
        // The proposal never drops below the filled drivers.
        return Ok(());
    };
    if number > MAX_DRIVERS {
        return Err(DutyError::OutOfRange);
    }
    if number > 0 && !required.drivers {
        return Err(DutyError::NoDriversNeeded);
    }
    if number < filled {
        return Err(DutyError::BelowFilled { filled });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schedule::tests::{CLUB, CLUB_TEAM, OPPONENT, TEAM, game};

    const SCHEDULE: RequiredDuties = RequiredDuties {
        schedule: true,
        drivers: false,
    };
    const DRIVERS: RequiredDuties = RequiredDuties {
        schedule: false,
        drivers: true,
    };
    const NOTHING: RequiredDuties = RequiredDuties {
        schedule: false,
        drivers: false,
    };
    const NEW_TEAM: TeamNumber = TeamNumber(186);

    fn team(number: TeamNumber) -> TeamRef {
        TeamRef {
            number,
            name: format!("Team {}", number.0),
        }
    }

    fn teams(places: &[((PlaceKind, u8), TeamNumber)]) -> StoredDuties {
        StoredDuties {
            teams: places
                .iter()
                .map(|&(place, number)| (place, team(number)))
                .collect(),
            ..StoredDuties::default()
        }
    }

    fn drivers(number: Option<u8>) -> StoredDuties {
        StoredDuties {
            drivers: number,
            ..StoredDuties::default()
        }
    }

    fn none() -> BTreeSet<(DutyKind, u8)> {
        BTreeSet::new()
    }

    fn of(places: &[DutyPlace]) -> Vec<(DutyKind, u8)> {
        places
            .iter()
            .map(|place| (place.kind, place.number))
            .collect()
    }

    #[test]
    fn a_home_game_of_another_club_team_requires_the_schedule() {
        assert_eq!(
            required_duties(&game(1, CLUB_TEAM, OPPONENT), CLUB, TEAM),
            SCHEDULE
        );
    }

    #[test]
    fn a_home_game_of_the_team_requires_the_schedule() {
        assert_eq!(
            required_duties(&game(1, TEAM, OPPONENT), CLUB, TEAM),
            SCHEDULE
        );
    }

    #[test]
    fn an_away_game_of_the_team_at_another_club_requires_drivers() {
        assert_eq!(
            required_duties(&game(1, OPPONENT, TEAM), CLUB, TEAM),
            DRIVERS
        );
    }

    #[test]
    fn an_away_game_in_the_clubs_own_hall_requires_the_schedule_and_no_drivers() {
        // Another club team is the home team: the game is a home game of the club.
        assert_eq!(
            required_duties(&game(1, CLUB_TEAM, TEAM), CLUB, TEAM),
            SCHEDULE
        );
    }

    #[test]
    fn an_away_game_of_another_club_team_requires_nothing() {
        assert_eq!(
            required_duties(&game(1, OPPONENT, CLUB_TEAM), CLUB, TEAM),
            NOTHING
        );
    }

    #[test]
    fn players_playing_on_a_date_count_only_plays_games() {
        use crate::players::{PlayerId, RoleId};
        use crate::season::Season;
        let date = |m, d| NaiveDate::from_ymd_opt(2026, m, d).expect("valid date");
        let role = |id, player, role, from, until| PlayerRole {
            id: RoleId(id),
            player: PlayerId(player),
            season: Season::from_start_year(2026),
            role,
            from,
            until,
        };
        let roles = [
            role(1, 1, Role::PlaysGames, date(9, 1), None),
            role(2, 2, Role::OnlyTrains, date(9, 1), None),
            role(3, 3, Role::PlaysGames, date(9, 1), Some(date(10, 15))),
            role(4, 4, Role::PlaysGames, date(11, 1), None),
        ];
        assert_eq!(players_playing_on(&roles, date(10, 31)), 1);
        assert_eq!(players_playing_on(&roles, date(10, 1)), 2);
        assert_eq!(players_playing_on(&roles, date(11, 7)), 2);
    }

    #[test]
    fn the_proposal_is_one_driver_for_every_three_players() {
        let proposals: Vec<u8> = [0, 1, 3, 4, 10, 31]
            .into_iter()
            .map(driver_proposal)
            .collect();
        assert_eq!(proposals, [0, 1, 1, 2, 4, 10]);
    }

    #[test]
    fn the_club_teams_come_from_the_games_and_the_places() {
        let club = Club {
            number: CLUB,
            name: "Club".to_owned(),
            town: None,
        };
        let mut gone = game(2, OPPONENT, CLUB_TEAM);
        gone.away.team_name = "Club X12 1".to_owned();
        let games = [game(1, TEAM, OPPONENT), gone];
        // A team that is in no game of the season any more, named by a place.
        let named = TeamRef {
            number: TeamNumber(300),
            name: "A1 1".to_owned(),
        };
        let stored = BTreeMap::from([(
            7,
            StoredDuties {
                teams: BTreeMap::from([((PlaceKind::Table, 1), named.clone())]),
                ..StoredDuties::default()
            },
        )]);
        let list = club_teams(&club, &games, &stored);
        let names: Vec<&str> = list.iter().map(|team| team.name.as_str()).collect();
        // Sorted by name; the opponent left out.
        assert_eq!(names, ["A1 1", "Team 184", "X12 1"]);
        assert_eq!(list[0], named);
    }

    #[test]
    fn a_game_is_kept_for_a_named_place_or_drivers_above_zero() {
        let entered = StoredDuties {
            schedule_entered: true,
            ..StoredDuties::default()
        };
        let stored = BTreeMap::from([
            (1, teams(&[((PlaceKind::Referee, 2), CLUB_TEAM)])),
            (2, drivers(Some(3))),
            (3, drivers(Some(0))),
            (4, entered),
            (5, drivers(None)),
        ]);
        assert_eq!(kept_games(&stored), BTreeSet::from([1, 2]));
    }

    #[test]
    fn a_home_game_not_entered_has_four_empty_places() {
        let places = game_places(SCHEDULE, None, TEAM, &none(), 10, true, false).expect("places");
        let part = places.schedule.as_ref().expect("schedule");
        let order: Vec<(PlaceKind, u8)> = part.places.iter().map(|p| (p.kind, p.number)).collect();
        assert_eq!(order, SCHEDULE_PLACES);
        assert!(part.places.iter().all(|place| place.team.is_none()));
        assert!(!part.entered && part.required && part.counts);
        assert!(part.of_team.is_empty());
        assert_eq!(places.drivers, None);
        assert!(not_entered(&places));
        assert!(!has_team_places(&places));
    }

    #[test]
    fn a_home_game_entered_with_empty_places_is_not_open() {
        let stored = StoredDuties {
            schedule_entered: true,
            ..StoredDuties::default()
        };
        let places =
            game_places(SCHEDULE, Some(&stored), TEAM, &none(), 10, true, false).expect("places");
        assert!(!not_entered(&places));
        assert_eq!(open_places(&places), 0);
    }

    #[test]
    fn places_of_other_teams_are_not_places_of_the_team() {
        let stored = teams(&[
            ((PlaceKind::Referee, 1), CLUB_TEAM),
            ((PlaceKind::Table, 2), CLUB_TEAM),
        ]);
        let places =
            game_places(SCHEDULE, Some(&stored), TEAM, &none(), 10, true, false).expect("places");
        let part = places.schedule.as_ref().expect("schedule");
        assert_eq!(part.places[0].team, Some(team(CLUB_TEAM)));
        assert!(part.of_team.is_empty());
        assert_eq!(open_places(&places), 0);
    }

    #[test]
    fn the_places_of_the_team_are_in_the_order_of_the_places() {
        let stored = teams(&[
            ((PlaceKind::Table, 2), TEAM),
            ((PlaceKind::Referee, 1), TEAM),
            ((PlaceKind::Referee, 2), CLUB_TEAM),
            ((PlaceKind::Table, 1), TEAM),
        ]);
        let places =
            game_places(SCHEDULE, Some(&stored), TEAM, &none(), 10, true, false).expect("places");
        let part = places.schedule.as_ref().expect("schedule");
        assert_eq!(
            of(&part.of_team),
            [
                (DutyKind::Referee, 1),
                (DutyKind::Table, 1),
                (DutyKind::Table, 2)
            ]
        );
        assert_eq!(open_places(&places), 3);
        assert!(has_team_places(&places));
        // A player on table 1 (story B).
        let filled = BTreeSet::from([(DutyKind::Table, 1)]);
        let places =
            game_places(SCHEDULE, Some(&stored), TEAM, &filled, 10, true, false).expect("places");
        assert_eq!(open_places(&places), 2);
    }

    #[test]
    fn a_coming_game_that_is_gone_does_not_count_and_a_past_one_does() {
        let stored = teams(&[((PlaceKind::Referee, 1), TEAM)]);
        let coming =
            game_places(SCHEDULE, Some(&stored), TEAM, &none(), 10, false, false).expect("places");
        assert!(!coming.schedule.as_ref().expect("schedule").counts);
        assert_eq!(open_places(&coming), 0);
        assert!(!not_entered(&coming));
        assert!(!has_team_places(&coming));
        let past =
            game_places(SCHEDULE, Some(&stored), TEAM, &none(), 10, false, true).expect("places");
        assert!(past.schedule.as_ref().expect("schedule").counts);
        assert!(has_team_places(&past));
    }

    #[test]
    fn a_swapped_game_keeps_its_places_and_its_drivers_count() {
        // A table place of the team was saved; home and away swapped and it is now an away game of
        // the team at another club.
        let stored = teams(&[((PlaceKind::Table, 1), TEAM)]);
        let places =
            game_places(DRIVERS, Some(&stored), TEAM, &none(), 4, true, false).expect("places");
        let part = places.schedule.as_ref().expect("schedule");
        assert!(!part.required && !part.counts);
        assert_eq!(of(&part.of_team), [(DutyKind::Table, 1)]);
        assert!(!not_entered(&places));
        let drivers = places.drivers.as_ref().expect("drivers");
        assert!(drivers.required && drivers.counts);
        // Only the 2 proposed drivers are open; the table place does not count.
        assert_eq!(open_places(&places), 2);
    }

    #[test]
    fn another_team_chosen_keeps_the_places_of_the_old_team() {
        // TEAM had table 1 and 2; NEW_TEAM is chosen now.
        let stored = teams(&[((PlaceKind::Table, 1), TEAM), ((PlaceKind::Table, 2), TEAM)]);
        let places = game_places(SCHEDULE, Some(&stored), NEW_TEAM, &none(), 10, true, false)
            .expect("places");
        let part = places.schedule.as_ref().expect("schedule");
        assert!(part.of_team.is_empty());
        assert_eq!(part.places[2].team, Some(team(TEAM)));
        assert_eq!(open_places(&places), 0);
    }

    #[test]
    fn a_game_with_nothing_required_or_kept_has_no_places() {
        assert_eq!(
            game_places(NOTHING, None, TEAM, &none(), 10, true, false),
            None
        );
        let stored = StoredDuties {
            schedule_entered: true,
            drivers: Some(0),
            ..StoredDuties::default()
        };
        assert_eq!(
            game_places(NOTHING, Some(&stored), TEAM, &none(), 10, true, false),
            None
        );
    }

    #[test]
    fn drivers_follow_the_proposal_until_a_number_is_set() {
        let proposed = game_places(DRIVERS, None, TEAM, &none(), 10, true, false).expect("places");
        let part = proposed.drivers.as_ref().expect("drivers");
        assert!(part.proposed);
        assert_eq!((part.places.len(), part.proposal), (4, 4));
        assert_eq!(proposed.schedule, None);

        let set = game_places(
            DRIVERS,
            Some(&drivers(Some(2))),
            TEAM,
            &none(),
            10,
            true,
            false,
        )
        .expect("places");
        let part = set.drivers.as_ref().expect("drivers");
        assert!(!part.proposed);
        assert_eq!(part.places.len(), 2);

        let no_players = game_places(DRIVERS, None, TEAM, &none(), 0, true, false).expect("places");
        assert!(no_players.drivers.expect("drivers").places.is_empty());
    }

    #[test]
    fn a_proposal_below_the_filled_drivers_keeps_them() {
        // 6 players: a proposal of 2; 3 drivers were filled when the team was larger.
        let filled = BTreeSet::from([
            (DutyKind::Drive, 1),
            (DutyKind::Drive, 2),
            (DutyKind::Drive, 3),
        ]);
        let places = game_places(DRIVERS, None, TEAM, &filled, 6, true, false).expect("places");
        let part = places.drivers.as_ref().expect("drivers");
        assert_eq!(part.places.len(), 3);
        assert!(part.more_than_proposed);
        assert_eq!(open_places(&places), 0);

        // A set number above the proposal is not "more than proposed".
        let set = game_places(
            DRIVERS,
            Some(&drivers(Some(4))),
            TEAM,
            &none(),
            6,
            true,
            false,
        )
        .expect("places");
        assert!(!set.drivers.expect("drivers").more_than_proposed);
    }

    #[test]
    fn drivers_no_longer_needed_stay_until_cleared_and_do_not_count() {
        let places = game_places(
            NOTHING,
            Some(&drivers(Some(3))),
            TEAM,
            &none(),
            9,
            true,
            false,
        )
        .expect("places");
        let part = places.drivers.as_ref().expect("drivers");
        assert!(!part.required && !part.counts && !part.proposed);
        assert_eq!(part.places.len(), 3);
        assert_eq!(open_places(&places), 0);
        // Past, the drivers count.
        let past = game_places(
            NOTHING,
            Some(&drivers(Some(3))),
            TEAM,
            &none(),
            9,
            true,
            true,
        )
        .expect("places");
        assert!(past.drivers.expect("drivers").counts);
    }

    #[test]
    fn a_team_is_picked_from_the_list() {
        let list = [team(CLUB_TEAM), team(TEAM)];
        assert_eq!(pick_team(&list, TEAM), Ok(team(TEAM)));
        assert_eq!(
            pick_team(&list, TeamNumber(999)),
            Err(DutyError::UnknownTeam)
        );
    }

    #[test]
    fn a_change_of_a_place_is_checked() {
        let ours = team(TEAM);
        let other = team(CLUB_TEAM);
        assert_eq!(check_team(SCHEDULE, None, Some(&ours), false), Ok(()));
        assert_eq!(
            check_team(DRIVERS, None, Some(&ours), false),
            Err(DutyError::NotAHomeGame)
        );
        // "—" is always allowed, also on a game that is no longer a home game.
        assert_eq!(check_team(NOTHING, Some(&ours), None, false), Ok(()));
        // A place with a player keeps its team; the same team again is no change.
        assert_eq!(
            check_team(SCHEDULE, Some(&ours), Some(&other), true),
            Err(DutyError::PlayerOnPlace)
        );
        assert_eq!(
            check_team(SCHEDULE, Some(&ours), None, true),
            Err(DutyError::PlayerOnPlace)
        );
        assert_eq!(check_team(SCHEDULE, Some(&ours), Some(&ours), true), Ok(()));
    }

    #[test]
    fn schedule_entered_is_checked() {
        assert_eq!(check_entered(SCHEDULE, true), Ok(()));
        assert_eq!(check_entered(DRIVERS, true), Err(DutyError::NotAHomeGame));
        assert_eq!(check_entered(NOTHING, false), Ok(()));
    }

    #[test]
    fn a_number_of_drivers_is_checked() {
        assert_eq!(check_drivers(DRIVERS, Some(10), 0), Ok(()));
        assert_eq!(
            check_drivers(DRIVERS, Some(11), 0),
            Err(DutyError::OutOfRange)
        );
        assert_eq!(
            check_drivers(SCHEDULE, Some(2), 0),
            Err(DutyError::NoDriversNeeded)
        );
        // 0 and the proposal clear a number that is no longer needed.
        assert_eq!(check_drivers(NOTHING, Some(0), 0), Ok(()));
        assert_eq!(check_drivers(NOTHING, None, 0), Ok(()));
        assert_eq!(
            check_drivers(DRIVERS, Some(1), 2),
            Err(DutyError::BelowFilled { filled: 2 })
        );
        // The proposal never drops below the filled drivers.
        assert_eq!(check_drivers(DRIVERS, None, 2), Ok(()));
    }
}
