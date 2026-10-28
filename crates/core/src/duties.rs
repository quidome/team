//! Duty places (story A of duties). A game requires duties: at each home game of the club the 4
//! places of the referee committee (referee 1 and 2, table 1 and 2), each given to a club team or
//! empty; at an away game of the team at another club its drivers. `game_places` is the only rule
//! for the places of a game: "Season", the page of a game, the start page and the list of changes
//! all use it (decision 0022).
//!
//! Story B: a player on each place of the team (`Assignment`), with the outcome after the game.
//! A reading of the game list never changes what is stored: a past game counts what is stored.

use std::collections::{BTreeMap, BTreeSet};

use chrono::NaiveDate;

use crate::club::{Club, ClubNumber};
use crate::game::{Game, TeamNumber};
use crate::players::{Player, PlayerId, PlayerRole, Role};
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

/// Whether `player` has the role "plays games" on `date`: only such a player gets a new duty
/// place on that date.
pub fn plays_games_on(roles: &[PlayerRole], player: PlayerId, date: NaiveDate) -> bool {
    roles
        .iter()
        .any(|role| role.player == player && role.role == Role::PlaysGames && role.covers(date))
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
/// longer in the schedule: a place names a team, drivers are set above 0, or a player is on a
/// place (`assignments`, by game).
pub fn kept_games(
    stored: &BTreeMap<i64, StoredDuties>,
    assignments: &BTreeMap<i64, Vec<Assignment>>,
) -> BTreeSet<i64> {
    stored
        .iter()
        .filter(|(_, duties)| !duties.teams.is_empty() || duties.drivers.is_some_and(|d| d > 0))
        .map(|(&id, _)| id)
        .chain(
            assignments
                .iter()
                .filter(|(_, list)| !list.is_empty())
                .map(|(&id, _)| id),
        )
        .collect()
}

/// What happened on a filled place after the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// The planned player did it (also the state before the game).
    AsPlanned,
    /// Another player did it ("Someone else").
    DoneBy(PlayerId),
    /// Nobody did it ("Nobody"); the planned player stays visible.
    NotDone,
}

/// A player on a duty place of the team.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Assignment {
    pub kind: DutyKind,
    pub number: u8,
    pub planned: PlayerId,
    pub outcome: Outcome,
}

impl Assignment {
    /// The players this place takes at its game: the planned player, and another player who did it.
    fn players(&self) -> impl Iterator<Item = PlayerId> {
        let done_by = match self.outcome {
            Outcome::DoneBy(player) => Some(player),
            Outcome::AsPlanned | Outcome::NotDone => None,
        };
        std::iter::once(self.planned).chain(done_by)
    }
}

/// Who a filled place counts for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Counted {
    /// A game before today: done by this player.
    Done(PlayerId),
    /// A game of today or later: planned for this player.
    Planned(PlayerId),
    Nobody,
}

