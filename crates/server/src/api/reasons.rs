//! `/api/reasons`: the reasons for an absence (attendance at training, story A).

use axum::Json;
use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};
use team_core::{ReasonError, clean_reason_name};
use utoipa::ToSchema;

use crate::app::AppState;
use crate::db::StoreError;
use crate::error::AppError;
use crate::reasons::{self, StoredReason};

/// A reason for an absence. (The schema name `Reason` is taken by the choice of club and team.)
#[derive(Debug, Serialize, ToSchema)]
pub struct AbsenceReason {
    pub id: i64,
    pub name: String,
    pub valid: bool,
    /// Not offered for new absences; still shown on old ones.
    pub hidden: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct NewReason {
    pub name: String,
    pub valid: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ReasonChange {
    pub name: String,
    pub valid: bool,
    pub hidden: bool,
}

impl From<ReasonError> for AppError {
    fn from(error: ReasonError) -> Self {
        Self::BadRequest(match error {
            ReasonError::NameRequired => "reason_name_required",
            ReasonError::NameTaken => "reason_name_taken",
        })
    }
}

pub(crate) fn view(reason: StoredReason) -> AbsenceReason {
    AbsenceReason {
        id: reason.id,
        name: reason.name,
        valid: reason.valid,
        hidden: reason.hidden,
    }
}

async fn all(state: &AppState) -> Result<Vec<AbsenceReason>, AppError> {
    Ok(reasons::all(&state.pool)
        .await?
        .into_iter()
        .map(view)
        .collect())
}

/// A name taken by a reason saved at the same time fails on the unique index.
fn taken(error: StoreError) -> AppError {
    if error.is_unique_violation() {
        ReasonError::NameTaken.into()
    } else {
        error.into()
    }
}

#[utoipa::path(
    get,
    path = "/api/reasons",
    operation_id = "reasons",
    responses(
        (status = 200, description = "All reasons in their order, also the hidden ones", body = [AbsenceReason]),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<AbsenceReason>>, AppError> {
    Ok(Json(all(&state).await?))
}

#[utoipa::path(
    post,
    path = "/api/reasons",
    operation_id = "addReason",
    request_body = NewReason,
    responses(
        (status = 200, description = "All reasons; the new one last", body = [AbsenceReason]),
        (status = 400, description = "`reason_name_required`, `reason_name_taken`"),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn add(
    State(state): State<AppState>,
    Json(body): Json<NewReason>,
) -> Result<Json<Vec<AbsenceReason>>, AppError> {
    let stored = reasons::all(&state.pool).await?;
    let others: Vec<&str> = stored.iter().map(|reason| reason.name.as_str()).collect();
    let name = clean_reason_name(&body.name, &others)?;
    reasons::insert(&state.pool, &name, body.valid)
        .await
        .map_err(taken)?;
    Ok(Json(all(&state).await?))
}

#[utoipa::path(
    put,
    path = "/api/reasons/{id}",
    operation_id = "changeReason",
    params(("id" = i64, Path, description = "The reason")),
    request_body = ReasonChange,
    responses(
        (status = 200, description = "All reasons", body = [AbsenceReason]),
        (status = 400, description = "`reason_name_required`, `reason_name_taken`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such reason"),
    )
)]
pub async fn change(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<ReasonChange>,
) -> Result<Json<Vec<AbsenceReason>>, AppError> {
    let stored = reasons::all(&state.pool).await?;
    if !stored.iter().any(|reason| reason.id == id) {
        return Err(AppError::NotFound);
    }
    let others: Vec<&str> = stored
        .iter()
        .filter(|reason| reason.id != id)
        .map(|reason| reason.name.as_str())
        .collect();
    let name = clean_reason_name(&body.name, &others)?;
    if !reasons::update(&state.pool, id, &name, body.valid, body.hidden)
        .await
        .map_err(taken)?
    {
        return Err(AppError::NotFound);
    }
    Ok(Json(all(&state).await?))
}
