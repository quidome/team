//! The page of a game and its duties through the API (story A of duties; the players on the
//! places, story B).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::time::Duration;

use common::Harness;
use common::game_list::{M16_1, M16_2, Reader, game_mut, games_with, utc};
use serde_json::{Value, json};
use sqlx::PgPool;

async fn ok(reader: &Reader, method: &str, path: &str, body: Option<Value>) -> Value {
    let (status, answer) = reader.send(method, path, body).await;
    assert_eq!(status, 200, "{method} {path}: {answer}");
    answer
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

/// The id of the first coming game that `find` accepts.
async fn first_game(reader: &Reader, find: impl Fn(&Value) -> bool) -> i64 {
    season_games(reader)
        .await
        .into_iter()
        .find(|game| find(game))
        .unwrap()["id"]
        .as_i64()
        .unwrap()
}

/// A home game of another club team.
async fn duty_game(reader: &Reader) -> i64 {
    first_game(reader, |game| game["reason"] == "duty").await
}

/// An away game of M16 1 at another club.
async fn away_game(reader: &Reader) -> i64 {
    first_game(reader, |game| {
        game["away"] == "Blue Drakes M16 1"
            && !game["home"].as_str().unwrap().starts_with("Blue Drakes")
    })
    .await
}

/// A home game of M16 1.
async fn home_game(reader: &Reader) -> i64 {
    first_game(reader, |game| game["home"] == "Blue Drakes M16 1").await
}

fn game_path(id: i64) -> String {
    format!("/api/games/2026-2027/{id}")
}

fn place_path(id: i64, kind: &str, number: u8) -> String {
    format!("/api/games/2026-2027/{id}/places/{kind}/{number}")
}

async fn set_place(reader: &Reader, id: i64, kind: &str, number: u8, team: Option<i64>) -> Value {
    ok(
        reader,
        "PUT",
        &place_path(id, kind, number),
        Some(json!({ "team": team })),
    )
    .await
}

/// The places of the team on a page, as "referee 1", "table 2".
fn of_team(page: &Value) -> Vec<String> {
    page["game"]["places"]["schedule"]["of_team"]
        .as_array()
        .unwrap()
        .iter()
        .map(|place| format!("{} {}", place["kind"].as_str().unwrap(), place["number"]))
        .collect()
}

/// The team numbers of the 4 places on a page (`None`: empty).
fn place_teams(page: &Value) -> Vec<Option<i64>> {
    page["game"]["places"]["schedule"]["places"]
        .as_array()
        .unwrap()
        .iter()
        .map(|place| place["team"]["number"].as_i64())
        .collect()
}

async fn add_players(reader: &Reader, count: usize) {
    for n in 0..count {
        ok(
            reader,
            "POST",
            "/api/players",
            Some(
                json!({ "first_name": format!("Player {n}"), "last_name": null,
                         "role": { "role": "plays_games", "from": "2026-09-01" } }),
            ),
        )
        .await;
    }
}

/// A team of the club other than M16 1 from the pull-down list of a page.
fn other_team(page: &Value) -> i64 {
    page["club_teams"]
        .as_array()
        .unwrap()
        .iter()
        .map(|team| team["number"].as_i64().unwrap())
        .find(|&number| number != M16_1)
        .unwrap()
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_game_routes_need_a_login(pool: PgPool) {
    let harness = Harness::start(pool).await;
    for path in [
        "/api/games/2026-2027/1",
        "/api/games/2026-2027/1/places/referee/1",
        "/api/games/2026-2027/1/schedule-entered",
        "/api/games/2026-2027/1/drivers",
    ] {
        let response = harness.get(path, None).await;
        assert_eq!(response.status(), 401, "{path}");
    }
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_home_game_of_another_club_team_has_four_places_given_to_club_teams(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let id = duty_game(&reader).await;

    let page = ok(&reader, "GET", &game_path(id), None).await;
    assert_eq!(page["team"]["name"], "M16 1");
    assert_eq!(place_teams(&page), [None, None, None, None]);
    let schedule = &page["game"]["places"]["schedule"];
    assert_eq!(schedule["entered"], false);
    assert_eq!(schedule["required"], true);
    assert_eq!(page["game"]["places"]["not_entered"], true);
    // The club teams of the season, sorted by name, without opponents.
    let names: Vec<&str> = page["club_teams"]
        .as_array()
        .unwrap()
        .iter()
        .map(|team| team["name"].as_str().unwrap())
        .collect();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    assert_eq!(names, sorted);
    assert!(names.contains(&"M16 1"));
    assert!(names.iter().all(|name| !name.contains("Blue Drakes")));

    let other = other_team(&page);
    set_place(&reader, id, "referee", 1, Some(M16_1)).await;
    set_place(&reader, id, "referee", 2, Some(other)).await;
    set_place(&reader, id, "table", 1, Some(M16_1)).await;
    let page = set_place(&reader, id, "table", 2, Some(M16_1)).await;
    assert_eq!(
        place_teams(&page),
        [Some(M16_1), Some(other), Some(M16_1), Some(M16_1)]
    );
    assert_eq!(of_team(&page), ["referee 1", "table 1", "table 2"]);
    assert_eq!(page["game"]["places"]["open_places"], 3);

    // "—" removes the team of a place.
    let page = set_place(&reader, id, "table", 2, None).await;
    assert_eq!(of_team(&page), ["referee 1", "table 1"]);
    let rows: i64 = sqlx::query_scalar("SELECT count(*) FROM duty_teams WHERE source_id = $1")
        .bind(id)
        .fetch_one(reader.pool())
        .await
        .unwrap();
    assert_eq!(rows, 3);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn schedule_entered_goes_on_and_off_and_keeps_its_first_moment(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let id = duty_game(&reader).await;
    let path = format!("{}/schedule-entered", game_path(id));

    let page = ok(&reader, "PUT", &path, None).await;
    assert_eq!(page["game"]["places"]["schedule"]["entered"], true);
    assert_eq!(page["game"]["places"]["not_entered"], false);
    let moment = || async {
        sqlx::query_scalar::<_, Option<chrono::DateTime<chrono::Utc>>>(
            "SELECT schedule_entered_at FROM game_duties WHERE source_id = $1",
        )
        .bind(id)
        .fetch_one(reader.pool())
        .await
        .unwrap()
    };
    let first = moment().await;
    reader.harness.clock.advance(chrono::TimeDelta::minutes(5));
    ok(&reader, "PUT", &path, None).await;
    assert_eq!(moment().await, first);

    let page = ok(&reader, "DELETE", &path, None).await;
    assert_eq!(page["game"]["places"]["schedule"]["entered"], false);
    assert_eq!(moment().await, None);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_home_game_of_the_team_has_the_same_four_places(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let id = home_game(&reader).await;
    let page = ok(&reader, "GET", &game_path(id), None).await;
    assert_eq!(place_teams(&page), [None, None, None, None]);
    assert!(page["game"]["places"]["drivers"].is_null());
    let other = other_team(&page);
    let page = set_place(&reader, id, "referee", 1, Some(other)).await;
    assert_eq!(place_teams(&page)[0], Some(other));
    assert!(of_team(&page).is_empty());
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn an_away_game_proposes_drivers_until_a_number_is_set(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    add_players(&reader, 4).await;
    let id = away_game(&reader).await;
    let path = format!("{}/drivers", game_path(id));

    let page = ok(&reader, "GET", &game_path(id), None).await;
    let drivers = &page["game"]["places"]["drivers"];
    assert!(page["game"]["places"]["schedule"].is_null());
    assert_eq!(drivers["proposal"], 2);
    assert_eq!(drivers["proposed"], true);
    assert_eq!(drivers["places"].as_array().unwrap().len(), 2);

    let page = ok(&reader, "PUT", &path, Some(json!({ "drivers": 3 }))).await;
    let drivers = &page["game"]["places"]["drivers"];
    assert_eq!(drivers["proposed"], false);
    assert_eq!(drivers["places"].as_array().unwrap().len(), 3);

    // A set number stays when the team grows.
    add_players(&reader, 6).await;
    let page = ok(&reader, "GET", &game_path(id), None).await;
    assert_eq!(page["game"]["places"]["drivers"]["proposal"], 4);
    assert_eq!(
        page["game"]["places"]["drivers"]["places"]
            .as_array()
            .unwrap()
            .len(),
        3
    );

    // "Use proposal".
    let page = ok(&reader, "PUT", &path, Some(json!({ "drivers": null }))).await;
    assert_eq!(page["game"]["places"]["drivers"]["proposed"], true);
    assert_eq!(
        page["game"]["places"]["drivers"]["places"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_write_that_the_game_does_not_allow_is_refused(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let duty = duty_game(&reader).await;
    let away = away_game(&reader).await;
    let refused = |method: &'static str, path: String, body: Option<Value>, code: &'static str| {
        let reader = &reader;
        async move {
            let (status, answer) = reader.send(method, &path, body).await;
            assert_eq!((status, &answer["error"]), (400, &json!(code)), "{path}");
        }
    };
    let team = Some(json!({ "team": M16_1 }));
    refused(
        "PUT",
        place_path(duty, "drive", 1),
        team.clone(),
        "invalid_place",
    )
    .await;
    refused(
        "PUT",
        place_path(duty, "referee", 3),
        team.clone(),
        "invalid_place",
    )
    .await;
    refused(
        "PUT",
        place_path(duty, "referee", 1),
        Some(json!({ "team": 999_999 })),
        "unknown_team",
    )
    .await;
    refused(
        "PUT",
        place_path(away, "table", 1),
        team.clone(),
        "not_a_home_game",
    )
    .await;
    refused(
        "PUT",
        format!("{}/schedule-entered", game_path(away)),
        None,
        "not_a_home_game",
    )
    .await;
    refused(
        "PUT",
        format!("{}/drivers", game_path(duty)),
        Some(json!({ "drivers": 2 })),
        "no_drivers_needed",
    )
    .await;
    refused(
        "PUT",
        format!("{}/drivers", game_path(away)),
        Some(json!({ "drivers": 11 })),
        "out_of_range",
    )
    .await;
    refused(
        "GET",
        "/api/games/2026-27/1".to_owned(),
        None,
        "invalid_season",
    )
    .await;
    refused(
        "GET",
        "/api/games/2026-2027/x".to_owned(),
        None,
        "invalid_game",
    )
    .await;

    // A game id that does not exist, and a game of the club outside the schedule (an away game
    // of another club team).
    let (status, _) = reader.send("GET", &game_path(1), None).await;
    assert_eq!(status, 404);
    let outside: i64 = sqlx::query_scalar(
        "SELECT source_id FROM games WHERE away_club_number = 37 AND home_club_number <> 37
           AND away_team_number <> $1 LIMIT 1",
    )
    .bind(M16_1)
    .fetch_one(reader.pool())
    .await
    .unwrap();
    let (status, _) = reader.send("GET", &game_path(outside), None).await;
    assert_eq!(status, 404);
    // Nothing was written by a refused write.
    let rows: i64 = sqlx::query_scalar("SELECT count(*) FROM game_duties")
        .fetch_one(reader.pool())
        .await
        .unwrap();
    assert_eq!(rows, 0);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn an_earlier_season_can_be_changed(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let id = duty_game(&reader).await;
    reader.harness.clock.set(utc(2027, 7, 1, 10, 0));
    reader.log_in_again().await;
    let page = set_place(&reader, id, "table", 1, Some(M16_1)).await;
    assert_eq!(page["season"], "2026-2027");
    assert_eq!(page["current_season"], "2027-2028");
    assert_eq!(of_team(&page), ["table 1"]);
    // A past game counts.
    assert_eq!(page["game"]["places"]["schedule"]["counts"], true);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn another_team_chosen_keeps_the_places_of_the_old_team(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let id = first_game(&reader, |game| {
        game["reason"] == "duty"
            && game["home"] != "Blue Drakes M16 2"
            && game["away"] != "Blue Drakes M16 2"
    })
    .await;
    set_place(&reader, id, "table", 1, Some(M16_1)).await;
    set_place(&reader, id, "table", 2, Some(M16_2)).await;

    reader.choose(M16_2).await;
    let page = ok(&reader, "GET", &game_path(id), None).await;
    assert_eq!(page["team"]["name"], "M16 2");
    assert_eq!(place_teams(&page)[2..], [Some(M16_1), Some(M16_2)]);
    assert_eq!(of_team(&page), ["table 2"]);

    reader.choose(M16_1).await;
    let page = ok(&reader, "GET", &game_path(id), None).await;
    assert_eq!(of_team(&page), ["table 1"]);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_place_waits_for_an_insert_of_the_same_place_on_another_screen(pool: PgPool) {
    let reader = Reader::chosen(pool.clone()).await;
    reader.read_good().await;
    let id = duty_game(&reader).await;
    let page = ok(&reader, "GET", &game_path(id), None).await;
    let other = other_team(&page);
    // Another screen inserts the same place in an open transaction.
    let mut screen = pool.begin().await.unwrap();
    sqlx::query(
        "INSERT INTO game_duties (club_number, season_year, source_id, saved_at)
         VALUES (37, 2026, $1, now())",
    )
    .bind(id)
    .execute(&mut *screen)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO duty_teams
             (club_number, season_year, source_id, kind, number, team_number, team_name)
         VALUES (37, 2026, $1, 'referee', 1, $2, 'other')",
    )
    .bind(id)
    .bind(other)
    .execute(&mut *screen)
    .await
    .unwrap();
    let path = place_path(id, "referee", 1);
    let save = reader.send("PUT", &path, Some(json!({ "team": M16_1 })));
    tokio::pin!(save);
    let waited = tokio::time::timeout(Duration::from_millis(300), &mut save).await;
    assert!(
        waited.is_err(),
        "the save did not wait for the other screen"
    );
    screen.commit().await.unwrap();
    // The last save wins; one place holds one team.
    let (status, page) = save.await;
    assert_eq!(status, 200, "{page}");
    assert_eq!(place_teams(&page)[0], Some(M16_1));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_save_waits_for_a_save_of_the_same_game_on_another_screen(pool: PgPool) {
    let reader = Reader::chosen(pool.clone()).await;
    reader.read_good().await;
    let id = away_game(&reader).await;
    let path = format!("{}/drivers", game_path(id));
    ok(&reader, "PUT", &path, Some(json!({ "drivers": 1 }))).await;
    // Another screen holds the row of the game in an open transaction. This test cannot fail
    // without the lock (the update waits by itself);
    // `a_place_waits_for_a_player_put_on_it_on_another_screen` can.
    let mut screen = pool.begin().await.unwrap();
    sqlx::query("SELECT 1 FROM game_duties WHERE source_id = $1 FOR UPDATE")
        .bind(id)
        .execute(&mut *screen)
        .await
        .unwrap();
    let save = reader.send("PUT", &path, Some(json!({ "drivers": 2 })));
    tokio::pin!(save);
    let waited = tokio::time::timeout(Duration::from_millis(300), &mut save).await;
    assert!(
        waited.is_err(),
        "the save did not wait for the other screen"
    );
    screen.commit().await.unwrap();
    let (status, page) = save.await;
    assert_eq!(status, 200, "{page}");
    assert_eq!(
        page["game"]["places"]["drivers"]["places"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_failed_save_writes_nothing(pool: PgPool) {
    let reader = Reader::chosen(pool.clone()).await;
    reader.read_good().await;
    let id = duty_game(&reader).await;
    // The second write of the save (the place) fails.
    sqlx::raw_sql(
        "CREATE FUNCTION refuse_duty_teams() RETURNS trigger LANGUAGE plpgsql AS
           $$ BEGIN RAISE EXCEPTION 'refused by the test'; END $$;
         CREATE TRIGGER refuse BEFORE INSERT ON duty_teams
           FOR EACH ROW EXECUTE FUNCTION refuse_duty_teams();",
    )
    .execute(&pool)
    .await
    .unwrap();
    let (status, _) = reader
        .send(
            "PUT",
            &place_path(id, "referee", 1),
            Some(json!({ "team": M16_1 })),
        )
        .await;
    assert_eq!(status, 500);
    let rows: i64 = sqlx::query_scalar("SELECT count(*) FROM game_duties")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(rows, 0);
}

// The duties on the answers of "Season", the start page and the list of changes (step 3).

/// The game event `id` of "Season" (of the season of the clock), if it shows.
async fn season_game(reader: &Reader, id: i64) -> Option<Value> {
    season_games(reader)
        .await
        .into_iter()
        .find(|game| game["id"] == id)
}

async fn start(reader: &Reader) -> Value {
    ok(reader, "GET", "/api/start", None).await
}

/// The game ids of the open item "schedule not entered" (empty without it).
fn not_entered_ids(start: &Value) -> Vec<i64> {
    start["open_items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["kind"] == "schedule_not_entered")
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

/// The change of game `id` in the list of changes.
async fn change_of(reader: &Reader, id: i64) -> Value {
    let answer = ok(reader, "GET", "/api/changes", None).await;
    assert_eq!(answer["season"], "2026-2027");
    answer["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|change| change["id"] == id)
        .unwrap()
        .clone()
}

/// Swaps home and away of a game of the source.
fn swap(game: &mut Value) {
    let object = game.as_object_mut().unwrap();
    let keys: Vec<String> = object
        .keys()
        .filter(|key| key.starts_with("thuis_"))
        .cloned()
        .collect();
    for home in keys {
        let away = home.replacen("thuis_", "uit_", 1);
        let home_value = object.remove(&home).unwrap();
        if let Some(away_value) = object.insert(away, home_value) {
            object.insert(home, away_value);
        }
    }
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn season_shows_the_duties_of_its_games(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let id = duty_game(&reader).await;
    set_place(&reader, id, "table", 1, Some(M16_1)).await;
    let game = season_game(&reader, id).await.unwrap();
    assert_eq!(game["places"]["schedule"]["of_team"][0]["kind"], "table");
    assert_eq!(game["places"]["not_entered"], true);
    assert_eq!(game["places"]["open_places"], 1);
    // A home game of the team has the 4 places too; an away game its drivers.
    let home = season_game(&reader, home_game(&reader).await)
        .await
        .unwrap();
    assert_eq!(
        home["places"]["schedule"]["places"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    let away = season_game(&reader, away_game(&reader).await)
        .await
        .unwrap();
    assert!(!away["places"]["drivers"].is_null());
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_moved_game_keeps_its_duties_and_its_change_has_them(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let id = duty_game(&reader).await;
    set_place(&reader, id, "referee", 1, Some(M16_1)).await;
    ok(
        &reader,
        "PUT",
        &format!("{}/schedule-entered", game_path(id)),
        None,
    )
    .await;
    reader.answer(games_with(|games| {
        game_mut(games, id)["tijd"] = json!("09:15");
    }));
    reader.read_good().await;
    let page = ok(&reader, "GET", &game_path(id), None).await;
    assert_eq!(page["game"]["start_time"], "09:15");
    assert_eq!(of_team(&page), ["referee 1"]);
    assert_eq!(page["game"]["places"]["schedule"]["entered"], true);
    let change = change_of(&reader, id).await;
    assert_eq!(change["kind"], "moved");
    assert_eq!(
        change["places"]["schedule"]["of_team"][0]["kind"],
        "referee"
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_swapped_game_stays_on_season_with_its_places_and_its_change_has_them(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let id = duty_game(&reader).await;
    set_place(&reader, id, "table", 1, Some(M16_1)).await;
    // Home and away swap: another club team now plays away at another club.
    reader.answer(games_with(|games| swap(game_mut(games, id))));
    reader.read_good().await;

    let game = season_game(&reader, id).await.unwrap();
    assert_eq!(game["reason"], "kept");
    assert_eq!(game["places"]["schedule"]["required"], false);
    assert_eq!(game["places"]["schedule"]["counts"], false);
    assert_eq!(game["places"]["open_places"], 0);
    let change = change_of(&reader, id).await;
    assert_eq!(change["places"]["schedule"]["required"], false);
    assert!(!not_entered_ids(&start(&reader).await).contains(&id));

    // "—" removes the place; the game then leaves "Season".
    set_place(&reader, id, "table", 1, None).await;
    ok(
        &reader,
        "POST",
        "/api/changes/seen",
        Some(json!({ "changes": [
        { "id": id, "version": change["version"] }
    ] })),
    )
    .await;
    assert!(season_game(&reader, id).await.is_none());
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_gone_game_keeps_its_places_on_season_and_counts_again_when_past(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let id = duty_game(&reader).await;
    set_place(&reader, id, "referee", 2, Some(M16_1)).await;
    reader.answer(games_with(|games| games.retain(|game| game["id"] != id)));
    reader.read_good().await;
    let change = change_of(&reader, id).await;
    ok(
        &reader,
        "POST",
        "/api/changes/seen",
        Some(json!({ "changes": [
        { "id": id, "version": change["version"] }
    ] })),
    )
    .await;
    // After "seen" it still shows, for its places; it does not count and is no open item.
    let game = season_game(&reader, id).await.unwrap();
    assert_eq!(game["in_list"], false);
    assert_eq!(game["places"]["schedule"]["counts"], false);
    assert!(!not_entered_ids(&start(&reader).await).contains(&id));
    // After its date it counts.
    reader.harness.clock.set(utc(2027, 5, 1, 12, 0));
    reader.log_in_again().await;
    let game = season_game(&reader, id).await.unwrap();
    assert_eq!(game["places"]["schedule"]["counts"], true);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_team_that_leaves_the_game_list_keeps_its_name_on_the_place(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let id = duty_game(&reader).await;
    let page = ok(&reader, "GET", &game_path(id), None).await;
    // A club team that does not play this game.
    let (leaving, name) = page["club_teams"]
        .as_array()
        .unwrap()
        .iter()
        .map(|team| (team["number"].as_i64().unwrap(), team["name"].clone()))
        .find(|(number, name)| {
            let short = name.as_str().unwrap();
            *number != M16_1
                && !page["game"]["home"].as_str().unwrap().ends_with(short)
                && !page["game"]["away"].as_str().unwrap().ends_with(short)
        })
        .unwrap();
    set_place(&reader, id, "referee", 1, Some(leaving)).await;
    // Every game of that team leaves the game list.
    reader.answer(games_with(|games| {
        games.retain(|game| game["thuis_ploeg_id"] != leaving && game["uit_ploeg_id"] != leaving);
    }));
    reader.read_good().await;
    let page = ok(&reader, "GET", &game_path(id), None).await;
    assert_eq!(
        page["game"]["places"]["schedule"]["places"][0]["team"]["name"],
        name
    );
    assert!(
        page["club_teams"]
            .as_array()
            .unwrap()
            .iter()
            .any(|team| team["number"] == leaving && team["name"] == name)
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn schedule_not_entered_lists_the_home_games_of_4_weeks_and_shrinks(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let answer = start(&reader).await;
    let item = answer["open_items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["kind"] == "schedule_not_entered")
        .unwrap()
        .clone();
    let games = item["games"].as_array().unwrap();
    // Mon 12 Oct: from today until Sun 8 Nov (today + 27 days), in time order.
    let dates: Vec<&str> = games
        .iter()
        .map(|game| game["date"].as_str().unwrap())
        .collect();
    assert!(!dates.is_empty());
    let mut sorted = dates.clone();
    sorted.sort_unstable();
    assert_eq!(dates, sorted);
    assert!(
        dates
            .iter()
            .all(|&date| ("2026-10-12"..="2026-11-08").contains(&date))
    );
    assert!(games.iter().all(|game| game["season"] == "2026-2027"));
    // The home games of 14 November are 33 days ahead: not in it.
    assert!(!dates.contains(&"2026-11-14"));

    let first = games[0]["id"].as_i64().unwrap();
    ok(
        &reader,
        "PUT",
        &format!("{}/schedule-entered", game_path(first)),
        None,
    )
    .await;
    let ids = not_entered_ids(&start(&reader).await);
    assert_eq!(ids.len(), games.len() - 1);
    assert!(!ids.contains(&first));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn next_shows_a_home_game_of_another_club_team_with_a_place_of_the_team(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    // Mon 26 Oct: "Next" runs until Sun 8 Nov, with the home games of 31 October.
    reader.harness.clock.set(utc(2026, 10, 26, 12, 0));
    reader.log_in_again().await;
    let ours = first_game(&reader, |game| {
        game["reason"] == "duty" && game["date"] == "2026-10-31"
    })
    .await;
    set_place(&reader, ours, "table", 1, Some(M16_1)).await;
    let next = start(&reader).await["next"].clone();
    let duty_ids: Vec<i64> = next
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["game"]["reason"] == "duty")
        .map(|event| event["game"]["id"].as_i64().unwrap())
        .collect();
    assert_eq!(duty_ids, [ours]);
    let event = next
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["game"]["id"] == ours)
        .unwrap();
    assert_eq!(
        event["game"]["places"]["schedule"]["of_team"][0]["kind"],
        "table"
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn last_has_all_games_of_the_last_date_with_places_also_without_a_training(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    // Two home games of other club teams on 3 October with places of the team; no trainings.
    let on_3_october: Vec<i64> = season_games(&reader)
        .await
        .into_iter()
        .filter(|game| game["reason"] == "duty" && game["date"] == "2026-10-03")
        .map(|game| game["id"].as_i64().unwrap())
        .take(2)
        .collect();
    assert_eq!(on_3_october.len(), 2);
    for &id in &on_3_october {
        set_place(&reader, id, "referee", 1, Some(M16_1)).await;
    }
    let answer = start(&reader).await;
    assert!(answer["last"].is_null());
    assert_eq!(answer["last_games_season"], "2026-2027");
    let mut ids: Vec<i64> = answer["last_games"]
        .as_array()
        .unwrap()
        .iter()
        .map(|event| event["game"]["id"].as_i64().unwrap())
        .collect();
    ids.sort_unstable();
    let mut expected = on_3_october.clone();
    expected.sort_unstable();
    assert_eq!(ids, expected);

    // In July the last games come from the season before.
    reader.harness.clock.set(utc(2027, 7, 2, 12, 0));
    reader.log_in_again().await;
    let answer = start(&reader).await;
    assert_eq!(answer["last_games_season"], "2026-2027");
    assert!(!answer["last_games"].as_array().unwrap().is_empty());
}

// The players on the places (story B of duties, step 3). The games are from the real game list:
// on Sat 3 Oct (past) and Sat 31 Oct 2026 (coming) the club plays at home in hall Bulwijk, 13:00
// on Veld 1 (X12 1) and 14:30 on Veld 2 (X14 1); on Sat 10 Oct (past) M16 1 plays away.

const X12_31_OCT: i64 = 1_081_923;
const X14_31_OCT: i64 = 1_083_716;
const X12_3_OCT: i64 = 1_081_913;
const AWAY_10_OCT: i64 = 1_073_147;

fn assignment_path(id: i64, kind: &str, number: u8) -> String {
    format!("/api/games/2026-2027/{id}/assignments/{kind}/{number}")
}

async fn assign(reader: &Reader, id: i64, kind: &str, number: u8, player: i64) -> (u16, Value) {
    reader
        .send(
            "PUT",
            &assignment_path(id, kind, number),
            Some(json!({ "player": player })),
        )
        .await
}

async fn add_player(reader: &Reader, first: &str, last: Option<&str>, role: &str) -> i64 {
    let answer = ok(
        reader,
        "POST",
        "/api/players",
        Some(json!({ "first_name": first, "last_name": last,
                     "role": { "role": role, "from": "2026-09-01" } })),
    )
    .await;
    answer["player"]["id"].as_i64().unwrap()
}

/// The player on the place of the team `index` of a page (table and referee places).
fn on_place(page: &Value, index: usize) -> &Value {
    &page["game"]["places"]["schedule"]["of_team"][index]["player"]
}

async fn assignments(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM duty_assignments")
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_assignment_routes_need_a_login(pool: PgPool) {
    let harness = Harness::start(pool).await;
    for path in [
        "/api/games/2026-2027/1/assignments/table/1",
        "/api/games/2026-2027/1/assignments/table/1/outcome",
    ] {
        let response = harness.get(path, None).await;
        assert_eq!(response.status(), 401, "{path}");
    }
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_player_goes_on_a_place_of_the_team_and_is_cleared(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    set_place(&reader, X12_31_OCT, "table", 1, Some(M16_1)).await;
    let anna = add_player(&reader, "Anna", Some("Bakker"), "plays_games").await;
    add_player(&reader, "Anna", Some("Visser"), "plays_games").await;

    let (status, page) = assign(&reader, X12_31_OCT, "table", 1, anna).await;
    assert_eq!(status, 200, "{page}");
    let player = on_place(&page, 0);
    assert_eq!(player["id"], anna);
    assert_eq!(player["name"], "Anna B.");
    assert_eq!(player["outcome"], "as_planned");
    assert_eq!(player["not_in_team"], false);
    assert_eq!(page["game"]["places"]["open_places"], 0);
    // "Season" shows the player too.
    let game = season_game(&reader, X12_31_OCT).await.unwrap();
    assert_eq!(
        game["places"]["schedule"]["of_team"][0]["player"]["name"],
        "Anna B."
    );

    let page = ok(
        &reader,
        "DELETE",
        &assignment_path(X12_31_OCT, "table", 1),
        None,
    )
    .await;
    assert!(on_place(&page, 0).is_null());
    assert_eq!(page["game"]["places"]["open_places"], 1);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_place_after_the_game_is_corrected(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    set_place(&reader, X12_3_OCT, "table", 1, Some(M16_1)).await;
    let anna = add_player(&reader, "Anna", None, "plays_games").await;
    let bram = add_player(&reader, "Bram", None, "plays_games").await;
    assign(&reader, X12_3_OCT, "table", 1, anna).await;
    let path = format!("{}/outcome", assignment_path(X12_3_OCT, "table", 1));

    let page = ok(
        &reader,
        "PUT",
        &path,
        Some(json!({ "outcome": "not_done" })),
    )
    .await;
    assert_eq!(on_place(&page, 0)["outcome"], "not_done");
    assert_eq!(on_place(&page, 0)["name"], "Anna");
    let page = ok(
        &reader,
        "PUT",
        &path,
        Some(json!({ "outcome": "done_by", "player": bram })),
    )
    .await;
    assert_eq!(on_place(&page, 0)["outcome"], "done_by");
    assert_eq!(on_place(&page, 0)["done_by"]["name"], "Bram");
    // Another planned player resets the correction.
    let (_, page) = assign(&reader, X12_3_OCT, "table", 1, bram).await;
    assert_eq!(on_place(&page, 0)["outcome"], "as_planned");

    // A coming game cannot be corrected; "done by" needs a player.
    set_place(&reader, X12_31_OCT, "table", 1, Some(M16_1)).await;
    assign(&reader, X12_31_OCT, "table", 1, anna).await;
    let coming = format!("{}/outcome", assignment_path(X12_31_OCT, "table", 1));
    let (status, answer) = reader
        .send("PUT", &coming, Some(json!({ "outcome": "not_done" })))
        .await;
    assert_eq!((status, answer["error"].as_str()), (400, Some("not_past")));
    let (status, answer) = reader
        .send("PUT", &path, Some(json!({ "outcome": "done_by" })))
        .await;
    assert_eq!(
        (status, answer["error"].as_str()),
        (400, Some("invalid_outcome"))
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn an_assignment_that_the_place_does_not_allow_is_refused(pool: PgPool) {
    let reader = Reader::chosen(pool.clone()).await;
    reader.read_good().await;
    let page = set_place(&reader, X12_31_OCT, "table", 1, Some(M16_1)).await;
    set_place(&reader, X12_31_OCT, "referee", 1, Some(M16_1)).await;
    set_place(&reader, X12_31_OCT, "table", 2, Some(other_team(&page))).await;
    let anna = add_player(&reader, "Anna", None, "plays_games").await;
    let cas = add_player(&reader, "Cas", None, "only_trains").await;
    assign(&reader, X12_31_OCT, "table", 1, anna).await;
    let before = assignments(&pool).await;

    for (kind, number, player, code) in [
        ("table", 2, anna, "not_a_place_of_the_team"),
        ("referee", 2, anna, "not_a_place_of_the_team"),
        ("referee", 1, anna, "player_at_game"),
        ("referee", 1, cas, "not_in_team"),
        ("referee", 1, 999_999, "unknown_player"),
        ("jury", 1, anna, "invalid_place"),
        ("drive", 11, anna, "invalid_place"),
    ] {
        let (status, answer) = assign(&reader, X12_31_OCT, kind, number, player).await;
        assert_eq!(
            (status, answer["error"].as_str()),
            (400, Some(code)),
            "{kind} {number}"
        );
    }
    assert_eq!(assignments(&pool).await, before);

    // The place of the team with a player keeps its team.
    let (status, answer) = reader
        .send(
            "PUT",
            &place_path(X12_31_OCT, "table", 1),
            Some(json!({ "team": null })),
        )
        .await;
    assert_eq!(
        (status, answer["error"].as_str()),
        (400, Some("player_on_place"))
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_close_clash_blocks_and_a_tight_one_does_not(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    for id in [X12_31_OCT, X14_31_OCT] {
        set_place(&reader, id, "table", 1, Some(M16_1)).await;
    }
    let anna = add_player(&reader, "Anna", None, "plays_games").await;
    assign(&reader, X12_31_OCT, "table", 1, anna).await;
    // 13:00 on Veld 1 and 14:30 on Veld 2: tight, so a warning only.
    let (status, page) = assign(&reader, X14_31_OCT, "table", 1, anna).await;
    assert_eq!(status, 200, "{page}");
    ok(
        &reader,
        "DELETE",
        &assignment_path(X14_31_OCT, "table", 1),
        None,
    )
    .await;
    // The X14 game moves to 14:00 on Veld 2: blocked.
    reader.answer(games_with(|games| {
        game_mut(games, X14_31_OCT)["tijd"] = json!("14:00");
    }));
    reader.read_good().await;
    let (status, answer) = assign(&reader, X14_31_OCT, "table", 1, anna).await;
    assert_eq!((status, answer["error"].as_str()), (400, Some("clash")));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn drivers_cannot_drop_below_the_highest_filled_place(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let id = away_game(&reader).await;
    let anna = add_player(&reader, "Anna", None, "plays_games").await;
    let path = format!("{}/drivers", game_path(id));
    ok(&reader, "PUT", &path, Some(json!({ "drivers": 3 }))).await;
    let (status, page) = assign(&reader, id, "drive", 3, anna).await;
    assert_eq!(status, 200, "{page}");
    let (status, answer) = reader
        .send("PUT", &path, Some(json!({ "drivers": 2 })))
        .await;
    assert_eq!(status, 400);
    assert_eq!(answer["error"], "places_filled");
    assert_eq!(
        (answer["filled"].as_i64(), answer["highest"].as_i64()),
        (Some(1), Some(3))
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_place_waits_for_a_player_put_on_it_on_another_screen(pool: PgPool) {
    // R4: without the row lock the change of team reads no player, and both saves succeed: a
    // player on a place of another team.
    let reader = Reader::chosen(pool.clone()).await;
    reader.read_good().await;
    let page = set_place(&reader, X12_31_OCT, "table", 1, Some(M16_1)).await;
    let other = other_team(&page);
    let anna = add_player(&reader, "Anna", None, "plays_games").await;
    // Another screen puts Anna on table 1 under the lock, and holds it.
    let mut screen = pool.begin().await.unwrap();
    sqlx::query("SELECT 1 FROM game_duties WHERE source_id = $1 FOR UPDATE")
        .bind(X12_31_OCT)
        .execute(&mut *screen)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO duty_assignments
             (club_number, season_year, source_id, kind, number, planned_player)
         VALUES (37, 2026, $1, 'table', 1, $2)",
    )
    .bind(X12_31_OCT)
    .bind(anna)
    .execute(&mut *screen)
    .await
    .unwrap();
    let path = place_path(X12_31_OCT, "table", 1);
    let save = reader.send("PUT", &path, Some(json!({ "team": other })));
    tokio::pin!(save);
    let waited = tokio::time::timeout(Duration::from_millis(300), &mut save).await;
    assert!(
        waited.is_err(),
        "the save did not wait for the other screen"
    );
    screen.commit().await.unwrap();
    let (status, answer) = save.await;
    assert_eq!(
        (status, answer["error"].as_str()),
        (400, Some("player_on_place"))
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_player_with_a_duty_cannot_be_deleted(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    set_place(&reader, X12_31_OCT, "table", 1, Some(M16_1)).await;
    let anna = add_player(&reader, "Anna", None, "plays_games").await;
    assign(&reader, X12_31_OCT, "table", 1, anna).await;
    let player = ok(&reader, "GET", &format!("/api/players/{anna}"), None).await;
    assert_eq!(player["has_duties"], true);
    let (status, answer) = reader
        .send("DELETE", &format!("/api/players/{anna}"), None)
        .await;
    assert_eq!(
        (status, answer["error"].as_str()),
        (400, Some("player_has_duties"))
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_past_game_with_a_player_on_a_proposed_driver_place_stays(pool: PgPool) {
    // R3: no number of drivers set and no place names a team; only the player keeps the game.
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let anna = add_player(&reader, "Anna", None, "plays_games").await;
    let (status, page) = assign(&reader, AWAY_10_OCT, "drive", 1, anna).await;
    assert_eq!(status, 200, "{page}");
    reader.answer(games_with(|games| {
        games.retain(|game| game["id"] != AWAY_10_OCT)
    }));
    reader.read_good().await;
    reader.log_in_again().await;
    let game = season_game(&reader, AWAY_10_OCT).await.unwrap();
    assert_eq!(game["in_list"], false);
    assert_eq!(game["places"]["drivers"]["counts"], true);
    assert_eq!(
        game["places"]["drivers"]["places"][0]["player"]["name"],
        "Anna"
    );
}
