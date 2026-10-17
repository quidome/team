//! The season totals through the API (attendance at training, story C). The clock starts on Monday
//! 12 October 2026, so the season is 2026-2027.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use chrono::NaiveDate;
use common::api::Coordinator;
use common::game_list::utc;
use serde_json::{Value, json};
use sqlx::PgPool;

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

async fn add_tuesdays(coordinator: &Coordinator) {
    coordinator
        .ok(
            "POST",
            "/api/weekly-trainings",
            Some(json!({ "weekday": 2, "start": "19:45", "end": "21:00",
                         "from": "2026-09-01", "until": "2027-05-25" })),
        )
        .await;
}

async fn reason_id(coordinator: &Coordinator, name: &str) -> i64 {
    let reasons = coordinator.ok("GET", "/api/reasons", None).await;
    reasons
        .as_array()
        .unwrap()
        .iter()
        .find(|reason| reason["name"] == name)
        .unwrap()["id"]
        .as_i64()
        .unwrap()
}

async fn open(coordinator: &Coordinator, on: &str) {
    coordinator
        .ok("PUT", &format!("/api/trainings/{on}/open"), None)
        .await;
}

async fn mark(coordinator: &Coordinator, on: &str, player: i64, reason: i64) {
    coordinator
        .ok(
            "PUT",
            &format!("/api/trainings/{on}/absences/{player}"),
            Some(json!({ "reason": reason })),
        )
        .await;
}

/// A training written past the routes, for a date the routes do not open.
async fn insert_training(pool: &PgPool, on: NaiveDate) {
    sqlx::query(
        "INSERT INTO trainings (date, start_time, end_time, created_at)
         VALUES ($1, '19:45', '21:00', now())",
    )
    .bind(on)
    .execute(pool)
    .await
    .unwrap();
}

/// (first name, there, valid, not valid, of N) of each row.
fn counts(rows: &Value) -> Vec<(String, u64, u64, u64, u64)> {
    rows.as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                row["first_name"].as_str().unwrap().to_owned(),
                row["there"].as_u64().unwrap(),
                row["absent_valid"].as_u64().unwrap(),
                row["absent_not_valid"].as_u64().unwrap(),
                row["trainings"].as_u64().unwrap(),
            )
        })
        .collect()
}

fn row(
    name: &str,
    there: u64,
    valid: u64,
    not_valid: u64,
    of: u64,
) -> (String, u64, u64, u64, u64) {
    (name.to_owned(), there, valid, not_valid, of)
}

/// Avery from Tue 1 Sep; Blake from Tue 1 Sep until Mon 14 Sep; trainings on the four Tuesdays of
/// September. Gives the ids of Avery and Blake.
async fn september(coordinator: &Coordinator, pool: &PgPool) -> (i64, i64) {
    add_tuesdays(coordinator).await;
    let avery = coordinator
        .add_player("Avery", "plays_games", "2026-09-01")
        .await;
    let blake = coordinator
        .add_player("Blake", "only_trains", "2026-09-01")
        .await;
    sqlx::query("UPDATE player_roles SET until_date = '2026-09-14' WHERE player_id = $1")
        .bind(blake)
        .execute(pool)
        .await
        .unwrap();
    for on in ["2026-09-01", "2026-09-08", "2026-09-15", "2026-09-22"] {
        open(coordinator, on).await;
    }
    (avery, blake)
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_overview_of_the_current_season(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    let (avery, blake) = september(&coordinator, &pool).await;
    let school = reason_id(&coordinator, "School").await;
    let ill = reason_id(&coordinator, "Ill").await;
    mark(&coordinator, "2026-09-15", avery, school).await;
    mark(&coordinator, "2026-09-22", avery, ill).await;
    mark(&coordinator, "2026-09-08", blake, ill).await;

    let overview = coordinator.ok("GET", "/api/attendance", None).await;
    assert_eq!(counts(&overview["team"]), [row("Avery", 2, 2, 0, 4)]);
    assert_eq!(counts(&overview["left"]), [row("Blake", 1, 1, 0, 2)]);

    // School becomes not valid: the count follows at once.
    coordinator
        .ok(
            "PUT",
            &format!("/api/reasons/{school}"),
            Some(json!({ "name": "School", "valid": false, "hidden": false })),
        )
        .await;
    let overview = coordinator.ok("GET", "/api/attendance", None).await;
    assert_eq!(overview["season"], "2026-2027");
    assert_eq!(overview["current_season"], "2026-2027");
    assert_eq!(overview["seasons"], json!(["2026-2027"]));
    assert_eq!(overview["trainings"], 4);
    assert_eq!(counts(&overview["team"]), [row("Avery", 2, 1, 1, 4)]);
    assert_eq!(
        overview["team"][0]["missed"],
        json!([
            { "date": "2026-09-22", "reason": "Ill", "valid": true },
            { "date": "2026-09-15", "reason": "School", "valid": false },
        ])
    );
    // Tue 29 Sep and Tue 6 Oct were never opened.
    assert_eq!(overview["not_opened"], 2);
    assert_eq!(overview["first_not_opened"], "2026-09-29");
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn an_earlier_season_can_be_chosen(pool: PgPool) {
    let mut coordinator = Coordinator::start(pool.clone()).await;
    september(&coordinator, &pool).await;

    // In the next season.
    coordinator.harness.clock.set(utc(2027, 7, 15, 10, 0));
    coordinator.log_in_again().await;
    let overview = coordinator.ok("GET", "/api/attendance", None).await;
    assert_eq!(overview["season"], "2027-2028");
    assert_eq!(overview["trainings"], 0);
    assert_eq!(overview["seasons"], json!(["2027-2028", "2026-2027"]));

    let overview = coordinator
        .ok("GET", "/api/attendance?season=2026-2027", None)
        .await;
    assert_eq!(overview["season"], "2026-2027");
    assert_eq!(overview["current_season"], "2027-2028");
    // Avery's role lasts until the end of the season: in the team on 30 June.
    assert_eq!(counts(&overview["team"]), [row("Avery", 4, 0, 0, 4)]);
    assert_eq!(counts(&overview["left"]), [row("Blake", 2, 0, 0, 2)]);

    // A season without data, chosen in the address, is in the choice too.
    let overview = coordinator
        .ok("GET", "/api/attendance?season=2024-2025", None)
        .await;
    assert_eq!(
        overview["seasons"],
        json!(["2027-2028", "2026-2027", "2024-2025"])
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_seasons_follow_the_date_rule(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    insert_training(&pool, date(2025, 6, 30)).await;
    insert_training(&pool, date(2025, 7, 1)).await;
    let overview = coordinator.ok("GET", "/api/attendance", None).await;
    assert_eq!(
        overview["seasons"],
        json!(["2026-2027", "2025-2026", "2024-2025"])
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_season_that_cannot_be_read_is_refused(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    let answer = coordinator
        .send("GET", "/api/attendance?season=abc", None)
        .await;
    assert_eq!(answer, (400, json!({ "error": "invalid_season" })));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn no_trainings_yet(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    add_tuesdays(&coordinator).await;
    coordinator
        .add_player("Avery", "plays_games", "2026-09-01")
        .await;
    let overview = coordinator.ok("GET", "/api/attendance", None).await;
    assert_eq!(overview["trainings"], 0);
    assert_eq!(overview["team"], json!([]));
    assert_eq!(overview["left"], json!([]));
}
