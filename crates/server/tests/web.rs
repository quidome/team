//! Serving the built front end (step 3 of the base).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use axum::http::{StatusCode, header};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use common::{Harness, INDEX_HTML, START_SCRIPT, body_text};

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn app_paths_get_index_html(pool: PgPool) {
    let harness = Harness::start(pool).await;
    for path in ["/", "/sample", "/some/deep/link"] {
        let response = harness.get(path, None).await;
        assert_eq!(response.status(), StatusCode::OK, "{path}");
        assert!(
            response.headers()[header::CONTENT_TYPE]
                .to_str()
                .unwrap()
                .starts_with("text/html"),
            "{path}"
        );
        assert_eq!(body_text(response).await, INDEX_HTML, "{path}");
    }
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn built_files_get_their_content_type(pool: PgPool) {
    let harness = Harness::start(pool).await;
    let response = harness.get("/_app/start.js", None).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "text/javascript");
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn api_paths_never_get_index_html(pool: PgPool) {
    let harness = Harness::start(pool).await;
    let response = harness.get("/api/does-not-exist", None).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert!(!body_text(response).await.contains("<html"));

    let token = harness.log_in(&["team_coordinator"]).await;
    let response = harness
        .get(
            "/api/does-not-exist",
            Some(&format!("team_session={token}")),
        )
        .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(!body_text(response).await.contains("<html"));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn policy_allows_exactly_the_start_script(pool: PgPool) {
    let harness = Harness::start(pool).await;
    let response = harness.get("/", None).await;
    let hash = STANDARD.encode(Sha256::digest(START_SCRIPT.as_bytes()));
    assert_eq!(
        response.headers()[header::CONTENT_SECURITY_POLICY],
        format!(
            "default-src 'self'; script-src 'self' 'sha256-{hash}'; \
             style-src 'self' 'unsafe-inline'; frame-ancestors 'none'"
        )
        .as_str()
    );
}
