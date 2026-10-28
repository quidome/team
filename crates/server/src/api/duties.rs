//! `GET /api/duties`: the duty totals of each player in a season, for the page "Duties" (story B
//! of duties). The numbers come from the same rule and the same games as the totals in the choice
//! of a game (`team_core::duty_totals`, decision 0022). Names never go to the log.

use axum::Json;
use axum::extract::{Query, State};
use serde::Serialize;
use team_core::{FollowedTeam, Role, duty_totals};
use utoipa::ToSchema;

use super::format::query_season;
use super::games::KindTotalsView;
use super::season::{SeasonPlaces, duties_of};
use super::trainings::SeasonQuery;
use crate::app::AppState;
use crate::clock::{current_season, today_in_netherlands};
use crate::db::StoreError;
use crate::error::AppError;
use crate::{followed_teams, games};

/// The totals of one player in the season.
#[derive(Debug, Serialize, ToSchema)]
pub struct DutyRowView {
    pub player: i64,
    /// The name shown ("Anna", "Anna B.").
    pub name: String,
    pub hidden: bool,
    pub totals: KindTotalsView,
}

/// The page "Duties".
#[derive(Debug, Serialize, ToSchema)]
pub struct DutiesOverview {
    /// For example "2026-2027".
    pub season: String,
    pub current_season: String,
    /// The seasons to choose from, newest first: those with a chosen team, the current season and
    /// the chosen season.
    pub seasons: Vec<String>,
    /// One row for each player with the role "plays games" in the season or with a duty in it,
    /// also players who left and hidden players; sorted by first name.
    pub rows: Vec<DutyRowView>,
}

#[utoipa::path(
    get,
    path = "/api/duties",
    operation_id = "duties",
    params(SeasonQuery),
    responses(
        (status = 200, description = "The duty totals of each player in the season and the seasons to choose from", body = DutiesOverview),
        (status = 400, description = "`invalid_season`"),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn get(
    State(state): State<AppState>,
    Query(query): Query<SeasonQuery>,
) -> Result<Json<DutiesOverview>, AppError> {
    let season = query_season(query.season, state.clock.as_ref())?;
    let current = current_season(state.clock.as_ref());
    let today = today_in_netherlands(state.clock.as_ref());

    let mut rows = Vec::new();
    if let Some(FollowedTeam {
        club,
        team: Some(team),
        ..
    }) = followed_teams::for_season(&state.pool, season).await?
    {
        let stored = {
            let mut db = state.pool.acquire().await.map_err(StoreError::from)?;
            games::stored(&mut db, club.number, season).await?
        };
        let duties = duties_of(&state, club.number, season).await?;
        let places = SeasonPlaces::new(&stored, &duties, club.number, team.number, today);
        let totals = duty_totals(&places.duties(&duties));
        // `players` is sorted by first name (`players::all`).
        rows = duties
            .players
            .iter()
            .filter(|player| {
                totals.contains_key(&player.id)
                    || duties
                        .roles
                        .iter()
                        .any(|role| role.player == player.id && role.role == Role::PlaysGames)
            })
            .map(|player| DutyRowView {
                player: player.id.0,
                name: duties.names.get(&player.id).cloned().unwrap_or_default(),
                hidden: player.hidden,
                totals: totals.get(&player.id).copied().unwrap_or_default().into(),
            })
            .collect();
    }

    let mut seasons = followed_teams::seasons(&state.pool).await?;
    seasons.extend([current, season]);
    seasons.sort();
    seasons.dedup();
    seasons.reverse();

    Ok(Json(DutiesOverview {
        season: season.to_string(),
        current_season: current.to_string(),
        seasons: seasons.iter().map(ToString::to_string).collect(),
        rows,
    }))
}
