//! `/api/trainings`: the trainings of a season, the next training, and one training with its
//! players (attendance at training, story B). Names never go to the log; only dates and ids do.

use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use team_core::{
    DayStatus, Season, SeasonRow, Training, absent_count, players_at, proposed_times, season_list,
};
use utoipa::ToSchema;

use super::players::RoleKind;
use super::reasons::{AbsenceReason, view as reason_view};
use crate::app::AppState;
use crate::clock::{current_season, today_in_netherlands};
use crate::error::AppError;
use crate::{players, reasons, trainings, weekly_trainings};

/// One row of the list: a training, or a training day that is not opened.
#[derive(Debug, Serialize, ToSchema)]
pub struct TrainingRow {
    /// For example "2026-10-13".
    pub date: String,
    /// For example "19:45".
    pub start: String,
    pub end: String,
    /// `not_opened`, `all_there`, `absent` or `cancelled`.
    pub status: String,
    /// The number of absences of players with a role on the date (0 when not opened).
    pub absent: usize,
    /// A training exists on the date (also when it is cancelled).
    pub opened: bool,
    /// A training on a date that is not a training day.
    pub extra: bool,
}

/// The list of a season: today and later, nearest first; before today, newest first.
#[derive(Debug, Serialize, ToSchema)]
pub struct TrainingList {
    pub season: String,
    pub today: String,
    pub season_first_day: String,
    pub season_last_day: String,
    pub coming: Vec<TrainingRow>,
    pub past: Vec<TrainingRow>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct SeasonQuery {
    /// For example "2026-2027"; the current season when left out.
    pub season: Option<String>,
}

/// The times of a training.
#[derive(Debug, Serialize, ToSchema)]
pub struct Times {
    pub start: String,
    pub end: String,
}

/// An opened training.
#[derive(Debug, Serialize, ToSchema)]
pub struct TrainingView {
    pub start: String,
    pub end: String,
    pub cancelled: bool,
}

/// A player at a training.
#[derive(Debug, Serialize, ToSchema)]
pub struct AtTrainingView {
    pub id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub role: RoleKind,
    /// The reason of the absence; `null` when the player is there.
    pub reason: Option<i64>,
}

/// One date: its training with the players, or nothing yet.
#[derive(Debug, Serialize, ToSchema)]
pub struct TrainingDay {
    pub date: String,
    pub season: String,
    /// `null` while no training exists on the date.
    pub training: Option<TrainingView>,
    /// The times "Add training" proposes: the training day's, or a weekly training's on the same
    /// day of the week; `null` when there are none.
    pub proposed_times: Option<Times>,
    pub absent: usize,
    /// The players with a role on the date, sorted by first name; empty while not opened.
    pub players: Vec<AtTrainingView>,
    /// All reasons in their order, also the hidden ones (they still show on old absences).
    pub reasons: Vec<AbsenceReason>,
}

pub(crate) fn day(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

fn clock_time(time: NaiveTime) -> String {
    time.format("%H:%M").to_string()
}

/// A date in a path, for example "2026-10-13"; else `invalid_date`.
pub(crate) fn path_date(text: &str) -> Result<NaiveDate, AppError> {
    NaiveDate::parse_from_str(text, "%Y-%m-%d").map_err(|_| AppError::BadRequest("invalid_date"))
}

fn row_view(row: &SeasonRow) -> TrainingRow {
    let (status, absent) = match row.status {
        DayStatus::NotOpened => ("not_opened", 0),
        DayStatus::AllThere => ("all_there", 0),
        DayStatus::Absent(n) => ("absent", n),
        DayStatus::Cancelled => ("cancelled", 0),
    };
    TrainingRow {
        date: day(row.date),
        start: clock_time(row.start),
        end: clock_time(row.end),
        status: status.to_owned(),
        absent,
        opened: row.training.is_some(),
        extra: row.extra,
    }
}

/// The list of `season`.
pub(crate) async fn list_of(state: &AppState, season: Season) -> Result<TrainingList, AppError> {
    let today = today_in_netherlands(state.clock.as_ref());
    let weekly = weekly_trainings::for_season(&state.pool, season).await?;
    let stored = trainings::for_season(&state.pool, season).await?;
    let ids: Vec<_> = stored.iter().map(|training| training.id).collect();
    let absences = trainings::absences_at(&state.pool, &ids).await?;
    let roles = players::roles_in(&state.pool, &[season]).await?;
    let list = season_list(season, &weekly, &stored, &absences, &roles, today);
    Ok(TrainingList {
        season: season.to_string(),
        today: day(today),
        season_first_day: day(season.first_day()),
        season_last_day: day(season.last_day()),
        coming: list.coming.iter().map(row_view).collect(),
        past: list.past.iter().map(row_view).collect(),
    })
}

/// The training day `date`: its training with the players, or nothing yet.
pub(crate) async fn day_of(state: &AppState, date: NaiveDate) -> Result<TrainingDay, AppError> {
    let season = Season::of(date);
    let weekly = weekly_trainings::for_season(&state.pool, season).await?;
    let training = trainings::on(&state.pool, date).await?;
    let reasons = reasons::all(&state.pool)
        .await?
        .into_iter()
        .map(reason_view)
        .collect();
    let mut view = TrainingDay {
        date: day(date),
        season: season.to_string(),
        training: None,
        proposed_times: proposed_times(&weekly, date).map(|(start, end)| Times {
            start: clock_time(start),
            end: clock_time(end),
        }),
        absent: 0,
        players: Vec::new(),
        reasons,
    };
    if let Some(training) = training {
        fill(state, season, &training, &mut view).await?;
    }
    Ok(view)
}

async fn fill(
    state: &AppState,
    season: Season,
    training: &Training,
    view: &mut TrainingDay,
) -> Result<(), AppError> {
    let all = players::all(&state.pool).await?;
    let roles = players::roles_in(&state.pool, &[season]).await?;
    let absences = trainings::absences_at(&state.pool, &[training.id]).await?;
    view.training = Some(TrainingView {
        start: clock_time(training.start),
        end: clock_time(training.end),
        cancelled: training.cancelled,
    });
    view.absent = absent_count(training, &absences, &roles);
    view.players = players_at(&all, &roles, &absences, training)
        .iter()
        .map(|entry| AtTrainingView {
            id: entry.player.id.0,
            first_name: entry.player.first_name.clone(),
            last_name: entry.player.last_name.clone(),
            role: entry.role.into(),
            reason: entry.absence.map(|reason| reason.0),
        })
        .collect();
    Ok(())
}

#[utoipa::path(
    get,
    path = "/api/trainings",
    operation_id = "trainings",
    params(SeasonQuery),
    responses(
        (status = 200, description = "The training days and trainings of the season, one row per date", body = TrainingList),
        (status = 400, description = "`invalid_season`"),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<SeasonQuery>,
) -> Result<Json<TrainingList>, AppError> {
    let season = match query.season {
        Some(text) => text
            .parse::<Season>()
            .map_err(|_| AppError::BadRequest("invalid_season"))?,
        None => current_season(state.clock.as_ref()),
    };
    Ok(Json(list_of(&state, season).await?))
}

#[utoipa::path(
    get,
    path = "/api/trainings/next",
    operation_id = "nextTraining",
    responses(
        (status = 200, description = "Today's training until midnight, else the next training day or training of the current season; `null` when there is none", body = Option<TrainingRow>),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn next(State(state): State<AppState>) -> Result<Json<Option<TrainingRow>>, AppError> {
    let season = current_season(state.clock.as_ref());
    let mut list = list_of(&state, season).await?;
    Ok(Json(if list.coming.is_empty() {
        None
    } else {
        Some(list.coming.swap_remove(0))
    }))
}

#[utoipa::path(
    get,
    path = "/api/trainings/{date}",
    operation_id = "training",
    params(("date" = String, Path, description = "The date, for example 2026-10-13")),
    responses(
        (status = 200, description = "The training of the date with its players, or nothing yet", body = TrainingDay),
        (status = 400, description = "`invalid_date`"),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn get(
    State(state): State<AppState>,
    Path(date): Path<String>,
) -> Result<Json<TrainingDay>, AppError> {
    Ok(Json(day_of(&state, path_date(&date)?).await?))
}
