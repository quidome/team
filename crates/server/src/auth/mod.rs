//! Login, sessions and the request rules (decision 0007).

pub mod login_cookie;
pub mod oidc;
pub mod pages;
pub mod routes;
pub mod sessions;

use axum::extract::{FromRequestParts, Request, State};
use axum::http::request::Parts;
use axum::http::{Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::CookieJar;

use crate::app::AppState;
use crate::error::AppError;
use sessions::SessionUser;

/// The logged-in user of this request. Only available behind [`require_session`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentUser(pub SessionUser);

impl<S: Send + Sync> FromRequestParts<S> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<CurrentUser>()
            .cloned()
            .ok_or(AppError::Unauthenticated)
    }
}

/// Finds the user of the session cookie, if the session is valid.
pub async fn session_user(
    state: &AppState,
    jar: &CookieJar,
) -> Result<Option<SessionUser>, AppError> {
    #[cfg(debug_assertions)]
    if state.config.dev_auth_bypass {
        return Ok(Some(dev_user()));
    }
    let Some(cookie) = jar.get(sessions::COOKIE_NAME) else {
        return Ok(None);
    };
    sessions::find_valid(&state.pool, cookie.value(), state.clock.now())
        .await
        .map_err(|error| AppError::Internal(error.into()))
}

/// The fixed user for `DEV_AUTH_BYPASS`. Only compiled in debug builds (decision 0007).
#[cfg(debug_assertions)]
fn dev_user() -> SessionUser {
    SessionUser {
        subject: "dev".to_owned(),
        name: "Developer".to_owned(),
    }
}

/// Middleware on the `/api` router: a request without a valid session gets 401.
pub async fn require_session(
    State(state): State<AppState>,
    jar: CookieJar,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let user = session_user(&state, &jar)
        .await?
        .ok_or(AppError::Unauthenticated)?;
    request.extensions_mut().insert(CurrentUser(user));
    Ok(next.run(request).await)
}

/// True for `prefix` itself and every path below it (`/api`, `/api/`, `/api/me`; not `/apis`).
pub fn is_under(path: &str, prefix: &str) -> bool {
    path.strip_prefix(prefix)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
}

/// Every request that changes something under `/api` or `/auth` must come from our own origin.
pub async fn require_same_origin(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let changes_data = !matches!(
        *request.method(),
        Method::GET | Method::HEAD | Method::OPTIONS
    );
    let path = request.uri().path();
    let guarded = is_under(path, "/api") || is_under(path, "/auth");
    if changes_data && guarded {
        let expected = state.config.origin.origin().ascii_serialization();
        let origin = request
            .headers()
            .get(header::ORIGIN)
            .and_then(|value| value.to_str().ok());
        if origin != Some(expected.as_str()) {
            return (StatusCode::FORBIDDEN, "Forbidden").into_response();
        }
    }
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::is_under;

    #[test]
    fn is_under_matches_the_prefix_and_paths_below_it() {
        for path in ["/api", "/api/", "/api/me", "/api/health/liveness"] {
            assert!(is_under(path, "/api"), "{path}");
        }
        for path in ["/apis", "/api-docs", "/", "/sample", "/auth"] {
            assert!(!is_under(path, "/api"), "{path}");
        }
    }
}
