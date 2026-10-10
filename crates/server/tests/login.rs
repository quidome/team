//! Login through a fake Pocket ID, sessions, logout and the request rules (step 2 of the base).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use chrono::TimeDelta;
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use common::{
    CLIENT_SECRET, Harness, ORIGIN, OTHER_KEY, TokenSpec, body_text, cookie_value, set_cookie,
};

const ROLE: &str = "team_coordinator";

async fn session_count(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM sessions")
        .fetch_one(pool)
        .await
        .unwrap()
}

fn session_cookie(value: &str) -> String {
    format!("team_session={value}")
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn login_redirects_to_pocket_id_with_pkce_and_scopes(pool: PgPool) {
    let harness = Harness::start(pool).await;
    let response = harness.get("/auth/login", None).await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let cookie = set_cookie(&response, "team_login").unwrap();
    for flag in [
        "HttpOnly",
        "Secure",
        "SameSite=Lax",
        "Path=/auth",
        "Max-Age=600",
    ] {
        assert!(cookie.contains(flag), "{flag} missing in {cookie}");
    }

    let login = harness.start_login().await;
    let query = |name: &str| {
        login
            .location
            .query_pairs()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.into_owned())
    };
    assert!(
        login
            .location
            .as_str()
            .starts_with(&format!("{}/authorize", harness.issuer()))
    );
    assert_eq!(query("scope").as_deref(), Some("openid profile groups"));
    assert_eq!(query("code_challenge_method").as_deref(), Some("S256"));
    assert_eq!(
        query("redirect_uri").as_deref(),
        Some("https://team.example.test/auth/callback")
    );
    assert!(query("code_challenge").is_some());
    assert!(!login.state.is_empty());
    assert!(!login.nonce.is_empty());
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn good_login_gives_a_session_and_me(pool: PgPool) {
    let harness = Harness::start(pool.clone()).await;
    let login = harness.start_login().await;
    let token = TokenSpec::new(&harness.issuer(), &login.nonce, &["family", ROLE]).sign();
    harness.answer_token(&token).await;

    let response = harness.callback(&login, &login.state).await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers()[header::LOCATION], "/");
    let cookie = set_cookie(&response, "team_session").unwrap();
    for flag in [
        "HttpOnly",
        "Secure",
        "SameSite=Lax",
        "Path=/",
        "Max-Age=28800",
    ] {
        assert!(cookie.contains(flag), "{flag} missing in {cookie}");
    }
    let token = cookie_value(&response, "team_session").unwrap();

    let me = harness.get("/api/me", Some(&session_cookie(&token))).await;
    assert_eq!(me.status(), StatusCode::OK);
    assert_eq!(body_text(me).await, r#"{"name":"Quinn Coordinator"}"#);

    // Only the hash of the cookie value is stored.
    let stored: Vec<u8> = sqlx::query_scalar("SELECT token_hash FROM sessions")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored, Sha256::digest(token.as_bytes()).to_vec());
    assert_ne!(stored, token.as_bytes());
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn login_without_the_role_is_refused(pool: PgPool) {
    let harness = Harness::start(pool.clone()).await;
    let login = harness.start_login().await;
    let token = TokenSpec::new(&harness.issuer(), &login.nonce, &["family"]).sign();
    harness.answer_token(&token).await;

    let response = harness.callback(&login, &login.state).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert!(cookie_value(&response, "team_session").is_none());
    assert!(body_text(response).await.contains("No access"));
    assert_eq!(session_count(&pool).await, 0);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn wrong_state_is_refused(pool: PgPool) {
    let harness = Harness::start(pool.clone()).await;
    let login = harness.start_login().await;
    let token = TokenSpec::new(&harness.issuer(), &login.nonce, &[ROLE]).sign();
    harness.answer_token(&token).await;

    let response = harness.callback(&login, "another-state").await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(session_count(&pool).await, 0);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn changed_login_cookie_is_refused(pool: PgPool) {
    let harness = Harness::start(pool.clone()).await;
    let mut login = harness.start_login().await;
    let token = TokenSpec::new(&harness.issuer(), &login.nonce, &[ROLE]).sign();
    harness.answer_token(&token).await;

    let (payload, _) = login.login_cookie.split_once('.').unwrap();
    login.login_cookie = format!("{payload}.AAAA");
    let response = harness.callback(&login, &login.state).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(session_count(&pool).await, 0);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn login_older_than_ten_minutes_is_refused(pool: PgPool) {
    let harness = Harness::start(pool.clone()).await;
    let login = harness.start_login().await;
    let token = TokenSpec::new(&harness.issuer(), &login.nonce, &[ROLE]).sign();
    harness.answer_token(&token).await;

    harness
        .clock
        .advance(TimeDelta::minutes(10) + TimeDelta::seconds(1));
    let response = harness.callback(&login, &login.state).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(session_count(&pool).await, 0);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn wrong_id_tokens_are_refused(pool: PgPool) {
    type Change = fn(&mut TokenSpec);
    let cases: [(&str, Change); 4] = [
        ("wrong nonce", |spec| {
            spec.nonce = "another-nonce".to_owned()
        }),
        ("wrong audience", |spec| {
            spec.audience = "another-client".to_owned();
        }),
        ("wrong issuer", |spec| {
            spec.issuer = "https://evil.example.test".to_owned();
        }),
        ("another signing key", |spec| spec.key = OTHER_KEY),
    ];
    for (case, change) in cases {
        let harness = Harness::start(pool.clone()).await;
        let login = harness.start_login().await;
        let mut spec = TokenSpec::new(&harness.issuer(), &login.nonce, &[ROLE]);
        change(&mut spec);
        harness.answer_token(&spec.sign()).await;

        let response = harness.callback(&login, &login.state).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{case}");
        assert_eq!(session_count(&pool).await, 0, "{case}");
    }
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn unreachable_pocket_id_gives_a_clear_page_and_health_stays_ok(pool: PgPool) {
    let harness = Harness::start_with(
        pool,
        &[("OIDC_ISSUER_URL", "http://127.0.0.1:9/unreachable")],
    )
    .await;
    let response = harness.get("/auth/login", None).await;
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    assert!(
        body_text(response)
            .await
            .contains("Login is not available now")
    );

    for path in ["/api/health/liveness", "/api/health/readiness"] {
        assert_eq!(
            harness.get(path, None).await.status(),
            StatusCode::OK,
            "{path}"
        );
    }
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn expired_session_is_unauthenticated(pool: PgPool) {
    let harness = Harness::start(pool).await;
    let token = harness.log_in(&[ROLE]).await;

    harness
        .clock
        .advance(TimeDelta::seconds(28_800) + TimeDelta::seconds(1));
    let response = harness.get("/api/me", Some(&session_cookie(&token))).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn logout_ends_the_session(pool: PgPool) {
    let harness = Harness::start(pool.clone()).await;
    let token = harness.log_in(&[ROLE]).await;

    let request = Request::post("/auth/logout")
        .header(header::ORIGIN, ORIGIN)
        .header(header::COOKIE, session_cookie(&token))
        .body(Body::empty())
        .unwrap();
    let response = harness.send(request).await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let removal = set_cookie(&response, "team_session").unwrap();
    assert!(
        removal.contains("Max-Age=0"),
        "cookie is cleared: {removal}"
    );
    assert_eq!(session_count(&pool).await, 0);

    let response = harness.get("/api/me", Some(&session_cookie(&token))).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn new_login_deletes_expired_sessions(pool: PgPool) {
    let harness = Harness::start(pool.clone()).await;
    harness.log_in(&[ROLE]).await;
    harness.clock.advance(TimeDelta::seconds(28_801));
    harness.log_in(&[ROLE]).await;
    assert_eq!(session_count(&pool).await, 1);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn changing_requests_need_our_origin(pool: PgPool) {
    let harness = Harness::start(pool).await;
    let token = harness.log_in(&[ROLE]).await;

    for origin in [Some("https://evil.example.test"), None] {
        let mut request =
            Request::post("/auth/logout").header(header::COOKIE, session_cookie(&token));
        if let Some(origin) = origin {
            request = request.header(header::ORIGIN, origin);
        }
        let response = harness.send(request.body(Body::empty()).unwrap()).await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN, "{origin:?}");
    }

    let request = Request::post("/auth/logout")
        .header(header::ORIGIN, ORIGIN)
        .header(header::COOKIE, session_cookie(&token))
        .body(Body::empty())
        .unwrap();
    assert_eq!(harness.send(request).await.status(), StatusCode::SEE_OTHER);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn unknown_api_path_with_login_is_not_found(pool: PgPool) {
    let harness = Harness::start(pool).await;
    let token = harness.log_in(&[ROLE]).await;
    let response = harness
        .get("/api/does-not-exist", Some(&session_cookie(&token)))
        .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn me_carries_the_security_headers(pool: PgPool) {
    let harness = Harness::start(pool).await;
    let token = harness.log_in(&[ROLE]).await;
    let response = harness.get("/api/me", Some(&session_cookie(&token))).await;
    let headers = response.headers();
    assert_eq!(
        headers[header::CONTENT_SECURITY_POLICY],
        "default-src 'self'; frame-ancestors 'none'"
    );
    assert_eq!(headers[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
    assert_eq!(headers[header::REFERRER_POLICY], "same-origin");
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn start_page_shows_login_state(pool: PgPool) {
    let harness = Harness::start(pool).await;
    let page = body_text(harness.get("/", None).await).await;
    assert!(page.contains("href=\"/auth/login\""));

    let token = harness.log_in(&[ROLE]).await;
    let page = body_text(harness.get("/", Some(&session_cookie(&token))).await).await;
    assert!(page.contains("Logged in as Quinn Coordinator"));
    assert!(page.contains("action=\"/auth/logout\""));
}

/// Captures log output written while the guard lives.
#[derive(Clone, Default)]
struct LogBuffer(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

impl std::io::Write for LogBuffer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn logs_contain_no_secrets(pool: PgPool) {
    let buffer = LogBuffer::default();
    let writer = buffer.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_writer(move || writer.clone())
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    let harness = Harness::start(pool).await;
    let login = harness.start_login().await;
    let id_token = TokenSpec::new(&harness.issuer(), &login.nonce, &[ROLE]).sign();
    harness.answer_token(&id_token).await;
    let response = harness.callback(&login, &login.state).await;
    let session = cookie_value(&response, "team_session").unwrap();
    harness
        .get("/api/me", Some(&session_cookie(&session)))
        .await;

    let logs = String::from_utf8(buffer.0.lock().unwrap().clone()).unwrap();
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
