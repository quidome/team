//! Errors that reach the client. Details go to the log only.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use crate::db::StoreError;
use crate::source::SourceError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("login required")]
    Unauthenticated,
    /// A request the client can fix; the reason is a short fixed code, for example `unknown_club`.
    #[error("bad request: {0}")]
    BadRequest(&'static str),
    #[error("not found")]
    NotFound,
    /// The game list cannot be reached, or answered with an error.
    #[error("game list not available: {0}")]
    SourceUnavailable(#[source] SourceError),
    /// The game list answered with data the tool does not understand.
    #[error("game list not understood: {0}")]
    SourceFormat(#[source] SourceError),
    /// Any failure the client cannot fix; the cause goes to the log only.
    #[error("internal error: {0}")]
    Internal(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl From<SourceError> for AppError {
    fn from(error: SourceError) -> Self {
        match error {
            SourceError::Format { .. } => Self::SourceFormat(error),
            SourceError::Unreachable(_) | SourceError::Status(_) => Self::SourceUnavailable(error),
            SourceError::Client(_) => Self::Internal(error.into()),
        }
    }
}

impl From<StoreError> for AppError {
    fn from(error: StoreError) -> Self {
        Self::Internal(error.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            Self::Unauthenticated => (StatusCode::UNAUTHORIZED, "unauthenticated"),
            Self::BadRequest(reason) => (StatusCode::BAD_REQUEST, *reason),
            Self::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            // The reader logs these with the club and season (`source/mod.rs`).
            Self::SourceUnavailable(_) => (StatusCode::BAD_GATEWAY, "source_unavailable"),
            Self::SourceFormat(_) => (StatusCode::BAD_GATEWAY, "source_format"),
            Self::Internal(error) => {
                tracing::error!(error = %error, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal")
            }
        };
        (status, Json(json!({ "error": code }))).into_response()
    }
}
