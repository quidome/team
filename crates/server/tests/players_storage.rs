//! Storage of players, roles, reasons and weekly trainings (attendance at training, story A).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use chrono::{NaiveDate, NaiveTime, TimeZone, Utc, Weekday};
use sqlx::PgPool;
use team::weekly_trainings::{self, WeeklyTrainingData};
use team::{players, reasons};
use team_core::{Role, Season, WeeklyTrainingId};

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

fn time(hour: u32, minute: u32) -> NaiveTime {
    NaiveTime::from_hms_opt(hour, minute, 0).unwrap()
}

fn season() -> Season {
    Season::from_start_year(2026)
}

fn tuesday(from: NaiveDate, until: NaiveDate) -> WeeklyTrainingData {
    WeeklyTrainingData {
        weekday: Weekday::Tue,
        start: time(19, 45),
        end: time(21, 0),
        from,
        until,
    }
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
async fn a_weekly_training_is_read_back_changed_and_deleted(pool: PgPool) {
    let id = weekly_trainings::insert(
        &pool,
        season(),
        tuesday(date(2026, 9, 1), date(2027, 5, 25)),
    )
    .await
    .unwrap();
    let stored = weekly_trainings::for_season(&pool, season()).await.unwrap();
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].id, id);
    assert_eq!(stored[0].weekday, Weekday::Tue);
    assert_eq!(stored[0].start, time(19, 45));
    assert_eq!(stored[0].from, date(2026, 9, 1));
    assert_eq!(stored[0].until, date(2027, 5, 25));
    assert!(
        weekly_trainings::for_season(&pool, Season::from_start_year(2025))
            .await
            .unwrap()
            .is_empty()
    );

    let thursday = WeeklyTrainingData {
        weekday: Weekday::Thu,
        ..tuesday(date(2026, 9, 3), date(2027, 5, 27))
    };
    assert!(
        weekly_trainings::update(&pool, season(), id, thursday)
            .await
            .unwrap()
    );
    let stored = weekly_trainings::for_season(&pool, season()).await.unwrap();
    assert_eq!(stored[0].weekday, Weekday::Thu);
    // Not a weekly training of that season, or none at all.
    let last_season = Season::from_start_year(2025);
    assert!(
        !weekly_trainings::update(&pool, last_season, id, thursday)
            .await
            .unwrap()
    );
    assert!(
        !weekly_trainings::update(&pool, season(), WeeklyTrainingId(id.0 + 1), thursday)
            .await
            .unwrap()
    );

    assert!(
        !weekly_trainings::delete(&pool, last_season, id)
            .await
            .unwrap()
    );
    assert!(weekly_trainings::delete(&pool, season(), id).await.unwrap());
    assert!(
        weekly_trainings::for_season(&pool, season())
            .await
            .unwrap()
            .is_empty()
    );
}
