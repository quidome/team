//! Migrations that change stored data: the data before them, and after them; and the keys of
//! tables that the code relies on.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use sqlx::PgPool;

/// Runs the SQL of every migration before `version`, without the record of migrations.
async fn migrate_before(pool: &PgPool, version: i64) {
    for migration in team::db::MIGRATOR.iter().filter(|m| m.version < version) {
        sqlx::raw_sql(&migration.sql).execute(pool).await.unwrap();
    }
}

/// Runs the SQL of the migration `version`.
async fn migrate_one(pool: &PgPool, version: i64) {
    let migration = team::db::MIGRATOR
        .iter()
        .find(|m| m.version == version)
        .unwrap();
    sqlx::raw_sql(&migration.sql).execute(pool).await.unwrap();
}

/// A training `days` from today (Dutch date), for example -1 for yesterday.
async fn insert_training(pool: &PgPool, days: i32) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO trainings (date, start_time, end_time, created_at)
         VALUES ((now() AT TIME ZONE 'Europe/Amsterdam')::date + $1, '19:45', '21:00', now())
         RETURNING id",
    )
    .bind(days)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[sqlx::test(migrations = false)]
async fn the_release_checks_the_trainings_before_today(pool: PgPool) {
    migrate_before(&pool, 10).await;
    let yesterday = insert_training(&pool, -1).await;
    let today = insert_training(&pool, 0).await;
    let tomorrow = insert_training(&pool, 1).await;

    migrate_one(&pool, 10).await;

    let checked: Vec<(i64, bool)> =
        sqlx::query_as("SELECT id, checked_at IS NOT NULL FROM trainings ORDER BY date")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(
        checked,
        [(yesterday, true), (today, false), (tomorrow, false)]
    );
}

/// Inserts a player (the name is not read here) and gives its id.
async fn insert_player(pool: &PgPool) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO players (first_name_enc, created_at) VALUES ('\\x00', now()) RETURNING id",
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

/// Puts `player` on the place `kind` `number` of game 1.
async fn assign(pool: &PgPool, kind: &str, number: i16, player: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO duty_assignments
             (club_number, season_year, source_id, kind, number, planned_player)
         VALUES (37, 2026, 1, $1, $2, $3)",
    )
    .bind(kind)
    .bind(number)
    .bind(player)
    .execute(pool)
    .await
    .map(|_| ())
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_keys_keep_one_player_on_a_place_and_one_place_for_a_player(pool: PgPool) {
    sqlx::raw_sql(
        "INSERT INTO places (number, hall, town) VALUES (554, 'Bulwijk', 'Woerden');
         INSERT INTO games (club_number, season_year, source_id, game_number, date, start_time,
             home_team_number, home_team_name, home_club_number, away_team_number,
             away_team_name, away_club_number, place_number, competition, in_list, first_read_at)
         VALUES (37, 2026, 1, '1', '2026-10-31', '13:00', 184, 'Blue Drakes M16 1', 37, 9,
             'Lions M16 1', 21, 554, 'M16', true, now());
         INSERT INTO game_duties (club_number, season_year, source_id, saved_at)
         VALUES (37, 2026, 1, now());",
    )
    .execute(&pool)
    .await
    .unwrap();
    let anna = insert_player(&pool).await;
    let bram = insert_player(&pool).await;

    assign(&pool, "table", 1, anna).await.unwrap();
    // A second player on the same place, and the same player on a second place.
    assert!(assign(&pool, "table", 1, bram).await.is_err());
    assert!(assign(&pool, "table", 2, anna).await.is_err());
    assign(&pool, "table", 2, bram).await.unwrap();

    // "Done by" needs its player, and only then.
    let outcome = |outcome: &'static str, done_by: Option<i64>| {
        sqlx::query(
            "UPDATE duty_assignments SET outcome = $1, done_by = $2
             WHERE kind = 'table' AND number = 1",
        )
        .bind(outcome)
        .bind(done_by)
        .execute(&pool)
    };
    assert!(outcome("done_by", None).await.is_err());
    assert!(outcome("not_done", Some(bram)).await.is_err());
    outcome("done_by", Some(bram)).await.unwrap();

    // A player with a duty cannot be deleted.
    let deleted = sqlx::query("DELETE FROM players WHERE id = $1")
        .bind(anna)
        .execute(&pool)
        .await;
    assert!(deleted.is_err());
}