/// Who `assignment` counts for; `past`: its game is before today (Dutch date).
pub fn who_did(assignment: &Assignment, past: bool) -> Counted {
    if !past {
        return Counted::Planned(assignment.planned);
    }
    match assignment.outcome {
        Outcome::AsPlanned => Counted::Done(assignment.planned),
        Outcome::DoneBy(player) => Counted::Done(player),
        Outcome::NotDone => Counted::Nobody,
    }
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
    let full = highest_driver(filled);
    // Driver places with players stay, also when the game no longer needs drivers.
    let drivers = (required.drivers || set.is_some_and(|d| d > 0) || full > 0).then(|| {
        let proposal = driver_proposal(players_on_date);
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

/// The highest driver place with a player; 0 for none.
pub fn highest_driver(filled: &BTreeSet<(DutyKind, u8)>) -> u8 {
    filled
        .iter()
        .filter(|(kind, _)| *kind == DutyKind::Drive)
        .map(|&(_, number)| number)
        .max()
        .unwrap_or(0)
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
    open_by_kind(places).values().sum()
}

/// The places of the team that count and have no player, for each kind; a kind without open
/// places is not in it.
pub fn open_by_kind(places: &GamePlaces) -> BTreeMap<DutyKind, usize> {
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
    let mut open = BTreeMap::new();
    for place in schedule.chain(drivers).filter(|place| !place.filled) {
        *open.entry(place.kind).or_insert(0) += 1;
    }
    open
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
    /// `filled` driver places have a player; the highest is place `highest`.
    #[error("{filled} places are filled, up to place {highest}")]
    BelowFilled { filled: u8, highest: u8 },
    #[error("a player is on the place")]
    PlayerOnPlace,
    #[error("the place is not a place of the team")]
    NotAPlaceOfTheTeam,
    #[error("the player already has a place at the game")]
    PlayerAtGame,
    #[error("the player does not play games on the date")]
    NotInTeam,
    #[error("the player has a clash")]
    Clash,
    #[error("the game is not past")]
    NotPast,
    #[error("no player is on the place")]
    NoPlayerOnPlace,
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

/// A number of drivers (`None`: the proposal), with `filled` the driver places with a player: at
/// most 10; above 0 only where drivers are needed; never below the highest filled place. 0, and
/// the proposal where no drivers are needed, clear a number that is no longer needed.
pub fn check_drivers(
    required: RequiredDuties,
    new: Option<u8>,
    filled: &BTreeSet<(DutyKind, u8)>,
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
    let highest = highest_driver(filled);
    if number < highest {
        let count = filled
            .iter()
            .filter(|(kind, _)| *kind == DutyKind::Drive)
            .count();
        return Err(DutyError::BelowFilled {
            filled: u8::try_from(count).unwrap_or(u8::MAX),
            highest,
        });
    }
    Ok(())
}

/// How close a clash is (step 2 works it out).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClashLevel {
    /// Can be chosen, with a note.
    Warning,
    /// Cannot be chosen.
    Blocked,
}

/// Whether the place `kind` `number` of `places` is a place of the team: a place that names the
/// team, or a driver place up to the number of drivers.
fn is_place_of_the_team(places: &GamePlaces, kind: DutyKind, number: u8) -> bool {
    match kind {
        DutyKind::Drive => places
            .drivers
            .as_ref()
            .is_some_and(|part| part.places.iter().any(|place| place.number == number)),
        DutyKind::Table | DutyKind::Referee => places.schedule.as_ref().is_some_and(|part| {
            part.of_team
                .iter()
                .any(|place| place.kind == kind && place.number == number)
        }),
    }
}

/// Whether `player` has another place at the game than `kind` `number` (`at_game`: the
/// assignments of the game), as planned player or as the player who did it.
fn elsewhere_at_game(at_game: &[Assignment], kind: DutyKind, number: u8, player: PlayerId) -> bool {
    at_game
        .iter()
        .filter(|other| (other.kind, other.number) != (kind, number))
        .any(|other| other.players().any(|p| p == player))
}

/// Putting `player` on the place `kind` `number` of a game with `places` and the assignments
/// `at_game`: only on a place of the team; not a player who has another place at the game; only a
/// player with the role "plays games" on the date (`plays_on_date`); not with a blocking `clash`.
/// The same player again is no change.
pub fn check_assignment(
    places: &GamePlaces,
    kind: DutyKind,
    number: u8,
    player: PlayerId,
    at_game: &[Assignment],
    plays_on_date: bool,
    clash: Option<ClashLevel>,
) -> Result<(), DutyError> {
    if !is_place_of_the_team(places, kind, number) {
        return Err(DutyError::NotAPlaceOfTheTeam);
    }
    let current = at_game
        .iter()
        .find(|other| (other.kind, other.number) == (kind, number));
    if current.is_some_and(|current| current.planned == player) {
        return Ok(());
    }
    if elsewhere_at_game(at_game, kind, number, player) {
        return Err(DutyError::PlayerAtGame);
    }
    if !plays_on_date {
        return Err(DutyError::NotInTeam);
    }
    if clash == Some(ClashLevel::Blocked) {
        return Err(DutyError::Clash);
    }
    Ok(())
}

/// A correction of the place `kind` `number` after the game (`past`), from `assignment` (the
/// player on it, if any) to `new`; `plays_on_date`: the player of `DoneBy` has the role "plays
/// games" on the date. Never blocked by a clash: it records what happened. Gives the outcome to
/// store: "done by" the planned player is "as planned".
pub fn check_outcome(
    past: bool,
    assignment: Option<&Assignment>,
    new: Outcome,
    at_game: &[Assignment],
    plays_on_date: bool,
) -> Result<Outcome, DutyError> {
    if !past {
        return Err(DutyError::NotPast);
    }
    let Some(assignment) = assignment else {
        return Err(DutyError::NoPlayerOnPlace);
    };
    let Outcome::DoneBy(player) = new else {
        return Ok(new);
    };
    if player == assignment.planned {
        return Ok(Outcome::AsPlanned);
    }
    if elsewhere_at_game(at_game, assignment.kind, assignment.number, player) {
        return Err(DutyError::PlayerAtGame);
    }
    if !plays_on_date {
        return Err(DutyError::NotInTeam);
    }
    Ok(new)
}

/// The duties of one game, as the totals and the clashes read them.
#[derive(Debug, Clone, Copy)]
pub struct GameDuties<'a> {
    pub game: &'a Game,
    pub places: &'a GamePlaces,
    pub assignments: &'a [Assignment],
    /// The game is before today (Dutch date).
    pub past: bool,
}

/// Whether the place of `assignment` counts. A coming game: its part counts and it is a place of
/// the team. A past game counts what is stored: its part counts (always, when past), also after a
/// swap or another team chosen (A1).
fn place_counts(duties: &GameDuties<'_>, assignment: &Assignment) -> bool {
    let part_counts = match assignment.kind {
        DutyKind::Drive => duties.places.drivers.as_ref().map(|part| part.counts),
        DutyKind::Table | DutyKind::Referee => {
            duties.places.schedule.as_ref().map(|part| part.counts)
        }
    };
    part_counts == Some(true)
        && (duties.past || is_place_of_the_team(duties.places, assignment.kind, assignment.number))
}

/// The duties of one kind of a player: done before today, planned from today.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Total {
    pub done: u32,
    pub planned: u32,
}

/// The duties of a player for each kind.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KindTotals {
    pub drive: Total,
    pub table: Total,
    pub referee: Total,
}

