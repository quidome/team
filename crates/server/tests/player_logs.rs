//! Player names are personal data of children: they never go to the log (attendance at training,
//! story A).
//!
//! This test has its own file (its own test process): see `LogBuffer` in `common`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::LogBuffer;
use common::api::Coordinator;
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_log_never_has_a_player_name(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    let (log, _guard) = LogBuffer::capture();

    let answer = coordinator
        .ok(
            "POST",
            "/api/players",
            Some(json!({ "first_name": "Zephyrine", "last_name": "Quistorp",
                         "role": { "role": "plays_games", "from": "2026-09-01" } })),
        )
        .await;
    let id = answer["player"]["id"].as_i64().unwrap();
    coordinator
        .ok(
            "PUT",
            &format!("/api/players/{id}"),
            Some(json!({ "first_name": "Zephyrina", "last_name": "Quistorpe" })),
        )
        .await;
    coordinator
        .ok(
            "POST",
            &format!("/api/players/{id}/role-change"),
            Some(json!({ "role": "only_trains", "from": "2026-11-01" })),
        )
        .await;
    coordinator.ok("GET", "/api/players", None).await;
    coordinator.ok("GET", "/api/team", None).await;

    let text = log.text();
    assert!(text.contains("player added"), "{text}");
    assert!(text.contains("role changed"), "{text}");
    for name in ["Zephyrin", "Quistorp"] {
        assert!(!text.contains(name), "a name in the log: {text}");
    }
}
