//! The start page through the API (story B of start page and season): the open items come from
//! the rules of their sources, "Last" and "Next".

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::Harness;
use common::game_list::{DRAKES, M16_1, Reader, data, game_mut, games_with, utc};
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

/// The events of `key` ("next" or "last_games") as on "Season", without how they look.
fn events(answer: &Value, key: &str) -> Vec<Value> {
    answer[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["event"].clone())
        .collect()
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

/// A player who plays games from `from`; its id.
async fn add_player(reader: &Reader, first: &str, from: &str) -> i64 {
    let answer = ok(
        reader,
        "POST",
        "/api/players",
        Some(json!({ "first_name": first, "role": { "role": "plays_games", "from": from } })),
    )
    .await;
    answer["player"]["id"].as_i64().unwrap()
}

/// `player` is absent at the training on `date`, with the reason School.
async fn absent_at(reader: &Reader, date: &str, player: i64) {
    let reasons = ok(reader, "GET", "/api/reasons", None).await;
    let school = reasons
        .as_array()
        .unwrap()
        .iter()
        .find(|reason| reason["name"] == "School")
        .unwrap()["id"]
        .clone();
    ok(
        reader,
        "PUT",
        &format!("/api/trainings/{date}/absences/{player}"),
        Some(json!({ "reason": school })),
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
    // The real games have home games of the club in the next 4 weeks.
    assert_eq!(kinds(&start(&reader).await), ["schedule_not_entered"]);

    // A game list with no games is held back.
    let mut empty = data("games-empty.json");
    empty["seizoen"] = json!("2026-2027");
    reader.answer(empty);
    let (_, held) = reader.read().await;
    assert_eq!(held["result"], "held_back", "{held}");
    assert!(!reader.latest().await["held_back"].is_null());
    assert_eq!(
        kinds(&start(&reader).await),
        ["reading_held_back", "schedule_not_entered"]
    );
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
async fn next_has_the_team_games_and_trainings_of_two_weeks(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    add_tuesdays(&reader, "2026-09-01", "2027-05-25").await;
    // Monday 26 October: until Sunday 8 November.
    at(&mut reader, 2026, 10, 26, 9).await;

    let answer = start(&reader).await;
    assert_eq!(answer["today"], "2026-10-26");
    assert_eq!(answer["show_season"], false);
    let next = events(&answer, "next");
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

    // A game of the team says home or away and its opponent; a training has neither. Nothing is
    // checked yet, so every row is normal.
    for item in answer["next"].as_array().unwrap() {
        let game = &item["event"]["game"];
        if game.is_null() {
            assert!(
                item["side"].is_null() && item["opponent"].is_null(),
                "{item}"
            );
        } else {
            let side = item["side"].as_str().unwrap();
            let opponent = item["opponent"].as_str().unwrap();
            let (ours, theirs) = if side == "home" {
                (&game["home"], &game["away"])
            } else {
                (&game["away"], &game["home"])
            };
            assert!(ours.as_str().unwrap().contains("M16 1"), "{item}");
            assert_eq!(theirs.as_str().unwrap(), opponent);
        }
        assert_eq!(item["look"], "normal", "{item}");
    }
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_new_season_without_a_team_has_trainings_only(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    at(&mut reader, 2027, 7, 1, 10).await;
    add_tuesdays(&reader, "2027-09-07", "2028-05-30").await;
    // The games of the season before that are not checked are an open item in July: this test is
    // about the new season, so the open item starts on the day of the clock.
    sqlx::query("UPDATE game_check_start SET from_date = '2027-07-01'")
        .execute(reader.pool())
        .await
        .unwrap();
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
    let next = events(&answer, "next");
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
    let next = events(&start(&reader).await, "next");
    assert_eq!(next[0]["game"]["id"], 9000001, "{next:?}");
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn in_july_the_open_items_and_the_header_cover_the_season_before(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    add_tuesdays(&reader, "2026-09-01", "2027-05-25").await;
    let avery = add_player(&reader, "Avery", "2026-09-01").await;
    ok(&reader, "PUT", "/api/trainings/2027-05-25/open", None).await;
    absent_at(&reader, "2027-05-25", avery).await;

    at(&mut reader, 2027, 7, 1, 10).await;
    let answer = start(&reader).await;
    let not_opened = item(&answer, "not_opened").unwrap();
    assert_eq!(not_opened["first"], "2026-09-01");
    assert_eq!(item(&answer, "not_checked").unwrap()["first"], "2027-05-25");
    assert_eq!(answer["show_season"], true);
    // No past training is shown as an event: only as an open item.
    assert!(answer["last_games"].as_array().unwrap().is_empty());

    // From September the season before is no longer covered.
    at(&mut reader, 2027, 9, 1, 10).await;
    let answer = start(&reader).await;
    assert_eq!(item(&answer, "not_opened"), None);
    assert_eq!(item(&answer, "not_checked"), None);
    assert_eq!(answer["show_season"], false);
}

// The warnings in time (story B of attendance at games).

/// The away game of M16 1 on Saturday 7 November 2026 at 11:15 (Dutch time, UTC+1).
const AWAY_7_NOV: i64 = 1_073_159;

fn away_path() -> String {
    format!("/api/games/2026-2027/{AWAY_7_NOV}")
}

/// `answer` of `player` at the away game of 7 November.
async fn answer_at_away(reader: &Reader, player: i64, answer: &str) {
    ok(
        reader,
        "PUT",
        &format!("{}/answers/{player}", away_path()),
        Some(json!({ "answer": answer })),
    )
    .await;
}

/// `count` players who play games, with the answer "coming" for the first `coming`.
async fn team_of(reader: &Reader, count: usize, coming: usize) -> Vec<i64> {
    let mut players = Vec::new();
    for n in 0..count {
        let player = add_player(reader, &format!("Player{n}"), "2026-09-01").await;
        if n < coming {
            answer_at_away(reader, player, "coming").await;
        }
        players.push(player);
    }
    players
}

/// The open item of `kind`, at the moment (UTC).
async fn item_at(
    reader: &mut Reader,
    kind: &str,
    (month, day, hour, minute): (u32, u32, u32, u32),
) -> Option<Value> {
    reader
        .harness
        .clock
        .set(utc(2026, month, day, hour, minute));
    reader.log_in_again().await;
    item(&start(reader).await, kind).cloned()
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn too_few_players_is_an_open_item_from_3_days_until_the_start(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    team_of(&reader, 7, 5).await;
    // A change not seen (another game gets a new start time): an item before the warnings.
    reader.answer(games_with(|games| {
        game_mut(games, 1_073_168)["tijd"] = json!("13:00");
    }));
    reader.read_good().await;

    // Tuesday 3 November 23:59 Dutch time: not yet.
    assert_eq!(
        item_at(&mut reader, "too_few_players", (11, 3, 22, 59)).await,
        None
    );
    // Wednesday 4 November 00:00 Dutch time.
    let found = item_at(&mut reader, "too_few_players", (11, 3, 23, 0))
        .await
        .unwrap();
    let games = found["games"].as_array().unwrap();
    assert_eq!(games.len(), 1, "{found}");
    assert_eq!(
        (
            &games[0]["id"],
            &games[0]["date"],
            &games[0]["coming"],
            &games[0]["no_answer"],
            &games[0]["enough"]
        ),
        (
            &json!(AWAY_7_NOV),
            &json!("2026-11-07"),
            &json!(5),
            &json!(2),
            &json!(7)
        )
    );
    // The order: after the items of the game list and the changes, before the later items.
    let answer = start(&reader).await;
    let kinds = kinds(&answer);
    let at = kinds
        .iter()
        .position(|kind| *kind == "too_few_players")
        .unwrap();
    for (n, kind) in kinds.iter().enumerate() {
        let earlier = ["reading_held_back", "out_of_date", "changes"].contains(kind);
        let later = [
            "driver_not_coming",
            "answers_moved",
            "schedule_not_entered",
            "open_places",
            "not_opened",
            "not_checked",
        ]
        .contains(kind);
        assert!(!earlier || n < at, "{kinds:?}");
        assert!(!later || n > at, "{kinds:?}");
    }
    assert!(
        kinds.len() > 1,
        "a later item is there to compare: {kinds:?}"
    );
    // One minute before the start, still; at the start time, no more.
    assert!(
        item_at(&mut reader, "too_few_players", (11, 7, 10, 14))
            .await
            .is_some()
    );
    assert_eq!(
        item_at(&mut reader, "too_few_players", (11, 7, 10, 15)).await,
        None
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn extra_players_count_for_the_7(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    team_of(&reader, 7, 5).await;
    ok(
        &reader,
        "PUT",
        &format!("{}/extra-players", away_path()),
        Some(json!({ "extra_players": 2 })),
    )
    .await;
    assert_eq!(
        item_at(&mut reader, "too_few_players", (11, 5, 9, 0)).await,
        None
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_driver_who_is_not_coming_is_an_open_item_at_once(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let anna = add_player(&reader, "Anna", "2026-09-01").await;
    ok(
        &reader,
        "PUT",
        &format!("{}/drivers", away_path()),
        Some(json!({ "drivers": 2 })),
    )
    .await;
    let place = format!("{}/assignments/drive/1", away_path());
    ok(&reader, "PUT", &place, Some(json!({ "player": anna }))).await;
    answer_at_away(&reader, anna, "not_coming").await;

    // Wednesday 28 October, 10 days before the game.
    let found = item_at(&mut reader, "driver_not_coming", (10, 28, 9, 0))
        .await
        .unwrap();
    assert_eq!(found["games"][0]["id"], AWAY_7_NOV);
    assert_eq!(found["games"][0]["drivers"], json!(["Anna"]));

    // "Coming" again: gone.
    answer_at_away(&reader, anna, "coming").await;
    assert_eq!(item(&start(&reader).await, "driver_not_coming"), None);
    // "Not coming", then the place cleared: gone.
    answer_at_away(&reader, anna, "not_coming").await;
    assert!(item(&start(&reader).await, "driver_not_coming").is_some());
    ok(&reader, "DELETE", &place, None).await;
    assert_eq!(item(&start(&reader).await, "driver_not_coming"), None);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_driver_warning_ends_at_the_start_time(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let anna = add_player(&reader, "Anna", "2026-09-01").await;
    ok(
        &reader,
        "PUT",
        &format!("{}/drivers", away_path()),
        Some(json!({ "drivers": 1 })),
    )
    .await;
    let place = format!("{}/assignments/drive/1", away_path());
    ok(&reader, "PUT", &place, Some(json!({ "player": anna }))).await;
    answer_at_away(&reader, anna, "not_coming").await;
    assert!(
        item_at(&mut reader, "driver_not_coming", (11, 7, 10, 14))
            .await
            .is_some()
    );
    assert_eq!(
        item_at(&mut reader, "driver_not_coming", (11, 7, 10, 15)).await,
        None
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_moved_game_is_an_open_item_from_3_days_before_its_new_date(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    team_of(&reader, 7, 7).await;
    // The game moves from Saturday 7 to Wednesday 11 November.
    reader.answer(games_with(|games| {
        game_mut(games, AWAY_7_NOV)["datum"] = json!("2026-11-11 11:15:00.000");
    }));
    reader.read_good().await;

    // Saturday 7 November 23:59 Dutch time: not yet; Sunday 8 November 00:00: the item.
    assert_eq!(
        item_at(&mut reader, "answers_moved", (11, 7, 22, 59)).await,
        None
    );
    let found = item_at(&mut reader, "answers_moved", (11, 7, 23, 0))
        .await
        .unwrap();
    assert_eq!(found["games"][0]["id"], AWAY_7_NOV);
    assert_eq!(found["games"][0]["date"], "2026-11-11");

    // "Seen" in the list of changes does not end it.
    let (status, body) = reader.send("GET", "/api/changes", None).await;
    assert_eq!(status, 200, "{body}");
    let seen: Vec<Value> = body["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|change| json!({ "id": change["id"], "version": change["version"] }))
        .collect();
    assert!(!seen.is_empty());
    ok(
        &reader,
        "POST",
        "/api/changes/seen",
        Some(json!({ "changes": seen })),
    )
    .await;
    assert!(item(&start(&reader).await, "answers_moved").is_some());
    // It ends at the start time (deviation D1).
    assert_eq!(
        item_at(&mut reader, "answers_moved", (11, 11, 10, 15)).await,
        None
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_game_that_left_the_game_list_gives_no_warning(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;
    team_of(&reader, 7, 5).await;
    reader.answer(games_with(|games| {
        games.retain(|game| game["id"] != AWAY_7_NOV);
    }));
    reader.read_good().await;
    assert_eq!(
        item_at(&mut reader, "too_few_players", (11, 5, 9, 0)).await,
        None
    );
}
