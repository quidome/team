//! `/api/attendance`: the season totals of each player, the group "Left the team", the training
//! days not opened and the seasons to choose from (attendance at training, story C). Names never
//! go to the log.

use axum::Json;
use axum::extract::{Query, State};
use serde::Serialize;
use team_core::{PlayerTotals, ReasonId, season_overview};
use utoipa::ToSchema;

use super::format::{day, query_season};
use super::season::{SeasonTrainings, trainings_of};
use super::trainings::SeasonQuery;
use crate::app::AppState;
use crate::clock::{current_season, today_in_netherlands};
use crate::error::AppError;
use crate::reasons::StoredReason;
use crate::{players, reasons, trainings};

/// A training that a player missed and that counts.
#[derive(Debug, Serialize, ToSchema)]
pub struct MissedView {
    /// For example "2026-10-06".
    pub date: String,
    pub reason: String,
    pub valid: bool,
}

/// The totals of one player in the season.
#[derive(Debug, Serialize, ToSchema)]
pub struct PlayerTotalsView {
    pub id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub there: usize,
    pub absent_valid: usize,
    pub absent_not_valid: usize,
    /// "of N trainings": the trainings that count for this player.
    pub trainings: usize,
    /// Newest first.
    pub missed: Vec<MissedView>,
}

/// The overview of a season on the page "Attendance".
#[derive(Debug, Serialize, ToSchema)]
pub struct AttendanceOverview {
    /// For example "2026-2027".
    pub season: String,
    pub current_season: String,
    /// The seasons to choose from, newest first: those with a role, a weekly training or a
    /// training, the current season and the chosen season.
    pub seasons: Vec<String>,
    /// The trainings of the season up to today that are not cancelled; 0: no trainings yet.
    pub trainings: usize,
    /// Training days before today without a training.
    pub not_opened: usize,
    /// The oldest of them.
    pub first_not_opened: Option<String>,
    /// A role today or later in the season (on its last day for an earlier season).
    pub team: Vec<PlayerTotalsView>,
    /// "Left the team": roles earlier in the season, none today or later.
    pub left: Vec<PlayerTotalsView>,
}

fn totals_view(totals: &PlayerTotals, reasons: &[StoredReason]) -> PlayerTotalsView {
    PlayerTotalsView {
        id: totals.player.id.0,
        first_name: totals.player.first_name.clone(),
        last_name: totals.player.last_name.clone(),
        there: totals.there,
        absent_valid: totals.absent_valid,
        absent_not_valid: totals.absent_not_valid,
        trainings: totals.trainings,
        missed: totals
            .missed
            .iter()
            .map(|missed| {
                let reason = reasons.iter().find(|reason| reason.id == missed.reason.0);
                MissedView {
                    date: day(missed.date),
                    reason: reason.map(|reason| reason.name.clone()).unwrap_or_default(),
                    valid: reason.is_some_and(|reason| reason.valid),
                }
            })
            .collect(),
    }
}

#[utoipa::path(
    get,
    path = "/api/attendance",
    operation_id = "attendance",
    params(SeasonQuery),
    responses(
        (status = 200, description = "The totals of each player in the season, the training days not opened and the seasons to choose from", body = AttendanceOverview),
        (status = 400, description = "`invalid_season`"),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn get(
    State(state): State<AppState>,
    Query(query): Query<SeasonQuery>,
) -> Result<Json<AttendanceOverview>, AppError> {
    let season = query_season(query.season, state.clock.as_ref())?;
    let current = current_season(state.clock.as_ref());
    let today = today_in_netherlands(state.clock.as_ref());
    let players = players::all(&state.pool, &state.config.player_name_key).await?;
    let SeasonTrainings {
        weekly,
        trainings: stored,
        absences,
        roles,
    } = trainings_of(&state, season).await?;
    let reasons = reasons::all(&state.pool).await?;
    let valid: Vec<(ReasonId, bool)> = reasons
        .iter()
        .map(|reason| (ReasonId(reason.id), reason.valid))
        .collect();
    let overview = season_overview(
        season, &players, &roles, &weekly, &stored, &absences, &valid, today,
    );

    let mut seasons = trainings::seasons(&state.pool).await?;
    seasons.extend([current, season]);
    seasons.sort();
    seasons.dedup();
    seasons.reverse();

    Ok(Json(AttendanceOverview {
        season: season.to_string(),
        current_season: current.to_string(),
        seasons: seasons.iter().map(ToString::to_string).collect(),
        trainings: overview.trainings,
        not_opened: overview.not_opened,
        first_not_opened: overview.first_not_opened.map(day),
        team: overview
            .team
            .iter()
            .map(|totals| totals_view(totals, &reasons))
            .collect(),
        left: overview
            .left
            .iter()
            .map(|totals| totals_view(totals, &reasons))
            .collect(),
    }))
}
