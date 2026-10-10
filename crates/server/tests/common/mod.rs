//! Shared helpers for the integration tests.

#![allow(dead_code)]

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, Response};
use http_body_util::BodyExt;
use sqlx::PgPool;
use team::config::Config;
use team::{AppState, app};
use tower::ServiceExt;

pub fn test_config() -> Config {
    Config::from_lookup(|name| {
        let value = match name {
            "DATABASE_URL" => "postgresql://unused-in-tests",
            "OIDC_ISSUER_URL" => "https://id.example.test",
            "OIDC_CLIENT_ID" => "team-coordinator",
            "OIDC_CLIENT_SECRET" => "client-secret",
            "SESSION_SECRET" => "0123456789abcdef0123456789abcdef",
            "ORIGIN" => "https://team.example.test",
            _ => return None,
        };
        Some(value.to_owned())
    })
    .unwrap_or_else(|error| panic!("test settings are valid: {error}"))
}

pub fn test_app(pool: PgPool) -> Router {
    app(AppState {
        pool,
        config: Arc::new(test_config()),
    })
}

pub async fn get(router: &Router, path: &str) -> Response<Body> {
    let request = Request::get(path)
        .body(Body::empty())
        .unwrap_or_else(|error| panic!("valid request: {error}"));
    router
        .clone()
        .oneshot(request)
        .await
        .unwrap_or_else(|error| panic!("router answers: {error}"))
}

pub async fn body_text(response: Response<Body>) -> String {
    let bytes = response
        .into_body()
        .collect()
        .await
        .unwrap_or_else(|error| panic!("body is readable: {error}"))
        .to_bytes();
    String::from_utf8_lossy(&bytes).into_owned()
}
