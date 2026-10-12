//! The log of a full login contains no secrets (step 2 of the base).
//!
//! This test has its own file (its own test process): tracing caches which log lines are enabled
//! per process, and other tests running at the same time can make that cache skip the lines this
//! test captures.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use sqlx::PgPool;

use common::{CLIENT_SECRET, Harness, LogBuffer, TokenSpec, cookie_value};

const ROLE: &str = "team_coordinator";

fn session_cookie(value: &str) -> String {
    format!("team_session={value}")
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn logs_contain_no_secrets(pool: PgPool) {
    let (buffer, _guard) = LogBuffer::capture();

    let harness = Harness::start(pool).await;
    let login = harness.start_login().await;
    let id_token = TokenSpec::new(&harness.issuer(), &login.nonce, &[ROLE]).sign();
    harness.answer_token(&id_token).await;
    let response = harness.callback(&login, &login.state).await;
    let session = cookie_value(&response, "team_session").unwrap();
    harness
        .get("/api/me", Some(&session_cookie(&session)))
        .await;

    let logs = buffer.text();
    assert!(logs.contains("logged in"), "the login is logged");
    for secret in [
        session.as_str(),
        login.login_cookie.as_str(),
        "code-for-tests",
        "access-token-for-tests",
        id_token.as_str(),
        CLIENT_SECRET,
        common::SESSION_SECRET,
    ] {
        assert!(!logs.contains(secret), "log contains a secret: {secret}");
    }
}
