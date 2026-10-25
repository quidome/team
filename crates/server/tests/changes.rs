//! The open changes (story 4): what a reading keeps, merges and removes, and what a new choice
//! removes.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::Harness;
use common::game_list::{M16_1, M16_2, Reader, game_mut, games_with, real_games, utc};
use serde_json::{Value, json};
use sqlx::PgPool;

/// A team game of M16 1 on 12 December 2026 at 14:30 in Bulwijk.
const TEAM_GAME: i64 = 1073178;
/// A team game of M16 1 on 7 November 2026 at 11:15 in Dudok Arena (place 552).
const DUDOK_GAME: i64 = 1073159;
/// A team game of M16 1 on 3 October 2026: a past game on 12 October.
const PAST_GAME: i64 = 1073142;
/// A team game of M16 1 on 31 October 2026 at 16:00.
const TIME_GAME: i64 = 1073152;
/// A home game of Blue Drakes X12 1 on 31 October 2026: a possible duty game.
const DUTY_GAME: i64 = 1081923;
/// An away game of Blue Drakes X12 1 on 25 October 2026: not in the schedule.
const OTHER_TEAM_AWAY: i64 = 1081919;
const NEW_GAME: i64 = 9_999_001;

/// The real games with the start time of `id` set to `time`.
fn at(id: i64, time: &str) -> Value {
    games_with(|games| game_mut(games, id)["tijd"] = json!(time))
}

/// The open changes: (source id, version, last seen).
async fn rows(pool: &PgPool) -> Vec<(i64, i64, Option<Value>)> {
    sqlx::query_as("SELECT source_id, version, last_seen FROM changes ORDER BY source_id")
        .fetch_all(pool)
        .await
        .unwrap()
}

async fn ids(pool: &PgPool) -> Vec<i64> {
    rows(pool).await.into_iter().map(|row| row.0).collect()
}

