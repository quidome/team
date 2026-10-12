//! The schedule through the API (story 3b): which schedule shows, its games, and their order.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::Harness;
use common::game_list::{DRAKES, M16_1, Reader, data, game_mut, games_with, utc};
use serde_json::{Value, json};
use sqlx::PgPool;

/// A team game of M16 1 on 31 October 2026 at 16:00.
const TIME_GAME: i64 = 1073152;

async fn schedule(reader: &Reader) -> Value {
    let (status, body) = reader.send("GET", "/api/schedule", None).await;
    assert_eq!(status, 200, "{body}");
    body["schedule"].clone()
}

fn games(schedule: &Value, list: &str) -> Vec<Value> {
    schedule[list].as_array().unwrap().clone()
}

fn count(games: &[Value], reason: &str) -> usize {
    games.iter().filter(|game| game["reason"] == reason).count()
}

/// An empty game list for `season`.
fn empty_season(season: &str) -> Value {
    let mut body = data("games-empty.json");
    body["seizoen"] = json!(season);
    body
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn without_login_the_schedule_needs_a_login(pool: PgPool) {
    let harness = Harness::start(pool).await;
    let response = harness.get("/api/schedule", None).await;
    assert_eq!(response.status(), 401);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn with_nothing_chosen_there_is_no_schedule(pool: PgPool) {
    let harness = Harness::start(pool).await;
    let cookie = format!(
        "team_session={}",
        harness.log_in(&["team_coordinator"]).await
    );
    let response = harness.get("/api/schedule", Some(&cookie)).await;
    assert_eq!(response.status(), 200);
    let body: Value = serde_json::from_str(&common::body_text(response).await).unwrap();
    assert_eq!(body, json!({ "schedule": null }));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn before_the_first_reading_the_schedule_is_empty(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    let schedule = schedule(&reader).await;
    assert_eq!(schedule["season"], "2026-2027");
    assert_eq!(schedule["last_season"], false);
    assert_eq!(schedule["club"]["number"], DRAKES);
    assert_eq!(schedule["team"]["number"], M16_1);
    assert_eq!(schedule["coming"], json!([]));
    assert_eq!(schedule["past"], json!([]));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn after_a_reading_the_schedule_has_the_team_games_and_the_club_home_games(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let schedule = schedule(&reader).await;
    let coming = games(&schedule, "coming");
    let past = games(&schedule, "past");

    assert_eq!((coming.len(), past.len()), (23, 14));
    assert_eq!((count(&coming, "team"), count(&coming, "duty")), (5, 18));
    assert_eq!(count(&past, "team"), 5);
    for game in coming.iter().filter(|game| game["reason"] == "team") {
        assert!(
            game["home"] == "Blue Drakes M16 1" || game["away"] == "Blue Drakes M16 1",
            "{game}"
        );
    }
    // A duty game is a home game of another club team, never an away game.
    for game in coming
        .iter()
        .chain(&past)
        .filter(|game| game["reason"] == "duty")
    {
        assert!(
            game["home"].as_str().unwrap().starts_with("Blue Drakes"),
            "{game}"
        );
    }
    assert_eq!(
        coming[0],
        json!({
            "id": coming[0]["id"], "date": "2026-10-31", "start_time": "13:00",
            "home": "Blue Drakes X12 1", "away": "De Dunckers X12 1",
            "hall": "Bulwijk", "town": "Woerden", "reason": "duty", "in_list": true
        })
    );
    assert!(past[0]["date"].as_str() > past[13]["date"].as_str());
    assert_eq!(schedule["season"], "2026-2027");
    assert_eq!(schedule["last_season"], false);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_start_time_that_is_not_known_has_no_time(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.answer(games_with(|games| {
        game_mut(games, TIME_GAME)["tijd"] = json!("00:00");
    }));
    reader.read_good().await;
    let coming = games(&schedule(&reader).await, "coming");

    let on_31_october: Vec<&Value> = coming
        .iter()
        .filter(|game| game["date"] == "2026-10-31")
        .collect();
    let times: Vec<&Value> = on_31_october
        .iter()
        .map(|game| &game["start_time"])
        .collect();
    assert_eq!(
        times,
        [
            &json!("13:00"),
            &json!("14:30"),
            &json!("17:30"),
            &Value::Null
        ]
    );
    assert_eq!(on_31_october[3]["id"], TIME_GAME);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_game_no_longer_in_the_game_list_stays_with_in_list_false(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let first = games(&schedule(&reader).await, "coming")[0]["id"].clone();

    reader.answer(games_with(|games| games.retain(|game| game["id"] != first)));
    reader.read_good().await;
    let coming = games(&schedule(&reader).await, "coming");
    assert_eq!(coming.len(), 23);
    assert_eq!(
        (&coming[0]["id"], &coming[0]["in_list"]),
        (&first, &json!(false))
    );
    assert!(coming[1..].iter().all(|game| game["in_list"] == true));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn in_a_new_season_the_schedule_of_last_season_shows(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    reader.harness.clock.set(utc(2027, 7, 1, 10, 0));
    reader.log_in_again().await;

    let schedule = schedule(&reader).await;
    assert_eq!(schedule["season"], "2026-2027");
    assert_eq!(schedule["last_season"], true);
    assert_eq!(schedule["team"]["number"], M16_1);
    assert_eq!(games(&schedule, "coming").len(), 0);
    assert_eq!(games(&schedule, "past").len(), 37);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_new_season_with_no_games_yet_still_shows_last_season(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    reader.harness.clock.set(utc(2027, 7, 1, 10, 0));
    reader.log_in_again().await;
    reader.answer(empty_season("2027-2028"));
    let (status, selection) = reader
        .send(
            "PUT",
            "/api/selection",
            Some(json!({ "club": DRAKES, "team": null })),
        )
        .await;
    assert_eq!(
        (status, &selection["reason"]),
        (200, &json!("no_games_yet")),
        "{selection}"
    );

    let schedule = schedule(&reader).await;
    assert_eq!(schedule["season"], "2026-2027");
    assert_eq!(schedule["last_season"], true);
    assert_eq!(games(&schedule, "past").len(), 37);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_team_needed_with_no_earlier_team_has_no_schedule(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.answer(empty_season("2026-2027"));
    let (status, selection) = reader
        .send(
            "PUT",
            "/api/selection",
            Some(json!({ "club": DRAKES, "team": null })),
        )
        .await;
    assert_eq!(
        (status, &selection["state"]),
        (200, &json!("team_needed")),
        "{selection}"
    );

    let (status, body) = reader.send("GET", "/api/schedule", None).await;
    assert_eq!((status, body), (200, json!({ "schedule": null })));
}
