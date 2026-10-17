//! `/api/team` and the actions of the team page: the team of the current season, "Change role" and
//! "Remove from the team" (attendance at training, story A). Names never go to the log; only ids
//! do.

use axum::Json;
use axum::extract::{Path, State};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use team_core::{
    Member, PlayerId, PlayerRole, RoleDates, RoleId, change_role, proposed_start, remove_from_team,
    team,
};
use utoipa::ToSchema;

use super::players::{RoleKind, RoleView, day, role_view};
use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgConnection;

use crate::app::AppState;
use crate::clock::{current_season, today_in_netherlands};
use crate::db::StoreError;
use crate::error::AppError;
use crate::{followed_teams, players};

/// A player in a group of the team page, with a role.
#[derive(Debug, Serialize, ToSchema)]
pub struct MemberView {
    pub id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    /// Today's role (or the coming one); in "left", the last role of the season; in
    /// "last_season", the last role of last season.
    pub role: RoleKind,
    /// The roles of the season shown (last season's for "last_season").
    pub roles: Vec<RoleView>,
    /// In the team, and today is the last day: the role today ends today, no role follows, and
    /// today is not the last day of the season. The row shows "Last day today".
    pub last_day_today: bool,
}

/// A player that "Add player" offers.
#[derive(Debug, Serialize, ToSchema)]
pub struct PlayerSummary {
    pub id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
}

/// The team of the current season.
#[derive(Debug, Serialize, ToSchema)]
pub struct TeamPageView {
    pub season: String,
    /// The name of the team chosen from the game list for the season, for example "U14-1";
    /// `null` while no team is chosen.
    pub team_name: Option<String>,
    /// Today's Dutch date: the proposed date of "Change role" and "Remove from the team".
    pub today: String,
    /// The first day of the season, for example "2026-07-01": the earliest date of a role.
    pub season_first_day: String,
    /// The last day of the season, for example "2027-06-30": the latest date of a role.
    pub season_last_day: String,
    /// The proposed "In the team from" of "Add player" and "Add all": 1 September of the season.
    pub proposed_from: String,
    /// Players with a role today or later in the season, sorted by first name.
    pub members: Vec<MemberView>,
    /// Players with a role earlier in the season and none today or later.
    pub left: Vec<MemberView>,
    /// Players who were in the team until the end of last season, have no role in this season and
    /// are not hidden: "Add all" brings them back. Empty as soon as one player of last season's
    /// team has a role in this season.
    pub last_season: Vec<MemberView>,
    /// The players "Add player" offers: not hidden and not in the team this season; last season's
    /// team first.
    pub candidates: Vec<PlayerSummary>,
}

/// The body of a refused request: the code, with details for some codes.
#[derive(Debug, Serialize, ToSchema)]
pub struct Refusal {
    /// For example `role_starts_later` or `changed_meanwhile`.
    pub error: String,
    /// For `role_starts_later`: the start of the player's later role, for example "2026-11-02".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// For `confirm_removal`: the roles "Remove from the team" would remove, in order of start.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<RoleView>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ChangeRole {
    pub role: RoleKind,
    /// The first day of the new role; the current role ends on the day before.
    #[schema(value_type = String, format = Date)]
    pub from: NaiveDate,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LeaveTeam {
    /// "Not in the team from": the first day without the team.
    #[schema(value_type = String, format = Date)]
    pub from: NaiveDate,
    /// The ids of the roles the coordinator confirmed to remove (roles that start on `from` or
    /// later); empty at first.
    #[serde(default)]
    pub removes: Vec<i64>,
}

/// A role without its id, as "Undo" sends it back.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RoleDatesView {
    pub role: RoleKind,
    #[schema(value_type = String, format = Date)]
    pub from: NaiveDate,
    /// `null` while the role continues.
    #[schema(value_type = Option<String>, format = Date)]
    pub until: Option<NaiveDate>,
}

impl From<&RoleDates> for RoleDatesView {
    fn from(role: &RoleDates) -> Self {
        RoleDatesView {
            role: role.role.into(),
            from: role.from,
            until: role.until,
        }
    }
}

/// What "Undo" sends back after "Remove from the team": the date of the removal and the
/// player's roles of the season from before it.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UndoRemoval {
    #[schema(value_type = String, format = Date)]
    pub from: NaiveDate,
    pub roles: Vec<RoleDatesView>,
}

/// The answer of "Remove from the team".
#[derive(Debug, Serialize, ToSchema)]
pub struct LeftTeam {
    pub team: TeamPageView,
    pub undo: UndoRemoval,
}

fn member_view(member: &Member) -> MemberView {
    MemberView {
        id: member.player.id.0,
        first_name: member.player.first_name.clone(),
        last_name: member.player.last_name.clone(),
        role: member.role.into(),
        roles: member.roles.iter().map(|role| role_view(role)).collect(),
        last_day_today: member.last_day_today,
    }
}

