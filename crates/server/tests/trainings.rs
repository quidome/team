//! Trainings through the API: the list of a season, the next training and one training with its
//! players (attendance at training, story B).

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

/// Moves the clock to that moment (UTC) and logs in again.
async fn at(
    coordinator: &mut Coordinator,
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
) {
    coordinator
        .harness
        .clock
        .set(utc(year, month, day, hour, minute));
    coordinator.log_in_again().await;
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

/// A training written past the routes (they come in step 2).
async fn insert_training(pool: &PgPool, on: NaiveDate, cancelled: bool) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO trainings (date, start_time, end_time, cancelled_at, created_at)
         VALUES ($1, '19:45', '21:00', CASE WHEN $2 THEN now() END, now()) RETURNING id",
    )
    .bind(on)
    .bind(cancelled)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn insert_absence(pool: &PgPool, training: i64, player: i64, reason: i64) {
    sqlx::query("INSERT INTO absences (training_id, player_id, reason_id) VALUES ($1, $2, $3)")
        .bind(training)
        .bind(player)
        .bind(reason)
        .execute(pool)
        .await
        .unwrap();
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

fn dates(rows: &Value) -> Vec<&str> {
    rows.as_array()
        .unwrap()
        .iter()
        .map(|row| row["date"].as_str().unwrap())
        .collect()
}

fn row<'a>(list: &'a Value, on: &str) -> &'a Value {
    list["coming"]
        .as_array()
        .unwrap()
        .iter()
        .chain(list["past"].as_array().unwrap())
        .find(|row| row["date"] == on)
        .unwrap()
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_list_has_one_row_per_date(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    add_tuesdays(&coordinator).await;
    let avery = coordinator
        .add_player("Avery", "plays_games", "2026-09-01")
        .await;
    let blake = coordinator
        .add_player("Blake", "only_trains", "2026-09-01")
        .await;
    let school = reason_id(&coordinator, "School").await;
    let tuesday = insert_training(&pool, date(2026, 10, 6), false).await;
    insert_absence(&pool, tuesday, avery, school).await;
    insert_absence(&pool, tuesday, blake, school).await;
    insert_training(&pool, date(2026, 9, 29), false).await;
    insert_training(&pool, date(2026, 9, 22), true).await;
    // An extra training on a Wednesday.
    insert_training(&pool, date(2026, 10, 7), false).await;

    let list = coordinator.ok("GET", "/api/trainings", None).await;
    assert_eq!(list["season"], "2026-2027");
    assert_eq!(list["today"], "2026-10-12");
    assert_eq!(
        dates(&list["coming"])[..3],
        ["2026-10-13", "2026-10-20", "2026-10-27"]
    );
    assert_eq!(
        dates(&list["past"])[..4],
        ["2026-10-07", "2026-10-06", "2026-09-29", "2026-09-22"]
    );
    assert_eq!(
        row(&list, "2026-10-06"),
        &json!({ "date": "2026-10-06", "start": "19:45", "end": "21:00", "status": "absent",
                 "absent": 2, "opened": true, "extra": false })
    );
    assert_eq!(row(&list, "2026-09-29")["status"], "all_there");
    assert_eq!(row(&list, "2026-09-22")["status"], "cancelled");
    assert_eq!(row(&list, "2026-10-07")["extra"], true);
    assert_eq!(row(&list, "2026-10-13")["status"], "not_opened");
    assert_eq!(row(&list, "2026-10-13")["opened"], false);

    // Last season has no weekly trainings and no trainings.
    let last = coordinator
        .ok("GET", "/api/trainings?season=2025-2026", None)
        .await;
    assert!(last["coming"].as_array().unwrap().is_empty());
    assert!(last["past"].as_array().unwrap().is_empty());
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_card_is_the_next_training(pool: PgPool) {
    let mut coordinator = Coordinator::start(pool.clone()).await;
    // No weekly trainings: no card.
    assert_eq!(
        coordinator.ok("GET", "/api/trainings/next", None).await,
        Value::Null
    );
    add_tuesdays(&coordinator).await;

    // Tue 13 Oct, 23:59 Dutch time (21:59 UTC): still today's training.
    at(&mut coordinator, 2026, 10, 13, 21, 59).await;
    let next = coordinator.ok("GET", "/api/trainings/next", None).await;
    assert_eq!(next["date"], "2026-10-13");
    assert_eq!(next["status"], "not_opened");

    // Wednesday: next Tuesday, also when it is cancelled.
    at(&mut coordinator, 2026, 10, 14, 10, 0).await;
    insert_training(&pool, date(2026, 10, 20), true).await;
    let next = coordinator.ok("GET", "/api/trainings/next", None).await;
    assert_eq!(next["date"], "2026-10-20");
    assert_eq!(next["status"], "cancelled");

    // An extra training on Thu 15 Oct comes first.
    insert_training(&pool, date(2026, 10, 15), false).await;
    let next = coordinator.ok("GET", "/api/trainings/next", None).await;
    assert_eq!(next["date"], "2026-10-15");
    assert_eq!(next["extra"], true);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_training_page_lists_the_players_with_a_role(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    add_tuesdays(&coordinator).await;
    let casey = coordinator
        .add_player("Casey", "plays_games", "2026-09-01")
        .await;
    coordinator
        .add_player("avery", "only_trains", "2026-09-01")
        .await;
    // Dana joins later: not at the training of Tue 6 Oct.
    coordinator
        .add_player("Dana", "plays_games", "2026-10-07")
        .await;
    let ill = reason_id(&coordinator, "Ill").await;

    // Not opened yet: no players, the times of the training day.
    let page = coordinator
        .ok("GET", "/api/trainings/2026-10-06", None)
        .await;
    assert_eq!(page["training"], Value::Null);
    assert_eq!(
        page["proposed_times"],
        json!({ "start": "19:45", "end": "21:00" })
    );
    assert!(page["players"].as_array().unwrap().is_empty());
    assert_eq!(page["reasons"].as_array().unwrap().len(), 6);

    let tuesday = insert_training(&pool, date(2026, 10, 6), false).await;
    insert_absence(&pool, tuesday, casey, ill).await;
    let page = coordinator
        .ok("GET", "/api/trainings/2026-10-06", None)
        .await;
    assert_eq!(page["season"], "2026-2027");
    assert_eq!(
        page["training"],
        json!({ "start": "19:45", "end": "21:00", "cancelled": false })
    );
    assert_eq!(page["absent"], 1);
    let players: Vec<(&Value, &Value, &Value)> = page["players"]
        .as_array()
        .unwrap()
        .iter()
        .map(|player| (&player["first_name"], &player["role"], &player["reason"]))
        .collect();
    assert_eq!(
        players,
        [
            (&json!("avery"), &json!("only_trains"), &Value::Null),
            (&json!("Casey"), &json!("plays_games"), &json!(ill)),
        ]
    );

    // A Wednesday: no proposed times without a weekly training on Wednesday.
    let page = coordinator
        .ok("GET", "/api/trainings/2026-10-07", None)
        .await;
    assert_eq!(page["proposed_times"], Value::Null);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_date_or_season_that_cannot_be_read_is_refused(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    let answer = coordinator
        .send("GET", "/api/trainings/13-10-2026", None)
        .await;
    assert_eq!(answer, (400, json!({ "error": "invalid_date" })));
    let answer = coordinator
        .send("GET", "/api/trainings?season=2026", None)
        .await;
    assert_eq!(answer, (400, json!({ "error": "invalid_season" })));
}
