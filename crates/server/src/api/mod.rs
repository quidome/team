//! The JSON API under `/api`, and its OpenAPI description.

pub mod changes;
pub mod clubs;
pub mod health;
pub mod me;
pub mod readings;
pub mod schedule;
pub mod selection;

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(title = "Team", description = "API of the team coordinator tool."),
    paths(
        health::liveness,
        health::readiness,
        me::me,
        selection::get,
        selection::save,
        clubs::search,
        clubs::teams,
        readings::read,
        readings::latest,
        readings::confirm_gone,
        schedule::get,
        changes::get,
        changes::seen
    )
)]
pub struct ApiDoc;

/// The OpenAPI description as pretty JSON (`team openapi`).
pub fn openapi_json() -> Result<String, serde_json::Error> {
    let mut doc = ApiDoc::openapi();
    // utoipa copies the package license, which this project does not set.
    doc.info.license = None;
    doc.to_pretty_json()
}
