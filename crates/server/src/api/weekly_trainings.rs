//! `/api/weekly-trainings`: the weekly trainings of the current season (attendance at training,
//! story A). A weekly training only suggests training days.

use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::{NaiveDate, NaiveTime, Weekday};
use serde::{Deserialize, Serialize};
use team_core::{
    Season, TrainingError, WeeklyTraining, WeeklyTrainingId, check_weekly_training, proposed_window,
};
use utoipa::{IntoParams, ToSchema};

use crate::app::AppState;
use crate::clock::current_season;
use crate::db::StoreError;
use crate::error::AppError;
use crate::weekly_trainings::{self, WeeklyTrainingData, weekday_from_number, weekday_number};

/// A weekly training: a day of the week with its times, in a window.
#[derive(Debug, Serialize, ToSchema)]
pub struct WeeklyTrainingView {
    pub id: i64,
    /// 1 (Monday) to 7 (Sunday).
    pub weekday: i16,
    /// For example "19:45".
    pub start: String,
    pub end: String,
    /// The first day of the window, for example "2026-09-01".
    pub from: String,
    /// The last day of the window.
    pub until: String,
}

/// The weekly trainings of the current season, sorted by the start of the window.
#[derive(Debug, Serialize, ToSchema)]
pub struct WeeklyTrainings {
    pub season: String,
    /// The first day of the season, for example "2026-07-01": the earliest date of a window.
    pub season_first_day: String,
    /// The last day of the season, for example "2027-06-30": the latest date of a window.
    pub season_last_day: String,
    pub weekly_trainings: Vec<WeeklyTrainingView>,
}

