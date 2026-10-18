//! The players and their roles in the team of each season (tables `players` and `player_roles`).
//! Names are personal data of children: they are stored encrypted (`cipher`) and never go to the
//! log.

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{PgConnection, PgExecutor, PgPool};
use team_core::{Player, PlayerId, PlayerRole, Role, RoleId, Season};

use crate::cipher::{NameField, NameKey, open, seal};
use crate::db::StoreError;

/// All players, sorted by first name, then last name (`Player::sort_key`).
pub async fn all(db: impl PgExecutor<'_>, key: &NameKey) -> Result<Vec<Player>, StoreError> {
    let rows = sqlx::query_as!(
        NameRow,
        "SELECT id, first_name_enc, last_name_enc, hidden FROM players"
    )
    .fetch_all(db)
    .await?;
    let mut players = rows
        .into_iter()
        .map(|row| row.into_player(key))
        .collect::<Result<Vec<_>, _>>()?;
    players.sort_by_cached_key(Player::sort_key);
    Ok(players)
}

pub async fn by_id(
    db: impl PgExecutor<'_>,
    key: &NameKey,
    id: PlayerId,
) -> Result<Option<Player>, StoreError> {
    let row = sqlx::query_as!(
        NameRow,
        "SELECT id, first_name_enc, last_name_enc, hidden FROM players WHERE id = $1",
        id.0
    )
    .fetch_optional(db)
    .await?;
    row.map(|row| row.into_player(key)).transpose()
}

pub async fn insert(
    db: impl PgExecutor<'_>,
    key: &NameKey,
    first_name: &str,
    last_name: Option<&str>,
    now: DateTime<Utc>,
) -> Result<PlayerId, StoreError> {
    let (first, last) = seal_names(key, first_name, last_name)?;
    let id = sqlx::query_scalar!(
        "INSERT INTO players (first_name_enc, last_name_enc, created_at) VALUES ($1, $2, $3)
         RETURNING id",
        first,
        last,
        now,
    )
    .fetch_one(db)
    .await?;
    Ok(PlayerId(id))
}

/// Changes the name; `false` when there is no such player.
pub async fn update_name(
    db: impl PgExecutor<'_>,
    key: &NameKey,
    id: PlayerId,
    first_name: &str,
    last_name: Option<&str>,
) -> Result<bool, StoreError> {
    let (first, last) = seal_names(key, first_name, last_name)?;
    let result = sqlx::query!(
        "UPDATE players
         SET first_name_enc = $2, last_name_enc = $3, first_name = NULL, last_name = NULL
         WHERE id = $1",
        id.0,
        first,
        last,
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected() == 1)
}

/// Encrypts the names of the players that still have a plain first name, and empties the plain
/// columns: players from before the encryption, and players that an older version added or renamed
/// during an update (it writes both plain columns, so a plain name is always the newer one, also
/// over an encrypted name). Runs in `team migrate`; returns how many players it encrypted.
pub async fn encrypt_plain_names(tx: &mut PgConnection, key: &NameKey) -> Result<u64, StoreError> {
    let rows = sqlx::query!(
        "SELECT id, first_name, last_name FROM players WHERE first_name IS NOT NULL
         ORDER BY id FOR UPDATE"
    )
    .fetch_all(&mut *tx)
    .await?;
    let mut count = 0;
    for row in rows {
        let Some(first_name) = row.first_name else {
            return Err(StoreError::Invalid(format!(
                "player {} has no name",
                row.id
            )));
        };
        let (first, last) = seal_names(key, &first_name, row.last_name.as_deref())?;
        sqlx::query!(
            "UPDATE players
             SET first_name_enc = $2, last_name_enc = $3, first_name = NULL, last_name = NULL
             WHERE id = $1",
            row.id,
            first,
            last,
        )
        .execute(&mut *tx)
        .await?;
        count += 1;
    }
    Ok(count)
}

/// Whether `key` reads the stored names: decrypts the name of the player with the lowest id, if
/// there is one with an encrypted name. A wrong key gives `StoreError::Name`.
pub async fn check_key(db: impl PgExecutor<'_>, key: &NameKey) -> Result<(), StoreError> {
    let row = sqlx::query_as!(
        NameRow,
        "SELECT id, first_name_enc, last_name_enc, hidden FROM players
         WHERE first_name_enc IS NOT NULL ORDER BY id LIMIT 1"
    )
    .fetch_optional(db)
    .await?;
    match row {
        Some(row) => row.into_player(key).map(|_| ()),
        None => Ok(()),
    }
}

/// The step of `team migrate` after the SQL migrations: checks the key against the stored names
/// first (so names are never encrypted with two keys), then encrypts the plain names in one
/// transaction. Returns how many players it encrypted.
pub async fn encrypt_after_migrations(pool: &PgPool, key: &NameKey) -> Result<u64, StoreError> {
    check_key(pool, key).await?;
    let mut tx = pool.begin().await?;
    let count = encrypt_plain_names(&mut tx, key).await?;
    tx.commit().await?;
    Ok(count)
}

fn seal_names(
    key: &NameKey,
    first_name: &str,
    last_name: Option<&str>,
) -> Result<(Vec<u8>, Option<Vec<u8>>), StoreError> {
    let first = seal(key, NameField::First, first_name).map_err(StoreError::Seal)?;
    let last = last_name
        .map(|last| seal(key, NameField::Last, last))
        .transpose()
        .map_err(StoreError::Seal)?;
    Ok((first, last))
}

struct NameRow {
    id: i64,
    first_name_enc: Option<Vec<u8>>,
    last_name_enc: Option<Vec<u8>>,
    hidden: bool,
}

impl NameRow {
    fn into_player(self, key: &NameKey) -> Result<Player, StoreError> {
        let player = PlayerId(self.id);
        let failed = |error| StoreError::Name { player, error };
        // A plain row, written by an older version during an update, until the next
        // `team migrate` encrypts it.
        let Some(first) = self.first_name_enc else {
            return Err(StoreError::Invalid(format!(
                "the name of player {} is not encrypted yet",
                self.id
            )));
        };
        Ok(Player {
            id: player,
            first_name: open(key, NameField::First, &first).map_err(failed)?,
            last_name: self
                .last_name_enc
                .map(|last| open(key, NameField::Last, &last))
                .transpose()
                .map_err(failed)?,
            hidden: self.hidden,
        })
    }
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
