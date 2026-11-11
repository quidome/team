//! "Not played" on the page of a game through the API (story E of attendance at games): the mark
//! holds for the date of the game, any game whose page opens can be marked, and a reading never
//! sets or clears it.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use chrono::NaiveDate;
use common::Harness;
use common::game_list::{Reader, game_mut, games_with};
use serde_json::{Value, json};
use sqlx::PgPool;

/// The away game of M16 1 on Saturday 7 November 2026 at 11:15.
const AWAY_7_NOV: i64 = 1_073_159;

async fn ok(reader: &Reader, method: &str, path: &str, body: Option<Value>) -> Value {
    let (status, answer) = reader.send(method, path, body).await;
    assert_eq!(status, 200, "{method} {path}: {answer}");
    answer
}

fn game_path(id: i64) -> String {
    format!("/api/games/2026-2027/{id}")
}

fn mark_path(id: i64) -> String {
    format!("{}/not-played", game_path(id))
}

/// The row of game `id` on "Season", if it is there.
async fn season_row(reader: &Reader, id: i64) -> Option<Value> {
    let answer = ok(reader, "GET", "/api/season", None).await;
    ["coming", "past"]
        .iter()
        .flat_map(|part| answer[*part].as_array().unwrap().clone())
        .map(|event| event["game"].clone())
        .find(|game| game["id"] == id)
}

/// The stored mark of game `id`: the date it holds for.
async fn stored_mark(reader: &Reader, id: i64) -> Option<NaiveDate> {
    sqlx::query_scalar::<_, NaiveDate>(
        "SELECT not_played_for FROM games_not_played WHERE season_year = 2026 AND source_id = $1",
    )
    .bind(id)
    .fetch_optional(&reader.harness.pool)
    .await
    .unwrap()
}

