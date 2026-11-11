//! "Checked" of a game through the API (story C of attendance at games): each "no answer" becomes
//! absent, "Undo check" brings back only those, and the check holds for the date of the game.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::Harness;
use common::game_list::{Reader, game_mut, games_with, utc};
use serde_json::{Value, json};
use sqlx::PgPool;

/// The away game of M16 1 on Saturday 10 October 2026 at 15:00, before the test clock.
const AWAY_10_OCT: i64 = 1_073_147;
/// The away game of M16 1 on Saturday 7 November 2026 at 11:15, after the test clock.
const AWAY_7_NOV: i64 = 1_073_159;

async fn ok(reader: &Reader, method: &str, path: &str, body: Option<Value>) -> Value {
    let (status, answer) = reader.send(method, path, body).await;
    assert_eq!(status, 200, "{method} {path}: {answer}");
    answer
}

fn game_path(id: i64) -> String {
    format!("/api/games/2026-2027/{id}")
}

fn checked_path(id: i64) -> String {
    format!("{}/checked", game_path(id))
}

async fn add_player(reader: &Reader, first: &str) -> i64 {
    ok(
        reader,
        "POST",
        "/api/players",
        Some(
            json!({ "first_name": first, "role": { "role": "plays_games", "from": "2026-09-01" } }),
        ),
    )
    .await["player"]["id"]
        .as_i64()
        .unwrap()
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

async fn answer(reader: &Reader, id: i64, player: i64, answer: &str, reason: Option<i64>) {
    ok(
        reader,
        "PUT",
        &format!("{}/answers/{player}", game_path(id)),
        Some(json!({ "answer": answer, "reason": reason })),
    )
    .await;
}

/// The answer and the reason of each player on the list of game `id`, by player.
async fn answers_of(reader: &Reader, id: i64) -> Vec<(i64, Value, Value)> {
    let page = ok(reader, "GET", &game_path(id), None).await;
    let mut rows: Vec<(i64, Value, Value)> = page["attendance"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                row["player"].as_i64().unwrap(),
                row["answer"].clone(),
                row["reason"].clone(),
            )
        })
        .collect();
    rows.sort_by_key(|row| row.0);
    rows
}

