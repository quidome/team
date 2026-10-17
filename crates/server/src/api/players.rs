//! `/api/players`: the players and their roles in the team of a season (attendance at training,
//! story A). Names never go to the log; only ids do.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgExecutor;
use team_core::{
    PlayerError, PlayerId, PlayerRole, Role, RoleId, Season, check_change, check_role, clean_name,
    role_on, team,
};
use utoipa::ToSchema;

use super::team::{self, TeamPageView};
use crate::app::AppState;
use crate::clock::{current_season, today_in_netherlands};
use crate::db::StoreError;
use crate::error::AppError;
use crate::players;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RoleKind {
    PlaysGames,
    OnlyTrains,
}

impl From<Role> for RoleKind {
    fn from(role: Role) -> Self {
        match role {
            Role::PlaysGames => Self::PlaysGames,
            Role::OnlyTrains => Self::OnlyTrains,
        }
    }
}

impl From<RoleKind> for Role {
    fn from(kind: RoleKind) -> Self {
        match kind {
            RoleKind::PlaysGames => Self::PlaysGames,
            RoleKind::OnlyTrains => Self::OnlyTrains,
        }
    }
}

/// A role of a player in the team of a season.
#[derive(Debug, Serialize, ToSchema)]
pub struct RoleView {
    pub id: i64,
    pub role: RoleKind,
    /// The first day, for example "2026-09-01".
    pub from: String,
    /// The last day; `null` while the role continues.
    pub until: Option<String>,
}

/// A player in the list of all players.
#[derive(Debug, Serialize, ToSchema)]
pub struct PlayerListItem {
    pub id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub hidden: bool,
    /// The role today; `null` when the player is not in the team today.
    pub role_today: Option<RoleKind>,
}

/// All players of the club, sorted by first name.
#[derive(Debug, Serialize, ToSchema)]
pub struct PlayersList {
    pub season: String,
    pub players: Vec<PlayerListItem>,
}

/// A role in a player's history.
#[derive(Debug, Serialize, ToSchema)]
pub struct HistoryRole {
    pub id: i64,
    /// The season of the role, for example "2026-2027".
    pub season: String,
    pub role: RoleKind,
    /// The first day, for example "2026-09-01".
    pub from: String,
    /// The last day; `null` while the role continues.
    pub until: Option<String>,
    /// Only a role of the current season can be corrected.
    pub correctable: bool,
}

