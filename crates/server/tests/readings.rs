//! Reading the game list on demand through the API (story 3a): the lock, the transaction, the
//! comparison, the guard and "these games are really gone".

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use chrono::{NaiveDate, NaiveTime, TimeDelta};
use common::Harness;
use common::game_list::{
    DRAKES, M16_1, M16_2, Reader, game_mut, games_with, in_m16_1_schedule, real_games, utc,
};
use serde_json::{Value, json};
use sqlx::PgPool;
use team::clock::Clock;
use wiremock::ResponseTemplate;

/// A team game of M16 1 on 12 December 2026.
const TEAM_GAME: i64 = 1073178;
/// A team game of M16 1 on 31 October 2026 at 16:00; "not known" in the base list.
const TIME_GAME: i64 = 1073152;
/// A home game of Blue Drakes X12 1 on 31 October 2026: a coming duty game. Changes to past games
/// are not counted, so the changed games must be coming.
const DUTY_GAME: i64 = 1081923;
/// An away game of Blue Drakes MSE 1 on 16 January 2027 with no known start time.
const OTHER_TEAM_AWAY: i64 = 1078886;
/// An away game of Blue Drakes M16 2.
const M16_2_AWAY: i64 = 1073597;
const NEW_GAME: i64 = 9_999_001;

fn counts(pairs: &[(&str, u32)]) -> Value {
    let mut counts = json!({
        "new": 0, "moved": 0, "start_time_known": 0, "home_and_away": 0, "team_changed": 0,
        "gone": 0, "came_back": 0, "into_schedule": 0, "out_of_schedule": 0
    });
    for (name, count) in pairs {
        counts[*name] = json!(count);
    }
    counts
}

/// The real games, with the start time of `TIME_GAME` not known.
fn base() -> Value {
    games_with(|games| {
        game_mut(games, TIME_GAME)["tijd"] = json!("00:00");
    })
}

/// `base` with: a team game on another date, the start time of `TIME_GAME` known, a duty game
/// removed, a new home game of another club team, and a start time for the away game of another
/// club team (no count).
fn changed() -> Value {
    games_with(|games| {
        game_mut(games, TEAM_GAME)["datum"] = json!("2026-12-19 14:30:00.000");
        game_mut(games, TIME_GAME)["tijd"] = json!("10:00");
        game_mut(games, OTHER_TEAM_AWAY)["tijd"] = json!("10:00");
        let mut new_game = game_mut(games, DUTY_GAME).clone();
        new_game["id"] = json!(NEW_GAME);
        new_game["nr"] = json!("NEW-1");
        games.retain(|game| game["id"] != DUTY_GAME);
        games.push(new_game);
    })
}

fn changed_counts() -> Value {
    counts(&[
        ("new", 1),
        ("moved", 1),
        ("start_time_known", 1),
        ("gone", 1),
    ])
}

async fn in_list(pool: &PgPool, id: i64) -> Option<bool> {
    sqlx::query_scalar("SELECT in_list FROM games WHERE source_id = $1 AND season_year = 2026")
        .bind(id)
        .fetch_optional(pool)
        .await
        .unwrap()
}

