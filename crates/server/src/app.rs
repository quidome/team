//! Builds the router. Tests call [`app`] directly.

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum_extra::extract::CookieJar;
use sqlx::PgPool;
use tower::ServiceExt;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use tracing::Span;

use crate::api::{
    changes, clubs, health, me, players, readings, reasons, schedule, selection, team,
    weekly_trainings,
};
use crate::auth;
use crate::auth::oidc::Oidc;
use crate::clock::Clock;
use crate::config::Config;
use crate::error::AppError;
use crate::source::Source;
use crate::web::Web;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub oidc: Arc<Oidc>,
    pub clock: Arc<dyn Clock>,
    pub web: Arc<Web>,
    pub source: Arc<Source>,
}

/// The log span of a request: method and path only. The query is left out, because the login
/// callback carries the one-time code and state in it.
fn request_span(request: &Request<Body>) -> Span {
    tracing::info_span!("request", method = %request.method(), path = %request.uri().path())
}

/// Every path that no route matched:
/// - `/api` and below: 401 without a session, else 404 (never the front end);
/// - `/auth` and below: 404;
/// - built files (under `/_app/`, or a name with an extension): the file, or 404;
/// - any other path: the file if it exists, else `index.html` (the app routes it).
async fn fallback(State(state): State<AppState>, jar: CookieJar, request: Request) -> Response {
    let path = request.uri().path().to_owned();
    if auth::is_under(&path, "/api") {
        return match auth::session_user(&state, &jar).await {
            Ok(Some(_)) => StatusCode::NOT_FOUND.into_response(),
            Ok(None) => AppError::Unauthenticated.into_response(),
            Err(error) => error.into_response(),
        };
    }
    if auth::is_under(&path, "/auth") {
        return StatusCode::NOT_FOUND.into_response();
    }
    let last_segment = path.rsplit('/').next().unwrap_or_default();
    let is_built_file = path.starts_with("/_app/") || last_segment.contains('.');
    let files = ServeDir::new(&state.web.dir);
    let served = if is_built_file {
        files.oneshot(request).await
    } else {
        files
            .fallback(ServeFile::new(state.web.index()))
            .oneshot(request)
            .await
    };
    match served {
        Ok(response) => with_cache_rule(&path, response.map(Body::new)),
        Err(error) => AppError::Internal(error.into()).into_response(),
    }
}

/// Files under `/_app/immutable/` have a hash in their name, so a new release has new names: the
/// browser may keep them for a year. Every other front-end file (`index.html` first) must be
/// checked with the server each time, so a new release shows at the next page load.
fn with_cache_rule(path: &str, mut response: Response) -> Response {
    let rule = if path.starts_with("/_app/immutable/") && response.status().is_success() {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static(rule));
    response
}

pub fn app(state: AppState) -> Router {
    // The policy carries the hash of the front end's start script; it is fixed at start.
    let content_security_policy = HeaderValue::from_str(&state.web.content_security_policy)
        .unwrap_or_else(|_| HeaderValue::from_static("default-src 'self'; frame-ancestors 'none'"));

    // Everything under /api needs a session by default. New routes go on this router; an unknown
    // /api path also gets 401 without a session, so nothing is open by mistake.
    let protected_api = Router::new()
        .route("/me", get(me::me))
        .route("/selection", get(selection::get).put(selection::save))
        .route("/clubs", get(clubs::search))
        .route("/clubs/{number}/teams", get(clubs::teams))
        .route("/readings", post(readings::read))
        .route("/readings/latest", get(readings::latest))
        .route("/readings/{id}/confirm-gone", post(readings::confirm_gone))
        .route("/schedule", get(schedule::get))
        .route("/changes", get(changes::get))
        .route("/changes/seen", post(changes::seen))
        .route("/players", get(players::list).post(players::add))
        .route(
            "/players/{id}",
            get(players::get)
                .put(players::change)
                .delete(players::delete),
        )
        .route("/players/carry", post(players::carry))
        .route("/players/{id}/roles", post(players::add_role))
        .route("/players/{id}/roles/{role}", put(players::correct_role))
        .route("/players/{id}/role-change", post(team::role_change))
        .route("/players/{id}/team-leave", post(team::team_leave))
        .route("/team", get(team::get))
        .route("/reasons", get(reasons::list).post(reasons::add))
        .route("/reasons/{id}", put(reasons::change))
        .route(
            "/weekly-trainings",
            get(weekly_trainings::list).post(weekly_trainings::add),
        )
        .route(
            "/weekly-trainings/proposal",
            get(weekly_trainings::proposal),
        )
        .route(
            "/weekly-trainings/{id}",
            put(weekly_trainings::change).delete(weekly_trainings::delete),
        )
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
        .nest("/api", api)
        .nest("/auth", auth_routes)
        .fallback(fallback)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_same_origin,
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::CONTENT_SECURITY_POLICY,
            content_security_policy,
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
