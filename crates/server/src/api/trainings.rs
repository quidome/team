//! `/api/trainings`: the trainings of a season, the next training, and one training with its
//! players (attendance at training, story B). Names never go to the log; only dates and ids do.

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use team_core::{
    AttendanceError, DayStatus, PlayerId, ReasonId, Season, SeasonRow, Training, absent_count,
    check_mark, check_training, day_times, players_at, proposed_times, season_list,
};
use utoipa::ToSchema;

use super::format::{clock_time, day, path_date, query_season, time};
use super::players::RoleKind;
use super::reasons::{AbsenceReason, view as reason_view};
use crate::app::AppState;
use crate::clock::{current_season, today_in_netherlands};
use crate::db::StoreError;
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
#[into_params(parameter_in = Query)]
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

/// "Add training" and "Change": the date and the times of a training.
#[derive(Debug, Deserialize, ToSchema)]
pub struct TrainingInput {
    #[schema(value_type = String, format = Date)]
    pub date: NaiveDate,
    /// For example "19:45".
    pub start: String,
    pub end: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MarkAbsent {
    pub reason: i64,
}

/// The answer of a mark: kept small, so a tap is quick.
#[derive(Debug, Serialize, ToSchema)]
pub struct Marked {
    pub player: i64,
    /// `null` when the player is there.
    pub reason: Option<i64>,
    /// The training's new number of absences.
    pub absent: usize,
}

impl From<AttendanceError> for AppError {
    fn from(error: AttendanceError) -> Self {
        Self::BadRequest(match error {
            AttendanceError::NotInTeam => "not_in_team",
            AttendanceError::ReasonHidden => "reason_hidden",
        })
    }
}

/// A date that has a training, refused by the unique index (a save at the same time).
fn taken(error: StoreError) -> AppError {
    if error.is_unique_violation() {
        AppError::BadRequest("training_exists")
    } else {
        error.into()
    }
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
    let season = query_season(query.season, state.clock.as_ref())?;
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

#[utoipa::path(
    put,
    path = "/api/trainings/{date}/open",
    operation_id = "openTraining",
    params(("date" = String, Path, description = "The date of a training day")),
    responses(
        (status = 200, description = "The training of the date, made with the times of its weekly training when it did not exist; safe to send twice", body = TrainingDay),
        (status = 400, description = "`invalid_date`, `not_a_training_day`"),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn open(
    State(state): State<AppState>,
    Path(date): Path<String>,
) -> Result<Json<TrainingDay>, AppError> {
    let date = path_date(&date)?;
    if trainings::on(&state.pool, date).await?.is_none() {
        let weekly = weekly_trainings::for_season(&state.pool, Season::of(date)).await?;
        let (start, end) = day_times(&weekly, date)?;
        trainings::open(&state.pool, date, start, end, state.clock.now()).await?;
        tracing::info!(%date, "training opened");
    }
    Ok(Json(day_of(&state, date).await?))
}

#[utoipa::path(
    post,
    path = "/api/trainings",
    operation_id = "addTraining",
    request_body = TrainingInput,
    responses(
        (status = 200, description = "The new training", body = TrainingDay),
        (status = 400, description = "`training_exists`, `outside_season`, `end_before_start`, `invalid_time`"),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn add(
    State(state): State<AppState>,
    Json(body): Json<TrainingInput>,
) -> Result<Json<TrainingDay>, AppError> {
    let season = current_season(state.clock.as_ref());
    let (start, end) = (time(&body.start)?, time(&body.end)?);
    let stored = trainings::for_season(&state.pool, season).await?;
    check_training(season, body.date, start, end, &stored, None)?;
    trainings::insert(&state.pool, body.date, start, end, state.clock.now())
        .await
        .map_err(taken)?;
    tracing::info!(date = %body.date, "training added");
    Ok(Json(day_of(&state, body.date).await?))
}

#[utoipa::path(
    put,
    path = "/api/trainings/{date}",
    operation_id = "changeTraining",
    params(("date" = String, Path, description = "The date of the training now")),
    request_body = TrainingInput,
    responses(
        (status = 200, description = "The training on its new date, with its attendance", body = TrainingDay),
        (status = 400, description = "`training_exists`, `outside_season`, `end_before_start`, `invalid_time`, `invalid_date`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No training on the date"),
    )
)]
pub async fn change(
    State(state): State<AppState>,
    Path(date): Path<String>,
    Json(body): Json<TrainingInput>,
) -> Result<Json<TrainingDay>, AppError> {
    let date = path_date(&date)?;
    let training = trainings::on(&state.pool, date)
        .await?
        .ok_or(AppError::NotFound)?;
    let season = Season::of(date);
    let (start, end) = (time(&body.start)?, time(&body.end)?);
    let stored = trainings::for_season(&state.pool, season).await?;
    check_training(season, body.date, start, end, &stored, Some(training.id))?;
    if !trainings::update(&state.pool, training.id, body.date, start, end)
        .await
        .map_err(taken)?
    {
        return Err(AppError::NotFound);
    }
    tracing::info!(from = %date, to = %body.date, "training changed");
    Ok(Json(day_of(&state, body.date).await?))
}

#[utoipa::path(
    post,
    path = "/api/trainings/{date}/cancel",
    operation_id = "cancelTraining",
    params(("date" = String, Path, description = "The date of a training or a training day")),
    responses(
        (status = 200, description = "The cancelled training; a training day gets a cancelled training", body = TrainingDay),
        (status = 400, description = "`invalid_date`, `not_a_training_day`"),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn cancel(
    State(state): State<AppState>,
    Path(date): Path<String>,
) -> Result<Json<TrainingDay>, AppError> {
    let date = path_date(&date)?;
    let (start, end) = match trainings::on(&state.pool, date).await? {
        Some(training) => (training.start, training.end),
        None => {
            let weekly = weekly_trainings::for_season(&state.pool, Season::of(date)).await?;
            day_times(&weekly, date)?
        }
    };
    trainings::cancel(&state.pool, date, start, end, state.clock.now()).await?;
    tracing::info!(%date, "training cancelled");
    Ok(Json(day_of(&state, date).await?))
}

#[utoipa::path(
    post,
    path = "/api/trainings/{date}/uncancel",
    operation_id = "uncancelTraining",
    params(("date" = String, Path, description = "The date of the training")),
    responses(
        (status = 200, description = "The training, no longer cancelled", body = TrainingDay),
        (status = 400, description = "`invalid_date`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No training on the date"),
    )
)]
pub async fn uncancel(
    State(state): State<AppState>,
    Path(date): Path<String>,
) -> Result<Json<TrainingDay>, AppError> {
    let date = path_date(&date)?;
    if !trainings::uncancel(&state.pool, date).await? {
        return Err(AppError::NotFound);
    }
    tracing::info!(%date, "training cancel undone");
    Ok(Json(day_of(&state, date).await?))
}

#[utoipa::path(
    delete,
    path = "/api/trainings/{date}",
    operation_id = "deleteTraining",
    params(("date" = String, Path, description = "The date of the training")),
    responses(
        (status = 204, description = "The training and its absences are deleted"),
        (status = 400, description = "`invalid_date`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No training on the date"),
    )
)]
pub async fn delete(
    State(state): State<AppState>,
    Path(date): Path<String>,
) -> Result<StatusCode, AppError> {
    let date = path_date(&date)?;
    if !trainings::delete(&state.pool, date).await? {
        return Err(AppError::NotFound);
    }
    tracing::info!(%date, "training deleted");
    Ok(StatusCode::NO_CONTENT)
}