/// One player with the history of all seasons.
#[derive(Debug, Serialize, ToSchema)]
pub struct PlayerDetail {
    pub id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub hidden: bool,
    /// The current season, for example "2026-2027".
    pub season: String,
    /// The first day of the current season: the earliest date of a corrected role.
    pub season_first_day: String,
    /// The last day of the current season: the latest date of a corrected role.
    pub season_last_day: String,
    /// All roles of all seasons, newest first.
    pub history: Vec<HistoryRole>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct NewRole {
    pub role: RoleKind,
    #[schema(value_type = String, format = Date)]
    pub from: NaiveDate,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct NewPlayer {
    pub first_name: String,
    pub last_name: Option<String>,
    /// The first role, if any.
    pub role: Option<NewRole>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PlayerChange {
    pub first_name: String,
    pub last_name: Option<String>,
    /// A hidden player is not offered in "Add player" of the team page and not in "Add all".
    pub hidden: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RoleChange {
    pub role: RoleKind,
    #[schema(value_type = String, format = Date)]
    pub from: NaiveDate,
    /// `null` while the role continues.
    #[schema(value_type = Option<String>, format = Date)]
    pub until: Option<NaiveDate>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CarryRequest {
    /// Players of last season to bring into this season ("Add all" sends all of them).
    pub players: Vec<i64>,
    /// "In the team from": the first day of their roles in this season.
    #[schema(value_type = String, format = Date)]
    pub from: NaiveDate,
}

/// A saved player, with the warning that another player has the same name.
#[derive(Debug, Serialize, ToSchema)]
pub struct SavedPlayer {
    pub player: PlayerDetail,
    pub same_name: bool,
}

impl From<PlayerError> for AppError {
    fn from(error: PlayerError) -> Self {
        Self::BadRequest(match error {
            PlayerError::RoleStartsLater(start) => {
                return Self::Refused {
                    status: StatusCode::BAD_REQUEST,
                    code: "role_starts_later",
                    details: json!({ "date": day(start) }),
                };
            }
            PlayerError::ChangedMeanwhile => {
                return Self::Refused {
                    status: StatusCode::CONFLICT,
                    code: "changed_meanwhile",
                    details: json!({}),
                };
            }
            PlayerError::SameRole => "same_role",
            PlayerError::FirstNameRequired => "first_name_required",
            PlayerError::UntilBeforeFrom => "until_before_from",
            PlayerError::OutsideSeason => "outside_season",
            PlayerError::RolesOverlap => "roles_overlap",
            PlayerError::ChangeBeforeRoleStart => "change_before_role_start",
            PlayerError::NotInTeam => "not_in_team",
        })
    }
}

pub(crate) fn day(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

pub(crate) fn role_view(role: &PlayerRole) -> RoleView {
    RoleView {
        id: role.id.0,
        role: role.role.into(),
        from: day(role.from),
        until: role.until.map(day),
    }
}

/// The roles of `player` in the current season and the season before.
async fn roles_around(
    db: impl PgExecutor<'_>,
    player: PlayerId,
    season: Season,
) -> Result<Vec<PlayerRole>, StoreError> {
    Ok(players::roles_of(db, player)
        .await?
        .into_iter()
        .filter(|role| role.season == season || role.season == season.previous())
        .collect())
}

/// The detail of one player, with the history of all seasons.
async fn detail(state: &AppState, id: PlayerId) -> Result<PlayerDetail, AppError> {
    let player = players::by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    let season = current_season(state.clock.as_ref());
    let mut roles = players::roles_of(&state.pool, id).await?;
    roles.reverse();
    Ok(PlayerDetail {
        id: player.id.0,
        first_name: player.first_name,
        last_name: player.last_name,
        hidden: player.hidden,
        season: season.to_string(),
        season_first_day: day(season.first_day()),
        season_last_day: day(season.last_day()),
        history: roles
            .iter()
            .map(|role| HistoryRole {
                id: role.id.0,
                season: role.season.to_string(),
                role: role.role.into(),
                from: day(role.from),
                until: role.until.map(day),
                correctable: role.season == season,
            })
            .collect(),
    })
}

#[utoipa::path(
    get,
    path = "/api/players",
    operation_id = "players",
    responses(
        (status = 200, description = "All players, hidden ones too, sorted by first name", body = PlayersList),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn list(State(state): State<AppState>) -> Result<Json<PlayersList>, AppError> {
    let season = current_season(state.clock.as_ref());
    let today = today_in_netherlands(state.clock.as_ref());
    let all = players::all(&state.pool).await?;
    let roles = players::roles_in(&state.pool, &[season]).await?;
    Ok(Json(PlayersList {
        season: season.to_string(),
        players: all
            .into_iter()
            .map(|player| PlayerListItem {
                role_today: role_on(&roles, player.id, today).map(|role| role.role.into()),
                id: player.id.0,
                first_name: player.first_name,
                last_name: player.last_name,
                hidden: player.hidden,
            })
            .collect(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/players/{id}",
    operation_id = "player",
    params(("id" = i64, Path, description = "The player")),
    responses(
        (status = 200, description = "The player with its history", body = PlayerDetail),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such player"),
    )
)]
pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<PlayerDetail>, AppError> {
    Ok(Json(detail(&state, PlayerId(id)).await?))
}

#[utoipa::path(
    post,
    path = "/api/players",
    operation_id = "addPlayer",
    request_body = NewPlayer,
    responses(
        (status = 200, description = "The new player", body = SavedPlayer),
        (status = 400, description = "`first_name_required`, `outside_season` (the role does not start in the current season)"),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn add(
    State(state): State<AppState>,
    Json(body): Json<NewPlayer>,
) -> Result<Json<SavedPlayer>, AppError> {
    let (first_name, last_name) = clean_name(&body.first_name, body.last_name.as_deref())?;
    let season = current_season(state.clock.as_ref());
    if let Some(role) = &body.role {
        check_role(season, role.from, None, &[])?;
    }
    let mut tx = state.pool.begin().await.map_err(StoreError::from)?;
    let id = players::insert(
        &mut *tx,
        &first_name,
        last_name.as_deref(),
        state.clock.now(),
    )
    .await?;
    if let Some(role) = &body.role {
        players::insert_role(&mut *tx, id, season, role.role.into(), role.from, None).await?;
    }
    let same_name =
        players::same_name_exists(&mut *tx, &first_name, last_name.as_deref(), Some(id)).await?;
    tx.commit().await.map_err(StoreError::from)?;
    tracing::info!(player = id.0, "player added");
    Ok(Json(SavedPlayer {
        player: detail(&state, id).await?,
        same_name,
    }))
}

#[utoipa::path(
    put,
    path = "/api/players/{id}",
    operation_id = "changePlayer",
    params(("id" = i64, Path, description = "The player")),
    request_body = PlayerChange,
    responses(
        (status = 200, description = "The player", body = SavedPlayer),
        (status = 400, description = "`first_name_required`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such player"),
    )
)]
pub async fn change(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<PlayerChange>,
) -> Result<Json<SavedPlayer>, AppError> {
    let id = PlayerId(id);
    let (first_name, last_name) = clean_name(&body.first_name, body.last_name.as_deref())?;
    // The name and "Hidden" together, or neither.
    let mut tx = state.pool.begin().await.map_err(StoreError::from)?;
    if !players::update_name(&mut *tx, id, &first_name, last_name.as_deref()).await? {
        return Err(AppError::NotFound);
    }
    players::set_hidden(&mut *tx, id, body.hidden).await?;
    let same_name =
        players::same_name_exists(&mut *tx, &first_name, last_name.as_deref(), Some(id)).await?;
    tx.commit().await.map_err(StoreError::from)?;
    tracing::info!(player = id.0, "player changed");
    Ok(Json(SavedPlayer {
        player: detail(&state, id).await?,
        same_name,
    }))
}

#[utoipa::path(
    delete,
    path = "/api/players/{id}",
    operation_id = "deletePlayer",
    params(("id" = i64, Path, description = "The player")),
    responses(
        (status = 204, description = "Deleted, with its roles"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such player"),
    )
)]
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    if !players::delete(&state.pool, PlayerId(id)).await? {
        return Err(AppError::NotFound);
    }
    tracing::info!(player = id, "player deleted");
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/players/{id}/roles",
    operation_id = "addRole",
    params(("id" = i64, Path, description = "The player")),
    request_body = RoleChange,
    responses(
        (status = 200, description = "The team", body = TeamPageView),
        (status = 400, description = "`until_before_from`, `outside_season`, `roles_overlap`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such player"),
    )
)]
pub async fn add_role(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<RoleChange>,
) -> Result<Json<TeamPageView>, AppError> {
    save_role(&state, PlayerId(id), None, body).await?;
    Ok(Json(team::load(&state).await?))
}

#[utoipa::path(
    put,
    path = "/api/players/{id}/roles/{role}",
    operation_id = "correctRole",
    params(
        ("id" = i64, Path, description = "The player"),
        ("role" = i64, Path, description = "The role"),
    ),
    request_body = RoleChange,
    responses(
        (status = 200, description = "The player with its history", body = PlayerDetail),
        (status = 400, description = "`until_before_from`, `outside_season` (also for a role of another season), `roles_overlap`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such player or role"),
    )
)]
pub async fn correct_role(
    State(state): State<AppState>,
    Path((id, role)): Path<(i64, i64)>,
    Json(body): Json<RoleChange>,
) -> Result<Json<PlayerDetail>, AppError> {
    save_role(&state, PlayerId(id), Some(RoleId(role)), body).await?;
    Ok(Json(detail(&state, PlayerId(id)).await?))
}

/// Adds a role (`role` is `None`) or changes one, under the player's lock, so two saves at the same
/// time cannot make an overlap. Only roles of the current season are saved or changed.
async fn save_role(
    state: &AppState,
    player: PlayerId,
    role: Option<RoleId>,
    body: RoleChange,
) -> Result<(), AppError> {
    let season = current_season(state.clock.as_ref());
    let mut tx = state.pool.begin().await.map_err(StoreError::from)?;
    if !players::lock(&mut tx, player).await? {
        return Err(AppError::NotFound);
    }
    let roles = players::roles_of(&mut *tx, player).await?;
    let (changed, others): (Vec<PlayerRole>, Vec<PlayerRole>) = roles
        .into_iter()
        .partition(|stored| Some(stored.id) == role);
    match (role, changed.first()) {
        (None, _) => check_role(season, body.from, body.until, &others)?,
        (Some(_), Some(changed)) => check_change(season, changed, body.from, body.until, &others)?,
        (Some(_), None) => return Err(AppError::NotFound),
    }
    match role {
        None => {
            players::insert_role(
                &mut *tx,
                player,
                season,
                body.role.into(),
                body.from,
                body.until,
            )
            .await?;
        }
        Some(id) => {
            players::update_role(
                &mut *tx,
                player,
                id,
                season,
                body.role.into(),
                body.from,
                body.until,
            )
            .await?;
        }
    }
    tx.commit().await.map_err(StoreError::from)?;
    tracing::info!(player = player.0, "role saved");
    Ok(())
}

#[utoipa::path(
    post,
    path = "/api/players/carry",
    operation_id = "carryPlayers",
    request_body = CarryRequest,
    responses(
        (status = 200, description = "The team; the listed players of last season have the same role in this season, from `from`", body = TeamPageView),
        (status = 400, description = "`outside_season`; no player is added"),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn carry(
    State(state): State<AppState>,
    Json(body): Json<CarryRequest>,
) -> Result<Json<TeamPageView>, AppError> {
    let season = current_season(state.clock.as_ref());
    let today = today_in_netherlands(state.clock.as_ref());
    let from = body.from;
    // Locks in order of id, so two "Add all" at the same time cannot block each other (a deadlock).
    let mut ids = body.players.clone();
    ids.sort_unstable();
    ids.dedup();
    let mut tx = state.pool.begin().await.map_err(StoreError::from)?;
    for id in ids.into_iter().map(PlayerId) {
        // Under the player's lock: two taps at the same time give one role. A player that already
        // has a role in this season, or did not stay until the end of last season, is skipped.
        if !players::lock(&mut tx, id).await? {
            continue;
        }
        let Some(player) = players::by_id(&mut *tx, id).await? else {
            continue;
        };
        let roles = roles_around(&mut *tx, id, season).await?;
        let page = team(std::slice::from_ref(&player), &roles, season, today);
        if let Some(member) = page.last_season.first() {
            check_role(season, from, None, &roles)?;
            players::insert_role(&mut *tx, id, season, member.role, from, None).await?;
            tracing::info!(player = id.0, "player carried into the season");
        }
    }
    tx.commit().await.map_err(StoreError::from)?;
    Ok(Json(team::load(&state).await?))
}
