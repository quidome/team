//! `/auth/login`, `/auth/callback`, `/auth/logout`, and the start page of step 2.

use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::TimeDelta;
use serde::Deserialize;
use team_core::may_access;

use super::login_cookie::{self, LoginAttempt};
use super::oidc::OidcError;
use super::sessions::{self, SessionUser};
use super::{pages, session_user};
use crate::app::AppState;
use crate::error::AppError;

fn login_cookie(value: String) -> Cookie<'static> {
    Cookie::build((login_cookie::NAME, value))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .path("/auth")
        .max_age(time_duration(login_cookie::MAX_AGE))
        .build()
}

fn removal(name: &'static str, path: &'static str) -> Cookie<'static> {
    Cookie::build((name, ""))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .path(path)
        .build()
}

fn time_duration(delta: TimeDelta) -> time::Duration {
    time::Duration::seconds(delta.num_seconds())
}

pub async fn login(State(state): State<AppState>, jar: CookieJar) -> Response {
    match state.oidc.start().await {
        Ok(started) => {
            let attempt = LoginAttempt {
                state: started.state,
                nonce: started.nonce,
                pkce_verifier: started.pkce_verifier,
                created_at: state.clock.now().timestamp(),
            };
            let value = login_cookie::encode(&attempt, &state.config.session_secret);
            (
                jar.add(login_cookie(value)),
                Redirect::to(started.authorize_url.as_str()),
            )
                .into_response()
        }
        Err(error) => {
            tracing::warn!(%error, "cannot start a login");
            pages::login_unavailable()
        }
    }
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

pub async fn callback(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<CallbackQuery>,
) -> Result<Response, AppError> {
    let now = state.clock.now();
    let jar_without_login = jar.clone().remove(removal(login_cookie::NAME, "/auth"));
    let failed = |reason: &str| {
        tracing::info!(reason, "login failed");
        Ok((jar_without_login.clone(), pages::login_failed()).into_response())
    };

    if let Some(error) = query.error.as_deref() {
        return failed(&format!("login server answered with an error: {error}"));
    }
    let Some(cookie) = jar.get(login_cookie::NAME) else {
        return failed("no login cookie");
    };
    let attempt = match login_cookie::decode(cookie.value(), &state.config.session_secret, now) {
        Ok(attempt) => attempt,
        Err(error) => return failed(&error.to_string()),
    };
    if query.state.as_deref() != Some(attempt.state.as_str()) {
        return failed("state does not match");
    }
    let Some(code) = query.code else {
        return failed("no code");
    };

    let identity = match state
        .oidc
        .finish(code, attempt.nonce, attempt.pkce_verifier)
        .await
    {
        Ok(identity) => identity,
        Err(OidcError::Unavailable(reason)) => {
            tracing::warn!(reason, "login server is not available");
            return Ok((jar_without_login, pages::login_unavailable()).into_response());
        }
        Err(OidcError::Invalid(reason)) => return failed(&reason),
    };

    if !may_access(&identity, &state.config.access_role) {
        tracing::info!(subject = identity.subject, "login refused: no access role");
        return Ok((jar_without_login, pages::no_access()).into_response());
    }

    let token = sessions::new_token();
    let ttl = TimeDelta::from_std(state.config.session_ttl)
        .map_err(|error| AppError::Internal(error.into()))?;
    let user = SessionUser {
        subject: identity.subject,
        name: identity.name,
    };
    sessions::create(&state.pool, &token, &user, now, now + ttl)
        .await
        .map_err(|error| AppError::Internal(error.into()))?;
    tracing::info!(subject = user.subject, "logged in");

    let session_cookie = Cookie::build((sessions::COOKIE_NAME, token))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(time_duration(ttl))
        .build();
    Ok((jar_without_login.add(session_cookie), Redirect::to("/")).into_response())
}

pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> Result<Response, AppError> {
    if let Some(cookie) = jar.get(sessions::COOKIE_NAME) {
        sessions::delete(&state.pool, cookie.value())
            .await
            .map_err(|error| AppError::Internal(error.into()))?;
    }
    Ok((
        jar.remove(removal(sessions::COOKIE_NAME, "/")),
        Redirect::to("/"),
    )
        .into_response())
}

/// The start page until the Svelte front end exists (step 3 of the base).
pub async fn start_page(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Response, AppError> {
    let user = session_user(&state, &jar).await?;
    Ok(pages::start(user.as_ref().map(|user| user.name.as_str())))
}
