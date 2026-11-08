//! "Checked" and "Undo check" answer the page of a game, which holds player names; the names never
//! go to the log (story C of attendance at games).
//!
//! This test has its own file (its own test process): see `LogBuffer` in `common`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::LogBuffer;
use common::game_list::Reader;
use serde_json::{Value, json};
use sqlx::PgPool;

/// The away game of M16 1 on Saturday 10 October 2026 at 15:00, before the test clock.
const AWAY_10_OCT: i64 = 1_073_147;

async fn ok(reader: &Reader, method: &str, path: &str, body: Option<Value>) -> Value {
    let (status, answer) = reader.send(method, path, body).await;
    assert_eq!(status, 200, "{method} {path}: {answer}");
    answer
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_log_of_checked_never_has_a_player_name(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    ok(
        &reader,
        "POST",
        "/api/players",
        Some(json!({ "first_name": "Zephyrine", "last_name": "Quistorp",
                     "role": { "role": "plays_games", "from": "2026-09-01" } })),
    )
    .await;
    let path = format!("/api/games/2026-2027/{AWAY_10_OCT}/checked");

    let (log, _guard) = LogBuffer::capture();
    let checked = ok(&reader, "PUT", &path, None).await;
    assert_eq!(checked["settled"], 1);
    assert!(
        checked.to_string().contains("Zephyrine"),
        "the page has the name"
    );
    ok(&reader, "DELETE", &path, None).await;
    let log = log.text();
    assert!(log.contains("game checked"), "the check is logged: {log}");
    assert!(!log.contains("Zephyrine"), "a name in the log: {log}");
    assert!(!log.contains("Quistorp"), "a name in the log: {log}");
}
