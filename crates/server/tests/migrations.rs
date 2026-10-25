//! Migrations that change stored data: the data before them, and after them.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use sqlx::PgPool;

/// Runs the SQL of every migration before `version`, without the record of migrations.
async fn migrate_before(pool: &PgPool, version: i64) {
    for migration in team::db::MIGRATOR.iter().filter(|m| m.version < version) {
        sqlx::raw_sql(&migration.sql).execute(pool).await.unwrap();
    }
}

/// Runs the SQL of the migration `version`.
async fn migrate_one(pool: &PgPool, version: i64) {
    let migration = team::db::MIGRATOR
        .iter()
        .find(|m| m.version == version)
        .unwrap();
    sqlx::raw_sql(&migration.sql).execute(pool).await.unwrap();
}

/// A training `days` from today (Dutch date), for example -1 for yesterday.
async fn insert_training(pool: &PgPool, days: i32) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO trainings (date, start_time, end_time, created_at)
         VALUES ((now() AT TIME ZONE 'Europe/Amsterdam')::date + $1, '19:45', '21:00', now())
         RETURNING id",
    )
    .bind(days)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[sqlx::test(migrations = false)]
async fn the_release_checks_the_trainings_before_today(pool: PgPool) {
    migrate_before(&pool, 10).await;
    let yesterday = insert_training(&pool, -1).await;
    let today = insert_training(&pool, 0).await;
    let tomorrow = insert_training(&pool, 1).await;

    migrate_one(&pool, 10).await;

    let checked: Vec<(i64, bool)> =
        sqlx::query_as("SELECT id, checked_at IS NOT NULL FROM trainings ORDER BY date")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(
        checked,
        [(yesterday, true), (today, false), (tomorrow, false)]
    );
}
