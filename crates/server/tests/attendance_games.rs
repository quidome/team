//! The games in the totals of "Attendance" (story D of attendance at games): only checked games of
//! the team that were played count, for a player on the list with an answer.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::game_list::{Reader, game_mut, games_with};
use serde_json::{Value, json};
use sqlx::PgPool;

/// The away game of M16 1 on Saturday 10 October 2026 at 15:00, before the test clock.
const AWAY_10_OCT: i64 = 1_073_147;

async fn ok(reader: &Reader, method: &str, path: &str, body: Option<Value>) -> Value {
    let (status, answer) = reader.send(method, path, body).await;
    assert_eq!(status, 200, "{method} {path}: {answer}");
    answer
}

fn game_path(id: i64) -> String {
    format!("/api/games/2026-2027/{id}")
}

async fn add_player(reader: &Reader, first: &str) -> i64 {
    ok(
        reader,
        "POST",
        "/api/players",
        Some(
            json!({ "first_name": first, "role": { "role": "plays_games", "from": "2026-09-01" } }),
        ),
    )
    .await["player"]["id"]
        .as_i64()
        .unwrap()
}

async fn add_reason(reader: &Reader, name: &str) -> i64 {
    let answer = ok(
        reader,
        "POST",
        "/api/reasons",
        Some(json!({ "name": name, "valid": true })),
    )
    .await;
    answer
        .as_array()
        .unwrap()
        .iter()
        .find(|reason| reason["name"] == name)
        .unwrap()["id"]
        .as_i64()
        .unwrap()
}

async fn answer(reader: &Reader, player: i64, answer: &str, reason: Option<i64>) {
    ok(
        reader,
        "PUT",
        &format!("{}/answers/{player}", game_path(AWAY_10_OCT)),
        Some(json!({ "answer": answer, "reason": reason })),
    )
    .await;
}

async fn attendance(reader: &Reader) -> Value {
    ok(reader, "GET", "/api/attendance?season=2026-2027", None).await
}

/// (first name, games, there, absent valid, absent not valid) of each row of the team.
fn games_of(overview: &Value) -> Vec<(String, i64, i64, i64, i64)> {
    overview["team"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let games = &row["games"];
            let n = |key: &str| games[key].as_i64().unwrap();
            (
                row["first_name"].as_str().unwrap().to_owned(),
                n("games"),
                n("there"),
                n("absent_valid"),
                n("absent_not_valid"),
            )
        })
        .collect()
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_checked_game_counts_each_answer_of_its_list(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let anna = add_player(&reader, "Anna").await;
    let bram = add_player(&reader, "Bram").await;
    add_player(&reader, "Cas").await;
    let exam = add_reason(&reader, "Exam").await;
    answer(&reader, anna, "coming", None).await;
    answer(&reader, bram, "not_coming", Some(exam)).await;

    // Not checked: no game counts, and nobody has a training, so there are no rows.
    let overview = attendance(&reader).await;
    assert_eq!(overview["games"], 0);
    assert_eq!(overview["team"], json!([]));

    let checked = format!("{}/checked", game_path(AWAY_10_OCT));
    ok(&reader, "PUT", &checked, None).await;
    let overview = attendance(&reader).await;
    assert_eq!(
        (overview["games"].clone(), overview["trainings"].clone()),
        (json!(1), json!(0))
    );
    // A player with games and no trainings has a row; Cas had no answer: absent, not valid.
    assert_eq!(
        games_of(&overview),
        [
            ("Anna".to_owned(), 1, 1, 0, 0),
            ("Bram".to_owned(), 1, 0, 1, 0),
            ("Cas".to_owned(), 1, 0, 0, 1),
        ]
    );
    assert_eq!(overview["team"][0]["trainings"], 0);
    assert_eq!(
        overview["team"][1]["missed"],
        json!([{ "date": "2026-10-10", "kind": "game", "reason": "Exam", "valid": true }])
    );
    assert_eq!(
        overview["team"][2]["missed"],
        json!([{ "date": "2026-10-10", "kind": "game", "reason": null, "valid": false }])
    );

    // "Undo check": the game no longer counts.
    ok(&reader, "DELETE", &checked, None).await;
    assert_eq!(attendance(&reader).await["team"], json!([]));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_not_played_or_moved_game_does_not_count(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let anna = add_player(&reader, "Anna").await;
    answer(&reader, anna, "coming", None).await;
    ok(
        &reader,
        "PUT",
        &format!("{}/checked", game_path(AWAY_10_OCT)),
        None,
    )
    .await;
    assert_eq!(attendance(&reader).await["games"], 1);

    let not_played = format!("{}/not-played", game_path(AWAY_10_OCT));
    ok(&reader, "PUT", &not_played, None).await;
    assert_eq!(attendance(&reader).await["games"], 0);
    ok(&reader, "DELETE", &not_played, None).await;
    assert_eq!(attendance(&reader).await["games"], 1);

    // Moved to Sunday 11 October: the check holds for 10 October only.
    reader.answer(games_with(|games| {
        game_mut(games, AWAY_10_OCT)["datum"] = json!("2026-10-11 15:00:00.000");
    }));
    reader.read_good().await;
    let overview = attendance(&reader).await;
    assert_eq!(overview["games"], 0);
    assert_eq!(overview["team"], json!([]));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_release_day_shows_only_in_its_season(pool: PgPool) {
    let reader = Reader::chosen(pool.clone()).await;
    reader.read_good().await;
    sqlx::query("UPDATE game_check_start SET from_date = '2026-10-10'")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(attendance(&reader).await["games_from"], "2026-10-10");
    let earlier = ok(&reader, "GET", "/api/attendance?season=2025-2026", None).await;
    assert_eq!(earlier["games_from"], Value::Null);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_checked_game_that_left_the_game_list_still_counts(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let anna = add_player(&reader, "Anna").await;
    answer(&reader, anna, "coming", None).await;
    ok(
        &reader,
        "PUT",
        &format!("{}/checked", game_path(AWAY_10_OCT)),
        None,
    )
    .await;

    // The game leaves the game list after it was played; its change, if shown, is seen.
    reader.answer(games_with(|games| {
        games.retain(|game| game["id"] != AWAY_10_OCT);
    }));
    reader.read_good().await;
    let changes = ok(&reader, "GET", "/api/changes", None).await;
    if let Some(change) = changes["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|change| change["id"] == AWAY_10_OCT)
    {
        ok(
            &reader,
            "POST",
            "/api/changes/seen",
            Some(json!({ "changes": [{ "id": AWAY_10_OCT, "version": change["version"] }] })),
        )
        .await;
    }
    let overview = attendance(&reader).await;
    assert_eq!(overview["games"], 1);
    assert_eq!(games_of(&overview), [("Anna".to_owned(), 1, 1, 0, 0)]);
}
