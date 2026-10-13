//! Database pool, migrations and the readiness check (decision 0004).

use std::collections::HashMap;
use std::time::Duration;

use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgConnection, PgPool};

/// The SQL files in `migrations/`, compiled into the binary.
pub static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");

/// A pool that connects on first use, so `team serve` starts even when the database is down;
/// readiness then reports it.
pub fn lazy_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect_lazy(database_url)
}

/// Applies the migrations that are not applied yet, under sqlx's lock.
pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    MIGRATOR.run(pool).await
}

/// The key of the advisory lock that every reading and every save of the choice takes ("team"
/// "read" in ASCII), so they run one after the other (decision 0005).
const READING_LOCK: i64 = 0x7465_616d_7265_6164;

/// Takes the reading lock until the end of the transaction.
pub async fn lock_readings(tx: &mut PgConnection) -> Result<(), StoreError> {
    sqlx::query!("SELECT pg_advisory_xact_lock($1)", READING_LOCK)
        .execute(tx)
        .await?;
    Ok(())
}

/// A failure of the stored data: of the database, or of data in it that the code does not
/// understand.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("stored data is not valid: {0}")]
    Invalid(String),
}

#[derive(Debug, thiserror::Error)]
pub enum NotReady {
    #[error("database is not reachable: {0}")]
    Database(#[from] sqlx::Error),
    #[error("migration {0} is not applied")]
    MissingMigration(i64),
    #[error("migration {0} failed")]
    FailedMigration(i64),
}

/// Ready when the database answers and every migration in this binary is applied with success.
/// The app never migrates by itself.
pub async fn check_ready(pool: &PgPool) -> Result<(), NotReady> {
    sqlx::query!("SELECT 1 AS one").fetch_one(pool).await?;

    let applied: HashMap<i64, bool> = sqlx::query!("SELECT version, success FROM _sqlx_migrations")
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| (row.version, row.success))
        .collect();

    for migration in MIGRATOR
        .iter()
        .filter(|migration| !migration.migration_type.is_down_migration())
    {
        match applied.get(&migration.version) {
            None => return Err(NotReady::MissingMigration(migration.version)),
            Some(false) => return Err(NotReady::FailedMigration(migration.version)),
            Some(true) => {}
        }
    }
    Ok(())
}