/// Chosen and read once: the next reading is not a first reading.
async fn read_once(pool: PgPool) -> Reader {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    reader
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_moved_game_keeps_one_open_change(pool: PgPool) {
    let reader = read_once(pool).await;
    reader.answer(at(TEAM_GAME, "16:00"));
    reader.read_good().await;

    let rows = rows(reader.pool()).await;
    assert_eq!(rows.len(), 1);
    let (id, version, last_seen) = &rows[0];
    assert_eq!((*id, *version), (TEAM_GAME, 1));
    let last_seen = last_seen.as_ref().unwrap();
    assert_eq!(last_seen["start_time"], "14:30:00");
    assert_eq!(last_seen["in_list"], true);

    // The same answer again changes nothing, also not the version.
    reader.read_good().await;
    assert_eq!(self::rows(reader.pool()).await, rows);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_second_move_keeps_the_first_old_value_and_a_move_back_removes_the_change(pool: PgPool) {
    let reader = read_once(pool).await;
    reader.answer(at(TEAM_GAME, "16:00"));
    reader.read_good().await;
    reader.answer(at(TEAM_GAME, "17:30"));
    reader.read_good().await;

    let rows = rows(reader.pool()).await;
    assert_eq!((rows[0].0, rows[0].1), (TEAM_GAME, 2));
    assert_eq!(rows[0].2.as_ref().unwrap()["start_time"], "14:30:00");

    reader.answer(real_games());
    reader.read_good().await;
    assert_eq!(ids(reader.pool()).await, Vec::<i64>::new());
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_first_reading_after_a_choice_keeps_no_changes(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.answer(at(TEAM_GAME, "16:00"));
    let body = reader.read_good().await;
    assert_eq!(body["first_reading"], true);
    assert_eq!(ids(reader.pool()).await, Vec::<i64>::new());
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_change_of_team_removes_the_open_changes_and_the_same_team_keeps_them(pool: PgPool) {
    let reader = read_once(pool).await;
    reader.answer(at(TEAM_GAME, "16:00"));
    reader.read_good().await;

    reader.choose(M16_1).await;
    assert_eq!(ids(reader.pool()).await, [TEAM_GAME]);

    reader.choose(M16_2).await;
    assert_eq!(ids(reader.pool()).await, Vec::<i64>::new());
    // The next reading is a first reading of M16 2 and records none.
    reader.answer(at(TEAM_GAME, "17:30"));
    let body = reader.read_good().await;
    assert_eq!(body["first_reading"], true);
    assert_eq!(ids(reader.pool()).await, Vec::<i64>::new());
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_first_choice_of_a_new_season_removes_the_changes_of_last_season(pool: PgPool) {
    let mut reader = read_once(pool).await;
    reader.answer(at(TEAM_GAME, "16:00"));
    reader.read_good().await;
    assert_eq!(ids(reader.pool()).await, [TEAM_GAME]);

    // 1 July 2027: season 2027-2028, no choice saved for it yet.
    reader.harness.clock.set(utc(2027, 7, 1, 10, 0));
    reader.log_in_again().await;
    reader.answer(games_with(|games| {
        for game in games.iter_mut() {
            game["seizoen"] = json!("2027-2028");
        }
    }));
    reader.choose(M16_1).await;
    assert_eq!(ids(reader.pool()).await, Vec::<i64>::new());
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_failure_while_storing_keeps_no_changes(pool: PgPool) {
    let reader = read_once(pool).await;
    sqlx::raw_sql(
        "CREATE FUNCTION fail_good_reading() RETURNS trigger LANGUAGE plpgsql AS
             $$ BEGIN RAISE EXCEPTION 'test failure'; END $$;
         CREATE TRIGGER fail_good_reading BEFORE INSERT ON readings FOR EACH ROW
             WHEN (NEW.result = 'good') EXECUTE FUNCTION fail_good_reading();",
    )
    .execute(reader.pool())
    .await
    .unwrap();
    reader.answer(at(TEAM_GAME, "16:00"));
    let (status, _) = reader.read().await;
    assert_eq!(status, 500);
    assert_eq!(ids(reader.pool()).await, Vec::<i64>::new());
    let start: Option<chrono::NaiveTime> = sqlx::query_scalar(
        "SELECT start_time FROM games WHERE source_id = $1 AND season_year = 2026",
    )
    .bind(TEAM_GAME)
    .fetch_one(reader.pool())
    .await
    .unwrap();
    assert_eq!(start, chrono::NaiveTime::from_hms_opt(14, 30, 0));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_game_as_last_seen_keeps_the_old_hall_name(pool: PgPool) {
    let reader = read_once(pool).await;
    reader.answer(at(DUDOK_GAME, "12:00"));
    reader.read_good().await;
    reader.answer(games_with(|games| {
        game_mut(games, DUDOK_GAME)["tijd"] = json!("12:00");
        for game in games.iter_mut().filter(|game| game["loc_id"] == 552) {
            game["loc_naam"] = json!("Dudok Arena Hilversum");
        }
    }));
    reader.read_good().await;

    let rows = rows(reader.pool()).await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].2.as_ref().unwrap()["hall"], "Dudok Arena");
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_move_of_a_past_game_is_not_kept_or_counted(pool: PgPool) {
    let reader = read_once(pool).await;
    reader.answer(at(PAST_GAME, "18:00"));
    let body = reader.read_good().await;
    assert_eq!(body["counts"]["moved"], 0);
    assert_eq!(ids(reader.pool()).await, Vec::<i64>::new());
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn really_gone_keeps_a_gone_change_for_each_coming_schedule_game(pool: PgPool) {
    let reader = read_once(pool).await;
    reader.answer(games_with(|games| games.truncate(30)));
    let (_, held) = reader.read().await;
    assert_eq!(held["result"], "held_back");
    let (status, body) = reader.confirm_gone(&held["id"]).await;
    assert_eq!((status, &body["result"]), (200, &json!("good")));

    let gone = body["counts"]["gone"].as_u64().unwrap();
    assert!(gone > 0);
    let rows = rows(reader.pool()).await;
    assert_eq!(u64::try_from(rows.len()).unwrap(), gone);
    assert!(
        rows.iter()
            .all(|(_, _, last_seen)| last_seen.as_ref().unwrap()["in_list"] == true)
    );
}

/// Home and away of a game of the source change places.
fn swap(game: &mut Value) {
    for field in ["ploeg_id", "ploeg", "club_id", "club"] {
        let home = game[format!("thuis_{field}")].clone();
        game[format!("thuis_{field}")] = game[format!("uit_{field}")].clone();
        game[format!("uit_{field}")] = home;
    }
}

async fn changes(reader: &Reader) -> Vec<Value> {
    let (status, body) = reader.send("GET", "/api/changes", None).await;
    assert_eq!(status, 200, "{body}");
    body["changes"].as_array().unwrap().clone()
}

/// The only change after one reading of `answer`.
async fn only_change(reader: &Reader, answer: Value) -> Value {
    reader.answer(answer);
    reader.read_good().await;
    let changes = changes(reader).await;
    assert_eq!(changes.len(), 1, "{changes:?}");
    changes[0].clone()
}

async fn seen(reader: &Reader, changes: Value) -> Value {
    let (status, body) = reader
        .send(
            "POST",
            "/api/changes/seen",
            Some(json!({ "changes": changes })),
        )
        .await;
    assert_eq!(status, 200, "{body}");
    body
}

/// The number of changes not seen (the open item of the start page) and the games of the current
/// season ("Season"), in the shape the schedule had.
async fn schedule(reader: &Reader) -> Value {
    let (status, start) = reader.send("GET", "/api/start", None).await;
    assert_eq!(status, 200, "{start}");
    let unseen = start["open_items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["kind"] == "changes")
        .map_or(json!(0), |item| item["count"].clone());
    let (status, season) = reader.send("GET", "/api/season", None).await;
    assert_eq!(status, 200, "{season}");
    let games = |part: &str| -> Vec<Value> {
        season[part]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| !event["game"].is_null())
            .map(|event| event["game"].clone())
            .collect()
    };
    json!({ "unseen_changes": unseen, "coming": games("coming"), "past": games("past") })
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_change_routes_need_a_login(pool: PgPool) {
    let harness = Harness::start(pool).await;
    assert_eq!(harness.get("/api/changes", None).await.status(), 401);
    let request = axum::http::Request::builder()
        .method("POST")
        .uri("/api/changes/seen")
        .header(axum::http::header::ORIGIN, common::ORIGIN)
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .body(axum::body::Body::from(r#"{"changes":[]}"#))
        .unwrap();
    assert_eq!(harness.send(request).await.status(), 401);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn with_nothing_chosen_there_are_no_changes(pool: PgPool) {
    let harness = Harness::start(pool).await;
    let cookie = format!(
        "team_session={}",
        harness.log_in(&["team_coordinator"]).await
    );
    let response = harness.get("/api/changes", Some(&cookie)).await;
    assert_eq!(response.status(), 200);
    let body: Value = serde_json::from_str(&common::body_text(response).await).unwrap();
    assert_eq!(body, json!({ "changes": [] }));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_moved_team_game_shows_its_old_and_new_start_time(pool: PgPool) {
    let reader = read_once(pool).await;
    let change = only_change(&reader, at(TEAM_GAME, "16:00")).await;
    assert_eq!(change["id"], TEAM_GAME);
    assert_eq!(change["version"], 1);
    assert_eq!(change["kind"], "moved");
    assert_eq!(change["differences"], json!(["start_time"]));
    assert_eq!(change["reason"], "team");
    assert_eq!(
        change["before"],
        json!({ "date": "2026-12-12", "start_time": "14:30", "home": "Blue Drakes M16 1",
                "away": "B.Z. '72 M16 1", "hall": "Bulwijk", "town": "Woerden" })
    );
    assert_eq!(change["after"]["start_time"], "16:00");
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_new_duty_game_is_new(pool: PgPool) {
    let reader = read_once(pool).await;
    let change = only_change(
        &reader,
        games_with(|games| {
            let mut new_game = game_mut(games, DUTY_GAME).clone();
            new_game["id"] = json!(NEW_GAME);
            new_game["nr"] = json!("NEW-1");
            games.push(new_game);
        }),
    )
    .await;
    assert_eq!(
        (&change["id"], &change["kind"], &change["reason"]),
        (&json!(NEW_GAME), &json!("new"), &json!("duty"))
    );
    assert_eq!(change["before"], Value::Null);
    assert_eq!(change["after"]["date"], "2026-10-31");
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_new_game_gone_and_back_before_it_is_seen_is_still_new(pool: PgPool) {
    let reader = read_once(pool).await;
    let with_new_game = || {
        games_with(|games| {
            let mut new_game = game_mut(games, DUTY_GAME).clone();
            new_game["id"] = json!(NEW_GAME);
            new_game["nr"] = json!("NEW-1");
            games.push(new_game);
        })
    };
    let change = only_change(&reader, with_new_game()).await;
    assert_eq!(change["kind"], "new");

    // Gone before the coordinator looked: nothing to show.
    reader.answer(real_games());
    reader.read_good().await;
    assert_eq!(changes(&reader).await, Vec::<Value>::new());

    // Back: still new, because the coordinator never saw it (epic review accept A2).
    let change = only_change(&reader, with_new_game()).await;
    assert_eq!(
        (&change["id"], &change["kind"], &change["before"]),
        (&json!(NEW_GAME), &json!("new"), &Value::Null)
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_start_time_that_becomes_known_is_start_time_known(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.answer(at(TIME_GAME, "00:00"));
    reader.read_good().await;
    let change = only_change(&reader, real_games()).await;
    assert_eq!(change["kind"], "start_time_known");
    assert_eq!(change["before"]["start_time"], Value::Null);
    assert_eq!(change["after"]["start_time"], "16:00");
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn home_and_away_swapped_and_a_changed_team(pool: PgPool) {
    let reader = read_once(pool).await;
    let change = only_change(
        &reader,
        games_with(|games| swap(game_mut(games, TEAM_GAME))),
    )
    .await;
    assert_eq!(change["kind"], "home_and_away");
    assert_eq!(change["before"]["home"], "Blue Drakes M16 1");
    assert_eq!(change["after"]["home"], "B.Z. '72 M16 1");

    reader.answer(real_games());
    reader.read_good().await;
    let change = only_change(
        &reader,
        games_with(|games| {
            let game = game_mut(games, TEAM_GAME);
            game["uit_ploeg_id"] = json!(9_999);
            game["uit_ploeg"] = json!("B.Z. '72 M16 2");
        }),
    )
    .await;
    assert_eq!(change["kind"], "team_changed");
    assert_eq!(change["after"]["away"], "B.Z. '72 M16 2");
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_gone_game_that_comes_back_came_back(pool: PgPool) {
    let reader = read_once(pool).await;
    let change = only_change(
        &reader,
        games_with(|games| games.retain(|game| game["id"] != TEAM_GAME)),
    )
    .await;
    assert_eq!(change["kind"], "gone");
    assert_eq!(change["after"], Value::Null);
    assert_eq!(change["before"]["date"], "2026-12-12");

    seen(
        &reader,
        json!([{ "id": TEAM_GAME, "version": change["version"] }]),
    )
    .await;
    let change = only_change(&reader, real_games()).await;
    assert_eq!(change["kind"], "came_back");
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn games_into_and_out_of_the_schedule_say_why(pool: PgPool) {
    let reader = read_once(pool).await;
    let change = only_change(
        &reader,
        games_with(|games| swap(game_mut(games, OTHER_TEAM_AWAY))),
    )
    .await;
    assert_eq!(
        (&change["kind"], &change["reason"]),
        (&json!("into_schedule"), &json!("duty"))
    );

    reader.answer(real_games());
    reader.read_good().await;
    let change = only_change(
        &reader,
        games_with(|games| swap(game_mut(games, DUTY_GAME))),
    )
    .await;
    assert_eq!(
        (&change["kind"], &change["reason"]),
        (&json!("out_of_schedule"), &json!("duty"))
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn seen_deletes_a_change_only_with_its_version(pool: PgPool) {
    let reader = read_once(pool).await;
    reader.answer(at(TEAM_GAME, "16:00"));
    reader.read_good().await;
    // A reading changed it again after the screen loaded version 1.
    reader.answer(at(TEAM_GAME, "17:30"));
    reader.read_good().await;

    let body = seen(&reader, json!([{ "id": TEAM_GAME, "version": 1 }])).await;
    assert_eq!(body, json!({ "open": 1 }));
    let body = seen(
        &reader,
        json!([{ "id": TEAM_GAME, "version": 2 }, { "id": 1, "version": 1 }]),
    )
    .await;
    assert_eq!(body, json!({ "open": 0 }));
    assert_eq!(changes(&reader).await, Vec::<Value>::new());
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_start_page_counts_unseen_changes_and_season_hides_a_gone_game_after_seen(
    pool: PgPool,
) {
    let reader = read_once(pool).await;
    assert_eq!(schedule(&reader).await["unseen_changes"], 0);

    reader.answer(games_with(|games| {
        games.retain(|game| game["id"] != TEAM_GAME)
    }));
    reader.read_good().await;
    let schedule_now = schedule(&reader).await;
    assert_eq!(schedule_now["unseen_changes"], 1);
    let gone = schedule_now["coming"]
        .as_array()
        .unwrap()
        .iter()
        .find(|game| game["id"] == TEAM_GAME)
        .unwrap();
    assert_eq!(gone["in_list"], false);

    seen(&reader, json!([{ "id": TEAM_GAME, "version": 1 }])).await;
    let schedule_now = schedule(&reader).await;
    assert_eq!(schedule_now["unseen_changes"], 0);
    assert!(
        schedule_now["coming"]
            .as_array()
            .unwrap()
            .iter()
            .all(|game| game["id"] != TEAM_GAME)
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_change_made_again_never_gets_an_old_version(pool: PgPool) {
    let reader = read_once(pool).await;
    let first = only_change(&reader, at(TEAM_GAME, "16:00")).await;
    // Moved back: no change; moved again: a new change of the same game.
    reader.answer(real_games());
    reader.read_good().await;
    let again = only_change(&reader, at(TEAM_GAME, "17:30")).await;
    assert_ne!(again["version"], first["version"]);

    // "Seen" from a page that still shows the first change does not hide the new one.
    let body = seen(
        &reader,
        json!([{ "id": TEAM_GAME, "version": first["version"] }]),
    )
    .await;
    assert_eq!(body, json!({ "open": 1 }));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_team_game_that_another_club_team_plays_now_is_a_change_of_the_team(pool: PgPool) {
    let reader = read_once(pool).await;
    let change = only_change(
        &reader,
        games_with(|games| {
            let game = game_mut(games, TEAM_GAME);
            game["thuis_ploeg_id"] = json!(M16_2);
            game["thuis_ploeg"] = json!("Blue Drakes M16 2");
        }),
    )
    .await;
    assert_eq!(
        (&change["kind"], &change["reason"]),
        (&json!("team_changed"), &json!("team"))
    );
    assert_eq!(change["after"]["home"], "Blue Drakes M16 2");
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_gone_game_whose_date_passed_is_hidden_with_its_change(pool: PgPool) {
    let mut reader = read_once(pool).await;
    reader.answer(games_with(|games| {
        games.retain(|game| game["id"] != TEAM_GAME)
    }));
    reader.read_good().await;
    assert_eq!(schedule(&reader).await["unseen_changes"], 1);

    // 13 December 2026: the gone game of 12 December is past, with no reading in between.
    reader.harness.clock.set(utc(2026, 12, 13, 12, 0));
    reader.log_in_again().await;
    let schedule_now = schedule(&reader).await;
    assert_eq!(schedule_now["unseen_changes"], 0);
    assert!(
        schedule_now["past"]
            .as_array()
            .unwrap()
            .iter()
            .all(|game| game["id"] != TEAM_GAME)
    );
    assert_eq!(changes(&reader).await, Vec::<Value>::new());
}