async fn date_of(pool: &PgPool, id: i64) -> NaiveDate {
    sqlx::query_scalar("SELECT date FROM games WHERE source_id = $1 AND season_year = 2026")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn count_rows(pool: &PgPool, sql: &str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(pool).await.unwrap()
}

/// The stored games, to compare before and after.
async fn snapshot(pool: &PgPool) -> Vec<(i64, NaiveDate, Option<NaiveTime>, bool)> {
    sqlx::query_as(
        "SELECT source_id, date, start_time, in_list FROM games ORDER BY season_year, source_id",
    )
    .fetch_all(pool)
    .await
    .unwrap()
}

/// Adds the counts of each kind.
fn sum(a: &Value, b: &Value) -> Value {
    let mut total = a.clone();
    for (name, value) in b.as_object().unwrap() {
        total[name] = json!(a[name].as_u64().unwrap() + value.as_u64().unwrap());
    }
    total
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn two_readings_at_the_same_time_count_each_change_once(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.answer(base());
    reader.read_good().await;

    // A slow answer, so both readings have read the source before either takes the lock.
    reader.answer_with(
        ResponseTemplate::new(200)
            .set_body_json(changed())
            .set_delay(std::time::Duration::from_millis(300)),
    );
    let ((status_a, a), (status_b, b)) = tokio::join!(reader.read(), reader.read());
    assert_eq!((status_a, status_b), (200, 200));
    assert_eq!(sum(&a["counts"], &b["counts"]), changed_counts());
    assert!(a["counts"] == counts(&[]) || b["counts"] == counts(&[]));

    let pool = reader.pool();
    assert_eq!(in_list(pool, DUTY_GAME).await, Some(false));
    assert_eq!(in_list(pool, NEW_GAME).await, Some(true));
    assert_eq!(
        date_of(pool, TEAM_GAME).await,
        NaiveDate::from_ymd_opt(2026, 12, 19).unwrap()
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_failure_while_storing_changes_nothing(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.answer(base());
    reader.read_good().await;
    let pool = reader.pool();
    let games_before = snapshot(pool).await;
    let places_before = count_rows(pool, "SELECT count(*) FROM places").await;

    sqlx::raw_sql(
        "CREATE FUNCTION fail_good_reading() RETURNS trigger LANGUAGE plpgsql AS
             $$ BEGIN RAISE EXCEPTION 'test failure'; END $$;
         CREATE TRIGGER fail_good_reading BEFORE INSERT ON readings FOR EACH ROW
             WHEN (NEW.result = 'good') EXECUTE FUNCTION fail_good_reading();",
    )
    .execute(pool)
    .await
    .unwrap();
    reader.answer(games_with(|games| {
        let mut new_game = game_mut(games, DUTY_GAME).clone();
        new_game["id"] = json!(NEW_GAME);
        new_game["loc_id"] = json!(99_999);
        games.push(new_game);
        game_mut(games, TEAM_GAME)["datum"] = json!("2026-12-19 14:30:00.000");
    }));

    let (status, body) = reader.read().await;
    assert_eq!((status, body), (500, json!({ "error": "internal" })));
    assert_eq!(snapshot(pool).await, games_before);
    assert_eq!(
        count_rows(pool, "SELECT count(*) FROM places").await,
        places_before
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn all_routes_need_a_login(pool: PgPool) {
    let harness = Harness::start(pool).await;
    for (method, path) in [
        ("POST", "/api/readings"),
        ("GET", "/api/readings/latest"),
        ("POST", "/api/readings/1/confirm-gone"),
    ] {
        let request = axum::http::Request::builder()
            .method(method)
            .uri(path)
            .header(axum::http::header::ORIGIN, common::ORIGIN)
            .body(axum::body::Body::empty())
            .unwrap();
        let response = harness.send(request).await;
        assert_eq!(response.status(), 401, "{method} {path}");
    }
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn without_a_choice_there_is_nothing_to_read(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    sqlx::query("DELETE FROM followed_teams")
        .execute(reader.pool())
        .await
        .unwrap();
    let (status, body) = reader.read().await;
    assert_eq!((status, body), (400, json!({ "error": "no_choice" })));
    assert_eq!(
        reader.latest().await,
        json!({ "latest": null, "last_good": null, "held_back": null })
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_first_reading_counts_nothing_and_stores_everything(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    let body = reader.read_good().await;
    assert_eq!(body["first_reading"], true);
    assert_eq!(body["games_read"], 71);
    assert_eq!(body["counts"], Value::Null);
    assert_eq!(
        (&body["hold"], &body["error"]),
        (&Value::Null, &Value::Null)
    );
    assert_eq!(body["finished_at"], "2026-10-12T12:00:00Z");

    let pool = reader.pool();
    assert_eq!(count_rows(pool, "SELECT count(*) FROM games").await, 71);
    assert_eq!(count_rows(pool, "SELECT count(*) FROM places").await, 27);

    let latest = reader.latest().await;
    assert_eq!(latest["latest"], body);
    assert_eq!(
        latest["last_good"],
        json!({ "finished_at": "2026-10-12T12:00:00Z", "games_read": 71 })
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_same_data_again_gives_no_changes(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let body = reader.read_good().await;
    assert_eq!(body["first_reading"], false);
    assert_eq!(body["counts"], counts(&[]));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn changes_are_counted_and_stored(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.answer(base());
    reader.read_good().await;

    reader.answer(changed());
    let body = reader.read_good().await;
    assert_eq!(body["counts"], changed_counts());
    let pool = reader.pool();
    assert_eq!(in_list(pool, DUTY_GAME).await, Some(false));
    assert_eq!(
        date_of(pool, TEAM_GAME).await,
        NaiveDate::from_ymd_opt(2026, 12, 19).unwrap()
    );
    let start: Option<NaiveTime> =
        sqlx::query_scalar("SELECT start_time FROM games WHERE source_id = $1")
            .bind(OTHER_TEAM_AWAY)
            .fetch_one(pool)
            .await
            .unwrap();
    assert_eq!(
        start,
        NaiveTime::from_hms_opt(10, 0, 0),
        "stored, not counted"
    );

    // The removed game is in the next reading again.
    reader.answer(games_with(|games| {
        game_mut(games, TEAM_GAME)["datum"] = json!("2026-12-19 14:30:00.000");
        game_mut(games, TIME_GAME)["tijd"] = json!("10:00");
        game_mut(games, OTHER_TEAM_AWAY)["tijd"] = json!("10:00");
        let mut new_game = game_mut(games, DUTY_GAME).clone();
        new_game["id"] = json!(NEW_GAME);
        new_game["nr"] = json!("NEW-1");
        games.push(new_game);
    }));
    let body = reader.read_good().await;
    assert_eq!(body["counts"], counts(&[("came_back", 1)]));
    assert_eq!(in_list(pool, DUTY_GAME).await, Some(true));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_change_outside_the_schedule_is_stored_but_not_counted(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    reader.answer(games_with(|games| {
        game_mut(games, M16_2_AWAY)["tijd"] = json!("13:45");
    }));
    let body = reader.read_good().await;
    assert_eq!(body["counts"], counts(&[]));
    let start: Option<NaiveTime> =
        sqlx::query_scalar("SELECT start_time FROM games WHERE source_id = $1")
            .bind(M16_2_AWAY)
            .fetch_one(reader.pool())
            .await
            .unwrap();
    assert_eq!(start, NaiveTime::from_hms_opt(13, 45, 0));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn an_empty_or_far_smaller_list_is_held_back(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let pool = reader.pool();
    let games_before = snapshot(pool).await;

    reader.answer(games_with(Vec::clear));
    let (status, body) = reader.read().await;
    assert_eq!(status, 200);
    assert_eq!(body["result"], "held_back");
    assert_eq!(
        body["hold"],
        json!({ "reason": "empty", "before": 71, "now": 0 })
    );
    assert_eq!(body["counts"], Value::Null);
    assert_eq!(snapshot(pool).await, games_before);

    reader.answer(games_with(|games| games.truncate(30)));
    let (_, body) = reader.read().await;
    assert_eq!(body["result"], "held_back");
    assert_eq!(
        body["hold"],
        json!({ "reason": "far_fewer", "before": 71, "now": 30 })
    );
    assert_eq!(snapshot(pool).await, games_before);
    assert_eq!(reader.latest().await["latest"]["result"], "held_back");

    reader.answer(real_games());
    reader.read_good().await;
    let latest = reader.latest().await;
    assert_eq!(latest["latest"]["result"], "good");
    assert_eq!(latest["latest"]["hold"], Value::Null);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn really_gone_counts_the_missing_games_as_gone(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    let good = reader.read_good().await;

    let smaller = games_with(|games| games.truncate(30));
    let missing_in_schedule = real_games()["wedstrijden"].as_array().unwrap()[30..]
        .iter()
        .filter(|game| in_m16_1_schedule(game))
        .count();
    assert!(missing_in_schedule > 0);
    reader.answer(smaller);
    let (_, held) = reader.read().await;
    assert_eq!(held["result"], "held_back");

    // Not held back, and an unknown reading.
    let (status, body) = reader.confirm_gone(&good["id"]).await;
    assert_eq!((status, body), (400, json!({ "error": "not_held_back" })));
    let (status, _) = reader.confirm_gone(&json!(999_999)).await;
    assert_eq!(status, 404);

    let (status, body) = reader.confirm_gone(&held["id"]).await;
    assert_eq!(status, 200);
    assert_eq!(body["result"], "good");
    assert_eq!(body["games_read"], 30);
    assert_eq!(
        body["counts"],
        counts(&[("gone", u32::try_from(missing_in_schedule).unwrap())])
    );
    let pool = reader.pool();
    assert_eq!(
        count_rows(pool, "SELECT count(*) FROM games WHERE NOT in_list").await,
        41
    );

    // The held-back reading is no longer the newest.
    let (status, body) = reader.confirm_gone(&held["id"]).await;
    assert_eq!((status, body), (400, json!({ "error": "not_held_back" })));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_failed_reading_keeps_the_games_and_shows_the_last_good_one(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let pool = reader.pool();
    let games_before = snapshot(pool).await;

    reader.harness.clock.advance(TimeDelta::hours(1));
    reader.answer_with(ResponseTemplate::new(503));
    let (status, body) = reader.read().await;
    assert_eq!(status, 200);
    assert_eq!(body["result"], "failed");
    assert_eq!(body["error"], "source_unavailable");
    assert_eq!(body["games_read"], Value::Null);
    assert_eq!(snapshot(pool).await, games_before);

    let latest = reader.latest().await;
    assert_eq!(latest["latest"]["error"], "source_unavailable");
    assert_eq!(
        latest["last_good"],
        json!({ "finished_at": "2026-10-12T12:00:00Z", "games_read": 71 })
    );

    reader.answer(json!({ "wedstrijden": "not a list" }));
    let (_, body) = reader.read().await;
    assert_eq!(body["error"], "source_format");
    assert_eq!(snapshot(pool).await, games_before);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_reading_never_changes_another_season(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    let pool = reader.pool();
    sqlx::raw_sql(&format!(
        "INSERT INTO places (number, hall, town) VALUES (1, 'Old hall', 'Old town');
         INSERT INTO games
             (club_number, season_year, source_id, game_number, date, start_time,
              home_team_number, home_team_name, home_club_number,
              away_team_number, away_team_name, away_club_number,
              place_number, court, competition, in_list, first_read_at)
         VALUES ({DRAKES}, 2025, {TEAM_GAME}, 'OLD', '2026-03-01', '11:00', 184, 'Old home', 37,
                 2, 'Old away', 99, 1, NULL, 'Old competition', true, now()),
                ({DRAKES}, 2025, 1, 'OLD-2', '2026-03-08', '11:00', 184, 'Old home', 37,
                 2, 'Old away', 99, 1, NULL, 'Old competition', true, now());"
    ))
    .execute(pool)
    .await
    .unwrap();
    let old = |pool: PgPool| async move {
        sqlx::query_as::<_, (i64, String, bool)>(
            "SELECT source_id, game_number, in_list FROM games WHERE season_year = 2025
             ORDER BY source_id",
        )
        .fetch_all(&pool)
        .await
        .unwrap()
    };
    let before = old(pool.clone()).await;

    reader.read_good().await;
    reader.answer(changed());
    reader.read_good().await;
    assert_eq!(old(pool.clone()).await, before);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn after_a_change_of_team_the_next_reading_is_a_first_reading(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    reader.read_good().await;

    reader.harness.clock.advance(TimeDelta::minutes(1));
    reader.choose(M16_2).await;
    let body = reader.read_good().await;
    assert_eq!(body["first_reading"], true);
    assert_eq!(body["counts"], Value::Null);
    assert_eq!(reader.read_good().await["first_reading"], false);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn in_a_new_season_the_kept_club_is_read(pool: PgPool) {
    let mut reader = Reader::chosen(pool).await;
    reader.read_good().await;

    // 1 July 2027: season 2027-2028, no choice saved for it yet.
    reader.harness.clock.set(utc(2027, 7, 1, 10, 0));
    reader.log_in_again().await;
    reader.answer(games_with(|games| {
        for game in games.iter_mut() {
            game["seizoen"] = json!("2027-2028");
        }
    }));
    let body = reader.read_good().await;
    assert_eq!(body["first_reading"], true);
    assert_eq!(body["games_read"], 71);
    let pool = reader.pool();
    assert_eq!(
        count_rows(pool, "SELECT count(*) FROM games WHERE season_year = 2027").await,
        71
    );
    assert_eq!(
        count_rows(pool, "SELECT count(*) FROM games WHERE season_year = 2026").await,
        71
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn saving_the_same_choice_again_still_counts_the_changes(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.answer(base());
    reader.read_good().await;

    reader.harness.clock.advance(TimeDelta::minutes(1));
    reader.choose(M16_1).await;
    reader.answer(changed());
    let body = reader.read_good().await;
    assert_eq!(body["first_reading"], false);
    assert_eq!(body["counts"], changed_counts());
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_failed_reading_does_not_hide_the_held_back_reading(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    reader.answer(games_with(|games| games.truncate(30)));
    let (_, held) = reader.read().await;
    assert_eq!(held["result"], "held_back");

    reader.answer_with(ResponseTemplate::new(503));
    let (_, failed) = reader.read().await;
    assert_eq!(failed["result"], "failed");
    let latest = reader.latest().await;
    assert_eq!(latest["latest"]["result"], "failed");
    assert_eq!(latest["held_back"], held);

    reader.answer(games_with(|games| games.truncate(30)));
    let (status, body) = reader.confirm_gone(&held["id"]).await;
    assert_eq!((status, &body["result"]), (200, &json!("good")));
    assert_eq!(reader.latest().await["held_back"], Value::Null);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn really_gone_never_loses_more_games_than_confirmed(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let pool = reader.pool();
    let games_before = snapshot(pool).await;
    reader.answer(games_with(|games| games.truncate(30)));
    let (_, held) = reader.read().await;

    // The game list now gives even fewer games than the warning showed.
    reader.answer(games_with(|games| games.truncate(10)));
    let (status, body) = reader.confirm_gone(&held["id"]).await;
    assert_eq!(status, 200);
    assert_eq!(body["result"], "held_back");
    assert_eq!(
        body["hold"],
        json!({ "reason": "far_fewer", "before": 71, "now": 10 })
    );
    assert_eq!(snapshot(pool).await, games_before);
    assert_eq!(reader.latest().await["held_back"], body);

    // The first warning is no longer the one to confirm.
    let (status, error) = reader.confirm_gone(&held["id"]).await;
    assert_eq!((status, error), (400, json!({ "error": "not_held_back" })));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn an_older_answer_is_not_stored_over_a_newer_one(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let pool = reader.pool();
    let games_before = snapshot(pool).await;

    // A reading that read the source later and stored its games first.
    let later = reader.harness.clock.now() + TimeDelta::seconds(5);
    let newer = team::readings::insert(
        pool,
        team_core::ClubNumber(DRAKES),
        "2026-2027".parse().unwrap(),
        Some(team_core::TeamNumber(M16_1)),
        later,
        later,
        &team::readings::ReadingResult::Good {
            games_read: 71,
            first_reading: false,
            counts: Some(team_core::Counts::default()),
        },
    )
    .await
    .unwrap();

    reader.answer(changed());
    let body = reader.read_good().await;
    assert_eq!(body["id"], newer);
    assert_eq!(snapshot(pool).await, games_before);
}
