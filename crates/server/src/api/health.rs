//! Health checks for Kubernetes. They need no login.

use axum::extract::State;
use axum::http::StatusCode;

use crate::app::AppState;
use crate::db;

/// 200 while the process runs.
pub async fn liveness() -> StatusCode {
    StatusCode::OK
}

/// 200 when the database answers and all migrations are applied; else 503.
pub async fn readiness(State(state): State<AppState>) -> StatusCode {
    match db::check_ready(&state.pool).await {
        Ok(()) => StatusCode::OK,
        Err(reason) => {
            tracing::warn!(%reason, "not ready");
            StatusCode::SERVICE_UNAVAILABLE
        }
    }
}
