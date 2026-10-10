//! Login sessions in PostgreSQL (decision 0007). The cookie holds a random token; the table holds
//! only its SHA-256 hash.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use rand::RngCore;
use sha2::{Digest, Sha256};
use sqlx::PgPool;

pub const COOKIE_NAME: &str = "team_session";

/// The person behind a valid session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionUser {
    pub subject: String,
    pub name: String,
}

/// A new random token for the cookie: 32 random bytes, base64url.
pub fn new_token() -> String {
    let mut bytes = [0_u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn hash_token(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}

pub async fn create(
    pool: &PgPool,
    token: &str,
    user: &SessionUser,
    now: DateTime<Utc>,
    expires_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let mut transaction = pool.begin().await?;
    sqlx::query!("DELETE FROM sessions WHERE expires_at <= $1", now)
        .execute(&mut *transaction)
        .await?;
    sqlx::query!(
        "INSERT INTO sessions (token_hash, subject, name, created_at, expires_at)
         VALUES ($1, $2, $3, $4, $5)",
        hash_token(token),
        user.subject,
        user.name,
        now,
        expires_at,
    )
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await
}

pub async fn find_valid(
    pool: &PgPool,
    token: &str,
    now: DateTime<Utc>,
) -> Result<Option<SessionUser>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT subject, name FROM sessions WHERE token_hash = $1 AND expires_at > $2",
        hash_token(token),
        now,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| SessionUser {
        subject: row.subject,
        name: row.name,
    }))
}

pub async fn delete(pool: &PgPool, token: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM sessions WHERE token_hash = $1",
        hash_token(token)
    )
    .execute(pool)
    .await?;
    Ok(())
}