impl KindTotals {
    pub fn of(&self, kind: DutyKind) -> Total {
        match kind {
            DutyKind::Drive => self.drive,
            DutyKind::Table => self.table,
            DutyKind::Referee => self.referee,
        }
    }

    fn of_mut(&mut self, kind: DutyKind) -> &mut Total {
        match kind {
            DutyKind::Drive => &mut self.drive,
            DutyKind::Table => &mut self.table,
            DutyKind::Referee => &mut self.referee,
        }
    }
}

/// The duty totals of `games` (usually one season): the only source of the numbers (decision
/// 0022). A player with no counted duty is not in it.
pub fn duty_totals(games: &[GameDuties<'_>]) -> BTreeMap<PlayerId, KindTotals> {
    let mut totals: BTreeMap<PlayerId, KindTotals> = BTreeMap::new();
    for duties in games {
        for assignment in duties.assignments {
            if !place_counts(duties, assignment) {
                continue;
            }
            let (player, done) = match who_did(assignment, duties.past) {
                Counted::Done(player) => (player, true),
                Counted::Planned(player) => (player, false),
                Counted::Nobody => continue,
            };
            let total = totals.entry(player).or_default().of_mut(assignment.kind);
            if done {
                total.done += 1;
            } else {
                total.planned += 1;
            }
        }
    }
    totals
}

/// The names shown for `players` (the players of a season and those on its places): the first
/// name; with the first letter of the last name ("Anna B.") when another of them has the same
/// first name, without case. A player without a last name: the first name only.
pub fn display_names(players: &[Player]) -> BTreeMap<PlayerId, String> {
    players
        .iter()
        .map(|player| {
            let first = player.first_name.to_lowercase();
            let shared = players
                .iter()
                .any(|other| other.id != player.id && other.first_name.to_lowercase() == first);
            let initial = player
                .last_name
                .as_deref()
                .and_then(|last| last.chars().next())
                .filter(|_| shared);
            let name = match initial {
                Some(initial) => format!("{} {}.", player.first_name, initial.to_uppercase()),
                None => player.first_name.clone(),
            };
            (player.id, name)
        })
        .collect()
}

/// Start times this close on different courts block a choice.
const BLOCKED_WITHIN_MINUTES: i64 = 90;
/// Start times this close on different courts give a warning: games often run over 1:30.
const WARNING_WITHIN_MINUTES: i64 = 120;

/// The court of `game` for comparing: without case and outer spaces; `None` when not known.
fn court_of(game: &Game) -> Option<String> {
    game.place
        .court
        .as_deref()
        .map(|court| court.trim().to_lowercase())
        .filter(|court| !court.is_empty())
}

/// How close `a` and `b` are for one player (coordinator, 2026-10-27): never on another date or on
/// the same court of the same hall (the next game on a court waits for the one before); on
/// different courts by the start times, under 1:30 blocked, under 2:00 a warning; a start time not
/// known only warns. A game without a court is on another court.
pub fn clash_level(a: &Game, b: &Game) -> Option<ClashLevel> {
    if a.date != b.date {
        return None;
    }
    let same_court = a.place.number == b.place.number
        && court_of(a).is_some_and(|court| Some(court) == court_of(b));
    if same_court {
        return None;
    }
    let (Some(start_a), Some(start_b)) = (a.start_time, b.start_time) else {
        return Some(ClashLevel::Warning);
    };
    let apart = (start_a - start_b).num_minutes().abs();
    if apart < BLOCKED_WITHIN_MINUTES {
        Some(ClashLevel::Blocked)
    } else if apart < WARNING_WITHIN_MINUTES {
        Some(ClashLevel::Warning)
    } else {
        None
    }
}

/// What a clash is with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClashWhat {
    /// A duty place at another game ("also Table 13:00").
    Duty { kind: DutyKind },
    /// Another game of the team ("plays 14:30").
    Plays,
}

