//! Choosing the club and team through the API (story 2): search, teams, save, new season.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use axum::body::Body;
use axum::http::{Request, Response, header};
use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use common::{Harness, ORIGIN, body_text};
use serde_json::{Value, json};
use sqlx::PgPool;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

const DRAKES: i64 = 37;
/// A real club that gets an empty season from the fake source.
const GIANTS: i64 = 21;

fn data(name: &str) -> Value {
    let path = format!("{}/tests/data/game-list/{name}", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// This season's real games of Blue Drakes, presented as the games of `season`.
fn drakes_games(season: &str) -> Value {
    let mut body = data("games-37-2026-2027.json");
    for game in body["wedstrijden"].as_array_mut().unwrap() {
        game["seizoen"] = json!(season);
    }
    body
}

fn utc(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(year, month, day, hour, minute, 0)
        .single()
        .unwrap()
}

/// A logged-in harness at 11 October 2026 (season 2026-2027). See [`start_at`].
async fn start(pool: PgPool) -> (Harness, String) {
    start_at(pool, utc(2026, 10, 11, 12, 0)).await
}

/// A harness logged in at `now`, with a fake source that knows the real club list, Blue Drakes'
/// games of 2026-2027 and 2027-2028, and an empty season for Green Giants.
async fn start_at(pool: PgPool, now: DateTime<Utc>) -> (Harness, String) {
    let harness = Harness::start(pool).await;
    harness.clock.set(now);
    Mock::given(method("GET"))
        .and(path("/db/json/club.pl"))
        .respond_with(ResponseTemplate::new(200).set_body_json(data("clubs.json")))
        .mount(&harness.source)
        .await;
    for season in ["2026-2027", "2027-2028"] {
        Mock::given(method("GET"))
            .and(path("/db/json/wedstrijd.pl"))
            .and(query_param("clb_ID", DRAKES.to_string()))
            .and(query_param("seizoen", season))
            .respond_with(ResponseTemplate::new(200).set_body_json(drakes_games(season)))
            .mount(&harness.source)
            .await;
    }
    Mock::given(method("GET"))
        .and(path("/db/json/wedstrijd.pl"))
        .and(query_param("clb_ID", GIANTS.to_string()))
        .respond_with(ResponseTemplate::new(200).set_body_json(data("games-empty.json")))
        .mount(&harness.source)
        .await;
    let cookie = format!(
        "team_session={}",
        harness.log_in(&["team_coordinator"]).await
    );
    (harness, cookie)
}

async fn json_of(response: Response<Body>) -> Value {
    serde_json::from_str(&body_text(response).await).unwrap()
}

async fn get_json(harness: &Harness, path: &str, cookie: &str) -> (u16, Value) {
    let response = harness.get(path, Some(cookie)).await;
    let status = response.status().as_u16();
    (status, json_of(response).await)
}

async fn put_selection(harness: &Harness, cookie: &str, body: Value) -> (u16, Value) {
    let request = Request::put("/api/selection")
        .header(header::COOKIE, cookie)
        .header(header::ORIGIN, ORIGIN)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = harness.send(request).await;
    let status = response.status().as_u16();
    (status, json_of(response).await)
}

async fn rows(pool: &PgPool) -> Vec<(String, Option<String>, DateTime<Utc>)> {
    sqlx::query_as("SELECT season, team_name, chosen_at FROM followed_teams ORDER BY season")
        .fetch_all(pool)
        .await
        .unwrap()
}

fn drakes_club() -> Value {
    json!({ "number": 37, "name": "Blue Drakes", "town": "Woerden" })
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn all_routes_need_a_login(pool: PgPool) {
    let harness = Harness::start(pool).await;
    for path in [
        "/api/selection",
        "/api/clubs?search=drakes",
        "/api/clubs/37/teams",
    ] {
        assert_eq!(harness.get(path, None).await.status(), 401, "{path}");
    }
    let request = Request::put("/api/selection")
        .header(header::ORIGIN, ORIGIN)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"club":37,"team":184}"#))
        .unwrap();
    assert_eq!(harness.send(request).await.status(), 401);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn nothing_saved_is_not_chosen(pool: PgPool) {
    let (harness, cookie) = start(pool).await;
    let (status, body) = get_json(&harness, "/api/selection", &cookie).await;
    assert_eq!(status, 200);
    assert_eq!(
        body,
        json!({ "state": "not_chosen", "season": "2026-2027" })
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn search_finds_the_club_by_part_of_its_name(pool: PgPool) {
    let (harness, cookie) = start(pool).await;
    let (status, body) = get_json(&harness, "/api/clubs?search=drakes", &cookie).await;
    assert_eq!(status, 200);
    assert_eq!(body, json!([drakes_club()]));

    let (status, body) = get_json(&harness, "/api/clubs?search=zzz", &cookie).await;
    assert_eq!(status, 200);
    assert_eq!(body, json!([]));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn lists_the_own_teams_of_the_club(pool: PgPool) {
    let (harness, cookie) = start(pool).await;
    let (status, body) = get_json(&harness, "/api/clubs/37/teams", &cookie).await;
    assert_eq!(status, 200);
    assert_eq!(body["season"], "2026-2027");
    let names: Vec<&str> = body["teams"]
        .as_array()
        .unwrap()
        .iter()
        .map(|team| team["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        ["M16 1", "M16 2", "M18 1", "MSE 1", "X12 1", "X14 1"]
    );
    assert_eq!(body["teams"][0], json!({ "number": 184, "name": "M16 1" }));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn an_unknown_club_is_not_found(pool: PgPool) {
    let (harness, cookie) = start(pool).await;
    let (status, _) = get_json(&harness, "/api/clubs/999999/teams", &cookie).await;
    assert_eq!(status, 404);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_club_with_no_games_has_no_teams_yet(pool: PgPool) {
    let (harness, cookie) = start(pool).await;
    let (status, body) = get_json(&harness, "/api/clubs/21/teams", &cookie).await;
    assert_eq!(status, 200);
    assert_eq!(body, json!({ "season": "2026-2027", "teams": [] }));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn saves_the_club_and_team(pool: PgPool) {
    let (harness, cookie) = start(pool).await;
    let chosen = json!({
        "state": "chosen",
        "season": "2026-2027",
        "club": drakes_club(),
        "team": { "number": 184, "name": "M16 1" },
    });
    let (status, body) = put_selection(&harness, &cookie, json!({ "club": 37, "team": 184 })).await;
    assert_eq!(status, 200);
    assert_eq!(body, chosen);
    let (_, body) = get_json(&harness, "/api/selection", &cookie).await;
    assert_eq!(body, chosen);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn refuses_choices_that_do_not_fit(pool: PgPool) {
    let (harness, cookie) = start(pool).await;
    let cases = [
        (json!({ "club": 999999, "team": 184 }), "unknown_club"),
        (json!({ "club": 37, "team": 1688 }), "not_an_own_team"),
        (json!({ "club": 37 }), "team_required"),
    ];
    for (request, code) in cases {
        let (status, body) = put_selection(&harness, &cookie, request.clone()).await;
        assert_eq!(status, 400, "{request}");
        assert_eq!(body, json!({ "error": code }), "{request}");
    }
    assert!(rows(&harness.pool).await.is_empty());
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_club_with_no_games_is_saved_without_a_team(pool: PgPool) {
    let (harness, cookie) = start(pool).await;
    let (status, body) = put_selection(&harness, &cookie, json!({ "club": 21 })).await;
    assert_eq!(status, 200);
    assert_eq!(body["state"], "team_needed");
    assert_eq!(body["reason"], "no_games_yet");
    assert_eq!(body["club"]["name"], "Green Giants");
    assert_eq!(body["last"], Value::Null);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_save_needs_the_own_origin(pool: PgPool) {
    let (harness, cookie) = start(pool).await;
    let request = Request::put("/api/selection")
        .header(header::COOKIE, &cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"club":37,"team":184}"#))
        .unwrap();
    assert_eq!(harness.send(request).await.status(), 403);
    assert!(rows(&harness.pool).await.is_empty());
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn nothing_is_saved_when_the_source_cannot_be_reached(pool: PgPool) {
    let harness = Harness::start(pool).await;
    let cookie = format!(
        "team_session={}",
        harness.log_in(&["team_coordinator"]).await
    );
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&harness.source)
        .await;
    let (status, body) = put_selection(&harness, &cookie, json!({ "club": 37, "team": 184 })).await;
    assert_eq!(status, 502);
    assert_eq!(body, json!({ "error": "source_unavailable" }));
    let (_, body) = get_json(&harness, "/api/selection", &cookie).await;
    assert_eq!(body["state"], "not_chosen");
    let (status, body) = get_json(&harness, "/api/clubs?search=drakes", &cookie).await;
    assert_eq!(
        (status, body),
        (502, json!({ "error": "source_unavailable" }))
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn nothing_is_saved_when_the_source_is_not_understood(pool: PgPool) {
    let harness = Harness::start(pool).await;
    let cookie = format!(
        "team_session={}",
        harness.log_in(&["team_coordinator"]).await
    );
    Mock::given(method("GET"))
        .and(path("/db/json/club.pl"))
        .respond_with(ResponseTemplate::new(200).set_body_json(data("clubs.json")))
        .mount(&harness.source)
        .await;
    let mut games = drakes_games("2026-2027");
    games["wedstrijden"][0]
        .as_object_mut()
        .unwrap()
        .remove("tijd");
    Mock::given(method("GET"))
        .and(path("/db/json/wedstrijd.pl"))
        .respond_with(ResponseTemplate::new(200).set_body_json(games))
        .mount(&harness.source)
        .await;
    harness.clock.set(utc(2026, 10, 11, 12, 0));
    let (status, body) = put_selection(&harness, &cookie, json!({ "club": 37, "team": 184 })).await;
    assert_eq!(status, 502);
    assert_eq!(body, json!({ "error": "source_format" }));
    assert!(rows(&harness.pool).await.is_empty());
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_change_in_the_same_season_replaces_the_choice(pool: PgPool) {
    let (harness, cookie) = start(pool).await;
    put_selection(&harness, &cookie, json!({ "club": 37, "team": 184 })).await;
    harness.clock.advance(TimeDelta::hours(1));
    let (status, body) =
        put_selection(&harness, &cookie, json!({ "club": 37, "team": 1404 })).await;
    assert_eq!(status, 200);
    assert_eq!(body["team"]["name"], "M16 2");
    let rows = rows(&harness.pool).await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].1.as_deref(), Some("M16 2"));
    assert_eq!(rows[0].2, utc(2026, 10, 11, 13, 0));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_new_season_asks_for_the_team_again_and_keeps_last_season(pool: PgPool) {
    // 23:59 Dutch summer time on 31 July 2027: still season 2026-2027.
    let (harness, cookie) = start_at(pool, utc(2027, 7, 31, 21, 59)).await;
    let (status, body) = put_selection(&harness, &cookie, json!({ "club": 37, "team": 184 })).await;
    assert_eq!(status, 200);
    assert_eq!(body["season"], "2026-2027");

    // 00:00 Dutch summer time on 1 August 2027: season 2027-2028.
    harness.clock.set(utc(2027, 7, 31, 22, 0));
    let (_, body) = get_json(&harness, "/api/selection", &cookie).await;
    assert_eq!(
        body,
        json!({
            "state": "team_needed",
            "season": "2027-2028",
            "club": drakes_club(),
            "reason": "new_season",
            "last": {
                "season": "2026-2027",
                "club": drakes_club(),
                "team": { "number": 184, "name": "M16 1" },
            },
        })
    );

    let (status, body) = put_selection(&harness, &cookie, json!({ "club": 37, "team": 154 })).await;
    assert_eq!(status, 200);
    assert_eq!(body["state"], "chosen");
    assert_eq!(body["season"], "2027-2028");
    assert_eq!(body["team"]["name"], "M18 1");

    let rows = rows(&harness.pool).await;
    assert_eq!(
        rows,
        [
            (
                "2026-2027".to_owned(),
                Some("M16 1".to_owned()),
                utc(2027, 7, 31, 21, 59)
            ),
            (
                "2027-2028".to_owned(),
                Some("M18 1".to_owned()),
                utc(2027, 7, 31, 22, 0)
            ),
        ]
    );
}

/// Puts a choice in the table by hand, as if it was saved in an earlier (or later) season.
async fn insert_choice(pool: &PgPool, season: &str, club: (i64, &str), team: Option<(i64, &str)>) {
    sqlx::query(
        "INSERT INTO followed_teams
             (season, club_number, club_name, club_town, team_number, team_name, chosen_at)
         VALUES ($1, $2, $3, NULL, $4, $5, now())",
    )
    .bind(season)
    .bind(club.0)
    .bind(club.1)
    .bind(team.map(|(number, _)| number))
    .bind(team.map(|(_, name)| name))
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn saving_the_club_for_an_empty_new_season_keeps_last_season(pool: PgPool) {
    // Green Giants has no games in the new season (the normal case on 1 August).
    insert_choice(
        &pool,
        "2026-2027",
        (GIANTS, "Green Giants"),
        Some((5, "M16 1")),
    )
    .await;
    let (harness, cookie) = start_at(pool, utc(2027, 8, 1, 10, 0)).await;
    let last = json!({
        "season": "2026-2027",
        "club": { "number": GIANTS, "name": "Green Giants", "town": null },
        "team": { "number": 5, "name": "M16 1" },
    });

    let (_, body) = get_json(&harness, "/api/selection", &cookie).await;
    assert_eq!(body["reason"], "new_season");
    assert_eq!(body["last"], last);

    let (status, body) = put_selection(&harness, &cookie, json!({ "club": GIANTS })).await;
    assert_eq!(status, 200);
    assert_eq!(body["state"], "team_needed");
    assert_eq!(body["reason"], "no_games_yet");
    assert_eq!(body["season"], "2027-2028");
    assert_eq!(
        body["last"], last,
        "last season's team stays after the save"
    );

    let (_, body) = get_json(&harness, "/api/selection", &cookie).await;
    assert_eq!(body["last"], last);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_choice_of_a_later_season_does_not_hide_the_current_season(pool: PgPool) {
    // A row of 2027-2028 while the clock says 2026-2027 (a wrong clock, later set back).
    insert_choice(
        &pool,
        "2027-2028",
        (DRAKES, "Blue Drakes"),
        Some((154, "M18 1")),
    )
    .await;
    let (harness, cookie) = start(pool).await;

    let (_, body) = get_json(&harness, "/api/selection", &cookie).await;
    assert_eq!(
        body,
        json!({ "state": "not_chosen", "season": "2026-2027" })
    );

    let (status, body) = put_selection(&harness, &cookie, json!({ "club": 37, "team": 184 })).await;
    assert_eq!(status, 200);
    assert_eq!(body["season"], "2026-2027");
    assert_eq!(body["team"]["name"], "M16 1");
}
