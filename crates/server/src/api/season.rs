//! `GET /api/season`: all events of a season for the page "Season" (story A of start page and
//! season): the games of the team and the possible duty games, and the training days and
//! trainings, in one list with their ISO weeks.

use std::collections::{BTreeMap, BTreeSet};

use axum::Json;
use axum::extract::{Query, State};
use chrono::NaiveDate;
use serde::Serialize;
use team_core::{
    Absence, Club, ClubNumber, DriverPart, DutyKind, DutyPlace, Event, EventKind, FollowedTeam,
    Game, GamePlaces, PlaceKind, PlacesOf, PlayerRole, Schedule, ScheduleReason, ScheduledGame,
    Season, SeasonList, StoredDuties, Team, TeamNumber, TeamRef, Training, WeeklyTraining,
    game_places, kept_games, not_entered, open_places, players_playing_on, required_duties,
    schedule, season_events, season_list,
};
use utoipa::ToSchema;

use super::format::{day, query_season};
use super::trainings::{SeasonQuery, TrainingRow, row_view};
use crate::api::clubs::{ClubView, TeamView};
use crate::app::AppState;
use crate::clock::{current_season, today_in_netherlands};
use crate::db::StoreError;
use crate::error::AppError;
use crate::{api, duties, followed_teams, games, players, trainings, weekly_trainings};

/// A game of the schedule: the team's games and the home games of other club teams.
#[derive(Debug, Serialize, ToSchema)]
pub struct ScheduleGame {
    /// The source's game id.
    pub id: i64,
    /// Dutch local date, for example "2026-10-31".
    pub date: String,
    /// Dutch local start time, for example "13:00"; `null` when it is not known yet.
    pub start_time: Option<String>,
    /// Full team names, with the club name in front.
    pub home: String,
    pub away: String,
    pub hall: String,
    pub town: String,
    pub reason: ScheduleReasonView,
    /// False: the game is no longer in the game list; it shows until its change is seen, or
    /// while it keeps duties.
    pub in_list: bool,
    /// The duties of the game, when it has any (`team_core::game_places`).
    pub places: Option<PlacesView>,
}