async fn is_checked(reader: &Reader, id: i64) -> bool {
    ok(reader, "GET", &game_path(id), None).await["attendance"]["checked"]
        .as_bool()
        .unwrap()
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn checked_needs_a_login(pool: PgPool) {
    let harness = Harness::start(pool).await;
    for method in ["PUT", "DELETE"] {
        let request = axum::http::Request::builder()
            .method(method)
            .uri(checked_path(AWAY_10_OCT))
            .header(axum::http::header::ORIGIN, common::ORIGIN)
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(harness.send(request).await.status(), 401, "{method}");
    }
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn checked_settles_each_no_answer_and_undo_brings_back_only_those(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let anna = add_player(&reader, "Anna").await;
    let bram = add_player(&reader, "Bram").await;
    let cas = add_player(&reader, "Cas").await;
    let dora = add_player(&reader, "Dora").await;
    let school = add_reason(&reader, "Exam").await;
    // Anna was marked before the check.
    answer(&reader, AWAY_10_OCT, anna, "coming", None).await;

    let checked = ok(&reader, "PUT", &checked_path(AWAY_10_OCT), None).await;
    assert_eq!(checked["settled"], 3);
    let attendance = &checked["page"]["attendance"];
    assert_eq!(attendance["checked"], true);
    assert_eq!(attendance["count"]["coming"], 1);
    assert_eq!(attendance["count"]["not_coming"], 3);
    assert_eq!(attendance["count"]["no_answer"], 0);
    // Settled without a reason.
    assert!(
        answers_of(&reader, AWAY_10_OCT)
            .await
            .iter()
            .filter(|row| row.0 != anna)
            .all(|row| row.1 == "not_coming" && row.2.is_null())
    );

    // After the check: Bram is tapped "coming", Cas gets a reason (S34a).
    answer(&reader, AWAY_10_OCT, bram, "coming", None).await;
    answer(&reader, AWAY_10_OCT, cas, "not_coming", Some(school)).await;
    // A second "Checked" changes nothing.
    let again = ok(&reader, "PUT", &checked_path(AWAY_10_OCT), None).await;
    assert_eq!(again["settled"], 0);

    let page = ok(&reader, "DELETE", &checked_path(AWAY_10_OCT), None).await;
    assert_eq!(page["attendance"]["checked"], false);
    assert_eq!(
        answers_of(&reader, AWAY_10_OCT).await,
        [
            (anna, json!("coming"), json!(null)),
            (bram, json!("coming"), json!(null)),
            (cas, json!("not_coming"), json!(school)),
            (dora, json!(null), json!(null)),
        ]
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn checked_is_refused_before_the_start_time_and_on_another_game(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let (status, answer) = reader.send("PUT", &checked_path(AWAY_7_NOV), None).await;
    assert_eq!(
        (status, answer["error"].as_str()),
        (400, Some("checked_too_early"))
    );
    // A home game of another club team has no list.
    let season = ok(&reader, "GET", "/api/season?season=2026-2027", None).await;
    let duty = season["coming"]
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["game"]["reason"] == "duty")
        .unwrap()["game"]["id"]
        .as_i64()
        .unwrap();
    let (status, answer) = reader.send("PUT", &checked_path(duty), None).await;
    assert_eq!(
        (status, answer["error"].as_str()),
        (400, Some("not_a_team_game"))
    );
    // A game with no player on its list can be checked.
    let checked = ok(&reader, "PUT", &checked_path(AWAY_10_OCT), None).await;
    assert_eq!(checked["settled"], 0);
    assert_eq!(checked["page"]["attendance"]["checked"], true);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_moved_game_shows_its_settled_players_as_no_answer_until_it_moves_back(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let anna = add_player(&reader, "Anna").await;
    let bram = add_player(&reader, "Bram").await;
    let checked = ok(&reader, "PUT", &checked_path(AWAY_10_OCT), None).await;
    assert_eq!(checked["settled"], 2);

    // Moved to Sunday 11 October: the check has no effect, the settled players have no answer.
    reader.answer(games_with(|games| {
        game_mut(games, AWAY_10_OCT)["datum"] = json!("2026-10-11 15:00:00.000");
    }));
    reader.read_good().await;
    assert!(!is_checked(&reader, AWAY_10_OCT).await);
    assert!(
        answers_of(&reader, AWAY_10_OCT)
            .await
            .iter()
            .all(|row| row.1.is_null()),
        "the settled answers have no effect"
    );
    // The row has no count of settled answers either.
    let page = ok(&reader, "GET", &game_path(AWAY_10_OCT), None).await;
    assert_eq!(page["attendance"]["count"]["no_answer"], 2);

    // Back on the date of the check: both hold again.
    reader.answer(games_with(|_| {}));
    reader.read_good().await;
    assert!(is_checked(&reader, AWAY_10_OCT).await);
    assert_eq!(
        answers_of(&reader, AWAY_10_OCT).await,
        [
            (anna, json!("not_coming"), json!(null)),
            (bram, json!("not_coming"), json!(null)),
        ]
    );

    // Moved again and checked on the new date: the two players are absent again, and the answer
    // says so (S41).
    reader.answer(games_with(|games| {
        game_mut(games, AWAY_10_OCT)["datum"] = json!("2026-10-11 15:00:00.000");
    }));
    reader.read_good().await;
    let checked = ok(&reader, "PUT", &checked_path(AWAY_10_OCT), None).await;
    assert_eq!(checked["settled"], 2);
    assert!(is_checked(&reader, AWAY_10_OCT).await);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_not_played_game_can_be_checked(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    add_player(&reader, "Anna").await;
    ok(
        &reader,
        "PUT",
        &format!("{}/not-played", game_path(AWAY_10_OCT)),
        None,
    )
    .await;
    let checked = ok(&reader, "PUT", &checked_path(AWAY_10_OCT), None).await;
    assert_eq!(checked["settled"], 1);
    assert_eq!(checked["page"]["not_played"], true);
    assert_eq!(checked["page"]["attendance"]["checked"], true);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_second_check_waits_for_a_check_on_another_screen(pool: PgPool) {
    let reader = Reader::chosen(pool.clone()).await;
    reader.read_good().await;
    add_player(&reader, "Anna").await;
    // The row of the game exists; another screen is checking it at the same moment.
    ok(
        &reader,
        "PUT",
        &format!("{}/extra-players", game_path(AWAY_10_OCT)),
        Some(json!({ "extra_players": 0 })),
    )
    .await;
    // The other screen has locked the row of the game for its check, and stores the check only
    // after this request has started (as `game_attendance::check` does).
    let mut other = pool.begin().await.unwrap();
    sqlx::query(
        "SELECT checked_for FROM game_attendance
         WHERE season_year = 2026 AND source_id = $1 FOR UPDATE",
    )
    .bind(AWAY_10_OCT)
    .execute(&mut *other)
    .await
    .unwrap();
    let path = checked_path(AWAY_10_OCT);
    let checked = reader.send("PUT", &path, None);
    tokio::pin!(checked);
    let waited = tokio::time::timeout(std::time::Duration::from_millis(300), &mut checked).await;
    assert!(
        waited.is_err(),
        "the check did not wait for the other check"
    );
    sqlx::query(
        "UPDATE game_attendance SET checked_for = '2026-10-10'
         WHERE season_year = 2026 AND source_id = $1",
    )
    .bind(AWAY_10_OCT)
    .execute(&mut *other)
    .await
    .unwrap();
    other.commit().await.unwrap();
    let (status, answer) = checked.await;
    assert_eq!(status, 200, "{answer}");
    // The other screen checked first: this check changes nothing.
    assert_eq!(answer["settled"], 0);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_game_before_the_release_has_its_list_closed(pool: PgPool) {
    let reader = Reader::chosen(pool.clone()).await;
    reader.read_good().await;
    sqlx::query("UPDATE game_check_start SET from_date = '2026-10-11'")
        .execute(&pool)
        .await
        .unwrap();
    let page = ok(&reader, "GET", &game_path(AWAY_10_OCT), None).await;
    assert_eq!(page["attendance"]["before_release"], true);
    sqlx::query("UPDATE game_check_start SET from_date = '2026-10-10'")
        .execute(&pool)
        .await
        .unwrap();
    let page = ok(&reader, "GET", &game_path(AWAY_10_OCT), None).await;
    assert_eq!(page["attendance"]["before_release"], false);
}

/// The open item "games not checked" of the start page, as `(id of the oldest, count)`.
async fn not_checked_item(reader: &Reader) -> Option<(i64, i64)> {
    let start = ok(reader, "GET", "/api/start", None).await;
    start["open_items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["kind"] == "games_not_checked")
        .map(|item| {
            assert_eq!(item["first"]["season"], "2026-2027");
            (
                item["first"]["id"].as_i64().unwrap(),
                item["count"].as_i64().unwrap(),
            )
        })
}

async fn set_check_start(pool: &PgPool, from: &str) {
    sqlx::query("UPDATE game_check_start SET from_date = $1::date")
        .bind(from)
        .execute(pool)
        .await
        .unwrap();
}

/// `checked` on the row of game `id` among `events`.
fn row_checked(events: &Value, id: i64) -> Option<bool> {
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
        .map(|event| event["game"]["checked"].as_bool().unwrap())
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_past_game_from_the_release_that_is_not_checked_is_an_open_item(pool: PgPool) {
    let reader = Reader::chosen(pool.clone()).await;
    reader.read_good().await;
    // The release day is 4 November 2026 (migration 0016): after the test clock.
    assert_eq!(not_checked_item(&reader).await, None);
    let from: chrono::NaiveDate = sqlx::query_scalar("SELECT from_date FROM game_check_start")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(from.to_string(), "2026-11-04");

    set_check_start(&pool, "2026-10-10").await;
    assert_eq!(not_checked_item(&reader).await, Some((AWAY_10_OCT, 1)));
    // A game before the day of the release is never an open item.
    set_check_start(&pool, "2026-10-11").await;
    assert_eq!(not_checked_item(&reader).await, None);
    set_check_start(&pool, "2026-10-10").await;

    // "Checked" ends it, and the row of "Season" says so (a driver place gives the game places of
    // the team; a game of the team is not among the last duty games of the start page).
    ok(
        &reader,
        "PUT",
        &format!("{}/drivers", game_path(AWAY_10_OCT)),
        Some(json!({ "drivers": 1 })),
    )
    .await;
    let season = ok(&reader, "GET", "/api/season?season=2026-2027", None).await;
    assert_eq!(row_checked(&season["past"], AWAY_10_OCT), Some(false));
    ok(&reader, "PUT", &checked_path(AWAY_10_OCT), None).await;
    assert_eq!(not_checked_item(&reader).await, None);
    let season = ok(&reader, "GET", "/api/season?season=2026-2027", None).await;
    assert_eq!(row_checked(&season["past"], AWAY_10_OCT), Some(true));
    let start = ok(&reader, "GET", "/api/start", None).await;
    // A game of the team is not among the last duty games of the start page: it was an open item
    // until it was checked, and after that it is on "Season".
    assert_eq!(row_checked(&start["last_games"], AWAY_10_OCT), None);

    // "Undo check" brings it back; "Not played" ends it too.
    ok(&reader, "DELETE", &checked_path(AWAY_10_OCT), None).await;
    assert_eq!(not_checked_item(&reader).await, Some((AWAY_10_OCT, 1)));
    let not_played = format!("{}/not-played", game_path(AWAY_10_OCT));
    ok(&reader, "PUT", &not_played, None).await;
    assert_eq!(not_checked_item(&reader).await, None);
    ok(&reader, "DELETE", &not_played, None).await;
    assert_eq!(not_checked_item(&reader).await, Some((AWAY_10_OCT, 1)));

    // A game that left the game list is not an open item.
    reader.answer(games_with(|games| {
        games.retain(|game| game["id"] != AWAY_10_OCT);
    }));
    reader.read_good().await;
    assert_eq!(not_checked_item(&reader).await, None);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_role_moved_earlier_names_the_checked_games_without_an_answer(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    // Anna plays games from 11 October: she is not on the list of the game of 10 October.
    let anna = ok(
        &reader,
        "POST",
        "/api/players",
        Some(json!({ "first_name": "Anna", "role": { "role": "plays_games", "from": "2026-10-11" } })),
    )
    .await["player"]["id"]
        .as_i64()
        .unwrap();
    ok(&reader, "PUT", &checked_path(AWAY_10_OCT), None).await;
    let role = ok(&reader, "GET", &format!("/api/players/{anna}"), None).await["history"][0]["id"]
        .as_i64()
        .unwrap();

    // Her role now starts on 1 September: she has no answer at the checked game.
    let player = ok(
        &reader,
        "PUT",
        &format!("/api/players/{anna}/roles/{role}"),
        Some(json!({ "role": "plays_games", "from": "2026-09-01", "until": null })),
    )
    .await;
    assert_eq!(
        player["now_there"],
        json!([{ "player": anna, "trainings": [], "games": ["2026-10-10"] }])
    );
    let row = answers_of(&reader, AWAY_10_OCT).await;
    assert_eq!(row, [(anna, json!(null), json!(null))]);
}

/// Moves the game of 10 October to Sunday 11 October in the game list, and reads it.
async fn move_to_11_october(reader: &Reader) {
    reader.answer(games_with(|games| {
        game_mut(games, AWAY_10_OCT)["datum"] = json!("2026-10-11 15:00:00.000");
    }));
    reader.read_good().await;
}

async fn answers_moved(reader: &Reader) -> bool {
    ok(reader, "GET", &game_path(AWAY_10_OCT), None).await["attendance"]["answers_moved"]
        .as_bool()
        .unwrap()
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_game_checked_without_answers_still_notices_a_move(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let anna = add_player(&reader, "Anna").await;
    add_player(&reader, "Bram").await;
    // Nobody answered before the game; after "Checked" Anna is tapped "there".
    ok(&reader, "PUT", &checked_path(AWAY_10_OCT), None).await;
    answer(&reader, AWAY_10_OCT, anna, "coming", None).await;
    assert!(!answers_moved(&reader).await);
    // The game moves: Anna's answer is from before the move.
    move_to_11_october(&reader).await;
    assert!(answers_moved(&reader).await);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn after_a_move_the_settled_answers_do_not_hold_back_the_first_answer(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let anna = add_player(&reader, "Anna").await;
    add_player(&reader, "Bram").await;
    ok(&reader, "PUT", &checked_path(AWAY_10_OCT), None).await;
    move_to_11_october(&reader).await;
    // The settled answers have no effect on 11 October: Anna's answer is the first of that date.
    answer(&reader, AWAY_10_OCT, anna, "coming", None).await;
    assert!(!answers_moved(&reader).await);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn checked_reads_the_list_after_an_undo_check_on_another_screen(pool: PgPool) {
    let reader = Reader::chosen(pool.clone()).await;
    reader.read_good().await;
    let anna = add_player(&reader, "Anna").await;
    ok(&reader, "PUT", &checked_path(AWAY_10_OCT), None).await;
    // Another screen does "Undo check": it holds the lock while this "Checked" starts.
    let mut other = pool.begin().await.unwrap();
    sqlx::query(
        "SELECT checked_for FROM game_attendance
         WHERE season_year = 2026 AND source_id = $1 FOR UPDATE",
    )
    .bind(AWAY_10_OCT)
    .execute(&mut *other)
    .await
    .unwrap();
    let path = checked_path(AWAY_10_OCT);
    let checked = reader.send("PUT", &path, None);
    tokio::pin!(checked);
    let waited = tokio::time::timeout(std::time::Duration::from_millis(300), &mut checked).await;
    assert!(waited.is_err(), "the check did not wait for the undo");
    sqlx::query("DELETE FROM game_answers WHERE season_year = 2026 AND source_id = $1 AND settled")
        .bind(AWAY_10_OCT)
        .execute(&mut *other)
        .await
        .unwrap();
    sqlx::query(
        "UPDATE game_attendance SET checked_for = NULL
         WHERE season_year = 2026 AND source_id = $1",
    )
    .bind(AWAY_10_OCT)
    .execute(&mut *other)
    .await
    .unwrap();
    other.commit().await.unwrap();
    let (status, answer) = checked.await;
    assert_eq!(status, 200, "{answer}");
    // The last check wins: Anna, back at "no answer" by the undo, is absent again.
    assert_eq!(answer["settled"], 1);
    assert_eq!(
        answers_of(&reader, AWAY_10_OCT).await,
        [(anna, json!("not_coming"), json!(null))]
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_tap_at_the_same_moment_as_checked_wins(pool: PgPool) {
    let reader = Reader::chosen(pool.clone()).await;
    reader.read_good().await;
    let anna = add_player(&reader, "Anna").await;
    ok(
        &reader,
        "PUT",
        &format!("{}/extra-players", game_path(AWAY_10_OCT)),
        Some(json!({ "extra_players": 0 })),
    )
    .await;
    // Another screen saves Anna "there" and has not committed yet.
    let mut other = pool.begin().await.unwrap();
    sqlx::query(
        "INSERT INTO game_answers (club_number, season_year, source_id, player_id, answer)
         SELECT club_number, season_year, source_id, $2, 'coming' FROM game_attendance
         WHERE season_year = 2026 AND source_id = $1",
    )
    .bind(AWAY_10_OCT)
    .bind(anna)
    .execute(&mut *other)
    .await
    .unwrap();
    let path = checked_path(AWAY_10_OCT);
    let checked = reader.send("PUT", &path, None);
    tokio::pin!(checked);
    let waited = tokio::time::timeout(std::time::Duration::from_millis(300), &mut checked).await;
    assert!(waited.is_err(), "the check did not wait for the tap");
    other.commit().await.unwrap();
    let (status, answer) = checked.await;
    assert_eq!(status, 200, "{answer}");
    assert_eq!(
        answers_of(&reader, AWAY_10_OCT).await,
        [(anna, json!("coming"), json!(null))]
    );
}

/// Makes each write to `table` fail.
async fn fail_writes_to(pool: &PgPool, table: &str) {
    sqlx::raw_sql(&format!(
        "CREATE FUNCTION fail_write() RETURNS trigger LANGUAGE plpgsql AS
             $$ BEGIN RAISE EXCEPTION 'test failure'; END $$;
         CREATE TRIGGER fail_write BEFORE INSERT OR UPDATE OR DELETE ON {table} FOR EACH ROW
             EXECUTE FUNCTION fail_write();"
    ))
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_checked_that_fails_halfway_changes_nothing(pool: PgPool) {
    let reader = Reader::chosen(pool.clone()).await;
    reader.read_good().await;
    let anna = add_player(&reader, "Anna").await;
    // The check is stored before the settled answers: their insert fails.
    fail_writes_to(&pool, "game_answers").await;
    let (status, _) = reader.send("PUT", &checked_path(AWAY_10_OCT), None).await;
    assert_eq!(status, 500);
    assert!(!is_checked(&reader, AWAY_10_OCT).await);
    assert_eq!(
        answers_of(&reader, AWAY_10_OCT).await,
        [(anna, json!(null), json!(null))]
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn an_undo_check_that_fails_halfway_changes_nothing(pool: PgPool) {
    let reader = Reader::chosen(pool.clone()).await;
    reader.read_good().await;
    let anna = add_player(&reader, "Anna").await;
    ok(&reader, "PUT", &checked_path(AWAY_10_OCT), None).await;
    // The settled answers are deleted before the check: the update of the check fails.
    sqlx::raw_sql(
        "CREATE FUNCTION fail_write() RETURNS trigger LANGUAGE plpgsql AS
             $$ BEGIN RAISE EXCEPTION 'test failure'; END $$;
         CREATE TRIGGER fail_write BEFORE UPDATE ON game_attendance FOR EACH ROW
             EXECUTE FUNCTION fail_write();",
    )
    .execute(&pool)
    .await
    .unwrap();
    let (status, _) = reader
        .send("DELETE", &checked_path(AWAY_10_OCT), None)
        .await;
    assert_eq!(status, 500);
    assert!(is_checked(&reader, AWAY_10_OCT).await);
    assert_eq!(
        answers_of(&reader, AWAY_10_OCT).await,
        [(anna, json!("not_coming"), json!(null))]
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_row_of_a_game_of_today_says_started_from_its_start_time(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    add_player(&reader, "Anna").await;
    let started_in_next = |start: &Value| {
        start["next"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["event"]["game"]["id"] == AWAY_10_OCT)
            .map(|item| item["event"]["game"]["started"].as_bool().unwrap())
    };
    // Saturday 10 October, 15:00 in the Netherlands (13:00 UTC).
    reader.harness.clock.set(utc(2026, 10, 10, 12, 59));
    reader.log_in_again().await;
    let start = ok(&reader, "GET", "/api/start", None).await;
    assert_eq!(started_in_next(&start), Some(false));
    reader.harness.clock.set(utc(2026, 10, 10, 13, 0));
    reader.log_in_again().await;
    let start = ok(&reader, "GET", "/api/start", None).await;
    assert_eq!(started_in_next(&start), Some(true));
}
