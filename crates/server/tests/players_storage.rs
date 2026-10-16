//! Storage of players, roles, reasons and training settings (attendance at training, story A).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use chrono::{NaiveDate, NaiveTime, TimeZone, Utc, Weekday};
use sqlx::PgPool;
use team::{players, reasons, training_settings};
use team_core::{Role, Season, TrainingDay, TrainingDaySet};

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

fn time(hour: u32, minute: u32) -> NaiveTime {
    NaiveTime::from_hms_opt(hour, minute, 0).unwrap()
}

fn season() -> Season {
    Season::from_start_year(2026)
}

fn set(valid_from: NaiveDate, weekday: Weekday) -> TrainingDaySet {
    TrainingDaySet {
        valid_from,
        days: vec![TrainingDay {
            weekday,
            start: time(19, 45),
            end: time(21, 0),
        }],
    }
}

/// `training_settings::save`, replacing the stored sets from `set`'s date on.
async fn store(
    tx: &mut sqlx::PgConnection,
    season: Season,
    first: NaiveDate,
    last: NaiveDate,
    set: &TrainingDaySet,
) -> Result<(), team::db::StoreError> {
    training_settings::save(tx, season, first, last, set, set.valid_from).await
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_migration_adds_the_six_preset_reasons_in_order(pool: PgPool) {
    let all: Vec<(String, bool, bool)> = reasons::all(&pool)
        .await
        .unwrap()
        .into_iter()
        .map(|reason| (reason.name, reason.valid, reason.hidden))
        .collect();
    let expected = [
        ("School", true),
        ("Ill", true),
        ("Injured", true),
        ("Holiday", false),
        ("Work", false),
        ("No reason", false),
    ]
    .map(|(name, valid)| (name.to_owned(), valid, false));
    assert_eq!(all, expected);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_new_reason_comes_last_and_a_taken_name_is_refused(pool: PgPool) {
    reasons::insert(&pool, "Family", false).await.unwrap();
    let all = reasons::all(&pool).await.unwrap();
    assert_eq!(
        all.last().map(|reason| reason.name.as_str()),
        Some("Family")
    );

    let error = reasons::insert(&pool, " school ", true).await.unwrap_err();
    assert!(error.is_unique_violation(), "{error}");
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_player_with_two_roles_is_read_back_with_both(pool: PgPool) {
    let now = Utc.with_ymd_and_hms(2026, 10, 12, 8, 0, 0).unwrap();
    let id = players::insert(&pool, "Avery", None, now).await.unwrap();
    players::insert_role(
        &pool,
        id,
        season(),
        Role::OnlyTrains,
        date(2026, 9, 1),
        Some(date(2026, 11, 15)),
    )
    .await
    .unwrap();
    players::insert_role(
        &pool,
        id,
        season(),
        Role::PlaysGames,
        date(2027, 1, 10),
        None,
    )
    .await
    .unwrap();

    let roles = players::roles_of(&pool, id).await.unwrap();
    assert_eq!(roles.len(), 2);
    assert_eq!(roles[0].role, Role::OnlyTrains);
    assert_eq!(roles[0].until, Some(date(2026, 11, 15)));
    assert_eq!(roles[1].role, Role::PlaysGames);
    assert_eq!(roles[1].until, None);

    let in_season = players::roles_in(&pool, &[season()]).await.unwrap();
    assert_eq!(in_season.len(), 2);
    let earlier = players::roles_in(&pool, &[Season::from_start_year(2025)])
        .await
        .unwrap();
    assert!(earlier.is_empty());
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn deleting_a_player_deletes_its_roles(pool: PgPool) {
    let now = Utc.with_ymd_and_hms(2026, 10, 12, 8, 0, 0).unwrap();
    let id = players::insert(&pool, "Avery", None, now).await.unwrap();
    players::insert_role(
        &pool,
        id,
        season(),
        Role::PlaysGames,
        date(2026, 9, 1),
        None,
    )
    .await
    .unwrap();
    assert!(players::delete(&pool, id).await.unwrap());
    assert!(players::by_id(&pool, id).await.unwrap().is_none());
    assert!(players::roles_of(&pool, id).await.unwrap().is_empty());
    assert!(!players::delete(&pool, id).await.unwrap());
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_second_set_is_kept_next_to_the_first_and_the_same_date_replaces(pool: PgPool) {
    let first = date(2026, 9, 1);
    let last = date(2027, 5, 25);
    let mut tx = pool.begin().await.unwrap();
    store(&mut tx, season(), first, last, &set(first, Weekday::Tue))
        .await
        .unwrap();
    store(
        &mut tx,
        season(),
        first,
        last,
        &set(date(2027, 1, 5), Weekday::Wed),
    )
    .await
    .unwrap();
    // A second change on the same day replaces the set of that day.
    store(
        &mut tx,
        season(),
        first,
        last,
        &set(date(2027, 1, 5), Weekday::Thu),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let mut db = pool.acquire().await.unwrap();
    let stored = training_settings::for_season(&mut db, season())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.first, first);
    assert_eq!(stored.last, last);
    assert_eq!(
        stored.sets,
        vec![
            set(first, Weekday::Tue),
            set(date(2027, 1, 5), Weekday::Thu)
        ]
    );
    assert!(
        training_settings::for_season(&mut db, Season::from_start_year(2025))
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_save_removes_the_sets_from_its_date_on(pool: PgPool) {
    let first = date(2026, 9, 1);
    let last = date(2027, 5, 25);
    let mut tx = pool.begin().await.unwrap();
    for set in [
        set(first, Weekday::Mon),
        set(date(2026, 9, 8), Weekday::Tue),
        set(date(2026, 10, 6), Weekday::Wed),
    ] {
        store(&mut tx, season(), first, last, &set).await.unwrap();
    }
    store(
        &mut tx,
        season(),
        first,
        last,
        &set(date(2026, 9, 8), Weekday::Thu),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let mut db = pool.acquire().await.unwrap();
    let stored = training_settings::for_season(&mut db, season())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        stored.sets,
        vec![
            set(first, Weekday::Mon),
            set(date(2026, 9, 8), Weekday::Thu)
        ]
    );
}
