//! The message for the next game holds player names; they never go to the log (story C of
//! duties).
//!
//! This test has its own file (its own test process): see `LogBuffer` in `common`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::LogBuffer;
use common::game_list::{M16_1, Reader, utc};
use serde_json::{Value, json};
use sqlx::PgPool;

/// A home game of X12 1 on Saturday 31 October 2026 at 13:00.
const X12_31_OCT: i64 = 1_081_923;

async fn ok(reader: &Reader, method: &str, path: &str, body: Option<Value>) -> Value {
    let (status, answer) = reader.send(method, path, body).await;
    assert_eq!(status, 200, "{method} {path}: {answer}");
    answer
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_log_never_has_a_name_of_the_message(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    reader.harness.clock.set(utc(2026, 10, 26, 10, 0));
    reader.log_in_again().await;
    let game = format!("/api/games/2026-2027/{X12_31_OCT}");
    ok(
        &reader,
        "PUT",
        &format!("{game}/places/table/1"),
        Some(json!({ "team": M16_1 })),
    )
    .await;
    let player = ok(
        &reader,
        "POST",
        "/api/players",
        Some(json!({ "first_name": "Zephyrine", "last_name": "Quistorp",
                     "role": { "role": "plays_games", "from": "2026-09-01" } })),
    )
    .await["player"]["id"]
        .as_i64()
        .unwrap();
    ok(
        &reader,
        "PUT",
        &format!("{game}/assignments/table/1"),
        Some(json!({ "player": player })),
    )
    .await;

    let (log, _guard) = LogBuffer::capture();
    let message = ok(&reader, "GET", "/api/messages/next-game", None).await;
    ok(&reader, "GET", "/api/start", None).await;
    let text = message["text"].as_str().unwrap();
    assert!(text.contains("Tafel: Zephyrine"), "{text}");
    let log = log.text();
    assert!(
        log.contains("/api/messages/next-game"),
        "the request is logged: {log}"
    );
    assert!(!log.contains("Zephyrine"), "a name in the log: {log}");
}