/// The training on `date`, else 404.
async fn training_on(state: &AppState, date: &str) -> Result<Training, AppError> {
    trainings::on(&state.pool, path_date(date)?)
        .await?
        .ok_or(AppError::NotFound)
}

/// The answer of a mark: the player's reason now and the training's number of absences.
async fn marked(
    state: &AppState,
    training: &Training,
    player: PlayerId,
    reason: Option<ReasonId>,
) -> Result<Marked, AppError> {
    let absences = trainings::absences_at(&state.pool, &[training.id]).await?;
    let roles = players::roles_in(&state.pool, &[Season::of(training.date)]).await?;
    Ok(Marked {
        player: player.0,
        reason: reason.map(|reason| reason.0),
        absent: absent_count(training, &absences, &roles),
    })
}

#[utoipa::path(
    put,
    path = "/api/trainings/{date}/absences/{player}",
    operation_id = "markAbsent",
    params(
        ("date" = String, Path, description = "The date of the training"),
        ("player" = i64, Path, description = "The player"),
    ),
    request_body = MarkAbsent,
    responses(
        (status = 200, description = "The player is absent with the reason; the last mark wins", body = Marked),
        (status = 400, description = "`not_in_team`, `reason_hidden`, `invalid_date`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No training on the date, no such player or reason"),
    )
)]
pub async fn mark(
    State(state): State<AppState>,
    Path((date, player)): Path<(String, i64)>,
    Json(body): Json<MarkAbsent>,
) -> Result<Json<Marked>, AppError> {
    let training = training_on(&state, &date).await?;
    let player = PlayerId(player);
    if players::by_id(&state.pool, player).await?.is_none() {
        return Err(AppError::NotFound);
    }
    let reason = reasons::all(&state.pool)
        .await?
        .into_iter()
        .find(|reason| reason.id == body.reason)
        .ok_or(AppError::NotFound)?;
    let roles = players::roles_of(&state.pool, player).await?;
    let current = trainings::absences_at(&state.pool, &[training.id])
        .await?
        .into_iter()
        .find(|absence| absence.player == player)
        .map(|absence| absence.reason);
    let reason_id = ReasonId(reason.id);
    check_mark(player, &roles, &training, reason_id, reason.hidden, current)?;
    trainings::mark(&state.pool, training.id, player, reason_id).await?;
    tracing::info!(date = %training.date, player = player.0, "absence marked");
    Ok(Json(
        marked(&state, &training, player, Some(reason_id)).await?,
    ))
}

#[utoipa::path(
    delete,
    path = "/api/trainings/{date}/absences/{player}",
    operation_id = "markThere",
    params(
        ("date" = String, Path, description = "The date of the training"),
        ("player" = i64, Path, description = "The player"),
    ),
    responses(
        (status = 200, description = "The player is there; safe to send twice", body = Marked),
        (status = 400, description = "`invalid_date`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No training on the date"),
    )
)]
pub async fn there(
    State(state): State<AppState>,
    Path((date, player)): Path<(String, i64)>,
) -> Result<Json<Marked>, AppError> {
    let training = training_on(&state, &date).await?;
    let player = PlayerId(player);
    trainings::unmark(&state.pool, training.id, player).await?;
    tracing::info!(date = %training.date, player = player.0, "absence removed");
    Ok(Json(marked(&state, &training, player, None).await?))
}
