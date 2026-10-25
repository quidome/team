//! `GET /api/season`: all events of a season for the page "Season" (story A of start page and
//! season): the games of the team and the possible duty games, and the training days and
//! trainings, in one list with their ISO weeks.

use std::collections::BTreeSet;

use axum::Json;
use axum::extract::{Query, State};
use serde::Serialize;
use team_core::{
    Event, EventKind, FollowedTeam, Schedule, Season, schedule, season_events, season_list,
};
use utoipa::ToSchema;

use super::format::{day, query_season};
use super::trainings::{SeasonQuery, TrainingRow, row_view};
use crate::api::clubs::{ClubView, TeamView};
use crate::api::schedule::ScheduleGame;
use crate::app::AppState;
use crate::clock::{current_season, today_in_netherlands};
use crate::db::StoreError;
use crate::error::AppError;
use crate::{api, followed_teams, games, players, trainings, weekly_trainings};

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

fn event_view(event: &Event) -> Result<EventView, StoreError> {
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
    let choice = followed_teams::for_season(&state.pool, season).await?;

    let (games_state, club, team, schedule) = match choice {
        Some(FollowedTeam {
            club,
            team: Some(team),
            ..
        }) => {
            let mut db = state.pool.acquire().await.map_err(StoreError::from)?;
            let stored = games::stored(&mut db, club.number, season).await?;
            // In the current season a gone game shows while its change does, as in the schedule;
            // an earlier season shows no changes, so its gone games are hidden.
            let open: BTreeSet<i64> = if season == current {
                api::changes::list(&state, &mut db, club.number, team.number, season, &stored)
                    .await?
                    .iter()
                    .map(|change| change.id)
                    .collect()
            } else {
                BTreeSet::new()
            };
            let schedule = schedule(stored, club.number, team.number, today, &open);
            (
                GamesState::Shown,
                Some(club.into()),
                Some(team.into()),
                schedule,
            )
        }
        _ if season == current => (GamesState::TeamNeeded, None, None, Schedule::default()),
        _ => (GamesState::None, None, None, Schedule::default()),
    };

    let weekly = weekly_trainings::for_season(&state.pool, season).await?;
    let stored = trainings::for_season(&state.pool, season).await?;
    let ids: Vec<_> = stored.iter().map(|training| training.id).collect();
    let absences = trainings::absences_at(&state.pool, &ids).await?;
    let roles = players::roles_in(&state.pool, &[season]).await?;
    let list = season_list(season, &weekly, &stored, &absences, &roles, today);
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
        club,
        team,
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
