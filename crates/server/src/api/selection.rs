//! `GET /api/selection` and `PUT /api/selection`: the chosen club and team.

use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use team_core::{
    ChoiceState, ClubNumber, FollowedTeam, Season, TeamChoiceError, TeamNeededReason, TeamNumber,
    choice_state, choose_team, own_teams,
};
use utoipa::ToSchema;

use crate::api::clubs::{ClubView, TeamView, find_club};
use crate::app::AppState;
use crate::clock::current_season;
use crate::error::AppError;
use crate::followed_teams;

/// Why a team must be chosen.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    /// The club is saved for the current season, which had no games yet.
    NoGamesYet,
    /// A new season started and nothing is saved for it yet; the club stays.
    NewSeason,
}

/// The choice of an earlier season.
#[derive(Debug, Serialize, ToSchema)]
pub struct LastChoice {
    pub season: String,
    pub club: ClubView,
    pub team: Option<TeamView>,
}

/// The state of the choice. `season` is always the current season.
#[derive(Debug, Serialize, ToSchema)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Selection {
    NotChosen {
        season: String,
    },
    TeamNeeded {
        season: String,
        club: ClubView,
        reason: Reason,
        /// The choice of the newest earlier season, if any (for example last season's team).
        last: Option<LastChoice>,
    },
    Chosen {
        season: String,
        club: ClubView,
        team: TeamView,
    },
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SaveSelection {
    /// The source's club number.
    pub club: i64,
    /// The source's team number; leave it out only when the season has no games yet.
    pub team: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/api/selection",
    responses(
        (status = 200, description = "The state of the choice", body = Selection),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn get(State(state): State<AppState>) -> Result<Json<Selection>, AppError> {
    let season = current_season(state.clock.as_ref());
    Ok(Json(selection(&state, season).await?))
}

#[utoipa::path(
    put,
    path = "/api/selection",
    request_body = SaveSelection,
    responses(
        (status = 200, description = "Saved; the state of the choice", body = Selection),
        (status = 400, description = "`unknown_club`, `not_an_own_team` or `team_required`"),
        (status = 401, description = "Not logged in"),
        (status = 502, description = "The game list cannot be reached or is not understood; nothing is saved"),
    )
)]
pub async fn save(
    State(state): State<AppState>,
    Json(body): Json<SaveSelection>,
) -> Result<Json<Selection>, AppError> {
    let season = current_season(state.clock.as_ref());
    // All names come from the source, never from the browser. The source is read before the
    // database is touched, so a slow source holds no database connection.
    let club = find_club(&state, ClubNumber(body.club))
        .await?
        .ok_or(AppError::BadRequest("unknown_club"))?;
    let games = state.source.club_games(club.number, season).await?;
    let team =
        choose_team(own_teams(&club, &games), body.team.map(TeamNumber)).map_err(|error| {
            match error {
                TeamChoiceError::NotAnOwnTeam => AppError::BadRequest("not_an_own_team"),
                TeamChoiceError::TeamRequired => AppError::BadRequest("team_required"),
            }
        })?;
    let choice = FollowedTeam { season, club, team };
    followed_teams::save(&state.pool, &choice, state.clock.now()).await?;
    Ok(Json(selection(&state, season).await?))
}

async fn selection(state: &AppState, season: Season) -> Result<Selection, AppError> {
    let this_season = followed_teams::for_season(&state.pool, season).await?;
    let earlier = followed_teams::newest_before(&state.pool, season).await?;
    let current = season.to_string();
    Ok(match choice_state(this_season, earlier) {
        ChoiceState::NotChosen => Selection::NotChosen { season: current },
        ChoiceState::TeamNeeded { club, reason, last } => Selection::TeamNeeded {
            season: current,
            club: club.into(),
            reason: match reason {
                TeamNeededReason::NoGamesYet => Reason::NoGamesYet,
                TeamNeededReason::NewSeason => Reason::NewSeason,
            },
            last: last.map(|last| LastChoice {
                season: last.season.to_string(),
                club: last.club.into(),
                team: last.team.map(TeamView::from),
            }),
        },
        ChoiceState::Chosen { club, team, .. } => Selection::Chosen {
            season: current,
            club: club.into(),
            team: team.into(),
        },
    })
}