/// A home game of another club team without places of the team.
async fn duty_game(reader: &Reader) -> i64 {
    let answer = ok(reader, "GET", "/api/season", None).await;
    answer["coming"]
        .as_array()
        .unwrap()
        .iter()
        .map(|event| &event["game"])
        .find(|game| game["reason"] == "duty")
        .unwrap()["id"]
        .as_i64()
        .unwrap()
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_mark_needs_a_login(pool: PgPool) {
    let harness = Harness::start(pool).await;
    for method in ["PUT", "DELETE"] {
        let request = axum::http::Request::builder()
            .method(method)
            .uri(mark_path(AWAY_7_NOV))
            .header(axum::http::header::ORIGIN, common::ORIGIN)
            .body(axum::body::Body::empty())
            .unwrap();
        let response = harness.send(request).await;
        assert_eq!(response.status(), 401, "{method}");
    }
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn not_played_holds_for_the_date_of_the_game_and_undo_removes_it(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let page = ok(&reader, "GET", &game_path(AWAY_7_NOV), None).await;
    assert_eq!(page["not_played"], false);

    let page = ok(&reader, "PUT", &mark_path(AWAY_7_NOV), None).await;
    assert_eq!(page["not_played"], true);
    assert_eq!(
        stored_mark(&reader, AWAY_7_NOV).await,
        NaiveDate::from_ymd_opt(2026, 11, 7)
    );
    let row = season_row(&reader, AWAY_7_NOV).await.unwrap();
    assert_eq!(row["not_played"], true);

    let page = ok(&reader, "DELETE", &mark_path(AWAY_7_NOV), None).await;
    assert_eq!(page["not_played"], false);
    assert_eq!(stored_mark(&reader, AWAY_7_NOV).await, None);
    let row = season_row(&reader, AWAY_7_NOV).await.unwrap();
    assert_eq!(row["not_played"], false);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_last_tap_wins(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    ok(&reader, "PUT", &mark_path(AWAY_7_NOV), None).await;
    ok(&reader, "PUT", &mark_path(AWAY_7_NOV), None).await;
    ok(&reader, "DELETE", &mark_path(AWAY_7_NOV), None).await;
    ok(&reader, "DELETE", &mark_path(AWAY_7_NOV), None).await;
    let page = ok(&reader, "PUT", &mark_path(AWAY_7_NOV), None).await;
    assert_eq!(page["not_played"], true);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn any_game_whose_page_opens_can_be_marked(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let id = duty_game(&reader).await;
    let page = ok(&reader, "GET", &game_path(id), None).await;
    assert!(
        page["game"]["places"]["schedule"]["of_team"]
            .as_array()
            .is_none_or(Vec::is_empty)
    );
    let page = ok(&reader, "PUT", &mark_path(id), None).await;
    assert_eq!(page["not_played"], true);
    // A game that is not stored does not open.
    let (status, _) = reader.send("PUT", &mark_path(1), None).await;
    assert_eq!(status, 404);
    let (status, _) = reader
        .send("PUT", "/api/games/2026-2027/x/not-played", None)
        .await;
    assert_eq!(status, 400);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_reading_never_sets_or_clears_the_mark(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    ok(&reader, "PUT", &mark_path(AWAY_7_NOV), None).await;
    let marked = NaiveDate::from_ymd_opt(2026, 11, 7);

    // The game leaves the game list: the mark stays.
    reader.answer(games_with(|games| {
        games.retain(|game| game["id"] != AWAY_7_NOV);
    }));
    reader.read_good().await;
    assert_eq!(stored_mark(&reader, AWAY_7_NOV).await, marked);
    let page = ok(&reader, "GET", &game_path(AWAY_7_NOV), None).await;
    assert_eq!(
        (page["game"]["in_list"].clone(), page["not_played"].clone()),
        (json!(false), json!(true))
    );

    // It comes back on another date: a normal game of its new date, the row still stored.
    reader.answer(games_with(|games| {
        game_mut(games, AWAY_7_NOV)["datum"] = json!("2026-11-14 11:15:00.000");
    }));
    reader.read_good().await;
    assert_eq!(stored_mark(&reader, AWAY_7_NOV).await, marked);
    let page = ok(&reader, "GET", &game_path(AWAY_7_NOV), None).await;
    assert_eq!(page["game"]["date"], "2026-11-14");
    assert_eq!(page["not_played"], false);

    // Back on the date of the mark: not played again.
    reader.answer(games_with(|_| {}));
    reader.read_good().await;
    let page = ok(&reader, "GET", &game_path(AWAY_7_NOV), None).await;
    assert_eq!(page["not_played"], true);

    // A new tap gives the mark the date of the game now.
    reader.answer(games_with(|games| {
        game_mut(games, AWAY_7_NOV)["datum"] = json!("2026-11-14 11:15:00.000");
    }));
    reader.read_good().await;
    let page = ok(&reader, "PUT", &mark_path(AWAY_7_NOV), None).await;
    assert_eq!(page["not_played"], true);
    assert_eq!(
        stored_mark(&reader, AWAY_7_NOV).await,
        NaiveDate::from_ymd_opt(2026, 11, 14)
    );
}

/// The away game of M16 1 on Saturday 10 October 2026 at 15:00, before the test clock.
const AWAY_10_OCT: i64 = 1_073_147;

/// The drive totals of `player` on the page "Duties".
async fn drive_total(reader: &Reader, player: i64) -> Value {
    let duties = ok(reader, "GET", "/api/duties", None).await;
    duties["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["player"] == player)
        .map_or(json!(null), |row| row["totals"]["drive"].clone())
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_not_played_game_counts_its_drives_only_with_the_switch(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let player = ok(
        &reader,
        "POST",
        "/api/players",
        Some(json!({ "first_name": "Anna", "last_name": null,
                     "role": { "role": "plays_games", "from": "2026-09-01" } })),
    )
    .await["player"]["id"]
        .as_i64()
        .unwrap();
    let game = game_path(AWAY_10_OCT);
    ok(
        &reader,
        "PUT",
        &format!("{game}/drivers"),
        Some(json!({ "drivers": 1 })),
    )
    .await;
    ok(
        &reader,
        "PUT",
        &format!("{game}/assignments/drive/1"),
        Some(json!({ "player": player })),
    )
    .await;
    let played = drive_total(&reader, player).await;
    assert_eq!(played, json!({ "done": 1, "planned": 0 }));

    let page = ok(&reader, "PUT", &mark_path(AWAY_10_OCT), None).await;
    assert_eq!(page["drivers_drove"], false);
    assert_eq!(
        drive_total(&reader, player).await,
        json!({ "done": 0, "planned": 0 })
    );

    let path = format!("{game}/drivers-drove");
    let page = ok(&reader, "PUT", &path, None).await;
    assert_eq!(page["drivers_drove"], true);
    assert_eq!(drive_total(&reader, player).await, played);

    // "Undo" keeps the switch; it has effect again after a new "Not played".
    let page = ok(&reader, "DELETE", &mark_path(AWAY_10_OCT), None).await;
    assert_eq!(page["drivers_drove"], true);
    assert_eq!(drive_total(&reader, player).await, played);
    ok(&reader, "DELETE", &path, None).await;
    assert_eq!(
        drive_total(&reader, player).await,
        played,
        "the switch alone changes nothing"
    );
    ok(&reader, "PUT", &path, None).await;
    ok(&reader, "PUT", &mark_path(AWAY_10_OCT), None).await;
    assert_eq!(drive_total(&reader, player).await, played);
    let page = ok(&reader, "DELETE", &path, None).await;
    assert_eq!(page["drivers_drove"], false);
    assert_eq!(
        drive_total(&reader, player).await,
        json!({ "done": 0, "planned": 0 })
    );
}

/// The game `id` among the events of `part` of the start page.
fn start_game(start: &Value, part: &str, id: i64) -> Option<Value> {
    start[part]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["event"]["game"].clone())
        .find(|game| game["id"] == id)
}

/// The kinds of the open items of the start page that name game `id`.
async fn open_items_of(reader: &Reader, id: i64) -> Vec<String> {
    let start = ok(reader, "GET", "/api/start", None).await;
    start["open_items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| item.to_string().contains(&id.to_string()))
        .map(|item| item["kind"].as_str().unwrap().to_owned())
        .collect()
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_not_played_game_keeps_its_places_but_asks_for_no_driver(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let game = game_path(AWAY_7_NOV);
    ok(
        &reader,
        "PUT",
        &format!("{game}/drivers"),
        Some(json!({ "drivers": 2 })),
    )
    .await;
    // Thursday 5 November: the driver places of Saturday are in the window of the open items.
    reader
        .harness
        .clock
        .set(common::game_list::utc(2026, 11, 5, 9, 0));
    reader.log_in_again().await;
    let before = open_items_of(&reader, AWAY_7_NOV).await;
    assert!(before.contains(&"open_places".to_owned()), "{before:?}");

    let page = ok(&reader, "PUT", &mark_path(AWAY_7_NOV), None).await;
    let drivers = &page["game"]["places"]["drivers"];
    assert_eq!(drivers["places"].as_array().unwrap().len(), 2, "{drivers}");
    assert!(open_items_of(&reader, AWAY_7_NOV).await.is_empty());

    // The game day after the start time (11:15), with "The drivers drove" on: the drives count
    // in the totals, but the game is still no open item and in no overview (R2).
    reader
        .harness
        .clock
        .set(common::game_list::utc(2026, 11, 7, 12, 0));
    reader.log_in_again().await;
    let page = ok(&reader, "PUT", &format!("{game}/drivers-drove"), None).await;
    assert_eq!(
        (page["started"].clone(), page["drivers_drove"].clone()),
        (json!(true), json!(true))
    );
    assert!(open_items_of(&reader, AWAY_7_NOV).await.is_empty());
    let driving = ok(&reader, "GET", "/api/messages/driving", None).await;
    assert!(
        !driving["text"].as_str().unwrap().contains("7 nov"),
        "{driving}"
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_game_without_a_start_time_has_started_when_its_day_begins(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    // The source writes 00:00 when the start time is not known.
    reader.answer(games_with(|games| {
        game_mut(games, AWAY_7_NOV)["tijd"] = json!("00:00");
    }));
    reader.read_good().await;
    // 23:30 Dutch time on Friday 6 November (UTC+1).
    reader
        .harness
        .clock
        .set(common::game_list::utc(2026, 11, 6, 22, 30));
    reader.log_in_again().await;
    let page = ok(&reader, "GET", &game_path(AWAY_7_NOV), None).await;
    assert_eq!(page["game"]["start_time"], json!(null));
    assert_eq!(page["started"], false);
    // 00:30 on Saturday 7 November.
    reader
        .harness
        .clock
        .set(common::game_list::utc(2026, 11, 6, 23, 30));
    reader.log_in_again().await;
    let page = ok(&reader, "GET", &game_path(AWAY_7_NOV), None).await;
    assert_eq!(page["started"], true);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn not_played_waits_for_a_mark_of_the_same_game_on_another_screen(pool: PgPool) {
    let reader = Reader::chosen(pool.clone()).await;
    reader.read_good().await;
    // Another screen marks the game at the same moment: not committed yet.
    let mut other = pool.begin().await.unwrap();
    sqlx::query(
        "INSERT INTO games_not_played (club_number, season_year, source_id, not_played_for)
         SELECT club_number, season_year, source_id, date - 7 FROM games
         WHERE season_year = 2026 AND source_id = $1",
    )
    .bind(AWAY_7_NOV)
    .execute(&mut *other)
    .await
    .unwrap();
    let path = mark_path(AWAY_7_NOV);
    let mark = reader.send("PUT", &path, None);
    tokio::pin!(mark);
    let waited = tokio::time::timeout(std::time::Duration::from_millis(300), &mut mark).await;
    assert!(
        waited.is_err(),
        "the save did not wait for the other insert"
    );
    other.commit().await.unwrap();
    let (status, page) = mark.await;
    assert_eq!(status, 200, "{page}");
    // The last tap wins: the mark holds for the date of the game now.
    assert_eq!(page["not_played"], true);
    assert_eq!(
        stored_mark(&reader, AWAY_7_NOV).await,
        NaiveDate::from_ymd_opt(2026, 11, 7)
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_not_played_game_warns_about_nothing_and_is_in_no_message(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let game = game_path(AWAY_7_NOV);
    let player = ok(
        &reader,
        "POST",
        "/api/players",
        Some(json!({ "first_name": "Anna", "last_name": null,
                     "role": { "role": "plays_games", "from": "2026-09-01" } })),
    )
    .await["player"]["id"]
        .as_i64()
        .unwrap();
    ok(
        &reader,
        "PUT",
        &format!("{game}/drivers"),
        Some(json!({ "drivers": 1 })),
    )
    .await;
    ok(
        &reader,
        "PUT",
        &format!("{game}/assignments/drive/1"),
        Some(json!({ "player": player })),
    )
    .await;
    // Thursday 5 November: in the warning period, 0 of 7 coming, the driver not coming.
    reader
        .harness
        .clock
        .set(common::game_list::utc(2026, 11, 5, 9, 0));
    reader.log_in_again().await;
    ok(
        &reader,
        "PUT",
        &format!("{game}/answers/{player}"),
        Some(json!({ "answer": "not_coming" })),
    )
    .await;
    let kinds = |start: &Value| -> Vec<String> {
        start["open_items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["kind"].as_str().unwrap().to_owned())
            .collect()
    };
    let start = ok(&reader, "GET", "/api/start", None).await;
    assert!(
        kinds(&start).contains(&"too_few_players".to_owned()),
        "{start}"
    );
    assert!(
        kinds(&start).contains(&"driver_not_coming".to_owned()),
        "{start}"
    );
    let week = ok(&reader, "GET", "/api/messages/week", None).await;
    assert!(
        week["text"].as_str().unwrap().contains("Rijden: Anna"),
        "{week}"
    );

    let page = ok(&reader, "PUT", &mark_path(AWAY_7_NOV), None).await;
    assert_eq!(page["attendance"]["too_few"], false);
    let start = ok(&reader, "GET", "/api/start", None).await;
    let kinds = kinds(&start);
    assert!(!kinds.contains(&"too_few_players".to_owned()), "{kinds:?}");
    assert!(
        !kinds.contains(&"driver_not_coming".to_owned()),
        "{kinds:?}"
    );
    assert!(!kinds.contains(&"open_places".to_owned()), "{kinds:?}");
    // It stays in "Next", as not played.
    let next = start_game(&start, "next", AWAY_7_NOV).unwrap();
    assert_eq!(next["not_played"], true);
    // It leaves the message of the week and the driving overview.
    let week = ok(&reader, "GET", "/api/messages/week", None).await;
    assert!(
        week["text"]
            .as_str()
            .is_none_or(|text| !text.contains("Rijden") && !text.contains("Uit tegen")),
        "{week}"
    );
    let driving = ok(&reader, "GET", "/api/messages/driving", None).await;
    assert!(
        !driving["text"].as_str().unwrap().contains("Anna"),
        "{driving}"
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_not_played_duty_game_stays_in_the_last_duty_games_and_on_season(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    // A past home game of another club team on Sat 3 Oct, with a referee place of the team.
    let season = ok(&reader, "GET", "/api/season?season=2026-2027", None).await;
    let id = season["past"]
        .as_array()
        .unwrap()
        .iter()
        .map(|event| &event["game"])
        .find(|game| game["reason"] == "duty" && game["date"] == "2026-10-03")
        .unwrap()["id"]
        .as_i64()
        .unwrap();
    ok(
        &reader,
        "PUT",
        &format!("{}/places/referee/1", game_path(id)),
        Some(json!({ "team": common::game_list::M16_1 })),
    )
    .await;
    let start = ok(&reader, "GET", "/api/start", None).await;
    let last = start_game(&start, "last_games", id).unwrap();
    assert_eq!(last["not_played"], false);
    assert_eq!(look(&start, "last_games", id), "normal", "an open place");

    ok(&reader, "PUT", &mark_path(id), None).await;
    let start = ok(&reader, "GET", "/api/start", None).await;
    let last = start_game(&start, "last_games", id).unwrap();
    assert_eq!(last["not_played"], true);
    assert_eq!(look(&start, "last_games", id), "lighter");
    let row = season_row(&reader, id).await.unwrap();
    assert_eq!(row["not_played"], true);
}

/// How the row of game `id` in `part` of the start page looks.
fn look(start: &Value, part: &str, id: i64) -> String {
    start[part]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["event"]["game"]["id"] == id)
        .map(|item| item["look"].as_str().unwrap().to_owned())
        .unwrap()
}
