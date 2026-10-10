//! The login check on `/api` (decision 0007).
//!
//! Step 1 of the base: there is no login yet, so every request that needs a session is refused.
//! Step 2 replaces this with the session check.

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

use crate::error::AppError;

/// Middleware on the `/api` router: a request without a valid session gets 401.
pub async fn require_session(_request: Request, _next: Next) -> Result<Response, AppError> {
    Err(AppError::Unauthenticated)
}
