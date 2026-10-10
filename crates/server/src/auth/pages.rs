//! Small HTML pages for login results and, until the front end exists, the start page.

use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

fn page(title: &str, body: &str) -> Html<String> {
    Html(format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <title>{title} · Team</title></head><body><main>{body}</main></body></html>"
    ))
}

/// Escapes text for HTML (the name comes from the login server).
pub fn escape(text: &str) -> String {
    text.chars()
        .map(|character| match character {
            '&' => "&amp;".to_owned(),
            '<' => "&lt;".to_owned(),
            '>' => "&gt;".to_owned(),
            '"' => "&quot;".to_owned(),
            '\'' => "&#39;".to_owned(),
            other => other.to_string(),
        })
        .collect()
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

/// The start page of step 2 of the base; the Svelte front end replaces it.
pub fn start(name: Option<&str>) -> Response {
    let body = match name {
        Some(name) => format!(
            "<h1>Team</h1><p>Logged in as {}</p>\
             <form method=\"post\" action=\"/auth/logout\"><button type=\"submit\">Log out</button></form>",
            escape(name)
        ),
        None => "<h1>Team</h1><p><a href=\"/auth/login\">Log in</a></p>".to_owned(),
    };
    page("Team", &body).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_html() {
        assert_eq!(
            escape("<script>alert('x') & \"y\"</script>"),
            "&lt;script&gt;alert(&#39;x&#39;) &amp; &quot;y&quot;&lt;/script&gt;"
        );
    }
}
