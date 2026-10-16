//! The training settings through the API (attendance at training, story A).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::api::Coordinator;
use common::game_list::utc;
use serde_json::json;
use sqlx::PgPool;
use team::training_settings;
use team_core::Season;

fn tuesday() -> serde_json::Value {
    json!({ "weekday": 2, "start": "19:45", "end": "21:00" })
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn nothing_saved_yet(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    let settings = coordinator.ok("GET", "/api/training-settings", None).await;
    assert_eq!(
        settings,
        json!({ "season": "2026-2027", "saved": false, "first": null, "last": null, "days": [] })
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_proposal_follows_the_chosen_days(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    let proposal = coordinator
        .ok("GET", "/api/training-settings/proposal?weekdays=2", None)
        .await;
    assert_eq!(
        proposal,
        json!({ "first": "2026-09-01", "last": "2027-05-25" })
    );
    let proposal = coordinator
        .ok("GET", "/api/training-settings/proposal?weekdays=2,4", None)
        .await;
    assert_eq!(
        proposal,
        json!({ "first": "2026-09-01", "last": "2027-05-27" })
    );
    let (status, answer) = coordinator
        .send("GET", "/api/training-settings/proposal?weekdays=", None)
        .await;
    assert_eq!(
        (status, answer),
        (400, json!({ "error": "no_training_days" }))
    );
    let (_, answer) = coordinator
        .send("GET", "/api/training-settings/proposal?weekdays=8", None)
        .await;
    assert_eq!(answer, json!({ "error": "invalid_weekday" }));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn saved_settings_are_read_back(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    let saved = coordinator
        .ok(
            "PUT",
            "/api/training-settings",
            Some(json!({ "first": "2026-09-01", "last": "2027-05-25", "days": [tuesday()] })),
        )
        .await;
    let expected = json!({ "season": "2026-2027", "saved": true, "first": "2026-09-01",
                           "last": "2027-05-25", "days": [tuesday()] });
    assert_eq!(saved, expected);
    assert_eq!(
        coordinator.ok("GET", "/api/training-settings", None).await,
        expected
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_change_in_the_season_keeps_the_earlier_days(pool: PgPool) {
    let mut coordinator = Coordinator::start(pool).await;
    coordinator
        .ok(
            "PUT",
            "/api/training-settings",
            Some(json!({ "first": "2026-09-01", "last": "2027-05-25", "days": [tuesday()] })),
        )
        .await;
    coordinator.harness.clock.set(utc(2027, 1, 5, 12, 0));
    coordinator.log_in_again().await;
    let thursday = json!({ "weekday": 4, "start": "19:00", "end": "20:30" });
    let saved = coordinator
        .ok(
            "PUT",
            "/api/training-settings",
            Some(json!({ "first": "2026-09-01", "last": "2027-05-25", "days": [thursday] })),
        )
        .await;
    assert_eq!(saved["days"], json!([thursday]));

    let mut db = coordinator.harness.pool.acquire().await.unwrap();
    let stored = training_settings::for_season(&mut db, Season::from_start_year(2026))
        .await
        .unwrap()
        .unwrap();
    let valid_from: Vec<String> = stored
        .sets
        .iter()
        .map(|set| set.valid_from.to_string())
        .collect();
    assert_eq!(valid_from, ["2026-09-01", "2027-01-05"]);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn settings_errors_have_their_codes(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    let save = |first: &str, last: &str, days: serde_json::Value| {
        Some(json!({ "first": first, "last": last, "days": days }))
    };
    let cases = [
        (
            save(
                "2026-09-01",
                "2027-05-25",
                json!([{ "weekday": 2, "start": "21:00", "end": "19:45" }]),
            ),
            "end_before_start",
        ),
        (
            save("2027-05-25", "2026-09-01", json!([tuesday()])),
            "last_before_first",
        ),
        (
            save("2026-09-01", "2027-05-25", json!([])),
            "no_training_days",
        ),
        (
            save("2026-09-01", "2027-05-25", json!([tuesday(), tuesday()])),
            "duplicate_weekday",
        ),
        (
            save("2026-06-30", "2027-05-25", json!([tuesday()])),
            "outside_season",
        ),
        (
            save(
                "2026-09-01",
                "2027-05-25",
                json!([{ "weekday": 2, "start": "7 pm", "end": "21:00" }]),
            ),
            "invalid_time",
        ),
        (
            save(
                "2026-09-01",
                "2027-05-25",
                json!([{ "weekday": 0, "start": "19:45", "end": "21:00" }]),
            ),
            "invalid_weekday",
        ),
    ];
    for (body, code) in cases {
        let (status, answer) = coordinator
            .send("PUT", "/api/training-settings", body)
            .await;
        assert_eq!((status, answer), (400, json!({ "error": code })));
    }
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn before_the_season_the_last_save_wins(pool: PgPool) {
    let mut coordinator = Coordinator::start(pool).await;
    coordinator.harness.clock.set(utc(2026, 8, 15, 12, 0));
    coordinator.log_in_again().await;
    coordinator
        .ok(
            "PUT",
            "/api/training-settings",
            Some(json!({ "first": "2026-09-08", "last": "2027-05-25", "days": [tuesday()] })),
        )
        .await;
    // The second save moves the first training date earlier.
    let thursday = json!({ "weekday": 4, "start": "19:00", "end": "20:30" });
    coordinator
        .ok(
            "PUT",
            "/api/training-settings",
            Some(json!({ "first": "2026-09-01", "last": "2027-05-27", "days": [thursday] })),
        )
        .await;

    let mut db = coordinator.harness.pool.acquire().await.unwrap();
    let stored = training_settings::for_season(&mut db, Season::from_start_year(2026))
        .await
        .unwrap()
        .unwrap();
    let valid_from: Vec<String> = stored
        .sets
        .iter()
        .map(|set| set.valid_from.to_string())
        .collect();
    assert_eq!(valid_from, ["2026-09-01"]);

    coordinator.harness.clock.set(utc(2026, 9, 10, 12, 0));
    coordinator.log_in_again().await;
    let settings = coordinator.ok("GET", "/api/training-settings", None).await;
    assert_eq!(settings["days"], json!([thursday]));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_save_waits_for_another_save(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    // Another save holds the lock.
    let mut other = pool.begin().await.unwrap();
    training_settings::lock(&mut other).await.unwrap();

    let save = coordinator.send(
        "PUT",
        "/api/training-settings",
        Some(json!({ "first": "2026-09-01", "last": "2027-05-25", "days": [tuesday()] })),
    );
    tokio::pin!(save);
    let waited = tokio::time::timeout(std::time::Duration::from_millis(300), &mut save).await;
    assert!(waited.is_err(), "the save did not wait for the lock");
    other.commit().await.unwrap();
    let (status, _) = save.await;
    assert_eq!(status, 200);
}
