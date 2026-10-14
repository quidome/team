//! The nightly reading (story 5): when one check of the timer reads the game list, and that it
//! stores the same as a tap.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::game_list::{Reader, game_mut, games_with, utc};
use common::{Harness, ORIGIN};
use serde_json::json;
use team::nightly::{self, Tried};
use team::readings::{Reading, ReadingResult};
use team::source::Source;
use url::Url;
use wiremock::{MockServer, ResponseTemplate};

/// A team game of M16 1 on 12 December 2026.
const TEAM_GAME: i64 = 1073178;

/// One check of a new timer against the harness's fake game list and clock.
async fn tick(harness: &Harness) -> Option<Reading> {
    tick_with(harness, &mut Tried::default()).await.unwrap()
}

/// One check of the timer that already tried `tried`.
async fn tick_with(
    harness: &Harness,
    tried: &mut Tried,
) -> Result<Option<Reading>, team::reading::ReadingError> {
    let source = Source::new(
        Url::parse(&harness.source.uri()).unwrap(),
        &Url::parse(ORIGIN).unwrap(),
    )
    .unwrap();
    nightly::tick(&harness.pool, &source, harness.clock.as_ref(), tried).await
}

async fn requests(source: &MockServer) -> usize {
    source.received_requests().await.unwrap().len()
}

fn is_good(reading: &Reading) -> bool {
    matches!(reading.result, ReadingResult::Good { .. })
}

/// Blue Drakes M16 1 chosen and read on 12 October 2026 at 12:00 UTC.
async fn read_on_monday(pool: sqlx::PgPool) -> Reader {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    reader
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn without_a_choice_nothing_is_read(pool: sqlx::PgPool) {
    let harness = Harness::start(pool).await;
    harness.clock.set(utc(2026, 10, 13, 1, 1));
    assert!(tick(&harness).await.is_none());
    assert_eq!(requests(&harness.source).await, 0);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn in_a_new_season_without_a_team_nothing_is_read(pool: sqlx::PgPool) {
    let reader = read_on_monday(pool).await;
    let before = requests(&reader.harness.source).await;
    // 2 July 2027 03:30 Dutch time: season 2027-2028, the club is known but no team is chosen.
    reader.harness.clock.set(utc(2027, 7, 2, 1, 30));
    assert!(tick(&reader.harness).await.is_none());
    assert_eq!(requests(&reader.harness.source).await, before);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_nightly_reading_records_changes_as_a_tap_does(pool: sqlx::PgPool) {
    let mut reader = read_on_monday(pool).await;
    reader.answer(games_with(|games| {
        game_mut(games, TEAM_GAME)["datum"] = json!("2026-12-19 14:30:00.000");
    }));

    // 13 October 03:01 Dutch time.
    reader.harness.clock.set(utc(2026, 10, 13, 1, 1));
    let reading = tick(&reader.harness).await.expect("a nightly reading");
    assert!(is_good(&reading), "{reading:?}");

    reader.log_in_again().await;
    let (status, body) = reader.send("GET", "/api/changes", None).await;
    assert_eq!(status, 200);
    let changes = body["changes"].as_array().unwrap();
    assert_eq!(changes.len(), 1, "{body}");
    assert_eq!(changes[0]["id"], TEAM_GAME);
    assert_eq!(changes[0]["kind"], "moved");

    // One reading a night.
    reader.harness.clock.set(utc(2026, 10, 13, 1, 6));
    assert!(tick(&reader.harness).await.is_none());
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_pod_that_was_down_at_three_reads_when_it_checks(pool: sqlx::PgPool) {
    let reader = read_on_monday(pool).await;
    // 13 October 09:00 Dutch time, nothing read since Monday.
    reader.harness.clock.set(utc(2026, 10, 13, 7, 0));
    let reading = tick(&reader.harness).await.expect("a nightly reading");
    assert!(is_good(&reading), "{reading:?}");
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_tap_after_three_makes_the_nightly_reading_unnecessary(pool: sqlx::PgPool) {
    let mut reader = read_on_monday(pool).await;
    // A tap on 13 October 08:00 Dutch time.
    reader.harness.clock.set(utc(2026, 10, 13, 6, 0));
    reader.log_in_again().await;
    reader.read_good().await;

    reader.harness.clock.set(utc(2026, 10, 13, 7, 0));
    let before = requests(&reader.harness.source).await;
    assert!(tick(&reader.harness).await.is_none());
    assert_eq!(requests(&reader.harness.source).await, before);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_failed_night_is_not_tried_again_until_the_next_night(pool: sqlx::PgPool) {
    let reader = read_on_monday(pool).await;
    reader.answer_with(ResponseTemplate::new(500));

    reader.harness.clock.set(utc(2026, 10, 13, 1, 1));
    let reading = tick(&reader.harness).await.expect("a nightly reading");
    assert!(
        matches!(reading.result, ReadingResult::Failed(_)),
        "{reading:?}"
    );

    reader.harness.clock.set(utc(2026, 10, 13, 1, 6));
    assert!(tick(&reader.harness).await.is_none());

    reader.answer(games_with(|_| {}));
    reader.harness.clock.set(utc(2026, 10, 14, 1, 1));
    let reading = tick(&reader.harness).await.expect("the next night");
    assert!(is_good(&reading), "{reading:?}");
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn an_empty_game_list_at_night_is_held_back_for_the_coordinator(pool: sqlx::PgPool) {
    let mut reader = read_on_monday(pool).await;
    reader.answer(games_with(Vec::clear));

    reader.harness.clock.set(utc(2026, 10, 13, 1, 1));
    let reading = tick(&reader.harness).await.expect("a nightly reading");
    assert!(
        matches!(reading.result, ReadingResult::HeldBack(_)),
        "{reading:?}"
    );

    reader.log_in_again().await;
    let latest = reader.latest().await;
    assert_eq!(latest["held_back"]["id"], reading.id);
    assert_eq!(latest["held_back"]["hold"]["reason"], "empty");
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_reading_that_ends_in_an_error_is_not_tried_again_the_same_night(pool: sqlx::PgPool) {
    let reader = read_on_monday(pool).await;
    // Storing a good reading fails, so the reading ends in an error and stores nothing.
    sqlx::raw_sql(
        "CREATE FUNCTION fail_good_reading() RETURNS trigger LANGUAGE plpgsql AS
             $$ BEGIN RAISE EXCEPTION 'test failure'; END $$;
         CREATE TRIGGER fail_good_reading BEFORE INSERT ON readings FOR EACH ROW
             WHEN (NEW.result = 'good') EXECUTE FUNCTION fail_good_reading();",
    )
    .execute(reader.pool())
    .await
    .unwrap();
    let mut tried = Tried::default();

    reader.harness.clock.set(utc(2026, 10, 13, 1, 1));
    assert!(tick_with(&reader.harness, &mut tried).await.is_err());

    let before = requests(&reader.harness.source).await;
    reader.harness.clock.set(utc(2026, 10, 13, 1, 6));
    assert!(
        tick_with(&reader.harness, &mut tried)
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(requests(&reader.harness.source).await, before);

    sqlx::raw_sql("DROP TRIGGER fail_good_reading ON readings")
        .execute(reader.pool())
        .await
        .unwrap();
    reader.harness.clock.set(utc(2026, 10, 14, 1, 1));
    let reading = tick_with(&reader.harness, &mut tried)
        .await
        .unwrap()
        .expect("the next night");
    assert!(is_good(&reading), "{reading:?}");
}
