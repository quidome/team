//! The start page through the API (story B of start page and season): the open items come from
//! the rules of their sources, "Last" and "Next".

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::Harness;
use common::game_list::{DRAKES, M16_1, Reader, data, games_with, utc};
use serde_json::{Value, json};
use sqlx::PgPool;

async fn start(reader: &Reader) -> Value {
    let (status, body) = reader.send("GET", "/api/start", None).await;
    assert_eq!(status, 200, "{body}");
    body
}

async fn ok(reader: &Reader, method: &str, path: &str, body: Option<Value>) -> Value {
    let (status, answer) = reader.send(method, path, body).await;
    assert_eq!(status, 200, "{method} {path}: {answer}");
    answer
}

/// The open item of `kind`, if any.
fn item<'a>(answer: &'a Value, kind: &str) -> Option<&'a Value> {
    answer["open_items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["kind"] == kind)
}

fn kinds(answer: &Value) -> Vec<&str> {
    answer["open_items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["kind"].as_str().unwrap())
        .collect()
}

/// Tuesday trainings at 19:45 from `from` until `until` (the current season).
async fn add_tuesdays(reader: &Reader, from: &str, until: &str) {
    ok(
        reader,
        "POST",
        "/api/weekly-trainings",
        Some(json!({ "weekday": 2, "start": "19:45", "end": "21:00",
                     "from": from, "until": until })),
    )
    .await;
}

/// Moves the clock to that moment (UTC) and logs in again.
async fn at(reader: &mut Reader, year: i32, month: u32, day: u32, hour: u32) {
    reader.harness.clock.set(utc(year, month, day, hour, 0));
    reader.log_in_again().await;
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn without_login_the_start_page_needs_a_login(pool: PgPool) {
    let harness = Harness::start(pool).await;
    let response = harness.get("/api/start", None).await;
    assert_eq!(response.status(), 401);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_held_back_reading_is_an_open_item(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    assert_eq!(kinds(&start(&reader).await), Vec::<&str>::new());

    // A game list with no games is held back.
    let mut empty = data("games-empty.json");
    empty["seizoen"] = json!("2026-2027");
    reader.answer(empty);
    let (_, held) = reader.read().await;
    assert_eq!(held["result"], "held_back", "{held}");
    assert!(!reader.latest().await["held_back"].is_null());
    assert_eq!(kinds(&start(&reader).await), ["reading_held_back"]);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn out_of_date_is_an_open_item_with_the_dutch_date(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    // Three nightly run moments later, without a good reading.
    at(&mut reader, 2026, 10, 15, 12).await;
    let since = reader.latest().await["out_of_date_since"].clone();
    assert!(!since.is_null());
    assert_eq!(
        item(&start(&reader).await, "out_of_date"),
        Some(&json!({ "kind": "out_of_date", "since": "2026-10-12" }))
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn changes_not_seen_are_counted_as_the_list_of_changes(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let first = ok(&reader, "GET", "/api/season", None).await["coming"]
        .as_array()
        .unwrap()
        .iter()
        .find(|event| !event["game"].is_null())
        .unwrap()["game"]["id"]
        .clone();
    reader.answer(games_with(|games| games.retain(|game| game["id"] != first)));
    reader.read_good().await;

    let changes = ok(&reader, "GET", "/api/changes", None).await["changes"]
        .as_array()
        .unwrap()
        .len();
    assert!(changes > 0);
    assert_eq!(
        item(&start(&reader).await, "changes"),
        Some(&json!({ "kind": "changes", "count": changes }))
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn training_days_not_opened_are_those_of_attendance(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    add_tuesdays(&reader, "2026-09-01", "2027-05-25").await;
    ok(&reader, "PUT", "/api/trainings/2026-09-15/open", None).await;

    let attendance = ok(&reader, "GET", "/api/attendance", None).await;
    assert_eq!(
        item(&start(&reader).await, "not_opened"),
        Some(
            &json!({ "kind": "not_opened", "first": attendance["first_not_opened"],
                      "count": attendance["not_opened"] })
        )
    );
    assert_eq!(attendance["not_opened"], 5);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_past_training_not_checked_is_an_open_item_until_it_is_checked(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    add_tuesdays(&reader, "2026-09-01", "2027-05-25").await;
    ok(&reader, "PUT", "/api/trainings/2026-10-06/open", None).await;
    ok(&reader, "PUT", "/api/trainings/2026-09-29/open", None).await;
    // Today's training is not an open item yet.
    ok(&reader, "PUT", "/api/trainings/2026-10-13/open", None).await;
    assert_eq!(
        item(&start(&reader).await, "not_checked"),
        Some(&json!({ "kind": "not_checked", "first": "2026-09-29", "count": 2 }))
    );

    ok(&reader, "PUT", "/api/trainings/2026-09-29/checked", None).await;
    ok(&reader, "PUT", "/api/trainings/2026-10-06/checked", None).await;
    assert_eq!(item(&start(&reader).await, "not_checked"), None);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn last_is_the_newest_past_training_also_when_checked(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    add_tuesdays(&reader, "2026-09-01", "2027-05-25").await;
    ok(&reader, "PUT", "/api/trainings/2026-09-29/open", None).await;
    ok(&reader, "PUT", "/api/trainings/2026-10-06/open", None).await;
    ok(&reader, "PUT", "/api/trainings/2026-10-06/checked", None).await;
    // A cancelled training after it is skipped.
    ok(
        &reader,
        "POST",
        "/api/trainings",
        Some(json!({ "date": "2026-10-08", "start": "19:00", "end": "20:00" })),
    )
    .await;
    ok(&reader, "POST", "/api/trainings/2026-10-08/cancel", None).await;

    let last = start(&reader).await["last"]["training"].clone();
    assert_eq!(
        (&last["date"], &last["checked"], &last["status"]),
        (&json!("2026-10-06"), &json!(true), &json!("all_there"))
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn next_has_the_team_games_and_trainings_of_two_weeks(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    add_tuesdays(&reader, "2026-09-01", "2027-05-25").await;
    // Monday 26 October: until Sunday 8 November.
    at(&mut reader, 2026, 10, 26, 9).await;

    let next = start(&reader).await["next"].as_array().unwrap().clone();
    let dates: Vec<&str> = next
        .iter()
        .map(|event| event["date"].as_str().unwrap())
        .collect();
    assert!(
        dates
            .iter()
            .all(|date| ("2026-10-26"..="2026-11-08").contains(date)),
        "{dates:?}"
    );
    let mut sorted = dates.clone();
    sorted.sort_unstable();
    assert_eq!(dates, sorted);
    assert!(dates.contains(&"2026-10-27") && dates.contains(&"2026-11-03"));
    let games: Vec<&Value> = next
        .iter()
        .filter(|event| !event["game"].is_null())
        .collect();
    assert!(!games.is_empty());
    assert!(games.iter().all(|event| event["game"]["reason"] == "team"));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_new_season_without_a_team_has_trainings_only(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    at(&mut reader, 2027, 7, 1, 10).await;
    add_tuesdays(&reader, "2027-09-07", "2028-05-30").await;
    // A stored game of M16 1 in the new season, in the window: without a chosen team it must not
    // show.
    sqlx::query(
        "INSERT INTO games
             (club_number, season_year, source_id, game_number, date, start_time,
              home_team_number, home_team_name, home_club_number,
              away_team_number, away_team_name, away_club_number,
              place_number, court, competition, in_list, first_read_at)
         SELECT $1, 2027, 9000001, 'N1', '2027-07-05', '14:30', $2, 'Blue Drakes M16 1', $1,
                900, 'Rivals M16 1', 99, number, NULL, 'M16', true, now()
         FROM places LIMIT 1",
    )
    .bind(DRAKES)
    .bind(M16_1)
    .execute(reader.pool())
    .await
    .unwrap();

    let answer = start(&reader).await;
    // Nothing in two weeks: the first coming training, in September.
    let next = answer["next"].as_array().unwrap();
    assert_eq!(next.len(), 1);
    assert_eq!(next[0]["date"], "2027-09-07");
    assert!(next[0]["game"].is_null());
    assert_eq!(kinds(&answer), Vec::<&str>::new());

    // With the team chosen for the new season, the same stored game is in "Next": so the check
    // above can fail.
    sqlx::query(
        "INSERT INTO followed_teams
             (season, club_number, club_name, club_town, team_number, team_name, chosen_at)
         VALUES ('2027-2028', $1, 'Blue Drakes', NULL, $2, 'M16 1', now())",
    )
    .bind(DRAKES)
    .bind(M16_1)
    .execute(reader.pool())
    .await
    .unwrap();
    let next = start(&reader).await["next"].as_array().unwrap().clone();
    assert_eq!(next[0]["game"]["id"], 9000001, "{next:?}");
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn in_july_the_open_items_and_last_cover_the_season_before(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    add_tuesdays(&reader, "2026-09-01", "2027-05-25").await;
    ok(&reader, "PUT", "/api/trainings/2027-05-25/open", None).await;

    at(&mut reader, 2027, 7, 1, 10).await;
    let answer = start(&reader).await;
    let not_opened = item(&answer, "not_opened").unwrap();
    assert_eq!(not_opened["first"], "2026-09-01");
    assert_eq!(item(&answer, "not_checked").unwrap()["first"], "2027-05-25");
    assert_eq!(answer["last"]["training"]["date"], "2027-05-25");

    // From September the season before is no longer covered.
    at(&mut reader, 2027, 9, 1, 10).await;
    let answer = start(&reader).await;
    assert_eq!(item(&answer, "not_opened"), None);
    assert_eq!(item(&answer, "not_checked"), None);
    assert_eq!(answer["last"], Value::Null);
}
