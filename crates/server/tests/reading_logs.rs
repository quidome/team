//! A gone and a new game with the same game number write a warning to the log (story 3a).
//!
//! This test has its own file (its own test process): see `LogBuffer` in `common`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::LogBuffer;
use common::game_list::{Reader, game_mut, games_with};
use serde_json::json;
use sqlx::PgPool;

/// A team game of M16 1.
const TEAM_GAME: i64 = 1073178;

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_gone_and_a_new_game_with_the_same_number_are_logged(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;

    let (buffer, _guard) = LogBuffer::capture();
    reader.answer(games_with(|games| {
        game_mut(games, TEAM_GAME)["id"] = json!(9_999_002);
    }));
    let body = reader.read_good().await;
    assert_eq!(body["counts"]["gone"], 1);
    assert_eq!(body["counts"]["new"], 1);

    let logs = buffer.text();
    let line = logs
        .lines()
        .find(|line| line.contains("same game number"))
        .unwrap_or_else(|| panic!("no warning in the log:\n{logs}"));
    assert!(line.contains("WARN"), "{line}");
    assert!(line.contains(&format!("gone_id={TEAM_GAME}")), "{line}");
    assert!(line.contains("new_id=9999002"), "{line}");
    assert!(line.contains("game_number=\"499386\""), "{line}");
}
