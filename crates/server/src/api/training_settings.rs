//! `/api/training-settings`: the training days and the first and last training date of the current
//! season (attendance at training, story A).

use axum::Json;
use axum::extract::{Query, State};
use chrono::{NaiveDate, NaiveTime, Weekday};
use serde::{Deserialize, Serialize};
use team_core::{
    TrainingDay, TrainingError, proposed_dates, replaced_from, settings_change, shown_set,
};
use utoipa::{IntoParams, ToSchema};

use crate::app::AppState;
use crate::clock::{current_season, today_in_netherlands};
use crate::db::StoreError;
use crate::error::AppError;
use crate::training_settings::{self, weekday_from_number};

/// A day of the week the team trains, with its times.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TrainingDayView {
    /// 1 (Monday) to 7 (Sunday).
    pub weekday: i16,
    /// For example "19:45".
    pub start: String,
    pub end: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TrainingSettings {
    pub season: String,
    /// `false` while nothing is saved for the season; then the dates are `null` and `days` is
    /// empty.
    pub saved: bool,
    pub first: Option<String>,
    pub last: Option<String>,
    /// The training days that apply today (before the first training date: the first ones).
    pub days: Vec<TrainingDayView>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SettingsChange {
    #[schema(value_type = String, format = Date)]
    pub first: NaiveDate,
    #[schema(value_type = String, format = Date)]
    pub last: NaiveDate,
    pub days: Vec<TrainingDayView>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ProposalQuery {
    /// The chosen days of the week, for example "2" or "2,4".
    pub weekdays: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Proposal {
    pub first: String,
    pub last: String,
}

impl From<TrainingError> for AppError {
    fn from(error: TrainingError) -> Self {
        Self::BadRequest(match error {
            TrainingError::EndBeforeStart => "end_before_start",
            TrainingError::LastBeforeFirst => "last_before_first",
            TrainingError::NoTrainingDays => "no_training_days",
            TrainingError::DuplicateWeekday => "duplicate_weekday",
            TrainingError::OutsideSeason => "outside_season",
        })
    }
}

fn day(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

fn weekday(number: i16) -> Result<Weekday, AppError> {
    weekday_from_number(number).ok_or(AppError::BadRequest("invalid_weekday"))
}

fn time(text: &str) -> Result<NaiveTime, AppError> {
    NaiveTime::parse_from_str(text, "%H:%M").map_err(|_| AppError::BadRequest("invalid_time"))
}

fn day_view(day: &TrainingDay) -> Result<TrainingDayView, StoreError> {
    Ok(TrainingDayView {
        weekday: i16::try_from(day.weekday.number_from_monday())
            .map_err(|_| StoreError::Invalid(format!("weekday {}", day.weekday)))?,
        start: day.start.format("%H:%M").to_string(),
        end: day.end.format("%H:%M").to_string(),
    })
}

async fn current(state: &AppState) -> Result<TrainingSettings, AppError> {
    let season = current_season(state.clock.as_ref());
    let today = today_in_netherlands(state.clock.as_ref());
    let mut db = state.pool.acquire().await.map_err(StoreError::from)?;
    let Some(stored) = training_settings::for_season(&mut db, season).await? else {
        return Ok(TrainingSettings {
            season: season.to_string(),
            saved: false,
            first: None,
            last: None,
            days: Vec::new(),
        });
    };
    let days = shown_set(&stored, today)
        .map(|set| set.days.iter().map(day_view).collect::<Result<Vec<_>, _>>())
        .transpose()?
        .unwrap_or_default();
    Ok(TrainingSettings {
        season: season.to_string(),
        saved: true,
        first: Some(day(stored.first)),
        last: Some(day(stored.last)),
        days,
    })
}

#[utoipa::path(
    get,
    path = "/api/training-settings",
    operation_id = "trainingSettings",
    responses(
        (status = 200, description = "The training settings of the current season", body = TrainingSettings),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn get(State(state): State<AppState>) -> Result<Json<TrainingSettings>, AppError> {
    Ok(Json(current(&state).await?))
}

#[utoipa::path(
    get,
    path = "/api/training-settings/proposal",
    operation_id = "trainingDatesProposal",
    params(ProposalQuery),
    responses(
        (status = 200, description = "The first chosen day in September and the last in May", body = Proposal),
        (status = 400, description = "`no_training_days`, `invalid_weekday`"),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn proposal(
    State(state): State<AppState>,
    Query(query): Query<ProposalQuery>,
) -> Result<Json<Proposal>, AppError> {
    let weekdays = query
        .weekdays
        .split(',')
        .filter(|part| !part.trim().is_empty())
        .map(|part| {
            part.trim()
                .parse::<i16>()
                .map_err(|_| AppError::BadRequest("invalid_weekday"))
                .and_then(weekday)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let season = current_season(state.clock.as_ref());
    let (first, last) = proposed_dates(season, &weekdays).ok_or(TrainingError::NoTrainingDays)?;
    Ok(Json(Proposal {
        first: day(first),
        last: day(last),
    }))
}

#[utoipa::path(
    put,
    path = "/api/training-settings",
    operation_id = "saveTrainingSettings",
    request_body = SettingsChange,
    responses(
        (status = 200, description = "The saved settings", body = TrainingSettings),
        (status = 400, description = "`end_before_start`, `last_before_first`, `no_training_days`, `duplicate_weekday`, `outside_season`, `invalid_weekday`, `invalid_time`"),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn save(
    State(state): State<AppState>,
    Json(body): Json<SettingsChange>,
) -> Result<Json<TrainingSettings>, AppError> {
    let days = body
        .days
        .iter()
        .map(|day| {
            Ok(TrainingDay {
                weekday: weekday(day.weekday)?,
                start: time(&day.start)?,
                end: time(&day.end)?,
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;
    let season = current_season(state.clock.as_ref());
    let today = today_in_netherlands(state.clock.as_ref());
    let mut tx = state.pool.begin().await.map_err(StoreError::from)?;
    training_settings::lock(&mut tx).await?;
    let first_save = training_settings::for_season(&mut tx, season)
        .await?
        .is_none_or(|stored| stored.sets.is_empty());
    let set = settings_change(season, body.first, body.last, days, today, first_save)?;
    let replace_from = replaced_from(&set, today);
    training_settings::save(&mut tx, season, body.first, body.last, &set, replace_from).await?;
    tx.commit().await.map_err(StoreError::from)?;
    Ok(Json(current(&state).await?))
}
