//! `/api/games/{season}/{id}`: the page of a game and its duties (story A of duties). A game is a
//! game of the club chosen for its season, with a team, that is in the schedule or kept for its
//! duties; also when it left the game list. Every write: lock the game, read, the core check,
//! write, in one transaction; each answers the page.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use team_core::{
    Club, DutyError, Game, PlaceKind, RequiredDuties, ScheduleReason, ScheduledGame, Season,
    StoredDuties, Team, TeamNumber, check_drivers, check_entered, check_team, club_teams,
    in_schedule, kept_games, pick_team, required_duties,
};
use utoipa::ToSchema;

use super::season::{PlacesView, ScheduleGame, duties_of};
use crate::api::clubs::TeamView;
use crate::app::AppState;
use crate::clock::{current_season, today_in_netherlands};
use crate::db::StoreError;
use crate::duties::{self, GameKey};
use crate::error::AppError;
use crate::{followed_teams, games};

#[derive(Debug, Serialize, ToSchema)]
pub struct GamePage {
    /// The season of the game, for example "2026-2027".
    pub season: String,
    pub current_season: String,
    /// The team chosen for the game's season.
    pub team: TeamView,
    /// The game, with its duties.
    pub game: ScheduleGame,
    /// The pull-down list of the places: the club teams of the season, sorted by name.
    pub club_teams: Vec<TeamView>,
}

/// The team of a place; `null`: "—".
#[derive(Debug, Deserialize, ToSchema)]
pub struct PlaceInput {
    pub team: Option<i64>,
}

/// The number of drivers; `null`: the proposal.
#[derive(Debug, Deserialize, ToSchema)]
pub struct DriversInput {
    pub drivers: Option<u8>,
}

impl From<DutyError> for AppError {
    fn from(error: DutyError) -> Self {
        match error {
            DutyError::NotAHomeGame => Self::BadRequest("not_a_home_game"),
            DutyError::NoDriversNeeded => Self::BadRequest("no_drivers_needed"),
            DutyError::OutOfRange => Self::BadRequest("out_of_range"),
            DutyError::UnknownTeam => Self::BadRequest("unknown_team"),
            DutyError::PlayerOnPlace => Self::BadRequest("player_on_place"),
            DutyError::BelowFilled { filled } => Self::Refused {
                status: StatusCode::BAD_REQUEST,
                code: "places_filled",
                details: json!({ "filled": filled }),
            },
        }
    }
}

/// A game of a season: the choice of that season, the game, and what the page needs.
struct Found {
    season: Season,
    club: Club,
    team: Team,
    game: Game,
    in_list: bool,
    reason: ScheduleReason,
    /// The pull-down list of the places.
    club_teams: Vec<team_core::TeamRef>,
}

impl Found {
    fn key(&self) -> GameKey {
        GameKey {
            club: self.club.number,
            season: self.season,
            source_id: self.game.source_id,
        }
    }

    fn required(&self) -> RequiredDuties {
        required_duties(&self.game, self.club.number, self.team.number)
    }
}

/// The game `id` of `season`, from the choice of that season; 404 without a team, or for a game
/// that is not in the schedule and not kept for its duties.
async fn find(state: &AppState, season: &str, id: &str) -> Result<Found, AppError> {
    let season = season
        .parse::<Season>()
        .map_err(|_| AppError::BadRequest("invalid_season"))?;
    let id = id
        .parse::<i64>()
        .map_err(|_| AppError::BadRequest("invalid_game"))?;
    let Some(team_core::FollowedTeam {
        club,
        team: Some(team),
        ..
    }) = followed_teams::for_season(&state.pool, season).await?
    else {
        return Err(AppError::NotFound);
    };
    let mut db = state.pool.acquire().await.map_err(StoreError::from)?;
    let stored = games::stored(&mut db, club.number, season).await?;
    let duties = duties::for_season(&mut db, club.number, season).await?;
    let all: Vec<Game> = stored.iter().map(|stored| stored.game.clone()).collect();
    let club_teams = club_teams(&club, &all, &duties);
    let found = stored
        .into_iter()
        .find(|stored| stored.game.source_id == id)
        .ok_or(AppError::NotFound)?;
    let reason = in_schedule(&found.game, club.number, Some(team.number))
        .or_else(|| {
            kept_games(&duties)
                .contains(&id)
                .then_some(ScheduleReason::Kept)
        })
        .ok_or(AppError::NotFound)?;
    Ok(Found {
        season,
        club,
        team,
        game: found.game,
        in_list: found.in_list,
        reason,
        club_teams,
    })
}

