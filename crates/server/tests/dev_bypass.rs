//! The login bypass works in debug builds (decision 0007).
//!
//! That a release build ignores it is checked on the real image by `scripts/smoke-test.sh`
//! (decision 0010).

// The bypass code exists only in debug builds, so this whole test file does too.
#![cfg(debug_assertions)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use axum::http::StatusCode;
use sqlx::PgPool;

use common::{Harness, body_text};

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn dev_auth_bypass_works_in_debug_builds(pool: PgPool) {
    let harness = Harness::start_with(pool, &[("DEV_AUTH_BYPASS", "true")]).await;
    let response = harness.get("/api/me", None).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_text(response).await, r#"{"name":"Developer"}"#);
}
