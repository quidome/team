//! The message for the next game through the API (story C of duties), and the link on the start
//! page that follows the same rule.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::Harness;
use common::api::Coordinator;
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

/// The message and the link of the start page, which must agree. `send` is the request of a
/// logged-in test client.
async fn message_of<F, R>(send: F) -> Value
where
    F: Fn(&'static str) -> R,
    R: std::future::Future<Output = (u16, Value)>,
{
    let (status, message) = send("/api/messages/next-game").await;
    assert_eq!(status, 200, "{message}");
    let (status, start) = send("/api/start").await;
    assert_eq!(status, 200, "{start}");
    assert_eq!(
        start["messages"]["next_game"],
        !message["text"].is_null(),
        "the link and the message agree: {message}"
    );
    message["text"].clone()
}

async fn message(reader: &Reader) -> Value {
    message_of(|path| reader.send("GET", path, None)).await
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_message_needs_a_login(pool: PgPool) {
    let harness = Harness::start(pool).await;
    let response = harness.get("/api/messages/next-game", None).await;
    assert_eq!(response.status(), 401);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn without_a_team_there_is_no_message(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    let text = message_of(|path| coordinator.send("GET", path, None)).await;
    assert_eq!(text, Value::Null);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_message_has_the_next_game_and_the_places_of_this_week(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    // Monday 26 October 2026: the next game of M16 1 is at home on Saturday 31 October.
    reader.harness.clock.set(utc(2026, 10, 26, 10, 0));
    reader.log_in_again().await;
    assert_eq!(
        message(&reader).await,
        "Volgende wedstrijd: zaterdag 31 oktober, 16:00\n\
         Thuis tegen BC Utrecht Cangeroes M16 2\n\
         Bulwijk, Woerden"
    );

    // A table place of the team this week, with a player.
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
        Some(json!({ "first_name": "Anna", "last_name": null,
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
    assert_eq!(
        message(&reader).await,
        "Volgende wedstrijd: zaterdag 31 oktober, 16:00\n\
         Thuis tegen BC Utrecht Cangeroes M16 2\n\
         Bulwijk, Woerden\n\
         \n\
         Tafel en scheidsrechter deze week:\n\
         za 31 okt 13:00 Blue Drakes X12 1 – De Dunckers X12 1\n  \
         Tafel: Anna"
    );
}
