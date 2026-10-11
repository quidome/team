//! `GET /api/clubs?search=` and `GET /api/clubs/{number}/teams`: find the club, list its teams.

use axum::Json;
use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};
use team_core::{Club, ClubNumber, Team, own_teams, search_clubs};
use utoipa::{IntoParams, ToSchema};

use crate::app::AppState;
use crate::clock::current_season;
use crate::error::AppError;

#[derive(Debug, Serialize, ToSchema)]
pub struct ClubView {
    /// The source's club number.
    pub number: i64,
    pub name: String,
    /// `null` when the source has no town for the club.
    pub town: Option<String>,
}

impl From<Club> for ClubView {
    fn from(club: Club) -> Self {
        Self {
            number: club.number.0,
            name: club.name,
            town: club.town,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TeamView {
    /// The source's team number.
    pub number: i64,
    /// The short name, for example "M16 1".
    pub name: String,
}

impl From<Team> for TeamView {
    fn from(team: Team) -> Self {
        Self {
            number: team.number.0,
            name: team.name,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ClubTeams {
    /// The current season, for example "2026-2027".
    pub season: String,
    /// The club's own teams in the current season; empty when the season has no games yet.
    pub teams: Vec<TeamView>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ClubSearch {
    /// Part of the club name. Upper or lower case is not important.
    #[serde(default)]
    pub search: String,
}

#[utoipa::path(
    get,
    path = "/api/clubs",
    params(ClubSearch),
    responses(
        (status = 200, description = "Clubs whose name contains the search, sorted by name", body = [ClubView]),
        (status = 401, description = "Not logged in"),
        (status = 502, description = "The game list cannot be reached or is not understood"),
    )
)]
pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<ClubSearch>,
) -> Result<Json<Vec<ClubView>>, AppError> {
    if query.search.trim().is_empty() {
        return Ok(Json(Vec::new()));
    }
    let clubs = state.source.clubs().await?;
    Ok(Json(
        search_clubs(&clubs, &query.search)
            .into_iter()
            .map(ClubView::from)
            .collect(),
    ))
}

#[utoipa::path(
    get,
    path = "/api/clubs/{number}/teams",
    params(("number" = i64, Path, description = "The source's club number")),
    responses(
        (status = 200, description = "The club's own teams in the current season", body = ClubTeams),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "The club is not in the club list"),
        (status = 502, description = "The game list cannot be reached or is not understood"),
    )
)]
pub async fn teams(
    State(state): State<AppState>,
    Path(number): Path<i64>,
) -> Result<Json<ClubTeams>, AppError> {
    let season = current_season(state.clock.as_ref());
    let club = find_club(&state, ClubNumber(number))
        .await?
        .ok_or(AppError::NotFound)?;
    let games = state.source.club_games(club.number, season).await?;
    Ok(Json(ClubTeams {
        season: season.to_string(),
        teams: own_teams(&club, &games)
            .into_iter()
            .map(TeamView::from)
            .collect(),
    }))
}

/// The club from the club list. The source gives an empty game list for an unknown club number,
/// so the club list is the only check that a club exists.
pub async fn find_club(state: &AppState, number: ClubNumber) -> Result<Option<Club>, AppError> {
    let clubs = state.source.clubs().await?;
    Ok(clubs.into_iter().find(|club| club.number == number))
}
