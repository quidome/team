//! Trainings through the API: the list of a season, the next training and one training with its
//! players (attendance at training, story B).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::time::Duration;

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

/// The training rows of `GET /api/season{query}` (the page "Season"), as the season, today and
/// the parts "coming" and "past".
async fn training_list(coordinator: &Coordinator, query: &str) -> Value {
    let answer = coordinator
        .ok("GET", &format!("/api/season{query}"), None)
        .await;
    let rows = |part: &str| -> Vec<Value> {
        answer[part]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| !event["training"].is_null())
            .map(|event| event["training"].clone())
            .collect()
    };
    json!({ "season": answer["season"], "today": answer["today"],
            "coming": rows("coming"), "past": rows("past") })
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

    let list = training_list(&coordinator, "").await;
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
                 "absent": 2, "opened": true, "extra": false, "checked": false })
    );
    assert_eq!(row(&list, "2026-09-29")["status"], "all_there");
    assert_eq!(row(&list, "2026-09-22")["status"], "cancelled");
    assert_eq!(row(&list, "2026-10-07")["extra"], true);
    assert_eq!(row(&list, "2026-10-13")["status"], "not_opened");
    assert_eq!(row(&list, "2026-10-13")["opened"], false);

    // Last season has no weekly trainings and no trainings.
    let last = training_list(&coordinator, "?season=2025-2026").await;
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
        json!({ "start": "19:45", "end": "21:00", "cancelled": false, "checked": false, "can_check": true })
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
        .send("GET", "/api/season?season=2026", None)
        .await;
    assert_eq!(answer, (400, json!({ "error": "invalid_season" })));
}

async fn stored_trainings(pool: &PgPool) -> Vec<NaiveDate> {
    sqlx::query_scalar("SELECT date FROM trainings ORDER BY date")
        .fetch_all(pool)
        .await
        .unwrap()
}

async fn stored_absences(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM absences")
        .fetch_one(pool)
        .await
        .unwrap()
}

fn times(on: &str, start: &str, end: &str) -> Option<Value> {
    Some(json!({ "date": on, "start": start, "end": end }))
}

/// A Tuesday weekly training, two players from Tue 1 Sep, and the ids of the reasons School and Ill.
async fn team_with_tuesdays(coordinator: &Coordinator) -> (i64, i64, i64, i64) {
    add_tuesdays(coordinator).await;
    let avery = coordinator
        .add_player("Avery", "plays_games", "2026-09-01")
        .await;
    let blake = coordinator
        .add_player("Blake", "only_trains", "2026-09-01")
        .await;
    (
        avery,
        blake,
        reason_id(coordinator, "School").await,
        reason_id(coordinator, "Ill").await,
    )
}

