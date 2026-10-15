//! The reasons for an absence (table `reasons`), in their order.

use sqlx::PgExecutor;

use crate::db::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredReason {
    pub id: i64,
    pub name: String,
    pub valid: bool,
    pub hidden: bool,
}

/// All reasons, also the hidden ones, in their order.
pub async fn all(db: impl PgExecutor<'_>) -> Result<Vec<StoredReason>, StoreError> {
    let reasons = sqlx::query_as!(
        StoredReason,
        "SELECT id, name, valid, hidden FROM reasons ORDER BY position, id"
    )
    .fetch_all(db)
    .await?;
    Ok(reasons)
}

/// A new reason, last in the order. A name that is taken fails on the unique index
/// (`StoreError::is_unique_violation`).
pub async fn insert(db: impl PgExecutor<'_>, name: &str, valid: bool) -> Result<i64, StoreError> {
    let id = sqlx::query_scalar!(
        "INSERT INTO reasons (name, valid, position)
         VALUES ($1, $2, (SELECT coalesce(max(position), 0) + 1 FROM reasons))
         RETURNING id",
        name,
        valid,
    )
    .fetch_one(db)
    .await?;
    Ok(id)
}

/// Changes a reason; `false` when there is no such reason.
pub async fn update(
    db: impl PgExecutor<'_>,
    id: i64,
    name: &str,
    valid: bool,
    hidden: bool,
) -> Result<bool, StoreError> {
    let result = sqlx::query!(
        "UPDATE reasons SET name = $2, valid = $3, hidden = $4 WHERE id = $1",
        id,
        name,
        valid,
        hidden,
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected() == 1)
}