/// The team page of the current season.
pub async fn load(state: &AppState) -> Result<TeamPageView, StoreError> {
    let season = current_season(state.clock.as_ref());
    let today = today_in_netherlands(state.clock.as_ref());
    let all = players::all(&state.pool).await?;
    let roles = players::roles_in(&state.pool, &[season, season.previous()]).await?;
    let team_name = followed_teams::for_season(&state.pool, season)
        .await?
        .and_then(|choice| choice.team)
        .map(|team| team.name);
    let page = team(&all, &roles, season, today);
    Ok(TeamPageView {
        season: season.to_string(),
        team_name,
        today: day(today),
        season_first_day: day(season.first_day()),
        season_last_day: day(season.last_day()),
        proposed_from: day(proposed_start(season)),
        members: page.members.iter().map(member_view).collect(),
        left: page.left.iter().map(member_view).collect(),
        last_season: page.last_season.iter().map(member_view).collect(),
        candidates: page
            .candidates
            .iter()
            .map(|player| PlayerSummary {
                id: player.id.0,
                first_name: player.first_name.clone(),
                last_name: player.last_name.clone(),
            })
            .collect(),
    })
}

#[utoipa::path(
    get,
    path = "/api/team",
    operation_id = "team",
    responses(
        (status = 200, description = "The team of the current season", body = TeamPageView),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn get(State(state): State<AppState>) -> Result<Json<TeamPageView>, AppError> {
    Ok(Json(load(&state).await?))
}

#[utoipa::path(
    post,
    path = "/api/players/{id}/role-change",
    operation_id = "changeRole",
    params(("id" = i64, Path, description = "The player")),
    request_body = ChangeRole,
    responses(
        (status = 200, description = "The team; the role on `from` ends the day before, the new role starts on `from`", body = TeamPageView),
        (status = 400, description = "`not_in_team`, `role_starts_later` (with `date`), `same_role`, `change_before_role_start`, `outside_season`, `roles_overlap`", body = Refusal),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such player"),
    )
)]
pub async fn role_change(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<ChangeRole>,
) -> Result<Json<TeamPageView>, AppError> {
    let player = PlayerId(id);
    let season = current_season(state.clock.as_ref());
    let mut tx = state.pool.begin().await.map_err(StoreError::from)?;
    if !players::lock(&mut tx, player).await? {
        return Err(AppError::NotFound);
    }
    let roles = players::roles_of(&mut *tx, player).await?;
    let change = change_role(season, &roles, body.role.into(), body.from)?;
    // Both or neither: the end of the current role and the new role.
    end_role(&mut tx, &roles, change.ended, change.ended_until).await?;
    players::insert_role(
        &mut *tx,
        player,
        season,
        change.role,
        change.from,
        change.until,
    )
    .await?;
    tx.commit().await.map_err(StoreError::from)?;
    tracing::info!(player = player.0, "role changed");
    Ok(Json(load(&state).await?))
}

#[utoipa::path(
    post,
    path = "/api/players/{id}/team-leave",
    operation_id = "leaveTeam",
    params(("id" = i64, Path, description = "The player")),
    request_body = LeaveTeam,
    responses(
        (status = 200, description = "The team, and what \"Undo\" sends back; from `from` on the player has no roles in the season", body = LeftTeam),
        (status = 400, description = "`not_in_team`, `outside_season`", body = Refusal),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such player"),
        (status = 409, description = "`confirm_removal`: the removal removes roles that are not in `removes`; nothing is saved", body = Refusal),
    )
)]
pub async fn team_leave(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<LeaveTeam>,
) -> Result<Json<LeftTeam>, AppError> {
    let player = PlayerId(id);
    let season = current_season(state.clock.as_ref());
    let mut tx = state.pool.begin().await.map_err(StoreError::from)?;
    if !players::lock(&mut tx, player).await? {
        return Err(AppError::NotFound);
    }
    let roles = players::roles_of(&mut *tx, player).await?;
    let removal = remove_from_team(season, &roles, body.from)?;
    if removal
        .removed
        .iter()
        .any(|role| !body.removes.contains(&role.id.0))
    {
        let removed: Vec<RoleView> = removal.removed.iter().map(|role| role_view(role)).collect();
        return Err(AppError::Refused {
            status: StatusCode::CONFLICT,
            code: "confirm_removal",
            details: json!({ "roles": removed }),
        });
    }
    let before: Vec<RoleDatesView> = roles
        .iter()
        .filter(|role| role.season == season)
        .map(|role| RoleDatesView::from(&RoleDates::from(role)))
        .collect();
    // All or nothing: the end of one role and the removal of the others.
    if let Some((ended, until)) = removal.ended {
        end_role(&mut tx, &roles, ended, until).await?;
    }
    for role in &removal.removed {
        players::delete_role(&mut *tx, player, role.id).await?;
    }
    tx.commit().await.map_err(StoreError::from)?;
    tracing::info!(player = player.0, "player left the team");
    Ok(Json(LeftTeam {
        team: load(&state).await?,
        undo: UndoRemoval {
            from: body.from,
            roles: before,
        },
    }))
}

/// Gives the stored role `id` (one of `roles`) the end `until`.
async fn end_role(
    tx: &mut PgConnection,
    roles: &[PlayerRole],
    id: RoleId,
    until: NaiveDate,
) -> Result<(), AppError> {
    let Some(role) = roles.iter().find(|role| role.id == id) else {
        return Err(AppError::BadRequest("not_in_team"));
    };
    players::update_role(
        tx,
        role.player,
        role.id,
        role.season,
        role.role,
        role.from,
        Some(until),
    )
    .await?;
    Ok(())
}
