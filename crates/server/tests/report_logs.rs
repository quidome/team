//! The report holds player names; they never go to the log (decision 0033).
//!
//! This test has its own file (its own test process): see `LogBuffer` in `common`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::LogBuffer;
use common::game_list::{M16_1, Reader};
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
async fn the_log_never_has_a_name_of_the_report(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
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
    let report = ok(&reader, "GET", "/api/report", None).await;
    let text = report["copies"]["table"].as_str().unwrap();
    assert!(text.contains("Zephyrine 0 (+1 gepland)"), "{text}");
    assert_eq!(report["team"][0]["name"], "Zephyrine");
    let log = log.text();
    assert!(log.contains("/api/report"), "the request is logged: {log}");
    assert!(!log.contains("Zephyrine"), "a name in the log: {log}");
    assert!(!log.contains("Quistorp"), "a name in the log: {log}");
}
