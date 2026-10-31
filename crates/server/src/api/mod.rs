//! The JSON API under `/api`, and its OpenAPI description.

pub mod attendance;
pub mod changes;
pub mod clubs;
pub mod duties;
pub(crate) mod format;
pub mod games;
pub mod health;
pub mod me;
pub mod messages;
pub mod players;
pub mod readings;
pub mod reasons;
pub mod season;
pub mod selection;
pub mod start;
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
        season::get,
        games::get,
        games::set_place,
        games::set_entered,
        games::unset_entered,
        games::set_drivers,
        games::set_assignment,
        games::clear_assignment,
        games::set_outcome,
        start::get,
        messages::week,
        messages::committee,
        messages::driving,
        messages::table_referee,
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
        attendance::get,
        duties::get,
        trainings::get,
        trainings::open,
        trainings::add,
        trainings::change,
        trainings::cancel,
        trainings::uncancel,
        trainings::check,
        trainings::uncheck,
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
