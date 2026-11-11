//! Who comes to a game, through the API (story A of attendance at games).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::Harness;
use common::game_list::{DRAKES, M16_2, Reader, game_mut, games_with, utc};
use serde_json::{Value, json};
use sqlx::PgPool;

async fn ok(reader: &Reader, method: &str, path: &str, body: Option<Value>) -> Value {
    let (status, answer) = reader.send(method, path, body).await;
    assert_eq!(status, 200, "{method} {path}: {answer}");
    answer
}

async fn refused(reader: &Reader, method: &str, path: &str, body: Option<Value>, code: &str) {
    let (status, answer) = reader.send(method, path, body).await;
    assert_eq!(status, 400, "{method} {path}: {answer}");
    assert_eq!(answer["error"], code);
}

/// The games of the current season ("Season"), coming and past.
async fn season_games(reader: &Reader) -> Vec<Value> {
    let answer = ok(reader, "GET", "/api/season", None).await;
    ["coming", "past"]
        .iter()
        .flat_map(|part| answer[*part].as_array().unwrap().clone())
        .filter(|event| !event["game"].is_null())
        .map(|event| event["game"].clone())
        .collect()
}

/// The first coming game with `reason` ("team" or "duty").
async fn game_of(reader: &Reader, reason: &str) -> Value {
    season_games(reader)
        .await
        .into_iter()
        .find(|game| game["reason"] == reason)
        .unwrap()
}

async fn team_game(reader: &Reader) -> i64 {
    game_of(reader, "team").await["id"].as_i64().unwrap()
}

fn game_path(id: i64) -> String {
    format!("/api/games/2026-2027/{id}")
}

fn answer_path(id: i64, player: i64) -> String {
    format!("{}/answers/{player}", game_path(id))
}

