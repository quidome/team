//! The page "Duties" through the API: the duty totals of each player in a season (story B of
//! duties).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::Harness;
use common::game_list::{M16_1, Reader};
use serde_json::{Value, json};
use sqlx::PgPool;
use team::players;
use team_core::PlayerId;

const X12_3_OCT: i64 = 1_081_913;
const X12_31_OCT: i64 = 1_081_923;
const X14_31_OCT: i64 = 1_083_716;

async fn ok(reader: &Reader, method: &str, path: &str, body: Option<Value>) -> Value {
    let (status, answer) = reader.send(method, path, body).await;
    assert_eq!(status, 200, "{method} {path}: {answer}");
    answer
}

fn game_path(id: i64) -> String {
    format!("/api/games/2026-2027/{id}")
}

async fn table_of_team(reader: &Reader, id: i64) {
    ok(
        reader,
        "PUT",
        &format!("{}/places/table/1", game_path(id)),
        Some(json!({ "team": M16_1 })),
    )
    .await;
}

async fn assign(reader: &Reader, id: i64, player: i64) {
    ok(
        reader,
        "PUT",
        &format!("{}/assignments/table/1", game_path(id)),
        Some(json!({ "player": player })),
    )
    .await;
}

async fn add_player(reader: &Reader, first: &str, role: &str) -> i64 {
    let answer = ok(
        reader,
        "POST",
        "/api/players",
        Some(json!({ "first_name": first, "last_name": null,
                     "role": { "role": role, "from": "2026-09-01" } })),
    )
    .await;
    answer["player"]["id"].as_i64().unwrap()
}

/// The row of `player` on "Duties".
fn row(duties: &Value, player: i64) -> Option<&Value> {
    duties["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["player"] == player)
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_duties_need_a_login(pool: PgPool) {
    let harness = Harness::start(pool).await;
    let response = harness.get("/api/duties", None).await;
    assert_eq!(response.status(), 401);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn duties_give_the_numbers_of_the_choice_of_a_game(pool: PgPool) {
    let reader = Reader::chosen(pool.clone()).await;
    reader.read_good().await;
    for id in [X12_3_OCT, X12_31_OCT, X14_31_OCT] {
        table_of_team(&reader, id).await;
    }
    let bram = add_player(&reader, "Bram", "plays_games").await;
    let anna = add_player(&reader, "Anna", "plays_games").await;
    let cas = add_player(&reader, "Cas", "only_trains").await;
    assign(&reader, X12_3_OCT, anna).await;
    assign(&reader, X12_31_OCT, anna).await;
    players::set_hidden(&pool, PlayerId(bram), true)
        .await
        .unwrap();

    let duties = ok(&reader, "GET", "/api/duties", None).await;
    assert_eq!(duties["season"], "2026-2027");
    assert_eq!(duties["seasons"], json!(["2026-2027"]));
    let names: Vec<&str> = duties["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["name"].as_str().unwrap())
        .collect();
    // By first name; a player who only trains and has no duty has no row.
    assert_eq!(names, ["Anna", "Bram"]);
    assert!(row(&duties, cas).is_none());
    assert_eq!(row(&duties, bram).unwrap()["hidden"], true);

    // The same numbers as the choice of a game of the season.
    let page = ok(&reader, "GET", &game_path(X14_31_OCT), None).await;
    for player in [anna, bram] {
        let choice = page["players"]
            .as_array()
            .unwrap()
            .iter()
            .find(|choice| choice["id"] == player)
            .unwrap();
        assert_eq!(row(&duties, player).unwrap()["totals"], choice["totals"]);
    }
    assert_eq!(
        row(&duties, anna).unwrap()["totals"]["table"],
        json!({ "done": 1, "planned": 1 })
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_player_with_a_duty_and_no_role_plays_games_has_a_row(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    table_of_team(&reader, X12_3_OCT).await;
    let dora = add_player(&reader, "Dora", "plays_games").await;
    assign(&reader, X12_3_OCT, dora).await;
    // "Correct": Dora only trained the whole season; her past duty stays and counts.
    let role = ok(&reader, "GET", &format!("/api/players/{dora}"), None).await["history"][0]["id"]
        .as_i64()
        .unwrap();
    ok(
        &reader,
        "PUT",
        &format!("/api/players/{dora}/roles/{role}"),
        Some(json!({ "role": "only_trains", "from": "2026-09-01", "until": null })),
    )
    .await;

    let duties = ok(&reader, "GET", "/api/duties", None).await;
    assert_eq!(
        row(&duties, dora).unwrap()["totals"]["table"],
        json!({ "done": 1, "planned": 0 })
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_season_without_a_team_has_no_rows(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    add_player(&reader, "Anna", "plays_games").await;
    let duties = ok(&reader, "GET", "/api/duties?season=2025-2026", None).await;
    assert_eq!(duties["season"], "2025-2026");
    assert_eq!(duties["rows"], json!([]));
    assert_eq!(duties["seasons"], json!(["2026-2027", "2025-2026"]));

    let (status, answer) = reader.send("GET", "/api/duties?season=2026", None).await;
    assert_eq!(
        (status, answer["error"].as_str()),
        (400, Some("invalid_season"))
    );
}