/// The page of `found`, with the duties as stored now.
async fn page(state: &AppState, found: Found) -> Result<GamePage, AppError> {
    let today = today_in_netherlands(state.clock.as_ref());
    let duties = duties_of(state, found.club.number, found.season).await?;
    let places = duties.places_of(
        &found.game,
        found.in_list,
        found.club.number,
        found.team.number,
        today,
    );
    let mut game = ScheduleGame::from(ScheduledGame {
        game: found.game,
        reason: found.reason,
        in_list: found.in_list,
    });
    game.places = places.as_ref().map(PlacesView::from);
    Ok(GamePage {
        season: found.season.to_string(),
        current_season: current_season(state.clock.as_ref()).to_string(),
        team: TeamView::from(found.team),
        game,
        club_teams: found.club_teams.iter().map(TeamView::from).collect(),
    })
}

/// One write of the duties of `found`: lock the game, check against what is stored, write;
/// a refused or failed write rolls back.
async fn write<F>(state: &AppState, found: &Found, change: F) -> Result<(), AppError>
where
    F: AsyncFnOnce(&mut sqlx::PgConnection, &StoredDuties) -> Result<(), AppError>,
{
    let now = state.clock.now();
    let mut tx = state.pool.begin().await.map_err(StoreError::from)?;
    let stored = duties::lock(&mut tx, found.key(), now).await?;
    change(&mut tx, &stored).await?;
    tx.commit().await.map_err(StoreError::from)?;
    tracing::info!(season = %found.season, game = found.game.source_id, "duties saved");
    Ok(())
}

#[utoipa::path(
    get,
    path = "/api/games/{season}/{id}",
    operation_id = "game",
    params(
        ("season" = String, Path, description = "The season, for example 2026-2027"),
        ("id" = String, Path, description = "The source's game id"),
    ),
    responses(
        (status = 200, description = "The game with its duties", body = GamePage),
        (status = 400, description = "`invalid_season`, `invalid_game`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such game of the club chosen for that season"),
    )
)]
pub async fn get(
    State(state): State<AppState>,
    Path((season, id)): Path<(String, String)>,
) -> Result<Json<GamePage>, AppError> {
    let found = find(&state, &season, &id).await?;
    Ok(Json(page(&state, found).await?))
}

/// The place of the path: `referee` or `table`, 1 or 2.
fn place(kind: &str, number: &str) -> Result<(PlaceKind, u8), AppError> {
    let kind = match kind {
        "referee" => PlaceKind::Referee,
        "table" => PlaceKind::Table,
        _ => return Err(AppError::BadRequest("invalid_place")),
    };
    match number {
        "1" => Ok((kind, 1)),
        "2" => Ok((kind, 2)),
        _ => Err(AppError::BadRequest("invalid_place")),
    }
}

