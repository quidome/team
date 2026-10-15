//! `/api/players`: the players and their roles in the team of a season (attendance at training,
//! story A). Names never go to the log; only ids do.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use team_core::{
    Player, PlayerError, PlayerId, PlayerRole, Role, RoleId, Season, check_role, clean_name,
    role_on, roles_to_carry,
};
use utoipa::ToSchema;

use crate::app::AppState;
use crate::clock::{current_season, today_in_netherlands};
use crate::db::StoreError;
use crate::error::AppError;
use crate::{players, training_settings};

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

#[derive(Debug, Serialize, ToSchema)]
pub struct PlayerView {
    pub id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    /// Today's role (or the coming one); in "left", the last role of the season; in "last_season", the last role of
    /// last season; `null` in "others".
    pub role: Option<RoleKind>,
    /// The roles of the season shown (last season's for "last_season").
    pub roles: Vec<RoleView>,
}

/// The players for the current season.
#[derive(Debug, Serialize, ToSchema)]
pub struct PlayersView {
    pub season: String,
    /// Today's Dutch date: the proposed end for "Left the team".
    pub today: String,
    /// The proposed start of a new role: the first training date of the season, or today when
    /// trainings have started or nothing is set.
    pub proposed_from: String,
    /// Players with a role today or later in the season, sorted by first name.
    pub players: Vec<PlayerView>,
    /// Players whose roles in the season all ended before today.
    pub left: Vec<PlayerView>,
    /// Players who were in the team until the end of last season and have no role in this season:
    /// "Add all" brings them back.
    pub last_season: Vec<PlayerView>,
    /// Players with no role in this season who are not in `last_season`.
    pub others: Vec<PlayerView>,
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
pub struct NameChange {
    pub first_name: String,
    pub last_name: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RoleChange {
    pub role: RoleKind,
    #[schema(value_type = String, format = Date)]
    pub from: NaiveDate,
    /// `null` while the role continues; "Left the team" sets it, "Undo" clears it.
    #[schema(value_type = Option<String>, format = Date)]
    pub until: Option<NaiveDate>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CarryRequest {
    /// Players of last season to bring into this season ("Add all" sends all of them).
    pub players: Vec<i64>,
}

/// A saved player, with the warning that another player has the same name.
#[derive(Debug, Serialize, ToSchema)]
pub struct SavedPlayer {
    pub player: PlayerView,
    pub same_name: bool,
}

impl From<PlayerError> for AppError {
    fn from(error: PlayerError) -> Self {
        Self::BadRequest(match error {
            PlayerError::FirstNameRequired => "first_name_required",
            PlayerError::UntilBeforeFrom => "until_before_from",
            PlayerError::OutsideSeason => "outside_season",
            PlayerError::RolesOverlap => "roles_overlap",
        })
    }
}

fn day(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

fn role_view(role: &PlayerRole) -> RoleView {
    RoleView {
        id: role.id.0,
        role: role.role.into(),
        from: day(role.from),
        until: role.until.map(day),
    }
}

fn player_view(player: Player, role: Option<Role>, roles: &[&PlayerRole]) -> PlayerView {
    PlayerView {
        id: player.id.0,
        first_name: player.first_name,
        last_name: player.last_name,
        role: role.map(RoleKind::from),
        roles: roles.iter().map(|role| role_view(role)).collect(),
    }
}

/// The view of one player in the current season.
async fn one(state: &AppState, id: PlayerId) -> Result<PlayerView, AppError> {
    let player = players::by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    let season = current_season(state.clock.as_ref());
    let today = today_in_netherlands(state.clock.as_ref());
    let roles = players::roles_of(&state.pool, id).await?;
    let in_season: Vec<&PlayerRole> = roles.iter().filter(|role| role.season == season).collect();
    let role = role_on(&roles, id, today).map(|role| role.role);
    Ok(player_view(player, role, &in_season))
}

/// Groups the players for `season` on `today`.
pub fn group(
    season: Season,
    today: NaiveDate,
    proposed_from: NaiveDate,
    all: Vec<Player>,
    roles: &[PlayerRole],
) -> PlayersView {
    let previous = Season::from_start_year(season.start_year() - 1);
    let to_carry: Vec<PlayerId> = roles_to_carry(roles, season)
        .into_iter()
        .map(|(player, _)| player)
        .collect();
    let mut view = PlayersView {
        season: season.to_string(),
        today: day(today),
        proposed_from: day(proposed_from),
        players: Vec::new(),
        left: Vec::new(),
        last_season: Vec::new(),
        others: Vec::new(),
    };
    for player in all {
        let of_season = |wanted: Season| -> Vec<&PlayerRole> {
            roles
                .iter()
                .filter(|role| role.player == player.id && role.season == wanted)
                .collect()
        };
        let this = of_season(season);
        let last = of_season(previous);
        // A role today, or one that starts later in the season (for example from the first
        // training date, after "Add all" in summer).
        let current = role_on(roles, player.id, today)
            .filter(|role| role.season == season)
            .or_else(|| this.iter().copied().find(|role| role.from > today));
        if let Some(role) = current {
            let role = role.role;
            view.players.push(player_view(player, Some(role), &this));
        } else if let Some(newest) = this.last() {
            let role = newest.role;
            view.left.push(player_view(player, Some(role), &this));
        } else if let Some(newest) = last.last().filter(|_| to_carry.contains(&player.id)) {
            let role = newest.role;
            view.last_season
                .push(player_view(player, Some(role), &last));
        } else {
            view.others.push(player_view(player, None, &[]));
        }
    }
    view
}

/// The proposed start of a new role: the first training date of the season, or today when
/// trainings have started or nothing is set.
async fn proposed_from(
    state: &AppState,
    season: Season,
    today: NaiveDate,
) -> Result<NaiveDate, StoreError> {
    let mut db = state.pool.acquire().await?;
    let first = training_settings::for_season(&mut db, season)
        .await?
        .map(|stored| stored.first);
    Ok(first.map_or(today, |first| first.max(today)))
}

pub async fn load(state: &AppState) -> Result<PlayersView, StoreError> {
    let season = current_season(state.clock.as_ref());
    let today = today_in_netherlands(state.clock.as_ref());
    let previous = Season::from_start_year(season.start_year() - 1);
    let all = players::all(&state.pool).await?;
    let roles = players::roles_in(&state.pool, &[season, previous]).await?;
    let proposed_from = proposed_from(state, season, today).await?;
    Ok(group(season, today, proposed_from, all, &roles))
}

#[utoipa::path(
    get,
    path = "/api/players",
    operation_id = "players",
    responses(
        (status = 200, description = "The players for the current season", body = PlayersView),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn list(State(state): State<AppState>) -> Result<Json<PlayersView>, AppError> {
    Ok(Json(load(&state).await?))
}

#[utoipa::path(
    post,
    path = "/api/players",
    operation_id = "addPlayer",
    request_body = NewPlayer,
    responses(
        (status = 200, description = "The new player", body = SavedPlayer),
        (status = 400, description = "`first_name_required`, `outside_season`"),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn add(
    State(state): State<AppState>,
    Json(body): Json<NewPlayer>,
) -> Result<Json<SavedPlayer>, AppError> {
    let (first_name, last_name) = clean_name(&body.first_name, body.last_name.as_deref())?;
    if let Some(role) = &body.role {
        check_role(Season::of(role.from), role.from, None, &[])?;
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
        players::insert_role(
            &mut *tx,
            id,
            Season::of(role.from),
            role.role.into(),
            role.from,
            None,
        )
        .await?;
    }
    let same_name =
        players::same_name_exists(&mut *tx, &first_name, last_name.as_deref(), Some(id)).await?;
    tx.commit().await.map_err(StoreError::from)?;
    tracing::info!(player = id.0, "player added");
    Ok(Json(SavedPlayer {
        player: one(&state, id).await?,
        same_name,
    }))
}

#[utoipa::path(
    put,
    path = "/api/players/{id}",
    operation_id = "renamePlayer",
    params(("id" = i64, Path, description = "The player")),
    request_body = NameChange,
    responses(
        (status = 200, description = "The player", body = SavedPlayer),
        (status = 400, description = "`first_name_required`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such player"),
    )
)]
pub async fn rename(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<NameChange>,
) -> Result<Json<SavedPlayer>, AppError> {
    let id = PlayerId(id);
    let (first_name, last_name) = clean_name(&body.first_name, body.last_name.as_deref())?;
    if !players::update_name(&state.pool, id, &first_name, last_name.as_deref()).await? {
        return Err(AppError::NotFound);
    }
    let same_name =
        players::same_name_exists(&state.pool, &first_name, last_name.as_deref(), Some(id)).await?;
    Ok(Json(SavedPlayer {
        player: one(&state, id).await?,
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
        (status = 200, description = "The player", body = PlayerView),
        (status = 400, description = "`until_before_from`, `outside_season`, `roles_overlap`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such player"),
    )
)]
pub async fn add_role(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<RoleChange>,
) -> Result<Json<PlayerView>, AppError> {
    save_role(&state, PlayerId(id), None, body).await?;
    Ok(Json(one(&state, PlayerId(id)).await?))
}

#[utoipa::path(
    put,
    path = "/api/players/{id}/roles/{role}",
    operation_id = "changeRole",
    params(
        ("id" = i64, Path, description = "The player"),
        ("role" = i64, Path, description = "The role"),
    ),
    request_body = RoleChange,
    responses(
        (status = 200, description = "The player", body = PlayerView),
        (status = 400, description = "`until_before_from`, `outside_season`, `roles_overlap`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such player or role"),
    )
)]
pub async fn change_role(
    State(state): State<AppState>,
    Path((id, role)): Path<(i64, i64)>,
    Json(body): Json<RoleChange>,
) -> Result<Json<PlayerView>, AppError> {
    save_role(&state, PlayerId(id), Some(RoleId(role)), body).await?;
    Ok(Json(one(&state, PlayerId(id)).await?))
}

/// Adds a role (`role` is `None`) or changes one, under the player's lock, so two saves at the same
/// time cannot make an overlap.
async fn save_role(
    state: &AppState,
    player: PlayerId,
    role: Option<RoleId>,
    body: RoleChange,
) -> Result<(), AppError> {
    let season = Season::of(body.from);
    let mut tx = state.pool.begin().await.map_err(StoreError::from)?;
    if !players::lock(&mut tx, player).await? {
        return Err(AppError::NotFound);
    }
    let roles = players::roles_of(&mut *tx, player).await?;
    if role.is_some_and(|id| !roles.iter().any(|stored| stored.id == id)) {
        return Err(AppError::NotFound);
    }
    let others: Vec<PlayerRole> = roles
        .into_iter()
        .filter(|stored| Some(stored.id) != role)
        .collect();
    check_role(season, body.from, body.until, &others)?;
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
        (status = 200, description = "The players; the listed players of last season have the same role in this season", body = PlayersView),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn carry(
    State(state): State<AppState>,
    Json(body): Json<CarryRequest>,
) -> Result<Json<PlayersView>, AppError> {
    let season = current_season(state.clock.as_ref());
    let today = today_in_netherlands(state.clock.as_ref());
    let previous = Season::from_start_year(season.start_year() - 1);
    let from = proposed_from(&state, season, today).await?;
    let mut tx = state.pool.begin().await.map_err(StoreError::from)?;
    for id in body.players.iter().copied().map(PlayerId) {
        // Under the player's lock: two taps at the same time give one role. A player that already
        // has a role in this season, or did not stay until the end of last season, is skipped.
        if !players::lock(&mut tx, id).await? {
            continue;
        }
        let roles: Vec<PlayerRole> = players::roles_of(&mut *tx, id)
            .await?
            .into_iter()
            .filter(|role| role.season == season || role.season == previous)
            .collect();
        if let Some((_, role)) = roles_to_carry(&roles, season).first() {
            players::insert_role(&mut *tx, id, season, *role, from, None).await?;
            tracing::info!(player = id.0, "player carried into the season");
        }
    }
    tx.commit().await.map_err(StoreError::from)?;
    Ok(Json(load(&state).await?))
}