/// A new or changed weekly training.
#[derive(Debug, Deserialize, ToSchema)]
pub struct WeeklyTrainingInput {
    /// 1 (Monday) to 7 (Sunday).
    pub weekday: i16,
    /// For example "19:45".
    pub start: String,
    pub end: String,
    #[schema(value_type = String, format = Date)]
    pub from: NaiveDate,
    #[schema(value_type = String, format = Date)]
    pub until: NaiveDate,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ProposalQuery {
    /// The day of the week, 1 (Monday) to 7 (Sunday).
    pub weekday: i16,
}

/// The proposed window of a weekly training.
#[derive(Debug, Serialize, ToSchema)]
pub struct WindowProposal {
    pub from: String,
    pub until: String,
}

impl From<TrainingError> for AppError {
    fn from(error: TrainingError) -> Self {
        Self::BadRequest(match error {
            TrainingError::EndBeforeStart => "end_before_start",
            TrainingError::LastBeforeFirst => "last_before_first",
            TrainingError::OutsideSeason => "outside_season",
            TrainingError::WeeklyTrainingsOverlap => "weekly_trainings_overlap",
            TrainingError::TrainingExists => "training_exists",
            TrainingError::NotATrainingDay => "not_a_training_day",
        })
    }
}

fn day(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

fn clock_time(time: NaiveTime) -> String {
    time.format("%H:%M").to_string()
}

fn weekday(number: i16) -> Result<Weekday, AppError> {
    weekday_from_number(number).ok_or(AppError::BadRequest("invalid_weekday"))
}

fn time(text: &str) -> Result<NaiveTime, AppError> {
    NaiveTime::parse_from_str(text, "%H:%M").map_err(|_| AppError::BadRequest("invalid_time"))
}

fn view(training: &WeeklyTraining) -> Result<WeeklyTrainingView, StoreError> {
    Ok(WeeklyTrainingView {
        id: training.id.0,
        weekday: weekday_number(training.weekday)?,
        start: clock_time(training.start),
        end: clock_time(training.end),
        from: day(training.from),
        until: day(training.until),
    })
}

async fn load(state: &AppState, season: Season) -> Result<WeeklyTrainings, AppError> {
    let stored = weekly_trainings::for_season(&state.pool, season).await?;
    Ok(WeeklyTrainings {
        season: season.to_string(),
        season_first_day: day(season.first_day()),
        season_last_day: day(season.last_day()),
        weekly_trainings: stored.iter().map(view).collect::<Result<_, _>>()?,
    })
}

fn data(input: &WeeklyTrainingInput) -> Result<WeeklyTrainingData, AppError> {
    Ok(WeeklyTrainingData {
        weekday: weekday(input.weekday)?,
        start: time(&input.start)?,
        end: time(&input.end)?,
        from: input.from,
        until: input.until,
    })
}

/// Checks `data` against the other weekly trainings of `season` (without `changed`), under the
/// lock, and writes it with `write`.
async fn save(
    state: &AppState,
    season: Season,
    changed: Option<WeeklyTrainingId>,
    data: WeeklyTrainingData,
) -> Result<(), AppError> {
    let mut tx = state.pool.begin().await.map_err(StoreError::from)?;
    weekly_trainings::lock(&mut tx).await?;
    let stored = weekly_trainings::for_season(&mut *tx, season).await?;
    if changed.is_some_and(|id| !stored.iter().any(|training| training.id == id)) {
        return Err(AppError::NotFound);
    }
    let others: Vec<WeeklyTraining> = stored
        .into_iter()
        .filter(|training| Some(training.id) != changed)
        .collect();
    check_weekly_training(
        season,
        data.weekday,
        data.start,
        data.end,
        data.from,
        data.until,
        &others,
    )?;
    match changed {
        None => {
            weekly_trainings::insert(&mut *tx, season, data).await?;
        }
        Some(id) => {
            weekly_trainings::update(&mut *tx, season, id, data).await?;
        }
    }
    tx.commit().await.map_err(StoreError::from)?;
    Ok(())
}

#[utoipa::path(
    get,
    path = "/api/weekly-trainings",
    operation_id = "weeklyTrainings",
    responses(
        (status = 200, description = "The weekly trainings of the current season", body = WeeklyTrainings),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn list(State(state): State<AppState>) -> Result<Json<WeeklyTrainings>, AppError> {
    let season = current_season(state.clock.as_ref());
    Ok(Json(load(&state, season).await?))
}

#[utoipa::path(
    get,
    path = "/api/weekly-trainings/proposal",
    operation_id = "weeklyTrainingProposal",
    params(ProposalQuery),
    responses(
        (status = 200, description = "That day on or after 1 September until that day on or before 31 May", body = WindowProposal),
        (status = 400, description = "`invalid_weekday`"),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn proposal(
    State(state): State<AppState>,
    Query(query): Query<ProposalQuery>,
) -> Result<Json<WindowProposal>, AppError> {
    let season = current_season(state.clock.as_ref());
    let (from, until) = proposed_window(season, weekday(query.weekday)?)
        .ok_or(AppError::BadRequest("invalid_weekday"))?;
    Ok(Json(WindowProposal {
        from: day(from),
        until: day(until),
    }))
}

#[utoipa::path(
    post,
    path = "/api/weekly-trainings",
    operation_id = "addWeeklyTraining",
    request_body = WeeklyTrainingInput,
    responses(
        (status = 200, description = "The weekly trainings of the current season", body = WeeklyTrainings),
        (status = 400, description = "`end_before_start`, `last_before_first`, `outside_season`, `weekly_trainings_overlap`, `invalid_weekday`, `invalid_time`"),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn add(
    State(state): State<AppState>,
    Json(body): Json<WeeklyTrainingInput>,
) -> Result<Json<WeeklyTrainings>, AppError> {
    let season = current_season(state.clock.as_ref());
    save(&state, season, None, data(&body)?).await?;
    tracing::info!("weekly training added");
    Ok(Json(load(&state, season).await?))
}

#[utoipa::path(
    put,
    path = "/api/weekly-trainings/{id}",
    operation_id = "changeWeeklyTraining",
    params(("id" = i64, Path, description = "The weekly training")),
    request_body = WeeklyTrainingInput,
    responses(
        (status = 200, description = "The weekly trainings of the current season", body = WeeklyTrainings),
        (status = 400, description = "`end_before_start`, `last_before_first`, `outside_season`, `weekly_trainings_overlap`, `invalid_weekday`, `invalid_time`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such weekly training in the current season"),
    )
)]
pub async fn change(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<WeeklyTrainingInput>,
) -> Result<Json<WeeklyTrainings>, AppError> {
    let season = current_season(state.clock.as_ref());
    save(&state, season, Some(WeeklyTrainingId(id)), data(&body)?).await?;
    tracing::info!(weekly_training = id, "weekly training changed");
    Ok(Json(load(&state, season).await?))
}

#[utoipa::path(
    delete,
    path = "/api/weekly-trainings/{id}",
    operation_id = "deleteWeeklyTraining",
    params(("id" = i64, Path, description = "The weekly training")),
    responses(
        (status = 200, description = "The weekly trainings of the current season", body = WeeklyTrainings),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such weekly training in the current season"),
    )
)]
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<WeeklyTrainings>, AppError> {
    let season = current_season(state.clock.as_ref());
    let mut tx = state.pool.begin().await.map_err(StoreError::from)?;
    weekly_trainings::lock(&mut tx).await?;
    if !weekly_trainings::delete(&mut *tx, season, WeeklyTrainingId(id)).await? {
        return Err(AppError::NotFound);
    }
    tx.commit().await.map_err(StoreError::from)?;
    tracing::info!(weekly_training = id, "weekly training deleted");
    Ok(Json(load(&state, season).await?))
}
