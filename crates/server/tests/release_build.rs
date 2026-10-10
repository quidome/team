//! The login bypass exists only in debug builds (decision 0007).
//!
//! `cargo test` checks the debug side; CI also runs `cargo test --release --test release_build`
//! to check that a release build still needs login when `DEV_AUTH_BYPASS=true`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use axum::http::StatusCode;
use sqlx::PgPool;

use common::{Harness, body_text};

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn dev_auth_bypass_only_works_in_debug_builds(pool: PgPool) {
    let harness = Harness::start_with(pool, &[("DEV_AUTH_BYPASS", "true")]).await;
    let response = harness.get("/api/me", None).await;

    if cfg!(debug_assertions) {
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body_text(response).await, r#"{"name":"Developer"}"#);
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
