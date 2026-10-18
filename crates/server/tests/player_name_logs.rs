//! A stored name that cannot be decrypted gives a server error; the log names the player's id and
//! never a name (change 2026-10-18-encrypted-player-names).
//!
//! This test has its own file (its own test process): see `LogBuffer` in `common`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::LogBuffer;
use common::api::Coordinator;
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_changed_name_gives_a_server_error_and_no_name_in_the_log(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    let answer = coordinator
        .ok(
            "POST",
            "/api/players",
            Some(json!({ "first_name": "Zephyrine", "last_name": "Quistorp" })),
        )
        .await;
    let id = answer["player"]["id"].as_i64().unwrap();
    // One bit of the encrypted first name changes (after the version byte and the nonce).
    sqlx::query(
        "UPDATE players SET first_name_enc = set_byte(first_name_enc, 13, get_byte(first_name_enc, 13) # 1)
         WHERE id = $1",
    )
    .bind(id)
    .execute(&pool)
    .await
    .unwrap();

    let (log, _guard) = LogBuffer::capture();
    let (status, _) = coordinator.send("GET", "/api/players", None).await;
    assert_eq!(status, 500);

    let text = log.text();
    assert!(
        text.contains(&format!("the name of player {id} cannot be read")),
        "{text}"
    );
    for name in ["Zephyrine", "Quistorp"] {
        assert!(!text.contains(name), "a name in the log: {text}");
    }
}
