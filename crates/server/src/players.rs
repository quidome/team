//! The players and their roles in the team of each season (tables `players` and `player_roles`).
//! Names are personal data of children: they never go to the log.

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{PgConnection, PgExecutor};
use team_core::{Player, PlayerId, PlayerRole, Role, RoleId, Season};

use crate::db::StoreError;

/// All players, sorted by first name, then last name.
pub async fn all(db: impl PgExecutor<'_>) -> Result<Vec<Player>, StoreError> {
    let rows = sqlx::query!(
        "SELECT id, first_name, last_name, hidden FROM players
         ORDER BY lower(first_name), lower(coalesce(last_name, '')), id"
    )
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| Player {
            id: PlayerId(row.id),
            first_name: row.first_name,
            last_name: row.last_name,
            hidden: row.hidden,
        })
        .collect())
}

pub async fn by_id(db: impl PgExecutor<'_>, id: PlayerId) -> Result<Option<Player>, StoreError> {
    let row = sqlx::query!(
        "SELECT id, first_name, last_name, hidden FROM players WHERE id = $1",
        id.0
    )
    .fetch_optional(db)
    .await?;
    Ok(row.map(|row| Player {
        id: PlayerId(row.id),
        first_name: row.first_name,
        last_name: row.last_name,
        hidden: row.hidden,
    }))
}

/// Whether another player has the same first and last name (without case).
pub async fn same_name_exists(
    db: impl PgExecutor<'_>,
    first_name: &str,
    last_name: Option<&str>,
    except: Option<PlayerId>,
) -> Result<bool, StoreError> {
    let exists = sqlx::query_scalar!(
        r#"SELECT EXISTS (
               SELECT 1 FROM players
               WHERE lower(first_name) = lower($1)
                 AND lower(coalesce(last_name, '')) = lower(coalesce($2, ''))
                 AND id IS DISTINCT FROM $3
           ) AS "exists!""#,
        first_name,
        last_name,
        except.map(|id| id.0),
    )
    .fetch_one(db)
    .await?;
    Ok(exists)
}

pub async fn insert(
    db: impl PgExecutor<'_>,
    first_name: &str,
    last_name: Option<&str>,
    now: DateTime<Utc>,
) -> Result<PlayerId, StoreError> {
    let id = sqlx::query_scalar!(
        "INSERT INTO players (first_name, last_name, created_at) VALUES ($1, $2, $3) RETURNING id",
        first_name,
        last_name,
        now,
    )
    .fetch_one(db)
    .await?;
    Ok(PlayerId(id))
}

/// Changes the name; `false` when there is no such player.
pub async fn update_name(
    db: impl PgExecutor<'_>,
    id: PlayerId,
    first_name: &str,
    last_name: Option<&str>,
) -> Result<bool, StoreError> {
    let result = sqlx::query!(
        "UPDATE players SET first_name = $2, last_name = $3 WHERE id = $1",
        id.0,
        first_name,
        last_name,
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected() == 1)
}

/// Sets whether the player is hidden; `false` when there is no such player.
pub async fn set_hidden(
    db: impl PgExecutor<'_>,
    id: PlayerId,
    hidden: bool,
) -> Result<bool, StoreError> {
    let result = sqlx::query!("UPDATE players SET hidden = $2 WHERE id = $1", id.0, hidden)
        .execute(db)
        .await?;
    Ok(result.rows_affected() == 1)
}

/// Deletes the player and its roles; `false` when there is no such player.
pub async fn delete(db: impl PgExecutor<'_>, id: PlayerId) -> Result<bool, StoreError> {
    let result = sqlx::query!("DELETE FROM players WHERE id = $1", id.0)
        .execute(db)
        .await?;
    Ok(result.rows_affected() == 1)
}

