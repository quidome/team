//! "Checked" of a duty game through the API (story 2 of "Checked for every event"): a home game of
//! another club team where the team has table or referee places can be marked as checked from its
//! start time, for its date, only while it is a duty game.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::game_list::{M16_1, M16_2, Reader};
use serde_json::{Value, json};
use sqlx::PgPool;

async fn ok(reader: &Reader, method: &str, path: &str, body: Option<Value>) -> Value {
    let (status, answer) = reader.send(method, path, body).await;
    assert_eq!(status, 200, "{method} {path}: {answer}");
    answer
}

async fn refused(reader: &Reader, method: &str, path: &str, code: &str) {
    let (status, answer) = reader.send(method, path, None).await;
    assert_eq!(
        (status, answer["error"].as_str()),
        (400, Some(code)),
        "{path}"
    );
}

fn game_path(id: i64) -> String {
    format!("/api/games/2026-2027/{id}")
}

fn checked_path(id: i64) -> String {
    format!("{}/checked", game_path(id))
}

/// The first home game of another club team on `date` (the clock is 12 October 2026).
async fn duty_game(reader: &Reader, part: &str, date: &str) -> i64 {
    let season = ok(reader, "GET", "/api/season?season=2026-2027", None).await;
    season[part]
        .as_array()
        .unwrap()
        .iter()
        .map(|event| &event["game"])
        .find(|game| game["reason"] == "duty" && game["date"] == date)
        .unwrap()["id"]
        .as_i64()
        .unwrap()
}

async fn give_place(reader: &Reader, id: i64, team: i64) {
    ok(
        reader,
        "PUT",
        &format!("{}/places/referee/1", game_path(id)),
        Some(json!({ "team": team })),
    )
    .await;
}