fn absent(reason: i64) -> Option<Value> {
    Some(json!({ "reason": reason }))
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn open_twice_gives_one_training(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    let (avery, _, school, _) = team_with_tuesdays(&coordinator).await;
    let path = "/api/trainings/2026-10-06/open";
    // Another screen opens the day at the same moment: its insert is not committed yet.
    let mut other = pool.begin().await.unwrap();
    sqlx::query(
        "INSERT INTO trainings (date, start_time, end_time, created_at)
         VALUES ('2026-10-06', '19:45', '21:00', now())",
    )
    .execute(&mut *other)
    .await
    .unwrap();
    let open = coordinator.send("PUT", path, None);
    tokio::pin!(open);
    let waited = tokio::time::timeout(Duration::from_millis(300), &mut open).await;
    assert!(
        waited.is_err(),
        "the open did not wait for the other insert"
    );
    other.commit().await.unwrap();
    let (status, page) = open.await;
    assert_eq!(status, 200);
    assert_eq!(page["training"]["start"], "19:45");
    assert_eq!(page["players"].as_array().unwrap().len(), 2);
    assert_eq!(stored_trainings(&pool).await, [date(2026, 10, 6)]);

    // An absence noted, then the day opened again from another screen: the absence stays.
    coordinator
        .ok(
            "PUT",
            &format!("/api/trainings/2026-10-06/absences/{avery}"),
            absent(school),
        )
        .await;
    let page = coordinator.ok("PUT", path, None).await;
    assert_eq!(page["absent"], 1);
    assert_eq!(stored_trainings(&pool).await, [date(2026, 10, 6)]);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn open_needs_a_training_day(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    add_tuesdays(&coordinator).await;
    let answer = coordinator
        .send("PUT", "/api/trainings/2026-10-07/open", None)
        .await;
    assert_eq!(answer, (400, json!({ "error": "not_a_training_day" })));
    assert!(stored_trainings(&pool).await.is_empty());
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn add_and_change_a_training(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    let (avery, _, school, _) = team_with_tuesdays(&coordinator).await;
    let page = coordinator
        .ok(
            "POST",
            "/api/trainings",
            times("2026-10-22", "19:00", "20:30"),
        )
        .await;
    assert_eq!(
        page["training"],
        json!({ "start": "19:00", "end": "20:30", "cancelled": false, "checked": false, "can_check": false })
    );
    let list = training_list(&coordinator, "").await;
    assert_eq!(row(&list, "2026-10-22")["extra"], true);

    // "Change" to Tue 20 Oct: the absence stays.
    coordinator
        .ok(
            "PUT",
            &format!("/api/trainings/2026-10-22/absences/{avery}"),
            absent(school),
        )
        .await;
    let page = coordinator
        .ok(
            "PUT",
            "/api/trainings/2026-10-22",
            times("2026-10-20", "19:45", "21:00"),
        )
        .await;
    assert_eq!(page["date"], "2026-10-20");
    assert_eq!(page["absent"], 1);
    assert_eq!(stored_trainings(&pool).await, [date(2026, 10, 20)]);

    coordinator
        .ok("PUT", "/api/trainings/2026-10-27/open", None)
        .await;
    let cases = [
        (times("2026-10-27", "19:45", "21:00"), "training_exists"),
        (times("2027-07-01", "19:45", "21:00"), "outside_season"),
        (times("2026-10-20", "21:00", "19:45"), "end_before_start"),
        (times("2026-10-20", "7 pm", "21:00"), "invalid_time"),
    ];
    for (body, code) in cases {
        let answer = coordinator
            .send("PUT", "/api/trainings/2026-10-20", body)
            .await;
        assert_eq!(answer, (400, json!({ "error": code })), "{code}");
    }
    let answer = coordinator
        .send(
            "POST",
            "/api/trainings",
            times("2026-10-27", "19:45", "21:00"),
        )
        .await;
    assert_eq!(answer, (400, json!({ "error": "training_exists" })));
    let (status, _) = coordinator
        .send(
            "PUT",
            "/api/trainings/2026-10-21",
            times("2026-10-23", "19:45", "21:00"),
        )
        .await;
    assert_eq!(status, 404);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn two_adds_of_one_date_give_one_training(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    // Another screen adds Thu 22 Oct at the same moment: its insert is not committed yet, so the
    // check in the core does not see it, and the unique date in the database decides.
    let mut other = pool.begin().await.unwrap();
    sqlx::query(
        "INSERT INTO trainings (date, start_time, end_time, created_at)
         VALUES ('2026-10-22', '19:00', '20:30', now())",
    )
    .execute(&mut *other)
    .await
    .unwrap();
    let add = coordinator.send(
        "POST",
        "/api/trainings",
        times("2026-10-22", "19:00", "20:30"),
    );
    tokio::pin!(add);
    let waited = tokio::time::timeout(Duration::from_millis(300), &mut add).await;
    assert!(waited.is_err(), "the add did not wait for the other insert");
    other.commit().await.unwrap();
    assert_eq!(add.await, (400, json!({ "error": "training_exists" })));
    assert_eq!(stored_trainings(&pool).await, [date(2026, 10, 22)]);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn cancel_and_undo_keep_the_absences(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    let (avery, blake, school, ill) = team_with_tuesdays(&coordinator).await;
    coordinator
        .ok("PUT", "/api/trainings/2026-10-06/open", None)
        .await;
    for (player, reason) in [(avery, school), (blake, ill)] {
        coordinator
            .ok(
                "PUT",
                &format!("/api/trainings/2026-10-06/absences/{player}"),
                absent(reason),
            )
            .await;
    }
    let page = coordinator
        .ok("POST", "/api/trainings/2026-10-06/cancel", None)
        .await;
    assert_eq!(page["training"]["cancelled"], true);
    assert_eq!(stored_absences(&pool).await, 2);
    let list = training_list(&coordinator, "").await;
    assert_eq!(row(&list, "2026-10-06")["status"], "cancelled");

    let page = coordinator
        .ok("POST", "/api/trainings/2026-10-06/uncancel", None)
        .await;
    assert_eq!(page["training"]["cancelled"], false);
    let list = training_list(&coordinator, "").await;
    assert_eq!(row(&list, "2026-10-06")["status"], "absent");
    assert_eq!(row(&list, "2026-10-06")["absent"], 2);

    let (status, _) = coordinator
        .send("POST", "/api/trainings/2026-10-13/uncancel", None)
        .await;
    assert_eq!(status, 404);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn cancel_a_day_that_is_not_opened(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    add_tuesdays(&coordinator).await;
    let page = coordinator
        .ok("POST", "/api/trainings/2026-10-20/cancel", None)
        .await;
    assert_eq!(
        page["training"],
        json!({ "start": "19:45", "end": "21:00", "cancelled": true, "checked": false, "can_check": false })
    );
    let first: chrono::DateTime<chrono::Utc> =
        sqlx::query_scalar("SELECT cancelled_at FROM trainings")
            .fetch_one(&pool)
            .await
            .unwrap();
    // Cancelled again later: the first moment stays.
    coordinator
        .harness
        .clock
        .advance(chrono::Duration::hours(1));
    coordinator
        .ok("POST", "/api/trainings/2026-10-20/cancel", None)
        .await;
    let again: chrono::DateTime<chrono::Utc> =
        sqlx::query_scalar("SELECT cancelled_at FROM trainings")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(again, first);
    let answer = coordinator
        .send("POST", "/api/trainings/2026-10-21/cancel", None)
        .await;
    assert_eq!(answer, (400, json!({ "error": "not_a_training_day" })));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn delete_a_training_deletes_its_absences(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    let (avery, _, school, _) = team_with_tuesdays(&coordinator).await;
    coordinator
        .ok("PUT", "/api/trainings/2026-10-06/open", None)
        .await;
    coordinator
        .ok(
            "PUT",
            &format!("/api/trainings/2026-10-06/absences/{avery}"),
            absent(school),
        )
        .await;
    let (status, _) = coordinator
        .send("DELETE", "/api/trainings/2026-10-06", None)
        .await;
    assert_eq!(status, 204);
    assert!(stored_trainings(&pool).await.is_empty());
    assert_eq!(stored_absences(&pool).await, 0);
    let list = training_list(&coordinator, "").await;
    assert_eq!(row(&list, "2026-10-06")["status"], "not_opened");
    let (status, _) = coordinator
        .send("DELETE", "/api/trainings/2026-10-06", None)
        .await;
    assert_eq!(status, 404);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn mark_absent_and_there_again(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    let (avery, _, school, ill) = team_with_tuesdays(&coordinator).await;
    coordinator
        .ok("PUT", "/api/trainings/2026-10-06/open", None)
        .await;
    let path = format!("/api/trainings/2026-10-06/absences/{avery}");
    let marked = coordinator.ok("PUT", &path, absent(school)).await;
    assert_eq!(
        marked,
        json!({ "player": avery, "reason": school, "absent": 1 })
    );
    // Another reason: still one absence.
    let marked = coordinator.ok("PUT", &path, absent(ill)).await;
    assert_eq!(marked["reason"], ill);
    assert_eq!(stored_absences(&pool).await, 1);
    let marked = coordinator.ok("DELETE", &path, None).await;
    assert_eq!(
        marked,
        json!({ "player": avery, "reason": null, "absent": 0 })
    );
    // Safe to send twice.
    let (status, _) = coordinator.send("DELETE", &path, None).await;
    assert_eq!(status, 200);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn mark_errors_have_their_codes(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    let (avery, _, school, _) = team_with_tuesdays(&coordinator).await;
    // Dana joins after the training.
    let dana = coordinator
        .add_player("Dana", "plays_games", "2026-10-07")
        .await;
    coordinator
        .ok("PUT", "/api/trainings/2026-10-06/open", None)
        .await;
    let answer = coordinator
        .send(
            "PUT",
            &format!("/api/trainings/2026-10-06/absences/{dana}"),
            absent(school),
        )
        .await;
    assert_eq!(answer, (400, json!({ "error": "not_in_team" })));

    // A hidden reason is not offered for new marks.
    coordinator
        .ok(
            "PUT",
            &format!("/api/reasons/{school}"),
            Some(json!({ "name": "School", "valid": true, "hidden": true })),
        )
        .await;
    let answer = coordinator
        .send(
            "PUT",
            &format!("/api/trainings/2026-10-06/absences/{avery}"),
            absent(school),
        )
        .await;
    assert_eq!(answer, (400, json!({ "error": "reason_hidden" })));

    for path in [
        "/api/trainings/2026-10-06/absences/999999".to_owned(),
        format!("/api/trainings/2026-10-13/absences/{avery}"),
    ] {
        let (status, _) = coordinator.send("PUT", &path, absent(school)).await;
        assert_eq!(status, 404, "{path}");
    }
    let (status, _) = coordinator
        .send(
            "PUT",
            &format!("/api/trainings/2026-10-06/absences/{avery}"),
            absent(999_999),
        )
        .await;
    assert_eq!(status, 404);
    let (status, _) = coordinator
        .send(
            "DELETE",
            &format!("/api/trainings/2026-10-13/absences/{avery}"),
            None,
        )
        .await;
    assert_eq!(status, 404);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_training_of_last_season_stays_open(pool: PgPool) {
    let mut coordinator = Coordinator::start(pool.clone()).await;
    let (avery, _, school, _) = team_with_tuesdays(&coordinator).await;
    coordinator
        .ok("PUT", "/api/trainings/2027-05-25/open", None)
        .await;

    // In the next season, the last training of May is found in July.
    at(&mut coordinator, 2027, 7, 15, 10, 0).await;
    coordinator
        .ok(
            "PUT",
            &format!("/api/trainings/2027-05-25/absences/{avery}"),
            absent(school),
        )
        .await;
    let page = coordinator
        .ok(
            "PUT",
            "/api/trainings/2027-05-25",
            times("2027-05-25", "19:30", "21:00"),
        )
        .await;
    assert_eq!(page["training"]["start"], "19:30");
    assert_eq!(page["absent"], 1);
    coordinator
        .ok("POST", "/api/trainings/2027-05-25/cancel", None)
        .await;
    // A training day of last season can still be opened.
    coordinator
        .ok("PUT", "/api/trainings/2027-05-18/open", None)
        .await;
    assert_eq!(
        stored_trainings(&pool).await,
        [date(2027, 5, 18), date(2027, 5, 25)]
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_weekly_training_change_never_touches_a_training(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    let (avery, _, school, _) = team_with_tuesdays(&coordinator).await;
    coordinator
        .ok("PUT", "/api/trainings/2026-10-06/open", None)
        .await;
    coordinator
        .ok(
            "PUT",
            &format!("/api/trainings/2026-10-06/absences/{avery}"),
            absent(school),
        )
        .await;
    // The weekly training was a Wednesday at 19:00 all along.
    let weekly = coordinator.ok("GET", "/api/weekly-trainings", None).await;
    let id = weekly["weekly_trainings"][0]["id"].as_i64().unwrap();
    coordinator
        .ok(
            "PUT",
            &format!("/api/weekly-trainings/{id}"),
            Some(json!({ "weekday": 3, "start": "19:00", "end": "20:30",
                         "from": "2026-09-02", "until": "2027-05-26" })),
        )
        .await;
    let list = training_list(&coordinator, "").await;
    let tuesday = row(&list, "2026-10-06");
    assert_eq!(tuesday["start"], "19:45");
    assert_eq!(tuesday["status"], "absent");
    assert_eq!(tuesday["extra"], true);
    assert_eq!(row(&list, "2026-10-07")["status"], "not_opened");
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_player_with_absences_cannot_be_deleted(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    let (avery, blake, school, _) = team_with_tuesdays(&coordinator).await;
    coordinator
        .ok("PUT", "/api/trainings/2026-10-06/open", None)
        .await;
    coordinator
        .ok(
            "PUT",
            &format!("/api/trainings/2026-10-06/absences/{avery}"),
            absent(school),
        )
        .await;
    let answer = coordinator
        .send("DELETE", &format!("/api/players/{avery}"), None)
        .await;
    assert_eq!(answer, (400, json!({ "error": "player_has_absences" })));
    // The page offers "Delete player" only without absences.
    let player = coordinator
        .ok("GET", &format!("/api/players/{avery}"), None)
        .await;
    assert_eq!(player["has_absences"], true);
    assert_eq!(
        coordinator
            .ok("GET", &format!("/api/players/{blake}"), None)
            .await["has_absences"],
        false
    );
    assert_eq!(stored_absences(&pool).await, 1);
    // Blake has no absences: deleted.
    let (status, _) = coordinator
        .send("DELETE", &format!("/api/players/{blake}"), None)
        .await;
    assert_eq!(status, 204);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_role_moved_earlier_counts_as_there(pool: PgPool) {
    let mut coordinator = Coordinator::start(pool.clone()).await;
    add_tuesdays(&coordinator).await;
    coordinator
        .ok("PUT", "/api/trainings/2026-09-01/open", None)
        .await;
    coordinator
        .ok("PUT", "/api/trainings/2026-09-08/open", None)
        .await;

    // "Add player" from 1 Sep: there at both trainings.
    let saved = coordinator
        .ok(
            "POST",
            "/api/players",
            Some(json!({ "first_name": "Casey", "last_name": null,
                         "role": { "role": "plays_games", "from": "2026-09-01" } })),
        )
        .await;
    let casey = saved["player"]["id"].as_i64().unwrap();
    assert_eq!(
        saved["player"]["now_there"],
        json!([{ "player": casey, "trainings": ["2026-09-01", "2026-09-08"] }])
    );

    // "Correct" of a role from today to 1 Sep.
    let avery = coordinator
        .add_player("Avery", "plays_games", "2026-10-12")
        .await;
    let role = coordinator
        .ok("GET", &format!("/api/players/{avery}"), None)
        .await["history"][0]["id"]
        .as_i64()
        .unwrap();
    let player = coordinator
        .ok(
            "PUT",
            &format!("/api/players/{avery}/roles/{role}"),
            Some(json!({ "role": "plays_games", "from": "2026-09-01", "until": null })),
        )
        .await;
    assert_eq!(
        player["now_there"],
        json!([{ "player": avery, "trainings": ["2026-09-01", "2026-09-08"] }])
    );
    // A change that covers nothing new: empty.
    let player = coordinator
        .ok(
            "PUT",
            &format!("/api/players/{avery}/roles/{role}"),
            Some(json!({ "role": "only_trains", "from": "2026-09-01", "until": null })),
        )
        .await;
    assert_eq!(player["now_there"], json!([]));
    assert_eq!(
        coordinator.ok("GET", "/api/team", None).await["now_there"],
        json!([])
    );

    // "Add to the team" (from the list) and "Add all" answer it on the team page.
    let dana = coordinator
        .ok(
            "POST",
            "/api/players",
            Some(json!({ "first_name": "Dana", "last_name": null, "role": null })),
        )
        .await["player"]["id"]
        .as_i64()
        .unwrap();
    let page = coordinator
        .ok(
            "POST",
            &format!("/api/players/{dana}/roles"),
            Some(json!({ "role": "plays_games", "from": "2026-09-05", "until": null })),
        )
        .await;
    assert_eq!(
        page["now_there"],
        json!([{ "player": dana, "trainings": ["2026-09-08"] }])
    );

    // The next season: a training on Tue 7 Sep 2027, then "Add all" from 1 Sep.
    at(&mut coordinator, 2027, 7, 2, 10, 0).await;
    coordinator
        .ok(
            "POST",
            "/api/trainings",
            times("2027-09-07", "19:45", "21:00"),
        )
        .await;
    let page = coordinator
        .ok(
            "POST",
            "/api/players/carry",
            Some(json!({ "players": [casey], "from": "2027-09-01" })),
        )
        .await;
    assert_eq!(
        page["now_there"],
        json!([{ "player": casey, "trainings": ["2027-09-07"] }])
    );
}

/// The training row of `on` in `GET /api/season`.
async fn season_row(coordinator: &Coordinator, on: &str) -> Value {
    row(&training_list(coordinator, "").await, on).clone()
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn check_and_undo_check_a_training(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    team_with_tuesdays(&coordinator).await;
    coordinator
        .ok("PUT", "/api/trainings/2026-10-06/open", None)
        .await;
    assert_eq!(
        season_row(&coordinator, "2026-10-06").await["checked"],
        false
    );

    let page = coordinator
        .ok("PUT", "/api/trainings/2026-10-06/checked", None)
        .await;
    assert_eq!(page["training"]["checked"], true);
    assert_eq!(page["players"].as_array().unwrap().len(), 2);
    assert_eq!(page["training"]["can_check"], false);
    let again = coordinator
        .ok("GET", "/api/trainings/2026-10-06", None)
        .await;
    assert_eq!(again["training"]["checked"], true);
    assert_eq!(
        season_row(&coordinator, "2026-10-06").await["checked"],
        true
    );

    let page = coordinator
        .ok("DELETE", "/api/trainings/2026-10-06/checked", None)
        .await;
    assert_eq!(page["training"]["checked"], false);
    assert_eq!(
        season_row(&coordinator, "2026-10-06").await["checked"],
        false
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn check_errors_have_their_codes(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    team_with_tuesdays(&coordinator).await;
    // Tue 6 Oct is a training day that is not opened.
    let answer = coordinator
        .send("PUT", "/api/trainings/2026-10-06/checked", None)
        .await;
    assert_eq!(answer, (400, json!({ "error": "not_opened" })));
    let answer = coordinator
        .send("DELETE", "/api/trainings/2026-10-06/checked", None)
        .await;
    assert_eq!(answer, (400, json!({ "error": "not_opened" })));
    // Tue 13 Oct is tomorrow: opened (an absence noted in advance), but not complete yet.
    coordinator
        .ok("PUT", "/api/trainings/2026-10-13/open", None)
        .await;
    let page = coordinator
        .ok("GET", "/api/trainings/2026-10-13", None)
        .await;
    assert_eq!(page["training"]["can_check"], false);
    let answer = coordinator
        .send("PUT", "/api/trainings/2026-10-13/checked", None)
        .await;
    assert_eq!(answer, (400, json!({ "error": "not_yet" })));
    let answer = coordinator
        .send("PUT", "/api/trainings/13-10-2026/checked", None)
        .await;
    assert_eq!(answer, (400, json!({ "error": "invalid_date" })));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_check_stays_through_marks_cancel_and_change(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    let (avery, _, school, _) = team_with_tuesdays(&coordinator).await;
    coordinator
        .ok("PUT", "/api/trainings/2026-10-06/open", None)
        .await;
    coordinator
        .ok("PUT", "/api/trainings/2026-10-06/checked", None)
        .await;

    coordinator
        .ok(
            "PUT",
            &format!("/api/trainings/2026-10-06/absences/{avery}"),
            absent(school),
        )
        .await;
    let page = coordinator
        .ok("POST", "/api/trainings/2026-10-06/cancel", None)
        .await;
    assert_eq!(page["training"]["checked"], true);
    let page = coordinator
        .ok("POST", "/api/trainings/2026-10-06/uncancel", None)
        .await;
    assert_eq!(page["training"]["checked"], true);
    // Moved to Thursday of next week: it stays checked, also after today.
    let page = coordinator
        .ok(
            "PUT",
            "/api/trainings/2026-10-06",
            times("2026-10-15", "19:45", "21:00"),
        )
        .await;
    assert_eq!(page["date"], "2026-10-15");
    assert_eq!(page["training"]["checked"], true);
    // A checked training after today can be checked again (the tap came twice).
    let page = coordinator
        .ok("PUT", "/api/trainings/2026-10-15/checked", None)
        .await;
    assert_eq!(page["training"]["checked"], true);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn delete_training_removes_the_check(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    team_with_tuesdays(&coordinator).await;
    coordinator
        .ok("PUT", "/api/trainings/2026-10-06/open", None)
        .await;
    coordinator
        .ok("PUT", "/api/trainings/2026-10-06/checked", None)
        .await;
    let (status, _) = coordinator
        .send("DELETE", "/api/trainings/2026-10-06", None)
        .await;
    assert_eq!(status, 204);
    let page = coordinator
        .ok("PUT", "/api/trainings/2026-10-06/open", None)
        .await;
    assert_eq!(page["training"]["checked"], false);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_check_does_not_wait_for_a_mark(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    let (avery, _, school, _) = team_with_tuesdays(&coordinator).await;
    let page = coordinator
        .ok("PUT", "/api/trainings/2026-10-06/open", None)
        .await;
    assert_eq!(page["training"]["checked"], false);
    let training: i64 = sqlx::query_scalar("SELECT id FROM trainings WHERE date = '2026-10-06'")
        .fetch_one(&pool)
        .await
        .unwrap();
    // Another screen marks Avery absent at the same moment: its insert is not committed yet.
    let mut other = pool.begin().await.unwrap();
    insert_absence_in(&mut other, training, avery, school).await;

    let check = coordinator.send("PUT", "/api/trainings/2026-10-06/checked", None);
    let answer = tokio::time::timeout(Duration::from_millis(300), check)
        .await
        .expect("the check waited for the mark");
    assert_eq!(answer.0, 200, "{}", answer.1);
    other.commit().await.unwrap();

    let page = coordinator
        .ok("GET", "/api/trainings/2026-10-06", None)
        .await;
    assert_eq!(page["training"]["checked"], true);
    assert_eq!(page["absent"], 1);
}

async fn insert_absence_in(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    training: i64,
    player: i64,
    reason: i64,
) {
    sqlx::query("INSERT INTO absences (training_id, player_id, reason_id) VALUES ($1, $2, $3)")
        .bind(training)
        .bind(player)
        .bind(reason)
        .execute(&mut **tx)
        .await
        .unwrap();
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_cancelled_training_is_not_checked(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    team_with_tuesdays(&coordinator).await;
    let page = coordinator
        .ok("POST", "/api/trainings/2026-10-06/cancel", None)
        .await;
    assert_eq!(page["training"]["can_check"], false);
    let answer = coordinator
        .send("PUT", "/api/trainings/2026-10-06/checked", None)
        .await;
    assert_eq!(answer, (400, json!({ "error": "cancelled" })));
    let page = coordinator
        .ok("POST", "/api/trainings/2026-10-06/uncancel", None)
        .await;
    assert_eq!(page["training"]["can_check"], true);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_check_waits_for_an_undo_check_on_another_screen(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    team_with_tuesdays(&coordinator).await;
    // A checked training moved to Thu 15 Oct, after today: it stays checked.
    coordinator
        .ok("PUT", "/api/trainings/2026-10-06/open", None)
        .await;
    coordinator
        .ok("PUT", "/api/trainings/2026-10-06/checked", None)
        .await;
    coordinator
        .ok(
            "PUT",
            "/api/trainings/2026-10-06",
            times("2026-10-15", "19:45", "21:00"),
        )
        .await;
    // Another screen undoes the check: not committed yet.
    let mut other = pool.begin().await.unwrap();
    sqlx::query("UPDATE trainings SET checked_at = NULL WHERE date = '2026-10-15'")
        .execute(&mut *other)
        .await
        .unwrap();

    let check = coordinator.send("PUT", "/api/trainings/2026-10-15/checked", None);
    tokio::pin!(check);
    let waited = tokio::time::timeout(Duration::from_millis(300), &mut check).await;
    assert!(
        waited.is_err(),
        "the check did not wait for the other screen"
    );
    other.commit().await.unwrap();
    // After the undo, the training is after today and not checked: the rule refuses it.
    assert_eq!(check.await, (400, json!({ "error": "not_yet" })));
    let page = coordinator
        .ok("GET", "/api/trainings/2026-10-15", None)
        .await;
    assert_eq!(page["training"]["checked"], false);
}
