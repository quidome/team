//! The weekly trainings through the API (attendance at training, story A).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::time::Duration;

use common::api::Coordinator;
use common::game_list::utc;
use serde_json::{Value, json};
use sqlx::PgPool;
use team::weekly_trainings;

fn tuesday() -> Value {
    json!({ "weekday": 2, "start": "19:45", "end": "21:00",
            "from": "2026-09-01", "until": "2027-05-25" })
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn nothing_saved_yet(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    let list = coordinator.ok("GET", "/api/weekly-trainings", None).await;
    assert_eq!(
        list,
        json!({ "season": "2026-2027", "season_first_day": "2026-07-01",
                "season_last_day": "2027-06-30", "weekly_trainings": [], "proposals": [] })
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_proposed_window_is_the_first_day_of_september_and_the_last_of_may(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    let proposal = coordinator
        .ok("GET", "/api/weekly-trainings/proposal?weekday=2", None)
        .await;
    assert_eq!(
        proposal,
        json!({ "from": "2026-09-01", "until": "2027-05-25" })
    );
    let proposal = coordinator
        .ok("GET", "/api/weekly-trainings/proposal?weekday=4", None)
        .await;
    assert_eq!(
        proposal,
        json!({ "from": "2026-09-03", "until": "2027-05-27" })
    );
    let (status, answer) = coordinator
        .send("GET", "/api/weekly-trainings/proposal?weekday=8", None)
        .await;
    assert_eq!(
        (status, answer),
        (400, json!({ "error": "invalid_weekday" }))
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn add_change_and_delete_a_weekly_training(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    let list = coordinator
        .ok("POST", "/api/weekly-trainings", Some(tuesday()))
        .await;
    let id = list["weekly_trainings"][0]["id"].as_i64().unwrap();
    assert_eq!(
        list["weekly_trainings"],
        json!([{ "id": id, "weekday": 2, "start": "19:45", "end": "21:00",
                 "from": "2026-09-01", "until": "2027-05-25" }])
    );

    // A Thursday next to it is fine.
    let thursday = json!({ "weekday": 4, "start": "19:00", "end": "20:30",
                           "from": "2026-09-03", "until": "2027-05-27" });
    let list = coordinator
        .ok("POST", "/api/weekly-trainings", Some(thursday))
        .await;
    assert_eq!(list["weekly_trainings"].as_array().unwrap().len(), 2);

    // "Change": the Tuesday was a Wednesday by mistake.
    let wednesday = json!({ "weekday": 3, "start": "19:45", "end": "21:00",
                            "from": "2026-09-02", "until": "2027-05-26" });
    let list = coordinator
        .ok(
            "PUT",
            &format!("/api/weekly-trainings/{id}"),
            Some(wednesday),
        )
        .await;
    let changed: Vec<&Value> = list["weekly_trainings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|training| training["id"] == id)
        .collect();
    assert_eq!(changed[0]["weekday"], 3);
    assert_eq!(changed[0]["from"], "2026-09-02");

    let list = coordinator
        .ok("DELETE", &format!("/api/weekly-trainings/{id}"), None)
        .await;
    assert_eq!(list["weekly_trainings"].as_array().unwrap().len(), 1);
    assert_eq!(list["weekly_trainings"][0]["weekday"], 4);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn an_overlapping_weekly_training_is_refused_and_nothing_is_stored(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    coordinator
        .ok("POST", "/api/weekly-trainings", Some(tuesday()))
        .await;
    let from_january = json!({ "weekday": 2, "start": "20:00", "end": "21:30",
                               "from": "2027-01-05", "until": "2027-05-25" });
    let (status, answer) = coordinator
        .send("POST", "/api/weekly-trainings", Some(from_january))
        .await;
    assert_eq!(
        (status, answer),
        (400, json!({ "error": "weekly_trainings_overlap" }))
    );
    let list = coordinator.ok("GET", "/api/weekly-trainings", None).await;
    assert_eq!(list["weekly_trainings"].as_array().unwrap().len(), 1);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn weekly_training_errors_have_their_codes(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    let cases = [
        (
            json!({ "start": "21:00", "end": "19:45" }),
            "end_before_start",
        ),
        (
            json!({ "from": "2026-09-01", "until": "2026-08-31" }),
            "last_before_first",
        ),
        (json!({ "from": "2026-06-30" }), "outside_season"),
        (json!({ "until": "2027-07-01" }), "outside_season"),
        (json!({ "weekday": 0 }), "invalid_weekday"),
        (json!({ "start": "7 pm" }), "invalid_time"),
    ];
    for (change, code) in cases {
        let mut body = tuesday();
        for (key, value) in change.as_object().unwrap() {
            body[key] = value.clone();
        }
        let (status, answer) = coordinator
            .send("POST", "/api/weekly-trainings", Some(body))
            .await;
        assert_eq!(
            (status, answer),
            (400, json!({ "error": code })),
            "{change}"
        );
    }
    let (status, _) = coordinator
        .send("PUT", "/api/weekly-trainings/999", Some(tuesday()))
        .await;
    assert_eq!(status, 404);
    let (status, _) = coordinator
        .send("DELETE", "/api/weekly-trainings/999", None)
        .await;
    assert_eq!(status, 404);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_save_waits_for_another_save(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    // Another save holds the lock.
    let mut other = pool.begin().await.unwrap();
    weekly_trainings::lock(&mut other).await.unwrap();

    let save = coordinator.send("POST", "/api/weekly-trainings", Some(tuesday()));
    tokio::pin!(save);
    let waited = tokio::time::timeout(Duration::from_millis(300), &mut save).await;
    assert!(waited.is_err(), "the save did not wait for the lock");
    other.commit().await.unwrap();
    let (status, _) = save.await;
    assert_eq!(status, 200);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn last_seasons_weekly_trainings_are_proposed(pool: PgPool) {
    let mut coordinator = Coordinator::start(pool).await;
    coordinator
        .ok("POST", "/api/weekly-trainings", Some(tuesday()))
        .await;
    let thursday = json!({ "weekday": 4, "start": "19:00", "end": "20:30",
                           "from": "2026-09-03", "until": "2027-05-27" });
    let list = coordinator
        .ok("POST", "/api/weekly-trainings", Some(thursday))
        .await;
    // Last season (2025-2026) had no weekly trainings: nothing is proposed.
    assert_eq!(list["proposals"], json!([]));

    // The next season: both of last season's weekly trainings, with new windows.
    coordinator.harness.clock.set(utc(2027, 8, 15, 12, 0));
    coordinator.log_in_again().await;
    let list = coordinator.ok("GET", "/api/weekly-trainings", None).await;
    assert_eq!(list["season"], "2027-2028");
    assert_eq!(
        list["proposals"],
        json!([{ "weekday": 4, "start": "19:00", "end": "20:30",
                 "from": "2027-09-02", "until": "2028-05-25" },
               { "weekday": 2, "start": "19:45", "end": "21:00",
                 "from": "2027-09-07", "until": "2028-05-30" }])
    );
    // Tuesday saved with "Add": Thursday is still proposed.
    let list = coordinator
        .ok(
            "POST",
            "/api/weekly-trainings",
            Some(json!({ "weekday": 2, "start": "19:45", "end": "21:00",
                         "from": "2027-09-07", "until": "2028-05-30" })),
        )
        .await;
    assert_eq!(
        list["proposals"],
        json!([{ "weekday": 4, "start": "19:00", "end": "20:30",
                 "from": "2027-09-02", "until": "2028-05-25" }])
    );
}
