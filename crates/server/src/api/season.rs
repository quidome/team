//! `GET /api/season`: all events of a season for the page "Season" (story A of start page and
//! season): the games of the team and the possible duty games, and the training days and
//! trainings, in one list with their ISO weeks.

use std::collections::BTreeSet;

use axum::Json;
use axum::extract::{Query, State};
use chrono::NaiveDate;
use serde::Serialize;
use team_core::{
    Absence, Club, Event, EventKind, FollowedTeam, PlayerRole, Schedule, ScheduleReason,
    ScheduledGame, Season, SeasonList, Team, Training, WeeklyTraining, schedule, season_events,
    season_list,
};
use utoipa::ToSchema;

use super::format::{day, query_season};
use super::trainings::{SeasonQuery, TrainingRow, row_view};
use crate::api::clubs::{ClubView, TeamView};
use crate::app::AppState;
use crate::clock::{current_season, today_in_netherlands};
use crate::db::StoreError;
use crate::error::AppError;
use crate::{api, followed_teams, games, players, trainings, weekly_trainings};

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
    /// False: the game is no longer in the game list; it shows until its change is seen.
    pub in_list: bool,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleReasonView {
    /// A game of the team, home or away.
    Team,
    /// A home game of another club team: a possible duty game.
    Duty,
}

impl From<ScheduleReason> for ScheduleReasonView {
    fn from(reason: ScheduleReason) -> Self {
        match reason {
            ScheduleReason::TeamGame => Self::Team,
            ScheduleReason::DutyGame => Self::Duty,
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
    let SeasonGames {
        state: games_state,
        club,
        team,
        schedule,
        ..
    } = games_of(&state, season, current, today).await?;
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
            .map(event_view)
            .collect::<Result<_, _>>()?,
        past: events
            .past
            .iter()
            .map(event_view)
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
    let schedule = schedule(stored, club.number, team.number, today, &open);
    Ok(SeasonGames {
        state: GamesState::Shown,
        club: Some(club),
        team: Some(team),
        schedule,
        unseen_changes,
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
