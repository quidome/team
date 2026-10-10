//! Health checks, migrations, the closed `/api` router and security headers (step 1 of the base).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use axum::http::{StatusCode, header};
use sqlx::PgPool;

use common::{Harness, body_text};

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn liveness_needs_no_login(pool: PgPool) {
    let response = Harness::start(pool)
        .await
        .get("/api/health/liveness", None)
        .await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn readiness_is_ok_when_migrated(pool: PgPool) {
    let response = Harness::start(pool)
        .await
        .get("/api/health/readiness", None)
        .await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test(migrations = false)]
async fn readiness_fails_without_migrations(pool: PgPool) {
    let response = Harness::start(pool)
        .await
        .get("/api/health/readiness", None)
        .await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[sqlx::test(migrations = false)]
async fn readiness_fails_with_a_missing_migration(pool: PgPool) {
    team::db::migrate(&pool).await.unwrap();
    let latest = team::db::MIGRATOR
        .iter()
        .map(|migration| migration.version)
        .max()
        .unwrap();
    sqlx::query("DELETE FROM _sqlx_migrations WHERE version = $1")
        .bind(latest)
        .execute(&pool)
        .await
        .unwrap();

    let response = Harness::start(pool)
        .await
        .get("/api/health/readiness", None)
        .await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[sqlx::test(migrations = false)]
async fn migrate_twice_applies_nothing_the_second_time(pool: PgPool) {
    team::db::migrate(&pool).await.unwrap();
    let count = || async {
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM _sqlx_migrations")
            .fetch_one(&pool)
            .await
            .unwrap()
    };
    let after_first = count().await;

    team::db::migrate(&pool).await.unwrap();

    assert_eq!(count().await, after_first);
    assert_eq!(
        after_first,
        i64::try_from(team::db::MIGRATOR.iter().count()).unwrap()
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn unknown_api_path_without_login_is_unauthenticated(pool: PgPool) {
    let response = Harness::start(pool)
        .await
        .get("/api/does-not-exist", None)
        .await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(body_text(response).await, r#"{"error":"unauthenticated"}"#);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn responses_carry_the_security_headers(pool: PgPool) {
    let harness = Harness::start(pool).await;
    for path in ["/", "/api/health/liveness", "/api/does-not-exist"] {
        let response = harness.get(path, None).await;
        let headers = response.headers();
        assert_eq!(
            headers[header::CONTENT_SECURITY_POLICY],
            "default-src 'self'; frame-ancestors 'none'",
            "{path}"
        );
        assert_eq!(headers[header::X_CONTENT_TYPE_OPTIONS], "nosniff", "{path}");
        assert_eq!(headers[header::REFERRER_POLICY], "same-origin", "{path}");
    }
}
