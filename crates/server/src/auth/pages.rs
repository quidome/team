//! Small HTML pages for login results.

use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

fn page(title: &str, body: &str) -> Html<String> {
    Html(format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <title>{title} · Team</title></head><body><main>{body}</main></body></html>"
    ))
}

pub fn login_failed() -> Response {
    (
        StatusCode::BAD_REQUEST,
        page(
            "Login failed",
            "<h1>Login failed</h1><p>Please try again.</p><p><a href=\"/auth/login\">Log in</a></p>",
        ),
    )
        .into_response()
}

pub fn no_access() -> Response {
    (
        StatusCode::FORBIDDEN,
        page(
            "No access",
            "<h1>No access</h1><p>Your account may not use this tool. Ask the coordinator.</p>",
        ),
    )
        .into_response()
}

pub fn login_unavailable() -> Response {
    (
        StatusCode::BAD_GATEWAY,
        page(
            "Login not available",
            "<h1>Login is not available now</h1><p>Please try again later.</p>\
             <p><a href=\"/auth/login\">Log in</a></p>",
        ),
    )
        .into_response()
}