/// The duties of a game, in two parts that count on their own.
#[derive(Debug, Serialize, ToSchema)]
pub struct PlacesView {
    /// The 4 places of the referee committee at a home game of the club.
    pub schedule: Option<SchedulePartView>,
    /// The drivers of an away game of the team.
    pub drivers: Option<DriverPartView>,
    /// A home game of the club whose schedule is not entered and whose places count.
    pub not_entered: bool,
    /// The places of the team that count and have no player.
    pub open_places: usize,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SchedulePartView {
    /// Referee 1, referee 2, table 1, table 2.
    pub places: Vec<SchedulePlaceView>,
    pub entered: bool,
    /// The places that name the team chosen for the game's season.
    pub of_team: Vec<PlaceView>,
    /// False: the game is no longer a home game of the club.
    pub required: bool,
    /// False: a coming game that is gone or is no longer a home game of the club.
    pub counts: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DriverPartView {
    pub places: Vec<PlaceView>,
    /// The proposal now: one driver for every 3 players with "plays games" on the date.
    pub proposal: u8,
    /// The number is the proposal (none was set).
    pub proposed: bool,
    /// The proposal dropped below the filled driver places.
    pub more_than_proposed: bool,
    /// False: the game no longer needs drivers of the team.
    pub required: bool,
    pub counts: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SchedulePlaceView {
    pub kind: PlaceKindView,
    pub number: u8,
    /// `null`: empty ("—").
    pub team: Option<TeamView>,
}

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlaceKindView {
    Referee,
    Table,
}

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DutyKindView {
    Drive,
    Table,
    Referee,
}

/// A duty place of the team.
#[derive(Debug, Serialize, ToSchema)]
pub struct PlaceView {
    pub kind: DutyKindView,
    pub number: u8,
    /// A player is on the place.
    pub filled: bool,
}

impl From<PlaceKind> for PlaceKindView {
    fn from(kind: PlaceKind) -> Self {
        match kind {
            PlaceKind::Referee => Self::Referee,
            PlaceKind::Table => Self::Table,
        }
    }
}

impl From<DutyKind> for DutyKindView {
    fn from(kind: DutyKind) -> Self {
        match kind {
            DutyKind::Drive => Self::Drive,
            DutyKind::Table => Self::Table,
            DutyKind::Referee => Self::Referee,
        }
    }
}

impl From<&TeamRef> for TeamView {
    fn from(team: &TeamRef) -> Self {
        Self {
            number: team.number.0,
            name: team.name.clone(),
        }
    }
}

fn place_view(place: &DutyPlace) -> PlaceView {
    PlaceView {
        kind: place.kind.into(),
        number: place.number,
        filled: place.filled,
    }
}

fn drivers_view(part: &DriverPart) -> DriverPartView {
    DriverPartView {
        places: part.places.iter().map(place_view).collect(),
        proposal: part.proposal,
        proposed: part.proposed,
        more_than_proposed: part.more_than_proposed,
        required: part.required,
        counts: part.counts,
    }
}

impl From<&GamePlaces> for PlacesView {
    fn from(places: &GamePlaces) -> Self {
        Self {
            schedule: places.schedule.as_ref().map(|part| SchedulePartView {
                places: part
                    .places
                    .iter()
                    .map(|place| SchedulePlaceView {
                        kind: place.kind.into(),
                        number: place.number,
                        team: place.team.as_ref().map(TeamView::from),
                    })
                    .collect(),
                entered: part.entered,
                of_team: part.of_team.iter().map(place_view).collect(),
                required: part.required,
                counts: part.counts,
            }),
            drivers: places.drivers.as_ref().map(drivers_view),
            not_entered: not_entered(places),
            open_places: open_places(places),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleReasonView {
    /// A game of the team, home or away.
    Team,
    /// A home game of another club team: a possible duty game.
    Duty,
    /// No longer in the schedule (for example home and away swapped), kept for its duties.
    Kept,
}

/// What the duties of the games of a season are made from.
pub(crate) struct SeasonDuties {
    pub stored: BTreeMap<i64, StoredDuties>,
    /// The roles of the season, for the driver proposal.
    pub roles: Vec<PlayerRole>,
}

impl SeasonDuties {
    /// The duties of `game` for `team` of `club` (decision 0022: the only rule is
    /// `team_core::game_places`). No places are filled before story B.
    pub fn places_of(
        &self,
        game: &Game,
        in_list: bool,
        club: ClubNumber,
        team: TeamNumber,
        today: NaiveDate,
    ) -> Option<GamePlaces> {
        game_places(
            required_duties(game, club, team),
            self.stored.get(&game.source_id),
            team,
            &BTreeSet::new(),
            players_playing_on(&self.roles, game.date),
            in_list,
            game.date < today,
        )
    }
}

/// The stored duties and the roles of `season` of `club`.
pub(crate) async fn duties_of(
    state: &AppState,
    club: ClubNumber,
    season: Season,
) -> Result<SeasonDuties, AppError> {
    let mut db = state.pool.acquire().await.map_err(StoreError::from)?;
    let stored = duties::for_season(&mut db, club, season).await?;
    let roles = players::roles_in(&mut *db, &[season]).await?;
    Ok(SeasonDuties { stored, roles })
}

impl From<ScheduleReason> for ScheduleReasonView {
    fn from(reason: ScheduleReason) -> Self {
        match reason {
            ScheduleReason::TeamGame => Self::Team,
            ScheduleReason::DutyGame => Self::Duty,
            ScheduleReason::Kept => Self::Kept,
        }
    }
}

impl From<ScheduledGame> for ScheduleGame {
    fn from(scheduled: ScheduledGame) -> Self {
        let game = scheduled.game;
        Self {
            id: game.source_id,
            date: game.date.format("%Y-%m-%d").to_string(),
            start_time: game.start_time.map(|time| time.format("%H:%M").to_string()),
            home: game.home.team_name,
            away: game.away.team_name,
            hall: game.place.hall,
            town: game.place.town,
            reason: scheduled.reason.into(),
            in_list: scheduled.in_list,
            places: None,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SeasonAnswer {
    /// The season of the events, for example "2026-2027".
    pub season: String,
    pub current_season: String,
    /// Dutch date: events of today and later are coming.
    pub today: String,
    /// The first and last day of the season, for "Add training".
    pub season_first_day: String,
    pub season_last_day: String,
    /// The seasons with trainings, roles, weekly trainings or a choice, and the current and the
    /// chosen season; newest first.
    pub seasons: Vec<String>,
    pub games: GamesState,
    /// The club and team of the season's choice; `null` without a choice (or without a team).
    pub club: Option<ClubView>,
    pub team: Option<TeamView>,
    /// Today and later, nearest first.
    pub coming: Vec<EventView>,
    /// Before today, newest first.
    pub past: Vec<EventView>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum GamesState {
    /// A team is chosen for the season: its games are in the list.
    Shown,
    /// The current season has no team yet: only trainings.
    TeamNeeded,
    /// An earlier season without a chosen team: only trainings.
    None,
}

/// One event: a game or a training (or a training day not opened). Exactly one of `game` and
/// `training` is set.
#[derive(Debug, Serialize, ToSchema)]
pub struct EventView {
    /// Dutch local date, for example "2026-10-31".
    pub date: String,
    /// The ISO week number.
    pub week: u32,
    /// The Monday and the Sunday of the week.
    pub week_first_day: String,
    pub week_last_day: String,
    pub game: Option<ScheduleGame>,
    pub training: Option<TrainingRow>,
}

/// The event with the duties of its game, if it has any.
pub(crate) fn event_view_with(event: &Event, places: &PlacesOf) -> Result<EventView, StoreError> {
    let mut view = event_view(event)?;
    if let (Some(game), EventKind::Game(scheduled)) = (view.game.as_mut(), event.kind) {
        game.places = places.get(&scheduled.game.source_id).map(PlacesView::from);
    }
    Ok(view)
}

pub(crate) fn event_view(event: &Event) -> Result<EventView, StoreError> {
    let (week, monday, sunday) = event
        .week()
        .ok_or_else(|| StoreError::Invalid(format!("no week for {}", event.date)))?;
    let (game, training) = match event.kind {
        EventKind::Game(scheduled) => (Some(ScheduleGame::from(scheduled.clone())), None),
        EventKind::Training(row) => (None, Some(row_view(row))),
    };
    Ok(EventView {
        date: day(event.date),
        week,
        week_first_day: day(monday),
        week_last_day: day(sunday),
        game,
        training,
    })
}

#[utoipa::path(
    get,
    path = "/api/season",
    operation_id = "season",
    params(SeasonQuery),
    responses(
        (status = 200, description = "All events of the season, coming and past", body = SeasonAnswer),
        (status = 400, description = "`invalid_season`"),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn get(
    State(state): State<AppState>,
    Query(query): Query<SeasonQuery>,
) -> Result<Json<SeasonAnswer>, AppError> {
    let season = query_season(query.season, state.clock.as_ref())?;
    let current = current_season(state.clock.as_ref());
    let today = today_in_netherlands(state.clock.as_ref());
    let games = games_of(&state, season, current, today).await?;
    let places = places_of_season(&games, today);
    let SeasonGames {
        state: games_state,
        club,
        team,
        schedule,
        ..
    } = games;
    let trainings = trainings_of(&state, season).await?;
    let list = trainings.list(season, today);
    let events = season_events(&schedule, &list);

    let seasons = seasons(&state, current, season).await?;
    Ok(Json(SeasonAnswer {
        season: season.to_string(),
        current_season: current.to_string(),
        today: day(today),
        season_first_day: day(season.first_day()),
        season_last_day: day(season.last_day()),
        seasons: seasons.iter().map(ToString::to_string).collect(),
        games: games_state,
        club: club.map(ClubView::from),
        team: team.map(TeamView::from),
        coming: events
            .coming
            .iter()
            .map(|event| event_view_with(event, &places))
            .collect::<Result<_, _>>()?,
        past: events
            .past
            .iter()
            .map(|event| event_view_with(event, &places))
            .collect::<Result<_, _>>()?,
    }))
}

/// The games of a season for its own choice.
pub(crate) struct SeasonGames {
    pub state: GamesState,
    pub club: Option<Club>,
    pub team: Option<Team>,
    /// Empty without a chosen team.
    pub schedule: Schedule,
    /// The changes not seen yet (the list of changes); 0 outside the current season.
    pub unseen_changes: u32,
    /// The stored duties and the roles of the season; empty without a chosen team.
    pub duties: SeasonDuties,
}

/// The duties of the games of `games` (the schedule of a season), by source id; empty without a
/// team.
pub(crate) fn places_of_season(games: &SeasonGames, today: NaiveDate) -> PlacesOf {
    let (Some(club), Some(team)) = (&games.club, &games.team) else {
        return PlacesOf::new();
    };
    games
        .schedule
        .coming
        .iter()
        .chain(&games.schedule.past)
        .filter_map(|scheduled| {
            let places = games.duties.places_of(
                &scheduled.game,
                scheduled.in_list,
                club.number,
                team.number,
                today,
            )?;
            Some((scheduled.game.source_id, places))
        })
        .collect()
}

/// The games of `season` from the team chosen for it. In the current season a gone game shows
/// while its change does, as in the list of changes; an earlier season shows no changes, so its
/// gone games are hidden.
pub(crate) async fn games_of(
    state: &AppState,
    season: Season,
    current: Season,
    today: NaiveDate,
) -> Result<SeasonGames, AppError> {
    let choice = followed_teams::for_season(&state.pool, season).await?;
    let Some(FollowedTeam {
        club,
        team: Some(team),
        ..
    }) = choice
    else {
        return Ok(SeasonGames {
            state: if season == current {
                GamesState::TeamNeeded
            } else {
                GamesState::None
            },
            club: None,
            team: None,
            schedule: Schedule::default(),
            unseen_changes: 0,
            duties: SeasonDuties {
                stored: BTreeMap::new(),
                roles: Vec::new(),
            },
        });
    };
    let mut db = state.pool.acquire().await.map_err(StoreError::from)?;
    let stored = games::stored(&mut db, club.number, season).await?;
    let changes = if season == current {
        api::changes::list(state, &mut db, club.number, team.number, season, &stored).await?
    } else {
        Vec::new()
    };
    let open: BTreeSet<i64> = changes.iter().map(|change| change.id).collect();
    let unseen_changes = u32::try_from(changes.len())
        .map_err(|_| StoreError::Invalid(format!("{} changes", changes.len())))?;
    let duties = SeasonDuties {
        stored: duties::for_season(&mut db, club.number, season).await?,
        roles: players::roles_in(&mut *db, &[season]).await?,
    };
    let schedule = schedule(
        stored,
        club.number,
        team.number,
        today,
        &open,
        &kept_games(&duties.stored),
    );
    Ok(SeasonGames {
        state: GamesState::Shown,
        club: Some(club),
        team: Some(team),
        schedule,
        unseen_changes,
        duties,
    })
}

/// What the training list of a season is made from; `list` borrows from it.
pub(crate) struct SeasonTrainings {
    pub weekly: Vec<WeeklyTraining>,
    pub trainings: Vec<Training>,
    pub absences: Vec<Absence>,
    /// The roles of the season, of all players.
    pub roles: Vec<PlayerRole>,
}

impl SeasonTrainings {
    /// The training days and trainings of `season` on `today` (`team_core::season_list`).
    pub fn list(&self, season: Season, today: NaiveDate) -> SeasonList<'_> {
        season_list(
            season,
            &self.weekly,
            &self.trainings,
            &self.absences,
            &self.roles,
            today,
        )
    }
}

/// The weekly trainings, trainings, absences and roles of `season`.
pub(crate) async fn trainings_of(
    state: &AppState,
    season: Season,
) -> Result<SeasonTrainings, AppError> {
    let weekly = weekly_trainings::for_season(&state.pool, season).await?;
    let trainings = trainings::for_season(&state.pool, season).await?;
    let ids: Vec<_> = trainings.iter().map(|training| training.id).collect();
    let absences = trainings::absences_at(&state.pool, &ids).await?;
    let roles = players::roles_in(&state.pool, &[season]).await?;
    Ok(SeasonTrainings {
        weekly,
        trainings,
        absences,
        roles,
    })
}

/// The seasons to choose from, newest first: those of "Attendance" and the seasons with a choice.
async fn seasons(
    state: &AppState,
    current: Season,
    chosen: Season,
) -> Result<Vec<Season>, StoreError> {
    let mut seasons: BTreeSet<Season> =
        trainings::seasons(&state.pool).await?.into_iter().collect();
    seasons.extend(followed_teams::seasons(&state.pool).await?);
    seasons.extend([current, chosen]);
    Ok(seasons.into_iter().rev().collect())
}