#[utoipa::path(
    put,
    path = "/api/games/{season}/{id}/places/{kind}/{number}",
    operation_id = "setPlace",
    params(
        ("season" = String, Path, description = "The season, for example 2026-2027"),
        ("id" = String, Path, description = "The source's game id"),
        ("kind" = String, Path, description = "`referee` or `table`"),
        ("number" = String, Path, description = "1 or 2"),
    ),
    request_body = PlaceInput,
    responses(
        (status = 200, description = "Saved; the game with its duties", body = GamePage),
        (status = 400, description = "`invalid_season`, `invalid_game`, `invalid_place`, `unknown_team`, `not_a_home_game`, `player_on_place`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such game of the club chosen for that season"),
    )
)]
pub async fn set_place(
    State(state): State<AppState>,
    Path((season, id, kind, number)): Path<(String, String, String, String)>,
    Json(input): Json<PlaceInput>,
) -> Result<Json<GamePage>, AppError> {
    let (kind, number) = place(&kind, &number)?;
    let found = find(&state, &season, &id).await?;
    let team = input
        .team
        .map(|team| pick_team(&found.club_teams, TeamNumber(team)))
        .transpose()?;
    let required = found.required();
    let now = state.clock.now();
    write(&state, &found, async |tx, stored| {
        // No player is on a place before story B.
        check_team(
            required,
            stored.teams.get(&(kind, number)),
            team.as_ref(),
            false,
        )?;
        duties::set_team(tx, found.key(), kind, number, team.as_ref(), now).await?;
        Ok(())
    })
    .await?;
    Ok(Json(page(&state, found).await?))
}

async fn entered(
    state: AppState,
    season: String,
    id: String,
    on: bool,
) -> Result<Json<GamePage>, AppError> {
    let found = find(&state, &season, &id).await?;
    check_entered(found.required(), on)?;
    let now = state.clock.now();
    write(&state, &found, async |tx, _| {
        duties::set_entered(tx, found.key(), on, now).await?;
        Ok(())
    })
    .await?;
    Ok(Json(page(&state, found).await?))
}

#[utoipa::path(
    put,
    path = "/api/games/{season}/{id}/schedule-entered",
    operation_id = "setScheduleEntered",
    params(
        ("season" = String, Path, description = "The season, for example 2026-2027"),
        ("id" = String, Path, description = "The source's game id"),
    ),
    responses(
        (status = 200, description = "Schedule entered; the game with its duties", body = GamePage),
        (status = 400, description = "`invalid_season`, `invalid_game`, `not_a_home_game`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such game of the club chosen for that season"),
    )
)]
pub async fn set_entered(
    State(state): State<AppState>,
    Path((season, id)): Path<(String, String)>,
) -> Result<Json<GamePage>, AppError> {
    entered(state, season, id, true).await
}

#[utoipa::path(
    delete,
    path = "/api/games/{season}/{id}/schedule-entered",
    operation_id = "unsetScheduleEntered",
    params(
        ("season" = String, Path, description = "The season, for example 2026-2027"),
        ("id" = String, Path, description = "The source's game id"),
    ),
    responses(
        (status = 200, description = "Schedule not entered; the game with its duties", body = GamePage),
        (status = 400, description = "`invalid_season`, `invalid_game`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such game of the club chosen for that season"),
    )
)]
pub async fn unset_entered(
    State(state): State<AppState>,
    Path((season, id)): Path<(String, String)>,
) -> Result<Json<GamePage>, AppError> {
    entered(state, season, id, false).await
}

#[utoipa::path(
    put,
    path = "/api/games/{season}/{id}/drivers",
    operation_id = "setDrivers",
    params(
        ("season" = String, Path, description = "The season, for example 2026-2027"),
        ("id" = String, Path, description = "The source's game id"),
    ),
    request_body = DriversInput,
    responses(
        (status = 200, description = "Saved; the game with its duties", body = GamePage),
        (status = 400, description = "`invalid_season`, `invalid_game`, `no_drivers_needed`, `out_of_range`, `places_filled` (with `filled`)"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such game of the club chosen for that season"),
    )
)]
pub async fn set_drivers(
    State(state): State<AppState>,
    Path((season, id)): Path<(String, String)>,
    Json(input): Json<DriversInput>,
) -> Result<Json<GamePage>, AppError> {
    let found = find(&state, &season, &id).await?;
    let required = found.required();
    let now = state.clock.now();
    write(&state, &found, async |tx, _| {
        // No driver place is filled before story B.
        check_drivers(required, input.drivers, 0)?;
        duties::set_drivers(tx, found.key(), input.drivers, now).await?;
        Ok(())
    })
    .await?;
    Ok(Json(page(&state, found).await?))
}
