//! Builds the router. Tests call [`app`] directly.

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::Request;
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware;
use axum::routing::{get, post};
use sqlx::PgPool;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use tracing::Span;

use crate::api::{health, me};
use crate::auth;
use crate::auth::oidc::Oidc;
use crate::clock::Clock;
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub oidc: Arc<Oidc>,
    pub clock: Arc<dyn Clock>,
}

/// The log span of a request: method and path only. The query is left out, because the login
/// callback carries the one-time code and state in it.
fn request_span(request: &Request<Body>) -> Span {
    tracing::info_span!("request", method = %request.method(), path = %request.uri().path())
}

pub fn app(state: AppState) -> Router {
    // Everything under /api needs a session by default. New routes go on this router; an unknown
    // /api path also gets 401 without a session, so nothing is open by mistake.
    let protected_api = Router::new()
        .route("/me", get(me::me))
        .fallback(|| async { StatusCode::NOT_FOUND })
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_session,
        ));

    let api = Router::new()
        .route("/health/liveness", get(health::liveness))
        .route("/health/readiness", get(health::readiness))
        .merge(protected_api);

    let auth_routes = Router::new()
        .route("/login", get(auth::routes::login))
        .route("/callback", get(auth::routes::callback))
        .route("/logout", post(auth::routes::logout));

    Router::new()
        .route("/", get(auth::routes::start_page))
        .nest("/api", api)
        .nest("/auth", auth_routes)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_same_origin,
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'self'; frame-ancestors 'none'"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::REFERRER_POLICY,
            HeaderValue::from_static("same-origin"),
        ))
        .layer(TraceLayer::new_for_http().make_span_with(request_span))
        .with_state(state)
}
