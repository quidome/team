//! Builds the router. Tests call [`app`] directly.

use std::sync::Arc;

use axum::Router;
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware;
use axum::routing::get;
use sqlx::PgPool;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;

use crate::api::health;
use crate::auth;
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
}

pub fn app(state: AppState) -> Router {
    // Everything under /api needs a session by default. New routes go on this router; an unknown
    // /api path also gets 401 without a session, so nothing is open by mistake.
    let protected_api = Router::new()
        .fallback(|| async { StatusCode::NOT_FOUND })
        .layer(middleware::from_fn(auth::require_session));

    let api = Router::new()
        .route("/health/liveness", get(health::liveness))
        .route("/health/readiness", get(health::readiness))
        .merge(protected_api);

    Router::new()
        .nest("/api", api)
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
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
