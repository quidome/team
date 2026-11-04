//! The open items of the start page hold the names of drivers who are not coming; the names never
//! go to the log (story B of attendance at games).
//!
//! This test has its own file (its own test process): see `LogBuffer` in `common`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::LogBuffer;
use common::game_list::{Reader, utc};
use serde_json::{Value, json};
use sqlx::PgPool;

/// The away game of M16 1 on Saturday 7 November 2026 at 11:15.
const AWAY_7_NOV: i64 = 1_073_159;

async fn ok(reader: &Reader, method: &str, path: &str, body: Option<Value>) -> Value {
    let (status, answer) = reader.send(method, path, body).await;
    assert_eq!(status, 200, "{method} {path}: {answer}");
    answer
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_log_never_has_the_name_of_a_driver_who_is_not_coming(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let game = format!("/api/games/2026-2027/{AWAY_7_NOV}");
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
        &format!("{game}/drivers"),
        Some(json!({ "drivers": 1 })),
    )
    .await;
    ok(
        &reader,
        "PUT",
        &format!("{game}/assignments/drive/1"),
        Some(json!({ "player": player })),
    )
    .await;
    reader.harness.clock.set(utc(2026, 11, 5, 9, 0));
    reader.log_in_again().await;

    let (log, _guard) = LogBuffer::capture();
    ok(
        &reader,
        "PUT",
        &format!("{game}/answers/{player}"),
        Some(json!({ "answer": "not_coming" })),
    )
    .await;
    let start = ok(&reader, "GET", "/api/start", None).await;
    let items = start["open_items"].as_array().unwrap();
    let driver = items
        .iter()
        .find(|item| item["kind"] == "driver_not_coming")
        .unwrap();
    assert_eq!(driver["games"][0]["drivers"], json!(["Zephyrine"]));
    let log = log.text();
    assert!(log.contains("/api/start"), "the request is logged: {log}");
    assert!(!log.contains("Zephyrine"), "a name in the log: {log}");
    assert!(!log.contains("Quistorp"), "a name in the log: {log}");
}