/// A clash of a player with another game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clash {
    pub level: ClashLevel,
    pub what: ClashWhat,
    /// The other game.
    pub game: i64,
    pub start: Option<chrono::NaiveTime>,
}

/// The clashes of `player` for a place at `game`: the places that count of the player at other
/// games of `season`, and the other games of the team (`team_games`, in the game list). The game
/// itself is never a clash (S22). One clash per other game; a duty there wins over playing there.
pub fn clashes(
    player: PlayerId,
    game: &Game,
    season: &[GameDuties<'_>],
    team_games: &[&Game],
) -> Vec<Clash> {
    let mut found: BTreeMap<i64, Clash> = BTreeMap::new();
    for other in team_games {
        if other.source_id == game.source_id {
            continue;
        }
        if let Some(level) = clash_level(game, other) {
            found.insert(
                other.source_id,
                Clash {
                    level,
                    what: ClashWhat::Plays,
                    game: other.source_id,
                    start: other.start_time,
                },
            );
        }
    }
    for duties in season {
        if duties.game.source_id == game.source_id {
            continue;
        }
        let Some(level) = clash_level(game, duties.game) else {
            continue;
        };
        let duty = duties.assignments.iter().find(|assignment| {
            place_counts(duties, assignment) && assignment.players().any(|p| p == player)
        });
        if let Some(assignment) = duty {
            found.insert(
                duties.game.source_id,
                Clash {
                    level,
                    what: ClashWhat::Duty {
                        kind: assignment.kind,
                    },
                    game: duties.game.source_id,
                    start: duties.game.start_time,
                },
            );
        }
    }
    let mut list: Vec<Clash> = found.into_values().collect();
    list.sort_by_key(|clash| (std::cmp::Reverse(clash.level), clash.start, clash.game));
    list
}

/// The closest clash in `clashes`, if any.
pub fn worst_clash(clashes: &[Clash]) -> Option<ClashLevel> {
    clashes.iter().map(|clash| clash.level).max()
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

    const ANNA: PlayerId = PlayerId(1);
    const BRAM: PlayerId = PlayerId(2);
    const CAS: PlayerId = PlayerId(3);

    fn planned(kind: DutyKind, number: u8, player: PlayerId) -> Assignment {
        Assignment {
            kind,
            number,
            planned: player,
            outcome: Outcome::AsPlanned,
        }
    }

    fn filled(places: &[(DutyKind, u8)]) -> BTreeSet<(DutyKind, u8)> {
        places.iter().copied().collect()
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
        assert!(plays_games_on(&roles, PlayerId(1), date(10, 31)));
        assert!(!plays_games_on(&roles, PlayerId(2), date(10, 31)));
        assert!(!plays_games_on(&roles, PlayerId(3), date(10, 31)));
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
        assert_eq!(
            kept_games(&stored, &BTreeMap::new()),
            BTreeSet::from([1, 2])
        );
    }

    #[test]
    fn a_game_with_a_player_on_a_proposed_driver_place_is_kept() {
        // R3: a past away game, drivers still proposed, a player on place 1; no stored row of
        // duties names a team or sets drivers.
        let stored = BTreeMap::from([(5, drivers(None))]);
        let assignments =
            BTreeMap::from([(5, vec![planned(DutyKind::Drive, 1, ANNA)]), (6, vec![])]);
        assert_eq!(kept_games(&stored, &assignments), BTreeSet::from([5]));
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
        assert_eq!(check_drivers(DRIVERS, Some(10), &none()), Ok(()));
        assert_eq!(
            check_drivers(DRIVERS, Some(11), &none()),
            Err(DutyError::OutOfRange)
        );
        assert_eq!(
            check_drivers(SCHEDULE, Some(2), &none()),
            Err(DutyError::NoDriversNeeded)
        );
        // 0 and the proposal clear a number that is no longer needed.
        assert_eq!(check_drivers(NOTHING, Some(0), &none()), Ok(()));
        assert_eq!(check_drivers(NOTHING, None, &none()), Ok(()));
        let two = filled(&[(DutyKind::Drive, 1), (DutyKind::Drive, 2)]);
        assert_eq!(
            check_drivers(DRIVERS, Some(1), &two),
            Err(DutyError::BelowFilled {
                filled: 2,
                highest: 2
            })
        );
        // The proposal never drops below the filled drivers.
        assert_eq!(check_drivers(DRIVERS, None, &two), Ok(()));
    }

    #[test]
    fn drivers_never_drop_below_the_highest_filled_place() {
        // Places 1 and 2 open, place 3 filled.
        let third = filled(&[(DutyKind::Drive, 3)]);
        assert_eq!(
            check_drivers(DRIVERS, Some(2), &third),
            Err(DutyError::BelowFilled {
                filled: 1,
                highest: 3
            })
        );
        assert_eq!(check_drivers(DRIVERS, Some(3), &third), Ok(()));
    }

    #[test]
    fn a_place_counts_as_done_after_the_game_and_as_planned_before() {
        let anna = planned(DutyKind::Table, 1, ANNA);
        assert_eq!(who_did(&anna, false), Counted::Planned(ANNA));
        assert_eq!(who_did(&anna, true), Counted::Done(ANNA));
        let bram = Assignment {
            outcome: Outcome::DoneBy(BRAM),
            ..anna
        };
        assert_eq!(who_did(&bram, true), Counted::Done(BRAM));
        let nobody = Assignment {
            outcome: Outcome::NotDone,
            ..anna
        };
        assert_eq!(who_did(&nobody, true), Counted::Nobody);
    }

    #[test]
    fn driver_places_with_players_stay_when_no_drivers_are_needed() {
        // An away game with proposed drivers and two players on them; home and away swapped.
        let two = filled(&[(DutyKind::Drive, 1), (DutyKind::Drive, 2)]);
        let coming = game_places(SCHEDULE, None, TEAM, &two, 10, true, false).expect("places");
        let part = coming.drivers.as_ref().expect("drivers");
        assert!(!part.required && !part.counts);
        assert_eq!(part.places.len(), 2);
        assert!(part.places.iter().all(|place| place.filled));
        // Past, they count as stored (A1).
        let past = game_places(SCHEDULE, None, TEAM, &two, 10, true, true).expect("places");
        assert!(past.drivers.expect("drivers").counts);
    }

    #[test]
    fn a_past_swapped_game_counts_its_stored_places() {
        // A1: a table place of the team with a player; home and away swapped before the game,
        // nobody cleared it, the date passed.
        let stored = teams(&[((PlaceKind::Table, 1), TEAM)]);
        let anna = filled(&[(DutyKind::Table, 1)]);
        let places =
            game_places(DRIVERS, Some(&stored), TEAM, &anna, 4, true, true).expect("places");
        let part = places.schedule.as_ref().expect("schedule");
        assert!(!part.required && part.counts);
        assert!(has_team_places(&places));
        // Coming, the same place does not count.
        let coming =
            game_places(DRIVERS, Some(&stored), TEAM, &anna, 4, true, false).expect("places");
        assert!(!coming.schedule.expect("schedule").counts);
    }

    fn table_places(team_on_table_2: TeamNumber) -> GamePlaces {
        let stored = teams(&[
            ((PlaceKind::Table, 1), TEAM),
            ((PlaceKind::Table, 2), team_on_table_2),
        ]);
        game_places(SCHEDULE, Some(&stored), TEAM, &none(), 10, true, false).expect("places")
    }

    #[test]
    fn a_player_goes_only_on_a_place_of_the_team() {
        let places = table_places(CLUB_TEAM);
        assert_eq!(
            check_assignment(&places, DutyKind::Table, 1, ANNA, &[], true, None),
            Ok(())
        );
        assert_eq!(
            check_assignment(&places, DutyKind::Table, 2, ANNA, &[], true, None),
            Err(DutyError::NotAPlaceOfTheTeam)
        );
        assert_eq!(
            check_assignment(&places, DutyKind::Referee, 1, ANNA, &[], true, None),
            Err(DutyError::NotAPlaceOfTheTeam)
        );
        // Drivers: only up to the number of driver places.
        let drivers = game_places(
            DRIVERS,
            Some(&drivers(Some(3))),
            TEAM,
            &none(),
            9,
            true,
            false,
        )
        .expect("places");
        assert_eq!(
            check_assignment(&drivers, DutyKind::Drive, 3, ANNA, &[], true, None),
            Ok(())
        );
        assert_eq!(
            check_assignment(&drivers, DutyKind::Drive, 4, ANNA, &[], true, None),
            Err(DutyError::NotAPlaceOfTheTeam)
        );
    }

    #[test]
    fn a_player_has_one_place_at_a_game() {
        let places = table_places(TEAM);
        let at_game = [planned(DutyKind::Table, 1, ANNA)];
        assert_eq!(
            check_assignment(&places, DutyKind::Table, 2, ANNA, &at_game, true, None),
            Err(DutyError::PlayerAtGame)
        );
        // The same player on the same place again is no change; another player replaces her.
        assert_eq!(
            check_assignment(&places, DutyKind::Table, 1, ANNA, &at_game, true, None),
            Ok(())
        );
        assert_eq!(
            check_assignment(&places, DutyKind::Table, 1, BRAM, &at_game, true, None),
            Ok(())
        );
        // A player who did another place ("Someone else") also has a place at the game.
        let done_by = [Assignment {
            outcome: Outcome::DoneBy(CAS),
            ..planned(DutyKind::Table, 1, ANNA)
        }];
        assert_eq!(
            check_assignment(&places, DutyKind::Table, 2, CAS, &done_by, true, None),
            Err(DutyError::PlayerAtGame)
        );
    }

    #[test]
    fn a_player_must_play_games_on_the_date_and_have_no_blocking_clash() {
        let places = table_places(TEAM);
        assert_eq!(
            check_assignment(&places, DutyKind::Table, 1, ANNA, &[], false, None),
            Err(DutyError::NotInTeam)
        );
        assert_eq!(
            check_assignment(
                &places,
                DutyKind::Table,
                1,
                ANNA,
                &[],
                true,
                Some(ClashLevel::Blocked)
            ),
            Err(DutyError::Clash)
        );
        assert_eq!(
            check_assignment(
                &places,
                DutyKind::Table,
                1,
                ANNA,
                &[],
                true,
                Some(ClashLevel::Warning)
            ),
            Ok(())
        );
    }

    #[test]
    fn a_correction_is_only_for_a_past_game_with_a_player_on_the_place() {
        let anna = planned(DutyKind::Table, 1, ANNA);
        let at_game = [anna, planned(DutyKind::Table, 2, CAS)];
        assert_eq!(
            check_outcome(false, Some(&anna), Outcome::NotDone, &at_game, true),
            Err(DutyError::NotPast)
        );
        assert_eq!(
            check_outcome(true, None, Outcome::NotDone, &at_game, true),
            Err(DutyError::NoPlayerOnPlace)
        );
        assert_eq!(
            check_outcome(true, Some(&anna), Outcome::NotDone, &at_game, true),
            Ok(Outcome::NotDone)
        );
        assert_eq!(
            check_outcome(true, Some(&anna), Outcome::DoneBy(BRAM), &at_game, true),
            Ok(Outcome::DoneBy(BRAM))
        );
        // "Someone else" who has another place at the game, or does not play games on the date.
        assert_eq!(
            check_outcome(true, Some(&anna), Outcome::DoneBy(CAS), &at_game, true),
            Err(DutyError::PlayerAtGame)
        );
        assert_eq!(
            check_outcome(true, Some(&anna), Outcome::DoneBy(BRAM), &at_game, false),
            Err(DutyError::NotInTeam)
        );
        // "Done by" the planned player is as planned.
        assert_eq!(
            check_outcome(true, Some(&anna), Outcome::DoneBy(ANNA), &at_game, true),
            Ok(Outcome::AsPlanned)
        );
    }

    fn at(source_id: i64, place: i64, court: Option<&str>, start: Option<(u32, u32)>) -> Game {
        let mut game = game(source_id, CLUB_TEAM, OPPONENT);
        game.date = NaiveDate::from_ymd_opt(2026, 10, 3).expect("valid date");
        game.place.number = crate::game::PlaceNumber(place);
        game.place.court = court.map(str::to_owned);
        game.start_time = start.and_then(|(h, m)| chrono::NaiveTime::from_hms_opt(h, m, 0));
        game
    }

    const BULWIJK: i64 = 554;
    const OTHER_HALL: i64 = 43;

    #[test]
    fn the_same_court_never_clashes() {
        let a = at(1, BULWIJK, Some("Veld 1"), Some((13, 0)));
        let b = at(2, BULWIJK, Some("Veld 1"), Some((13, 0)));
        assert_eq!(clash_level(&a, &b), None);
        // The game list writes the same court in two ways.
        let c = at(3, BULWIJK, Some(" veld 1"), Some((13, 30)));
        assert_eq!(clash_level(&a, &c), None);
    }

    #[test]
    fn other_courts_block_under_one_thirty_and_warn_under_two() {
        // The real day of 3 October 2026 in the club's hall: 13:00 Veld 1, 14:30 Veld 2.
        let table = at(1, BULWIJK, Some("Veld 1"), Some((13, 0)));
        let next = at(2, BULWIJK, Some("Veld 2"), Some((14, 30)));
        assert_eq!(clash_level(&table, &next), Some(ClashLevel::Warning));
        let close = at(3, BULWIJK, Some("Veld 2"), Some((14, 0)));
        assert_eq!(clash_level(&table, &close), Some(ClashLevel::Blocked));
        // Another hall.
        let away = |id, h, m| at(id, OTHER_HALL, Some("Veld 1"), Some((h, m)));
        assert_eq!(
            clash_level(&table, &away(4, 14, 0)),
            Some(ClashLevel::Blocked)
        );
        assert_eq!(
            clash_level(&table, &away(5, 14, 30)),
            Some(ClashLevel::Warning)
        );
        assert_eq!(clash_level(&table, &away(6, 15, 0)), None);
        assert_eq!(clash_level(&away(7, 15, 0), &table), None);
    }

    #[test]
    fn a_game_without_a_court_is_on_another_court() {
        let a = at(1, BULWIJK, Some("Veld 1"), Some((13, 0)));
        let b = at(2, BULWIJK, None, Some((13, 0)));
        assert_eq!(clash_level(&a, &b), Some(ClashLevel::Blocked));
        let c = at(3, BULWIJK, Some(""), Some((13, 0)));
        assert_eq!(clash_level(&b, &c), Some(ClashLevel::Blocked));
    }

    #[test]
    fn a_start_time_not_known_only_warns() {
        let known = at(1, BULWIJK, Some("Veld 1"), Some((13, 0)));
        let unknown = at(2, OTHER_HALL, Some("Veld 1"), None);
        assert_eq!(clash_level(&known, &unknown), Some(ClashLevel::Warning));
        // On the same court: nothing.
        let same_court = at(3, BULWIJK, Some("Veld 1"), None);
        assert_eq!(clash_level(&known, &same_court), None);
        // Another date: nothing.
        let mut later = unknown.clone();
        later.date = NaiveDate::from_ymd_opt(2026, 10, 4).expect("valid date");
        assert_eq!(clash_level(&known, &later), None);
    }

    fn table_one(players: &[Assignment], past: bool) -> (GamePlaces, Vec<Assignment>, bool) {
        let stored = teams(&[((PlaceKind::Table, 1), TEAM), ((PlaceKind::Table, 2), TEAM)]);
        let filled: BTreeSet<(DutyKind, u8)> = players.iter().map(|a| (a.kind, a.number)).collect();
        let places =
            game_places(SCHEDULE, Some(&stored), TEAM, &filled, 10, true, past).expect("places");
        (places, players.to_vec(), past)
    }

    #[test]
    fn a_player_has_clashes_with_duties_and_games_of_the_team() {
        let this = at(1, BULWIJK, Some("Veld 1"), Some((13, 0)));
        let duty_game = at(2, BULWIJK, Some("Veld 2"), Some((14, 0)));
        let plays = at(3, OTHER_HALL, Some("Veld 1"), Some((14, 30)));
        let (places, assignments, past) = table_one(&[planned(DutyKind::Table, 1, ANNA)], false);
        let season = [GameDuties {
            game: &duty_game,
            places: &places,
            assignments: &assignments,
            past,
        }];
        let found = clashes(ANNA, &this, &season, &[&plays, &this]);
        assert_eq!(
            found,
            [
                Clash {
                    level: ClashLevel::Blocked,
                    what: ClashWhat::Duty {
                        kind: DutyKind::Table
                    },
                    game: 2,
                    start: duty_game.start_time,
                },
                Clash {
                    level: ClashLevel::Warning,
                    what: ClashWhat::Plays,
                    game: 3,
                    start: plays.start_time,
                },
            ]
        );
        assert_eq!(worst_clash(&found), Some(ClashLevel::Blocked));
        // Bram has no duty: only the game of the team; the game itself is never a clash (S22).
        let bram = clashes(BRAM, &this, &season, &[&plays, &this]);
        assert_eq!(bram.len(), 1);
        assert_eq!(bram[0].what, ClashWhat::Plays);
        assert_eq!(clashes(BRAM, &this, &season, &[&this]), []);
    }

    #[test]
    fn totals_count_done_and_planned_per_kind() {
        let (past_places, past_list, _) = table_one(
            &[
                planned(DutyKind::Table, 1, ANNA),
                Assignment {
                    outcome: Outcome::DoneBy(CAS),
                    ..planned(DutyKind::Table, 2, BRAM)
                },
            ],
            true,
        );
        let (coming_places, coming_list, _) =
            table_one(&[planned(DutyKind::Table, 1, ANNA)], false);
        let one = game(1, CLUB_TEAM, OPPONENT);
        let two = game(2, CLUB_TEAM, OPPONENT);
        let games = [
            GameDuties {
                game: &one,
                places: &past_places,
                assignments: &past_list,
                past: true,
            },
            GameDuties {
                game: &two,
                places: &coming_places,
                assignments: &coming_list,
                past: false,
            },
        ];
        let totals = duty_totals(&games);
        let total = |player, kind| totals.get(&player).copied().unwrap_or_default().of(kind);
        assert_eq!(
            total(ANNA, DutyKind::Table),
            Total {
                done: 1,
                planned: 1
            }
        );
        assert_eq!(
            total(CAS, DutyKind::Table),
            Total {
                done: 1,
                planned: 0
            }
        );
        // Bram was planned but someone else did it.
        assert_eq!(totals.get(&BRAM), None);
        assert_eq!(total(ANNA, DutyKind::Referee), Total::default());
    }

    #[test]
    fn totals_leave_out_nobody_and_coming_games_that_do_not_count() {
        let not_done = [Assignment {
            outcome: Outcome::NotDone,
            ..planned(DutyKind::Table, 1, ANNA)
        }];
        let (places, list, _) = table_one(&not_done, true);
        let one = game(1, CLUB_TEAM, OPPONENT);
        let games = [GameDuties {
            game: &one,
            places: &places,
            assignments: &list,
            past: true,
        }];
        assert!(duty_totals(&games).is_empty());

        // A coming game that left the game list, and a coming swapped game.
        let stored = teams(&[((PlaceKind::Table, 1), TEAM)]);
        let anna = [planned(DutyKind::Table, 1, ANNA)];
        let filled = BTreeSet::from([(DutyKind::Table, 1)]);
        for (required, in_list) in [(SCHEDULE, false), (DRIVERS, true)] {
            let places = game_places(required, Some(&stored), TEAM, &filled, 10, in_list, false)
                .expect("places");
            let games = [GameDuties {
                game: &one,
                places: &places,
                assignments: &anna,
                past: false,
            }];
            assert!(duty_totals(&games).is_empty());
        }
    }

    #[test]
    fn a_past_game_counts_what_is_stored() {
        // A1: past and out of the game list, past and swapped, past after another team was chosen.
        let stored = teams(&[((PlaceKind::Table, 1), TEAM)]);
        let anna = [planned(DutyKind::Table, 1, ANNA)];
        let filled = BTreeSet::from([(DutyKind::Table, 1)]);
        let one = game(1, CLUB_TEAM, OPPONENT);
        for (required, in_list, team) in [
            (SCHEDULE, false, TEAM),
            (DRIVERS, true, TEAM),
            (SCHEDULE, true, NEW_TEAM),
        ] {
            let places = game_places(required, Some(&stored), team, &filled, 10, in_list, true)
                .expect("places");
            let games = [GameDuties {
                game: &one,
                places: &places,
                assignments: &anna,
                past: true,
            }];
            assert_eq!(
                duty_totals(&games)[&ANNA].table,
                Total {
                    done: 1,
                    planned: 0
                }
            );
        }
        // Past drivers of the old team after another team was chosen.
        let drive = [planned(DutyKind::Drive, 1, BRAM)];
        let filled = BTreeSet::from([(DutyKind::Drive, 1)]);
        let places = game_places(NOTHING, None, NEW_TEAM, &filled, 10, true, true).expect("places");
        let games = [GameDuties {
            game: &one,
            places: &places,
            assignments: &drive,
            past: true,
        }];
        assert_eq!(duty_totals(&games)[&BRAM].drive.done, 1);
    }

    fn player(id: i64, first: &str, last: Option<&str>) -> Player {
        Player {
            id: PlayerId(id),
            first_name: first.to_owned(),
            last_name: last.map(str::to_owned),
            hidden: false,
        }
    }

    #[test]
    fn two_players_with_the_same_first_name_get_an_initial() {
        let players = [
            player(1, "Anna", Some("Bakker")),
            player(2, "anna", Some("van Vliet")),
            player(3, "Bram", Some("Smit")),
            player(4, "Cas", None),
            player(5, "Cas", Some("de Wit")),
        ];
        let names = display_names(&players);
        let name = |id| names[&PlayerId(id)].as_str();
        assert_eq!(name(1), "Anna B.");
        assert_eq!(name(2), "anna V.");
        assert_eq!(name(3), "Bram");
        assert_eq!(name(4), "Cas");
        assert_eq!(name(5), "Cas D.");
    }
}