/// Locks the player's row until the end of the transaction, so two saves of its roles run one
/// after the other; `false` when there is no such player.
pub async fn lock(tx: &mut PgConnection, id: PlayerId) -> Result<bool, StoreError> {
    let row = sqlx::query_scalar!("SELECT id FROM players WHERE id = $1 FOR UPDATE", id.0)
        .fetch_optional(tx)
        .await?;
    Ok(row.is_some())
}

/// The roles of the given seasons, of all players, in order of start.
pub async fn roles_in(
    db: impl PgExecutor<'_>,
    seasons: &[Season],
) -> Result<Vec<PlayerRole>, StoreError> {
    let years: Vec<i32> = seasons.iter().map(|season| season.start_year()).collect();
    let rows = sqlx::query_as!(
        RoleRow,
        "SELECT id, player_id, season_year, role, from_date, until_date FROM player_roles
         WHERE season_year = ANY($1) ORDER BY from_date, id",
        &years,
    )
    .fetch_all(db)
    .await?;
    rows.into_iter().map(RoleRow::into_role).collect()
}

/// All roles of one player, in order of start.
pub async fn roles_of(
    db: impl PgExecutor<'_>,
    player: PlayerId,
) -> Result<Vec<PlayerRole>, StoreError> {
    let rows = sqlx::query_as!(
        RoleRow,
        "SELECT id, player_id, season_year, role, from_date, until_date FROM player_roles
         WHERE player_id = $1 ORDER BY from_date, id",
        player.0,
    )
    .fetch_all(db)
    .await?;
    rows.into_iter().map(RoleRow::into_role).collect()
}

/// A new role. The caller checked it (`team_core::check_role`) under the player's lock.
pub async fn insert_role(
    db: impl PgExecutor<'_>,
    player: PlayerId,
    season: Season,
    role: Role,
    from: NaiveDate,
    until: Option<NaiveDate>,
) -> Result<RoleId, StoreError> {
    let id = sqlx::query_scalar!(
        "INSERT INTO player_roles (player_id, season_year, role, from_date, until_date)
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
        player.0,
        season.start_year(),
        role_text(role),
        from,
        until,
    )
    .fetch_one(db)
    .await?;
    Ok(RoleId(id))
}

/// Changes a role of `player`; `false` when the player has no such role.
pub async fn update_role(
    db: impl PgExecutor<'_>,
    player: PlayerId,
    id: RoleId,
    season: Season,
    role: Role,
    from: NaiveDate,
    until: Option<NaiveDate>,
) -> Result<bool, StoreError> {
    let result = sqlx::query!(
        "UPDATE player_roles SET season_year = $3, role = $4, from_date = $5, until_date = $6
         WHERE id = $2 AND player_id = $1",
        player.0,
        id.0,
        season.start_year(),
        role_text(role),
        from,
        until,
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected() == 1)
}

/// Deletes a role of `player`; `false` when the player has no such role.
pub async fn delete_role(
    db: impl PgExecutor<'_>,
    player: PlayerId,
    id: RoleId,
) -> Result<bool, StoreError> {
    let result = sqlx::query!(
        "DELETE FROM player_roles WHERE id = $2 AND player_id = $1",
        player.0,
        id.0,
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected() == 1)
}

fn role_text(role: Role) -> &'static str {
    match role {
        Role::PlaysGames => "plays_games",
        Role::OnlyTrains => "only_trains",
    }
}

struct RoleRow {
    id: i64,
    player_id: i64,
    season_year: i32,
    role: String,
    from_date: NaiveDate,
    until_date: Option<NaiveDate>,
}

impl RoleRow {
    fn into_role(self) -> Result<PlayerRole, StoreError> {
        let role = match self.role.as_str() {
            "plays_games" => Role::PlaysGames,
            "only_trains" => Role::OnlyTrains,
            other => return Err(StoreError::Invalid(format!("unknown role {other:?}"))),
        };
        Ok(PlayerRole {
            id: RoleId(self.id),
            player: PlayerId(self.player_id),
            season: Season::from_start_year(self.season_year),
            role,
            from: self.from_date,
            until: self.until_date,
        })
    }
}