async fn add_player(reader: &Reader, first: &str, role: &str, from: &str) -> i64 {
    let answer = ok(
        reader,
        "POST",
        "/api/players",
        Some(json!({ "first_name": first, "role": { "role": role, "from": from } })),
    )
    .await;
    answer["player"]["id"].as_i64().unwrap()
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

async fn set(reader: &Reader, id: i64, player: i64, answer: &str, reason: Option<i64>) -> Value {
    ok(
        reader,
        "PUT",
        &answer_path(id, player),
        Some(json!({ "answer": answer, "reason": reason })),
    )
    .await
}

async fn attendance(reader: &Reader, id: i64) -> Value {
    ok(reader, "GET", &game_path(id), None).await["attendance"].clone()
}

/// The stored date of the answers of a game.
async fn answers_for(reader: &Reader, id: i64) -> Option<chrono::NaiveDate> {
    sqlx::query_scalar("SELECT answers_for FROM game_attendance WHERE source_id = $1")
        .bind(id)
        .fetch_one(reader.pool())
        .await
        .unwrap()
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_page_of_a_game_of_the_team_has_the_list(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let cas = add_player(&reader, "Cas", "plays_games", "2026-09-01").await;
    let anna = add_player(&reader, "Anna", "plays_games", "2026-09-01").await;
    let bram = add_player(&reader, "Bram", "plays_games", "2026-09-01").await;
    add_player(&reader, "Dirk", "only_trains", "2026-09-01").await;
    let id = team_game(&reader).await;

    let list = attendance(&reader, id).await;
    let names: Vec<_> = list["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["Anna", "Bram", "Cas"]);
    assert_eq!(list["count"]["no_answer"], 3);
    assert_eq!(list["extra_players"], 0);

    set(&reader, id, anna, "not_coming", None).await;
    set(&reader, id, cas, "coming", None).await;
    let list = attendance(&reader, id).await;
    let rows: Vec<_> = list["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| (row["player"].as_i64().unwrap(), row["answer"].clone()))
        .collect();
    assert_eq!(
        rows,
        [
            (bram, Value::Null),
            (cas, json!("coming")),
            (anna, json!("not_coming")),
        ]
    );
    assert_eq!(
        list["count"],
        json!({ "coming": 1, "not_coming": 1, "no_answer": 1 })
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn another_game_has_no_list_and_refuses_a_tap(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let anna = add_player(&reader, "Anna", "plays_games", "2026-09-01").await;
    let id = game_of(&reader, "duty").await["id"].as_i64().unwrap();
    assert!(attendance(&reader, id).await.is_null());
    let body = json!({ "answer": "coming", "reason": null });
    let path = answer_path(id, anna);
    refused(&reader, "PUT", &path, Some(body), "not_a_team_game").await;
    refused(&reader, "DELETE", &path, None, "not_a_team_game").await;
    let extra = format!("{}/extra-players", game_path(id));
    let body = json!({ "extra_players": 1 });
    refused(&reader, "PUT", &extra, Some(body), "not_a_team_game").await;
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_tap_saves_and_a_delete_goes_back_to_no_answer(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let anna = add_player(&reader, "Anna", "plays_games", "2026-09-01").await;
    let id = team_game(&reader).await;

    let saved = set(&reader, id, anna, "coming", None).await;
    assert_eq!(saved["row"]["answer"], "coming");
    assert_eq!(saved["count"]["coming"], 1);
    // The last tap wins.
    let saved = set(&reader, id, anna, "not_coming", None).await;
    assert_eq!(saved["row"]["answer"], "not_coming");
    assert_eq!(saved["count"]["coming"], 0);
    assert_eq!(saved["count"]["not_coming"], 1);

    let path = answer_path(id, anna);
    let saved = ok(&reader, "DELETE", &path, None).await;
    assert!(saved["row"]["answer"].is_null());
    assert_eq!(saved["count"]["no_answer"], 1);
    // Safe to send twice.
    ok(&reader, "DELETE", &path, None).await;
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_reason_is_saved_changed_removed_and_kept_when_coming(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let anna = add_player(&reader, "Anna", "plays_games", "2026-09-01").await;
    let id = team_game(&reader).await;
    let school = add_reason(&reader, "Exam").await;
    let holiday = add_reason(&reader, "Trip").await;

    let saved = set(&reader, id, anna, "not_coming", Some(school)).await;
    assert_eq!(saved["row"]["reason"], school);
    let saved = set(&reader, id, anna, "not_coming", Some(holiday)).await;
    assert_eq!(saved["row"]["reason"], holiday);
    let saved = set(&reader, id, anna, "coming", Some(holiday)).await;
    assert_eq!(saved["row"]["answer"], "coming");
    assert_eq!(saved["row"]["reason"], holiday);
    let saved = set(&reader, id, anna, "not_coming", Some(holiday)).await;
    assert_eq!(saved["row"]["reason"], holiday);
    let saved = set(&reader, id, anna, "not_coming", None).await;
    assert!(saved["row"]["reason"].is_null());
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_tap_is_refused_off_the_list_and_with_a_hidden_reason(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let anna = add_player(&reader, "Anna", "plays_games", "2026-09-01").await;
    let dirk = add_player(&reader, "Dirk", "only_trains", "2026-09-01").await;
    let id = team_game(&reader).await;
    let coming = json!({ "answer": "coming", "reason": null });
    let path = answer_path(id, dirk);
    refused(&reader, "PUT", &path, Some(coming), "not_on_list").await;
    refused(&reader, "DELETE", &path, None, "not_on_list").await;

    let reason = add_reason(&reader, "Exam").await;
    set(&reader, id, anna, "not_coming", Some(reason)).await;
    ok(
        &reader,
        "PUT",
        &format!("/api/reasons/{reason}"),
        Some(json!({ "name": "Exam", "valid": true, "hidden": true })),
    )
    .await;
    // The reason the answer already has stays allowed, so "Coming" and back keeps working.
    let saved = set(&reader, id, anna, "coming", Some(reason)).await;
    assert_eq!(saved["row"]["reason"], reason);
    // A new answer cannot take the hidden reason.
    ok(&reader, "DELETE", &answer_path(id, anna), None).await;
    let body = json!({ "answer": "not_coming", "reason": reason });
    let path = answer_path(id, anna);
    refused(&reader, "PUT", &path, Some(body), "reason_hidden").await;
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn extra_players_are_0_to_10_and_add_to_coming(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let anna = add_player(&reader, "Anna", "plays_games", "2026-09-01").await;
    let id = team_game(&reader).await;
    set(&reader, id, anna, "coming", None).await;
    let path = format!("{}/extra-players", game_path(id));

    let saved = ok(&reader, "PUT", &path, Some(json!({ "extra_players": 10 }))).await;
    assert!(saved["row"].is_null());
    assert_eq!(saved["extra_players"], 10);
    assert_eq!(saved["count"]["coming"], 11);
    let saved = ok(&reader, "PUT", &path, Some(json!({ "extra_players": 0 }))).await;
    assert_eq!(saved["count"]["coming"], 1);
    for number in [11, -1, 300] {
        let body = json!({ "extra_players": number });
        refused(&reader, "PUT", &path, Some(body), "out_of_range").await;
    }
    assert_eq!(attendance(&reader, id).await["extra_players"], 0);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_first_answer_sets_the_date_of_the_answers(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let anna = add_player(&reader, "Anna", "plays_games", "2026-09-01").await;
    let bram = add_player(&reader, "Bram", "plays_games", "2026-09-01").await;
    let game = game_of(&reader, "team").await;
    let id = game["id"].as_i64().unwrap();
    let first = game["date"].as_str().unwrap().parse().unwrap();

    set(&reader, id, anna, "coming", None).await;
    assert_eq!(answers_for(&reader, id).await, Some(first));

    // The game moves one day: a later answer leaves the date.
    let moved = first + chrono::TimeDelta::days(1);
    reader.answer(games_with(|games| {
        game_mut(games, id)["datum"] = json!(moved.to_string());
    }));
    reader.read_good().await;
    set(&reader, id, bram, "coming", None).await;
    assert_eq!(answers_for(&reader, id).await, Some(first));

    // After all answers are removed, a new first answer sets the new date.
    ok(&reader, "DELETE", &answer_path(id, anna), None).await;
    ok(&reader, "DELETE", &answer_path(id, bram), None).await;
    set(&reader, id, anna, "coming", None).await;
    assert_eq!(answers_for(&reader, id).await, Some(moved));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_player_with_an_answer_cannot_be_deleted_but_can_be_hidden(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let anna = add_player(&reader, "Anna", "plays_games", "2026-09-01").await;
    let id = team_game(&reader).await;
    set(&reader, id, anna, "coming", None).await;

    let player = format!("/api/players/{anna}");
    assert_eq!(ok(&reader, "GET", &player, None).await["has_answers"], true);
    refused(&reader, "DELETE", &player, None, "player_has_answers").await;
    ok(
        &reader,
        "PUT",
        &player,
        Some(json!({ "first_name": "Anna", "last_name": null, "hidden": true })),
    )
    .await;
    // A hidden player stays on the list.
    assert_eq!(attendance(&reader, id).await["rows"][0]["hidden"], true);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_gone_game_with_answers_stays_reachable(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let anna = add_player(&reader, "Anna", "plays_games", "2026-09-01").await;
    let id = team_game(&reader).await;
    set(&reader, id, anna, "coming", None).await;

    reader.answer(games_with(|games| games.retain(|game| game["id"] != id)));
    reader.read_good().await;
    let changes = ok(&reader, "GET", "/api/changes", None).await;
    let change = changes["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|change| change["id"] == id)
        .unwrap()
        .clone();
    ok(
        &reader,
        "POST",
        "/api/changes/seen",
        Some(json!({ "changes": [{ "id": id, "version": change["version"] }] })),
    )
    .await;

    let game = season_games(&reader)
        .await
        .into_iter()
        .find(|game| game["id"] == id)
        .unwrap();
    assert_eq!(game["in_list"], false);
    // No count on the row; the page of the game keeps its answers.
    assert!(game["attendance"].is_null());
    assert_eq!(attendance(&reader, id).await["count"]["coming"], 1);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_game_no_longer_of_the_team_with_answers_stays_reachable(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let anna = add_player(&reader, "Anna", "plays_games", "2026-09-01").await;
    let id = season_games(&reader)
        .await
        .into_iter()
        .find(|game| {
            game["reason"] == "team" && !game["home"].as_str().unwrap().starts_with("Blue Drakes")
        })
        .unwrap()["id"]
        .as_i64()
        .unwrap();
    set(&reader, id, anna, "coming", None).await;

    reader.choose(M16_2).await;
    let (status, page) = reader.send("GET", &game_path(id), None).await;
    assert_eq!(status, 200, "{page}");
    assert_eq!(page["game"]["reason"], "kept");
    assert!(page["attendance"].is_null());
}

/// The count of the game `id` in the events `events` of an answer.
fn count_in(events: &Value, id: i64) -> Option<Value> {
    events
        .as_array()
        .unwrap()
        .iter()
        // A row of the start page holds the event as on "Season" under "event".
        .map(|row| {
            if row["event"].is_null() {
                row
            } else {
                &row["event"]
            }
        })
        .find(|event| event["game"]["id"] == id)
        .map(|event| event["game"]["attendance"].clone())
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_rows_give_the_count_of_the_page_of_the_game(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let anna = add_player(&reader, "Anna", "plays_games", "2026-09-01").await;
    add_player(&reader, "Bram", "plays_games", "2026-09-01").await;
    let start = ok(&reader, "GET", "/api/start", None).await;
    let id = start["next"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["event"]["game"]["reason"] == "team")
        .unwrap()["event"]["game"]["id"]
        .as_i64()
        .unwrap();

    // Without an answer: on the start page, not on "Season".
    assert_eq!(
        count_in(&start["next"], id).unwrap(),
        json!({ "coming": 0, "not_coming": 0, "no_answer": 2 })
    );
    let season = ok(&reader, "GET", "/api/season", None).await;
    assert!(count_in(&season["coming"], id).unwrap().is_null());

    set(&reader, id, anna, "coming", None).await;
    let page = attendance(&reader, id).await["count"].clone();
    assert_eq!(page["coming"], 1);
    let start = ok(&reader, "GET", "/api/start", None).await;
    assert_eq!(count_in(&start["next"], id).unwrap(), page);
    let season = ok(&reader, "GET", "/api/season", None).await;
    assert_eq!(count_in(&season["coming"], id).unwrap(), page);

    // An extra player alone also shows on "Season".
    ok(&reader, "DELETE", &answer_path(id, anna), None).await;
    let extra = format!("{}/extra-players", game_path(id));
    ok(&reader, "PUT", &extra, Some(json!({ "extra_players": 2 }))).await;
    let season = ok(&reader, "GET", "/api/season", None).await;
    assert_eq!(count_in(&season["coming"], id).unwrap()["coming"], 2);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_game_of_another_team_has_no_count_on_a_row(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    add_player(&reader, "Anna", "plays_games", "2026-09-01").await;
    let id = game_of(&reader, "duty").await["id"].as_i64().unwrap();
    let season = ok(&reader, "GET", "/api/season", None).await;
    assert!(count_in(&season["coming"], id).unwrap().is_null());
}

// The warnings in time (story B of attendance at games).

/// The away game of M16 1 on Saturday 7 November 2026 at 11:15 (Dutch time, UTC+1).
const AWAY_7_NOV: i64 = 1_073_159;

/// Moves the clock to that moment (UTC) and logs in again.
async fn at(reader: &mut Reader, month: u32, day: u32, hour: u32, minute: u32) {
    reader
        .harness
        .clock
        .set(utc(2026, month, day, hour, minute));
    reader.log_in_again().await;
}

/// The kinds of the open items of the start page.
async fn open_kinds(reader: &Reader) -> Vec<String> {
    ok(reader, "GET", "/api/start", None).await["open_items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["kind"].as_str().unwrap().to_owned())
        .collect()
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn too_few_on_the_page_starts_earlier_once_the_game_has_an_answer(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let mut players = Vec::new();
    for n in 0..7 {
        players.push(add_player(&reader, &format!("Player{n}"), "plays_games", "2026-09-01").await);
    }
    // Monday 2 November, 5 days before: no answer, no line.
    at(&mut reader, 11, 2, 9, 0).await;
    let page = attendance(&reader, AWAY_7_NOV).await;
    assert_eq!(
        (&page["too_few"], &page["enough_players"]),
        (&json!(false), &json!(7))
    );
    // The first answer: the tap gives the line, the start page has no item yet.
    let tap = set(&reader, AWAY_7_NOV, players[0], "coming", None).await;
    assert_eq!(
        (&tap["too_few"], &tap["enough_players"]),
        (&json!(true), &json!(7))
    );
    assert_eq!(attendance(&reader, AWAY_7_NOV).await["too_few"], true);
    assert!(
        !open_kinds(&reader)
            .await
            .contains(&"too_few_players".to_owned())
    );
    // In the period: both.
    at(&mut reader, 11, 5, 9, 0).await;
    assert_eq!(attendance(&reader, AWAY_7_NOV).await["too_few"], true);
    assert!(
        open_kinds(&reader)
            .await
            .contains(&"too_few_players".to_owned())
    );
    // 7 coming: neither; the tap that makes the 7 says so.
    for &player in &players[1..6] {
        set(&reader, AWAY_7_NOV, player, "coming", None).await;
    }
    let tap = set(&reader, AWAY_7_NOV, players[6], "coming", None).await;
    assert_eq!(tap["too_few"], false);
    assert!(
        !open_kinds(&reader)
            .await
            .contains(&"too_few_players".to_owned())
    );
    // One player not coming, then at the start time: neither.
    set(&reader, AWAY_7_NOV, players[6], "not_coming", None).await;
    assert_eq!(attendance(&reader, AWAY_7_NOV).await["too_few"], true);
    at(&mut reader, 11, 7, 10, 15).await;
    assert_eq!(attendance(&reader, AWAY_7_NOV).await["too_few"], false);
    assert!(
        !open_kinds(&reader)
            .await
            .contains(&"too_few_players".to_owned())
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn answers_from_before_a_move_show_until_answers_checked(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let anna = add_player(&reader, "Anna", "plays_games", "2026-09-01").await;
    let tap = set(&reader, AWAY_7_NOV, anna, "coming", None).await;
    assert_eq!(tap["answers_moved"], false);
    // The game moves from Saturday 7 to Wednesday 11 November.
    reader.answer(games_with(|games| {
        game_mut(games, AWAY_7_NOV)["datum"] = json!("2026-11-11 11:15:00.000");
    }));
    reader.read_good().await;
    // On the page at once, also long before the warning period (12 October).
    assert_eq!(attendance(&reader, AWAY_7_NOV).await["answers_moved"], true);
    // A new answer does not confirm the old ones.
    let bram = add_player(&reader, "Bram", "plays_games", "2026-09-01").await;
    let tap = set(&reader, AWAY_7_NOV, bram, "coming", None).await;
    assert_eq!(tap["answers_moved"], true);
    at(&mut reader, 11, 9, 9, 0).await;
    assert!(
        open_kinds(&reader)
            .await
            .contains(&"answers_moved".to_owned())
    );

    // "Answers checked": the stored date is the date now, the note and the item are gone.
    let path = format!("{}/answers-checked", game_path(AWAY_7_NOV));
    let first = ok(&reader, "PUT", &path, None).await;
    assert_eq!(first["answers_moved"], false);
    assert!(first["row"].is_null());
    assert_eq!(
        answers_for(&reader, AWAY_7_NOV).await,
        Some(chrono::NaiveDate::from_ymd_opt(2026, 11, 11).unwrap())
    );
    assert_eq!(
        attendance(&reader, AWAY_7_NOV).await["answers_moved"],
        false
    );
    assert!(
        !open_kinds(&reader)
            .await
            .contains(&"answers_moved".to_owned())
    );
    // A second tap gives the same answer.
    let second = ok(&reader, "PUT", &path, None).await;
    assert_eq!(first, second);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn answers_checked_is_only_for_a_game_of_the_team(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let id = game_of(&reader, "duty").await["id"].as_i64().unwrap();
    let path = format!("{}/answers-checked", game_path(id));
    refused(&reader, "PUT", &path, None, "not_a_team_game").await;
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn answers_checked_needs_a_login(pool: PgPool) {
    let harness = Harness::start(pool).await;
    let request = axum::http::Request::builder()
        .method("PUT")
        .uri(format!("/api/games/2026-2027/{AWAY_7_NOV}/answers-checked"))
        .header(axum::http::header::ORIGIN, common::ORIGIN)
        .body(axum::body::Body::empty())
        .unwrap();
    assert_eq!(harness.send(request).await.status(), 401);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn answers_checked_waits_for_a_first_save_on_another_screen(pool: PgPool) {
    let reader = Reader::chosen(pool.clone()).await;
    reader.read_good().await;
    // Another screen makes the row of the game at the same moment: not committed yet.
    let mut other = pool.begin().await.unwrap();
    sqlx::query(
        "INSERT INTO game_attendance (club_number, season_year, source_id, extra_players)
         VALUES ($1, 2026, $2, 2)",
    )
    .bind(DRAKES)
    .bind(AWAY_7_NOV)
    .execute(&mut *other)
    .await
    .unwrap();
    let path = format!("{}/answers-checked", game_path(AWAY_7_NOV));
    let checked = reader.send("PUT", &path, None);
    tokio::pin!(checked);
    let waited = tokio::time::timeout(std::time::Duration::from_millis(300), &mut checked).await;
    assert!(
        waited.is_err(),
        "the save did not wait for the other insert"
    );
    other.commit().await.unwrap();
    let (status, answer) = checked.await;
    assert_eq!(status, 200, "{answer}");
    // Both saves are kept: the extra players of the other screen and the date of the answers.
    assert_eq!(answer["extra_players"], 2);
    assert_eq!(
        answers_for(&reader, AWAY_7_NOV).await,
        Some(chrono::NaiveDate::from_ymd_opt(2026, 11, 7).unwrap())
    );
}
