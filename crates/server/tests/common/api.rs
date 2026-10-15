//! A logged-in coordinator that sends JSON requests to the API (attendance at training).

use axum::body::Body;
use axum::http::{Request, header};
use serde_json::Value;
use sqlx::PgPool;

use super::game_list::utc;
use super::{Harness, ORIGIN, body_text};

pub struct Coordinator {
    pub harness: Harness,
    pub cookie: String,
}

impl Coordinator {
    /// A harness at 12 October 2026 12:00 UTC (season 2026-2027), logged in.
    pub async fn start(pool: PgPool) -> Self {
        let harness = Harness::start(pool).await;
        harness.clock.set(utc(2026, 10, 12, 12, 0));
        let cookie = format!(
            "team_session={}",
            harness.log_in(&["team_coordinator"]).await
        );
        Self { harness, cookie }
    }

    /// A new login, for example after the clock moved past the session's lifetime.
    pub async fn log_in_again(&mut self) {
        self.cookie = format!(
            "team_session={}",
            self.harness.log_in(&["team_coordinator"]).await
        );
    }

    pub async fn send(&self, method: &str, path: &str, body: Option<Value>) -> (u16, Value) {
        self.send_with(method, path, body, Some(&self.cookie), true)
            .await
    }

    pub async fn send_with(
        &self,
        method: &str,
        path: &str,
        body: Option<Value>,
        cookie: Option<&str>,
        origin: bool,
    ) -> (u16, Value) {
        let mut request = Request::builder()
            .method(method)
            .uri(path)
            .header(header::CONTENT_TYPE, "application/json");
        if let Some(cookie) = cookie {
            request = request.header(header::COOKIE, cookie);
        }
        if origin {
            request = request.header(header::ORIGIN, ORIGIN);
        }
        let body = body.map_or_else(Body::empty, |body| Body::from(body.to_string()));
        let response = self.harness.send(request.body(body).unwrap()).await;
        let status = response.status().as_u16();
        let text = body_text(response).await;
        (status, serde_json::from_str(&text).unwrap_or(Value::Null))
    }

    /// `send`, which must answer 200.
    pub async fn ok(&self, method: &str, path: &str, body: Option<Value>) -> Value {
        let (status, answer) = self.send(method, path, body).await;
        assert_eq!(status, 200, "{method} {path}: {answer}");
        answer
    }
}