/// The row of game `id` on "Season" is "· Checked".
async fn checked_on_season(reader: &Reader, id: i64) -> bool {
    let season = ok(reader, "GET", "/api/season?season=2026-2027", None).await;
    ["coming", "past"]
        .iter()
        .flat_map(|part| season[*part].as_array().unwrap().clone())
        .find(|event| event["game"]["id"] == id)
        .unwrap()["game"]["checked"]
        .as_bool()
        .unwrap()
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_duty_game_is_checked_from_its_start_time_and_undone(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let id = duty_game(&reader, "past", "2026-10-03").await;
    give_place(&reader, id, M16_1).await;
    let page = ok(&reader, "GET", &game_path(id), None).await;
    assert_eq!(page["duty_checked"], false);
    assert!(page["attendance"].is_null(), "a duty game has no list");

    let answer = ok(&reader, "PUT", &checked_path(id), None).await;
    assert_eq!(answer["settled"], 0);
    assert_eq!(answer["page"]["duty_checked"], true);
    assert!(checked_on_season(&reader, id).await);
    // A second check changes nothing.
    ok(&reader, "PUT", &checked_path(id), None).await;

    let page = ok(&reader, "DELETE", &checked_path(id), None).await;
    assert_eq!(page["duty_checked"], false);
    assert!(!checked_on_season(&reader, id).await);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_duty_check_is_refused_too_early_when_not_played_and_without_places(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    // Before the start time.
    let coming = duty_game(&reader, "coming", "2026-10-31").await;
    give_place(&reader, coming, M16_1).await;
    refused(&reader, "PUT", &checked_path(coming), "checked_too_early").await;

    // A home game of another club team without places of the team has no check.
    let past = duty_game(&reader, "past", "2026-10-03").await;
    let page = ok(&reader, "GET", &game_path(past), None).await;
    assert!(page["duty_checked"].is_null());
    refused(&reader, "PUT", &checked_path(past), "not_checkable").await;

    // A duty game marked "Not played" needs no check; its "Undo" is still allowed.
    give_place(&reader, past, M16_1).await;
    ok(
        &reader,
        "PUT",
        &format!("{}/not-played", game_path(past)),
        None,
    )
    .await;
    refused(&reader, "PUT", &checked_path(past), "duty_game_not_played").await;
    ok(&reader, "DELETE", &checked_path(past), None).await;
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_duty_check_holds_only_for_its_date_and_its_kind(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let id = duty_game(&reader, "past", "2026-10-03").await;
    give_place(&reader, id, M16_1).await;
    ok(&reader, "PUT", &checked_path(id), None).await;

    // The game moved a day earlier: the check has no effect.
    sqlx::query("UPDATE games SET date = date - 1 WHERE source_id = $1")
        .bind(id)
        .execute(reader.pool())
        .await
        .unwrap();
    let page = ok(&reader, "GET", &game_path(id), None).await;
    assert_eq!(page["duty_checked"], false);
    // Back on its date, the check holds again.
    sqlx::query("UPDATE games SET date = date + 1 WHERE source_id = $1")
        .bind(id)
        .execute(reader.pool())
        .await
        .unwrap();
    let page = ok(&reader, "GET", &game_path(id), None).await;
    assert_eq!(page["duty_checked"], true);

    // Another team chosen: the place no longer names the team, so the game has no duty check.
    reader.choose(M16_2).await;
    let page = ok(&reader, "GET", &game_path(id), None).await;
    assert!(page["duty_checked"].is_null());
    assert!(!checked_on_season(&reader, id).await);
}

/// The team number of the home team of game `id`, as stored.
async fn home_team(reader: &Reader, id: i64) -> i64 {
    sqlx::query_scalar("SELECT home_team_number FROM games WHERE source_id = $1")
        .bind(id)
        .fetch_one(reader.pool())
        .await
        .unwrap()
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_check_from_a_page_of_another_kind_is_refused(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let id = duty_game(&reader, "past", "2026-10-03").await;
    give_place(&reader, id, M16_1).await;
    // A page that still shows a team game (another team was chosen meanwhile).
    refused(
        &reader,
        "PUT",
        &format!("{}?kind=team", checked_path(id)),
        "game_changed",
    )
    .await;
    refused(
        &reader,
        "DELETE",
        &format!("{}?kind=team", checked_path(id)),
        "game_changed",
    )
    .await;
    ok(
        &reader,
        "PUT",
        &format!("{}?kind=duty", checked_path(id)),
        None,
    )
    .await;
    let page = ok(&reader, "GET", &game_path(id), None).await;
    assert_eq!(page["duty_checked"], true);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_duty_game_that_becomes_a_team_game_loses_its_duty_check(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let id = duty_game(&reader, "past", "2026-10-03").await;
    give_place(&reader, id, M16_1).await;
    ok(&reader, "PUT", &checked_path(id), None).await;
    // The home team of the game is chosen: it is now a team game, with its own check.
    reader.choose(home_team(&reader, id).await).await;
    let page = ok(&reader, "GET", &game_path(id), None).await;
    assert_eq!(page["game"]["reason"], "team");
    assert!(page["duty_checked"].is_null());
    assert!(!checked_on_season(&reader, id).await);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_duty_game_kept_for_its_duties_can_be_checked(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let id = duty_game(&reader, "past", "2026-10-03").await;
    give_place(&reader, id, M16_1).await;
    // It left the game list after it was played: kept for its duties.
    sqlx::query("UPDATE games SET in_list = false WHERE source_id = $1")
        .bind(id)
        .execute(reader.pool())
        .await
        .unwrap();
    let page = ok(&reader, "GET", &game_path(id), None).await;
    assert_eq!(page["duty_checked"], false, "{page}");
    let answer = ok(&reader, "PUT", &checked_path(id), None).await;
    assert_eq!(answer["page"]["duty_checked"], true);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_second_check_keeps_the_first_moment(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let id = duty_game(&reader, "past", "2026-10-03").await;
    give_place(&reader, id, M16_1).await;
    let moment = async || -> Option<chrono::DateTime<chrono::Utc>> {
        sqlx::query_scalar("SELECT checked_at FROM game_duties WHERE source_id = $1")
            .bind(id)
            .fetch_one(reader.pool())
            .await
            .unwrap()
    };
    ok(&reader, "PUT", &checked_path(id), None).await;
    let first = moment().await;
    assert!(first.is_some());
    reader
        .harness
        .clock
        .set(common::game_list::utc(2026, 10, 12, 13, 0));
    ok(&reader, "PUT", &checked_path(id), None).await;
    assert_eq!(moment().await, first);
    ok(&reader, "DELETE", &checked_path(id), None).await;
    assert_eq!(moment().await, None);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_checked_duty_game_marked_not_played_is_not_checked(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let id = duty_game(&reader, "past", "2026-10-03").await;
    give_place(&reader, id, M16_1).await;
    ok(&reader, "PUT", &checked_path(id), None).await;
    ok(
        &reader,
        "PUT",
        &format!("{}/not-played", game_path(id)),
        None,
    )
    .await;
    assert!(!checked_on_season(&reader, id).await);
    let page = ok(&reader, "GET", &game_path(id), None).await;
    assert_eq!(page["duty_checked"], false);
    // "Undo" of "Not played": the stored check holds again.
    ok(
        &reader,
        "DELETE",
        &format!("{}/not-played", game_path(id)),
        None,
    )
    .await;
    assert!(checked_on_season(&reader, id).await);
}

/// The ids of the games of the open item `kind` of the start page, oldest first.
async fn item_games(reader: &Reader, kind: &str) -> Vec<i64> {
    let start = ok(reader, "GET", "/api/start", None).await;
    start["open_items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["kind"] == kind)
        .map(|item| {
            item["games"]
                .as_array()
                .unwrap()
                .iter()
                .map(|game| game["id"].as_i64().unwrap())
                .collect()
        })
        .unwrap_or_default()
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_past_duty_game_that_is_not_checked_is_an_open_item(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    // Without places of the team no game is a duty game.
    assert!(
        item_games(&reader, "duty_games_not_checked")
            .await
            .is_empty()
    );
    let id = duty_game(&reader, "past", "2026-10-03").await;
    give_place(&reader, id, M16_1).await;
    assert_eq!(item_games(&reader, "duty_games_not_checked").await, [id]);
    // Each game of the item has what its line on the start page needs.
    let start = ok(&reader, "GET", "/api/start", None).await;
    let item = start["open_items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["kind"] == "duty_games_not_checked")
        .unwrap()
        .clone();
    let game = &item["games"][0];
    assert_eq!(game["season"], "2026-2027");
    assert_eq!(game["date"], "2026-10-03");
    assert!(game["home"].as_str().unwrap().starts_with("Blue Drakes"));
    // The items come in this order, at the end.
    let kinds: Vec<&str> = start["open_items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["kind"].as_str().unwrap())
        .collect();
    assert_eq!(
        &kinds[kinds.len() - 2..],
        ["games_not_checked", "duty_games_not_checked"]
    );

    ok(&reader, "PUT", &checked_path(id), None).await;
    assert!(
        item_games(&reader, "duty_games_not_checked")
            .await
            .is_empty()
    );
    ok(&reader, "DELETE", &checked_path(id), None).await;
    ok(
        &reader,
        "PUT",
        &format!("{}/not-played", game_path(id)),
        None,
    )
    .await;
    assert!(
        item_games(&reader, "duty_games_not_checked")
            .await
            .is_empty()
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn in_july_the_duty_games_of_the_season_before_are_an_open_item(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let id = duty_game(&reader, "past", "2026-10-03").await;
    give_place(&reader, id, M16_1).await;
    // July 2027: the new season has no team yet; the open items also cover the season before.
    reader
        .harness
        .clock
        .set(common::game_list::utc(2027, 7, 2, 10, 0));
    reader.log_in_again().await;
    let start = ok(&reader, "GET", "/api/start", None).await;
    let item = start["open_items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["kind"] == "duty_games_not_checked")
        .unwrap_or_else(|| panic!("{start}"))
        .clone();
    let games = item["games"].as_array().unwrap();
    assert_eq!(games.len(), 1, "{item}");
    assert_eq!(games[0]["id"], id);
    assert_eq!(games[0]["season"], "2026-2027");
    // From September the season before is no longer covered.
    reader
        .harness
        .clock
        .set(common::game_list::utc(2027, 9, 1, 10, 0));
    reader.log_in_again().await;
    assert!(
        item_games(&reader, "duty_games_not_checked")
            .await
            .is_empty()
    );
}
