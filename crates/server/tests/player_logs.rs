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
            Some(json!({ "first_name": "Zephyrina", "last_name": "Quistorpe", "hidden": true })),
        )
        .await;
    coordinator
        .ok(
            "POST",
            &format!("/api/players/{id}/role-change"),
            Some(json!({ "role": "only_trains", "from": "2026-11-01" })),
        )
        .await;
    // "Remove from the team" from today (it also removes the role from Sun 1 Nov), then "Undo".
    let roles = coordinator
        .ok("GET", &format!("/api/players/{id}"), None)
        .await["history"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|role| role["from"] == "2026-11-01")
        .map(|role| role["id"].as_i64().unwrap())
        .collect::<Vec<_>>();
    let left = coordinator
        .ok(
            "POST",
            &format!("/api/players/{id}/team-leave"),
            Some(json!({ "from": "2026-10-12", "removes": roles })),
        )
        .await;
    coordinator
        .ok(
            "POST",
            &format!("/api/players/{id}/team-undo"),
            Some(left["undo"].clone()),
        )
        .await;
    coordinator.ok("GET", "/api/players", None).await;
    coordinator
        .ok("GET", &format!("/api/players/{id}"), None)
        .await;
    coordinator.ok("GET", "/api/team", None).await;

    let text = log.text();
    assert!(text.contains("player added"), "{text}");
    assert!(text.contains("role changed"), "{text}");
    assert!(text.contains("player left the team"), "{text}");
    assert!(text.contains("removal undone"), "{text}");
    for name in ["Zephyrin", "Quistorp"] {
        assert!(!text.contains(name), "a name in the log: {text}");
    }
}
