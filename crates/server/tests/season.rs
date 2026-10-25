//! The page "Season" through the API (story A of start page and season): the events of a season,
//! games and trainings in one list with their weeks, and the seasons to choose.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::Harness;
use common::game_list::{M16_1, Reader, game_mut, games_with, utc};
use serde_json::{Value, json};
use sqlx::PgPool;

async fn season(reader: &Reader, query: &str) -> Value {
    let (status, body) = reader
        .send("GET", &format!("/api/season{query}"), None)
        .await;
    assert_eq!(status, 200, "{body}");
    body
}

fn events(answer: &Value, part: &str) -> Vec<Value> {
    answer[part].as_array().unwrap().clone()
}

fn trainings(events: &[Value]) -> Vec<&Value> {
    events
        .iter()
        .filter(|event| !event["training"].is_null())
        .collect()
}

fn games(events: &[Value]) -> Vec<&Value> {
    events
        .iter()
        .filter(|event| !event["game"].is_null())
        .collect()
}

/// Tuesday trainings at 19:45 from 1 September until 25 May of the current season.
async fn add_tuesdays(reader: &Reader, from: &str, until: &str) {
    let (status, body) = reader
        .send(
            "POST",
            "/api/weekly-trainings",
            Some(json!({ "weekday": 2, "start": "19:45", "end": "21:00",
                         "from": from, "until": until })),
        )
        .await;
    assert_eq!(status, 200, "{body}");
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn without_login_the_season_needs_a_login(pool: PgPool) {
    let harness = Harness::start(pool).await;
    let response = harness.get("/api/season", None).await;
    assert_eq!(response.status(), 401);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_current_season_has_games_and_trainings_in_order_with_their_weeks(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    add_tuesdays(&reader, "2026-09-01", "2027-05-25").await;
    // An extra training on Saturday 31 October at 15:00, between the games of that day.
    let (status, body) = reader
        .send(
            "POST",
            "/api/trainings",
            Some(json!({ "date": "2026-10-31", "start": "15:00", "end": "16:00" })),
        )
        .await;
    assert_eq!(status, 200, "{body}");

    let answer = season(&reader, "").await;
    assert_eq!(answer["season"], "2026-2027");
    assert_eq!(answer["current_season"], "2026-2027");
    assert_eq!(answer["today"], "2026-10-12");
    assert_eq!(answer["games"], "shown");
    assert_eq!(answer["team"]["number"], M16_1);
    assert_eq!(answer["season_first_day"], "2026-07-01");
    assert_eq!(answer["season_last_day"], "2027-06-30");

    let coming = events(&answer, "coming");
    let past = events(&answer, "past");
    // The schedule has 23 coming and 14 past games; the Tuesdays from 13 October until 25 May
    // are 33 days, from 1 September until 6 October 6 days, and one extra training.
    assert_eq!(games(&coming).len(), 23);
    assert_eq!(games(&past).len(), 14);
    assert_eq!(trainings(&coming).len(), 34);
    assert_eq!(trainings(&past).len(), 6);

    // The first coming event is the training day of Tuesday 13 October, in week 42.
    assert_eq!(
        (
            &coming[0]["date"],
            &coming[0]["week"],
            &coming[0]["week_first_day"],
            &coming[0]["week_last_day"],
            &coming[0]["training"]["status"],
            &coming[0]["game"]
        ),
        (
            &json!("2026-10-13"),
            &json!(42),
            &json!("2026-10-12"),
            &json!("2026-10-18"),
            &json!("not_opened"),
            &Value::Null
        )
    );

    // On 31 October: the games and the extra training in time order, in week 44.
    let on_31_october: Vec<&Value> = coming
        .iter()
        .filter(|event| event["date"] == "2026-10-31")
        .collect();
    let times: Vec<&Value> = on_31_october
        .iter()
        .map(|event| {
            if event["game"].is_null() {
                &event["training"]["start"]
            } else {
                &event["game"]["start_time"]
            }
        })
        .collect();
    assert_eq!(
        times,
        [
            &json!("13:00"),
            &json!("14:30"),
            &json!("15:00"),
            &json!("16:00"),
            &json!("17:30")
        ]
    );
    assert_eq!(on_31_october[2]["training"]["extra"], true);
    assert!(on_31_october.iter().all(|event| event["week"] == 44
        && event["week_first_day"] == "2026-10-26"
        && event["week_last_day"] == "2026-11-01"));

    // Past: newest first.
    assert_eq!(past[0]["date"], "2026-10-10");
    assert!(past[0]["date"].as_str() > past[past.len() - 1]["date"].as_str());
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_gone_game_shows_while_its_change_is_open(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let first = games(&events(&season(&reader, "").await, "coming"))[0]["game"]["id"].clone();
    reader.answer(games_with(|games| games.retain(|game| game["id"] != first)));
    reader.read_good().await;

    let coming = events(&season(&reader, "").await, "coming");
    let gone: Vec<&Value> = games(&coming)
        .into_iter()
        .filter(|event| event["game"]["id"] == first)
        .collect();
    assert_eq!(gone.len(), 1);
    assert_eq!(gone[0]["game"]["in_list"], false);

    let (status, changes) = reader.send("GET", "/api/changes", None).await;
    assert_eq!(status, 200);
    let seen: Vec<Value> = changes["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|change| json!({ "id": change["id"], "version": change["version"] }))
        .collect();
    let (status, _) = reader
        .send(
            "POST",
            "/api/changes/seen",
            Some(json!({ "changes": seen })),
        )
        .await;
    assert_eq!(status, 200);

    let coming = events(&season(&reader, "").await, "coming");
    assert!(
        games(&coming)
            .iter()
            .all(|event| event["game"]["id"] != first)
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_new_season_without_a_team_has_team_needed_and_only_trainings(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    reader.harness.clock.set(utc(2027, 7, 1, 10, 0));
    reader.log_in_again().await;
    add_tuesdays(&reader, "2027-09-07", "2027-10-26").await;

    let answer = season(&reader, "").await;
    assert_eq!(answer["season"], "2027-2028");
    assert_eq!(answer["games"], "team_needed");
    assert_eq!(answer["team"], Value::Null);
    let coming = events(&answer, "coming");
    assert_eq!(games(&coming).len(), 0);
    assert_eq!(trainings(&coming).len(), 8);
    assert_eq!(events(&answer, "past"), Vec::<Value>::new());
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn an_earlier_season_shows_its_games_from_its_own_choice_and_hides_gone_games(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let first = games(&events(&season(&reader, "").await, "coming"))[0]["game"]["id"].clone();
    // Gone, with its change still open.
    reader.answer(games_with(|games| games.retain(|game| game["id"] != first)));
    reader.read_good().await;
    reader.harness.clock.set(utc(2027, 7, 1, 10, 0));
    reader.log_in_again().await;

    let answer = season(&reader, "?season=2026-2027").await;
    assert_eq!(answer["season"], "2026-2027");
    assert_eq!(answer["current_season"], "2027-2028");
    assert_eq!(answer["games"], "shown");
    assert_eq!(answer["team"]["number"], M16_1);
    assert_eq!(events(&answer, "coming"), Vec::<Value>::new());
    let past = events(&answer, "past");
    // 37 games, without the gone one.
    assert_eq!(games(&past).len(), 36);
    assert!(
        games(&past)
            .iter()
            .all(|event| event["game"]["id"] != first)
    );

    // Without a choice of its own, an earlier season shows no games.
    let answer = season(&reader, "?season=2025-2026").await;
    assert_eq!(answer["games"], "none");
    assert_eq!(answer["past"], json!([]));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_season_with_only_a_choice_is_in_the_seasons(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.harness.clock.set(utc(2027, 7, 1, 10, 0));
    reader.log_in_again().await;
    // 2026-2027 has a choice, and no roles, weekly trainings or trainings.
    let answer = season(&reader, "").await;
    assert_eq!(answer["seasons"], json!(["2027-2028", "2026-2027"]));
    // A season where only a club was saved, without a team, is not offered.
    sqlx::query(
        "INSERT INTO followed_teams
             (season, club_number, club_name, club_town, team_number, team_name, chosen_at)
         VALUES ('2025-2026', 1, 'Blue Drakes', NULL, NULL, NULL, now())",
    )
    .execute(reader.pool())
    .await
    .unwrap();
    let answer = season(&reader, "").await;
    assert_eq!(answer["seasons"], json!(["2027-2028", "2026-2027"]));
    // A chosen season is offered also when nothing is in it.
    let answer = season(&reader, "?season=2024-2025").await;
    assert_eq!(
        answer["seasons"],
        json!(["2027-2028", "2026-2027", "2024-2025"])
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_season_that_cannot_be_read_is_refused(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    let (status, body) = reader.send("GET", "/api/season?season=abc", None).await;
    assert_eq!((status, body), (400, json!({ "error": "invalid_season" })));
}

/// A team game of M16 1 on 31 October 2026 at 16:00.
const TIME_GAME: i64 = 1073152;

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_start_time_that_is_not_known_comes_last_in_its_date(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.answer(games_with(|games| {
        game_mut(games, TIME_GAME)["tijd"] = json!("00:00");
    }));
    reader.read_good().await;

    let coming = events(&season(&reader, "").await, "coming");
    let on_31_october: Vec<&Value> = games(&coming)
        .into_iter()
        .filter(|event| event["date"] == "2026-10-31")
        .collect();
    let times: Vec<&Value> = on_31_october
        .iter()
        .map(|event| &event["game"]["start_time"])
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
    assert_eq!(on_31_october[3]["game"]["id"], TIME_GAME);
}
