//! The message of the week through the API (story C of duties), and the button on the start page
//! that follows the same rule.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::Harness;
use common::api::Coordinator;
use common::game_list::{M16_1, Reader, utc};
use serde_json::{Value, json};
use sqlx::PgPool;

/// A home game of X12 1 on Saturday 31 October 2026 at 13:00.
const X12_31_OCT: i64 = 1_081_923;
/// The away game of M16 1 on Saturday 7 November 2026 at 11:15.
const AWAY_7_NOV: i64 = 1_073_159;

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
    let (status, message) = send("/api/messages/week").await;
    assert_eq!(status, 200, "{message}");
    let (status, start) = send("/api/start").await;
    assert_eq!(status, 200, "{start}");
    assert_eq!(
        start["messages"]["week"],
        !message["text"].is_null(),
        "the button and the message agree: {message}"
    );
    message["text"].clone()
}

async fn add_player(reader: &Reader, first: &str) -> i64 {
    ok(
        reader,
        "POST",
        "/api/players",
        Some(json!({ "first_name": first, "last_name": null,
                     "role": { "role": "plays_games", "from": "2026-09-01" } })),
    )
    .await["player"]["id"]
        .as_i64()
        .unwrap()
}

async fn message(reader: &Reader) -> Value {
    message_of(|path| reader.send("GET", path, None)).await
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_message_needs_a_login(pool: PgPool) {
    let harness = Harness::start(pool).await;
    let response = harness.get("/api/messages/week", None).await;
    assert_eq!(response.status(), 401);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn without_a_team_there_is_no_message(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    let text = message_of(|path| coordinator.send("GET", path, None)).await;
    assert_eq!(text, Value::Null);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_message_has_the_game_and_the_places_of_this_week(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    // Monday 26 October 2026: this week M16 1 plays at home on Saturday 31 October.
    reader.harness.clock.set(utc(2026, 10, 26, 10, 0));
    reader.log_in_again().await;
    assert_eq!(
        message(&reader).await,
        "Week 26 oktober – 1 november\n\
         \n\
         Wedstrijd: zaterdag 31 oktober, 16:00\n\
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
    let player = add_player(&reader, "Anna").await;
    ok(
        &reader,
        "PUT",
        &format!("{game}/assignments/table/1"),
        Some(json!({ "player": player })),
    )
    .await;
    assert_eq!(
        message(&reader).await,
        "Week 26 oktober – 1 november\n\
         \n\
         Wedstrijd: zaterdag 31 oktober, 16:00\n\
         Thuis tegen BC Utrecht Cangeroes M16 2\n\
         Bulwijk, Woerden\n\
         \n\
         Tafel en scheidsrechter:\n\
         za 31 okt 13:00 Blue Drakes X12 1 – De Dunckers X12 1\n  \
         Tafel: Anna"
    );

    // Sunday 1 November: the game has left, the place stays, and a correction shows.
    let bram = add_player(&reader, "Bram").await;
    reader.harness.clock.set(utc(2026, 11, 1, 20, 0));
    reader.log_in_again().await;
    ok(
        &reader,
        "PUT",
        &format!("{game}/assignments/table/1/outcome"),
        Some(json!({ "outcome": "done_by", "player": bram })),
    )
    .await;
    assert_eq!(
        message(&reader).await,
        "Week 26 oktober – 1 november\n\
         \n\
         Tafel en scheidsrechter:\n\
         za 31 okt 13:00 Blue Drakes X12 1 – De Dunckers X12 1\n  \
         Tafel: Bram"
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_week_with_nothing_to_announce_has_no_message(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    // Sunday 1 November: the game of M16 1 was played, and the team has no places.
    reader.harness.clock.set(utc(2026, 11, 1, 20, 0));
    reader.log_in_again().await;
    assert_eq!(message(&reader).await, Value::Null);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn an_away_game_has_its_drivers(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    // Monday 2 November 2026: this week M16 1 plays away on Saturday 7 November.
    reader.harness.clock.set(utc(2026, 11, 2, 10, 0));
    reader.log_in_again().await;
    let game = format!("/api/games/2026-2027/{AWAY_7_NOV}");
    ok(
        &reader,
        "PUT",
        &format!("{game}/drivers"),
        Some(json!({ "drivers": 2 })),
    )
    .await;
    let player = add_player(&reader, "Bram").await;
    ok(
        &reader,
        "PUT",
        &format!("{game}/assignments/drive/1"),
        Some(json!({ "player": player })),
    )
    .await;
    assert_eq!(
        message(&reader).await,
        "Week 2 november – 8 november\n\
         \n\
         Wedstrijd: zaterdag 7 november, 11:15\n\
         Uit tegen De Dunckers M16 3\n\
         Dudok Arena, Hilversum\n\
         Rijden: Bram, nog 1 open"
    );
}
