//! The JSON API under `/api`, and its OpenAPI description.

pub mod changes;
pub mod clubs;
pub mod health;
pub mod me;
pub mod players;
pub mod readings;
pub mod reasons;
pub mod schedule;
pub mod selection;
pub mod team;
pub mod trainings;
pub mod weekly_trainings;

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
        changes::seen,
        players::list,
        players::get,
        players::add,
        players::change,
        players::delete,
        players::add_role,
        players::correct_role,
        players::carry,
        team::get,
        team::role_change,
        team::team_leave,
        team::team_undo,
        trainings::list,
        trainings::next,
        trainings::get,
        trainings::open,
        trainings::add,
        trainings::change,
        trainings::cancel,
        trainings::uncancel,
        trainings::delete,
        trainings::mark,
        trainings::there,
        reasons::list,
        reasons::add,
        reasons::change,
        weekly_trainings::list,
        weekly_trainings::proposal,
        weekly_trainings::add,
        weekly_trainings::change,
        weekly_trainings::delete
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
