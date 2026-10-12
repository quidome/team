//! A logged-in harness with Blue Drakes M16 1 chosen, and a fake game list whose answer each test
//! can change between readings (story 3a).

use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::{Request, header};
use chrono::{DateTime, TimeZone, Utc};
use serde_json::{Value, json};
use sqlx::PgPool;
use wiremock::matchers::{method, path};
use wiremock::{Mock, Respond, ResponseTemplate};

use super::{Harness, ORIGIN, body_text};

pub const DRAKES: i64 = 37;
/// Blue Drakes M16 1.
pub const M16_1: i64 = 184;
/// Blue Drakes M16 2.
pub const M16_2: i64 = 1404;

pub fn data(name: &str) -> Value {
    let path = format!("{}/tests/data/game-list/{name}", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// The real 2026-2027 games of Blue Drakes.
pub fn real_games() -> Value {
    data("games-37-2026-2027.json")
}

/// The real games, changed by `change` (on the list of games).
pub fn games_with(change: impl FnOnce(&mut Vec<Value>)) -> Value {
    let mut body = real_games();
    change(body["wedstrijden"].as_array_mut().unwrap());
    body
}

/// The game with source id `id` in a list of games.
pub fn game_mut(games: &mut [Value], id: i64) -> &mut Value {
    games.iter_mut().find(|game| game["id"] == id).unwrap()
}

/// Whether a game of the source is in the schedule of M16 1.
pub fn in_m16_1_schedule(game: &Value) -> bool {
    game["thuis_ploeg_id"] == M16_1
        || game["uit_ploeg_id"] == M16_1
        || game["thuis_club_id"] == DRAKES
}

pub fn utc(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(year, month, day, hour, minute, 0)
        .unwrap()
}

/// The answer of the fake game list, changed by the test.
#[derive(Clone)]
struct Answer(Arc<Mutex<ResponseTemplate>>);

impl Respond for Answer {
    fn respond(&self, _: &wiremock::Request) -> ResponseTemplate {
        self.0.lock().unwrap().clone()
    }
}

pub struct Reader {
    pub harness: Harness,
    pub cookie: String,
    answer: Answer,
}

impl Reader {
    /// A harness at 12 October 2026 (season 2026-2027), logged in, Blue Drakes M16 1 chosen, the
    /// game list answering the real games. Nothing is read yet.
    pub async fn chosen(pool: PgPool) -> Self {
        let harness = Harness::start(pool).await;
        harness.clock.set(utc(2026, 10, 12, 12, 0));
        Mock::given(method("GET"))
            .and(path("/db/json/club.pl"))
            .respond_with(ResponseTemplate::new(200).set_body_json(data("clubs.json")))
            .mount(&harness.source)
            .await;
        let answer = Answer(Arc::new(Mutex::new(
            ResponseTemplate::new(200).set_body_json(real_games()),
        )));
        Mock::given(method("GET"))
            .and(path("/db/json/wedstrijd.pl"))
            .respond_with(answer.clone())
            .mount(&harness.source)
            .await;
        let cookie = format!(
            "team_session={}",
            harness.log_in(&["team_coordinator"]).await
        );
        let reader = Self {
            harness,
            cookie,
            answer,
        };
        reader.choose(M16_1).await;
        reader
    }

    /// A new login, for example after the clock moved past the session's lifetime.
    pub async fn log_in_again(&mut self) {
        self.cookie = format!(
            "team_session={}",
            self.harness.log_in(&["team_coordinator"]).await
        );
    }

    /// The game list answers `body` from now on.
    pub fn answer(&self, body: Value) {
        self.answer_with(ResponseTemplate::new(200).set_body_json(body));
    }

    pub fn answer_with(&self, template: ResponseTemplate) {
        *self.answer.0.lock().unwrap() = template;
    }

    pub async fn choose(&self, team: i64) {
        let (status, body) = self
            .send(
                "PUT",
                "/api/selection",
                Some(json!({ "club": DRAKES, "team": team })),
            )
            .await;
        assert_eq!(status, 200, "{body}");
    }

    /// `POST /api/readings`.
    pub async fn read(&self) -> (u16, Value) {
        self.send("POST", "/api/readings", None).await
    }

    /// `POST /api/readings`, which must give a good reading.
    pub async fn read_good(&self) -> Value {
        let (status, body) = self.read().await;
        assert_eq!((status, &body["result"]), (200, &json!("good")), "{body}");
        body
    }

    pub async fn latest(&self) -> Value {
        let (status, body) = self.send("GET", "/api/readings/latest", None).await;
        assert_eq!(status, 200);
        body
    }

    pub async fn confirm_gone(&self, id: &Value) -> (u16, Value) {
        self.send("POST", &format!("/api/readings/{id}/confirm-gone"), None)
            .await
    }

    pub async fn send(&self, method: &str, path: &str, body: Option<Value>) -> (u16, Value) {
        let request = Request::builder()
            .method(method)
            .uri(path)
            .header(header::COOKIE, &self.cookie)
            .header(header::ORIGIN, ORIGIN)
            .header(header::CONTENT_TYPE, "application/json");
        let body = body.map_or_else(Body::empty, |body| Body::from(body.to_string()));
        let response = self.harness.send(request.body(body).unwrap()).await;
        let status = response.status().as_u16();
        let text = body_text(response).await;
        (status, serde_json::from_str(&text).unwrap_or(Value::Null))
    }

    pub fn pool(&self) -> &PgPool {
        &self.harness.pool
    }
}
