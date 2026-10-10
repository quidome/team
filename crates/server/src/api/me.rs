//! `GET /api/me`: who is logged in.

use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

use crate::auth::CurrentUser;

#[derive(Debug, Serialize, ToSchema)]
pub struct Me {
    /// The name to show.
    pub name: String,
}

#[utoipa::path(
    get,
    path = "/api/me",
    responses(
        (status = 200, description = "The logged-in user", body = Me),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn me(CurrentUser(user): CurrentUser) -> Json<Me> {
    Json(Me { name: user.name })
}
