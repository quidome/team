//! The player names are stored encrypted (change 2026-10-18-encrypted-player-names): the database
//! has no plain name, the order and the same-name warning stay, and `team migrate` encrypts the
//! players from before once.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use chrono::{NaiveDate, Utc};
use common::api::Coordinator;
use serde_json::json;
use sqlx::PgPool;
use sqlx::migrate::Migrator;
use team::cipher::NameKey;
use team::db::StoreError;
use team::players;
use team_core::{PlayerId, Season};

fn key() -> NameKey {
    NameKey::from_base64(common::PLAYER_NAME_KEY).unwrap()
}

/// The plain and the encrypted columns of a raw row.
type RawNames = (
    Option<String>,
    Option<String>,
    Option<Vec<u8>>,
    Option<Vec<u8>>,
);

fn contains(haystack: &[u8], needle: &str) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle.as_bytes())
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn names_are_not_stored_as_plain_text(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    coordinator
        .ok(
            "POST",
            "/api/players",
            Some(json!({ "first_name": "Anna", "last_name": "de Vries" })),
        )
        .await;

    let (first, last, first_enc, last_enc): RawNames =
        sqlx::query_as("SELECT first_name, last_name, first_name_enc, last_name_enc FROM players")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((first, last), (None, None));
    let (first_enc, last_enc) = (first_enc.unwrap(), last_enc.unwrap());
    assert!(!contains(&first_enc, "Anna"));
    assert!(!contains(&last_enc, "Vries"));

    let all = players::all(&pool, &key()).await.unwrap();
    assert_eq!(all[0].first_name, "Anna");
    assert_eq!(all[0].last_name.as_deref(), Some("de Vries"));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn encrypted_players_are_listed_in_order_and_the_same_name_is_found(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    for (first, last) in [
        ("bram", None),
        ("Anna", Some("de Vries")),
        ("anna", Some("Bakker")),
    ] {
        let answer = coordinator
            .ok(
                "POST",
                "/api/players",
                Some(json!({ "first_name": first, "last_name": last })),
            )
            .await;
        assert_eq!(answer["same_name"], false);
    }

    let list = coordinator.ok("GET", "/api/players", None).await;
    let names: Vec<(String, String)> = list["players"]
        .as_array()
        .unwrap()
        .iter()
        .map(|player| {
            (
                player["first_name"].as_str().unwrap().to_owned(),
                player["last_name"].as_str().unwrap_or("").to_owned(),
            )
        })
        .collect();
    assert_eq!(
        names,
        [
            ("anna".to_owned(), "Bakker".to_owned()),
            ("Anna".to_owned(), "de Vries".to_owned()),
            ("bram".to_owned(), String::new()),
        ]
    );

    let again = coordinator
        .ok(
            "POST",
            "/api/players",
            Some(json!({ "first_name": "ANNA", "last_name": "DE VRIES" })),
        )
        .await;
    assert_eq!(again["same_name"], true);
}

/// A database at migration 0006 (before the encryption), with the migration files copied to a
/// temporary folder.
async fn migrate_to_0006(pool: &PgPool) {
    let folder = tempfile::tempdir().unwrap();
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations");
    for entry in std::fs::read_dir(&source).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();
        if name.as_str() < "0007" {
            std::fs::copy(&path, folder.path().join(&name)).unwrap();
        }
    }
    let migrator = Migrator::new(folder.path()).await.unwrap();
    assert_eq!(migrator.iter().count(), 6);
    migrator.run(pool).await.unwrap();
}

#[sqlx::test(migrations = false)]
async fn encrypt_plain_names_encrypts_each_player_once(pool: PgPool) {
    migrate_to_0006(&pool).await;
    let now = Utc::now();
    let mut ids = Vec::new();
    for (first, last, hidden) in [
        ("Anna", Some("de Vries"), false),
        ("Bram", None, false),
        ("Cas", Some("Jansen"), true),
    ] {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO players (first_name, last_name, hidden, created_at)
             VALUES ($1, $2, $3, $4) RETURNING id",
        )
        .bind(first)
        .bind(last)
        .bind(hidden)
        .bind(now)
        .fetch_one(&pool)
        .await
        .unwrap();
        ids.push(id);
    }
    let anna = ids[0];
    let season = Season::from_start_year(2026);
    let tuesday = NaiveDate::from_ymd_opt(2026, 10, 6).unwrap();
    sqlx::query(
        "INSERT INTO player_roles (player_id, season_year, role, from_date) VALUES ($1, $2, 'plays_games', '2026-09-01')",
    )
    .bind(anna)
    .bind(season.start_year())
    .execute(&pool)
    .await
    .unwrap();
    let training: i64 = sqlx::query_scalar(
        "INSERT INTO trainings (date, start_time, end_time, created_at)
         VALUES ($1, '19:45', '21:00', now()) RETURNING id",
    )
    .bind(tuesday)
    .fetch_one(&pool)
    .await
    .unwrap();
    let reason: i64 = sqlx::query_scalar("SELECT id FROM reasons ORDER BY position LIMIT 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO absences (training_id, player_id, reason_id) VALUES ($1, $2, $3)")
        .bind(training)
        .bind(anna)
        .bind(reason)
        .execute(&pool)
        .await
        .unwrap();

    team::db::migrate(&pool).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    assert_eq!(
        players::encrypt_plain_names(&mut tx, &key()).await.unwrap(),
        3
    );
    tx.commit().await.unwrap();

    let plain: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM players WHERE first_name IS NOT NULL OR last_name IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(plain, 0);
    let all = players::all(&pool, &key()).await.unwrap();
    let read: Vec<(i64, &str, Option<&str>, bool)> = all
        .iter()
        .map(|player| {
            (
                player.id.0,
                player.first_name.as_str(),
                player.last_name.as_deref(),
                player.hidden,
            )
        })
        .collect();
    assert_eq!(
        read,
        [
            (ids[0], "Anna", Some("de Vries"), false),
            (ids[1], "Bram", None, false),
            (ids[2], "Cas", Some("Jansen"), true),
        ]
    );
    // The role and the absence still point at the player.
    assert_eq!(
        players::roles_of(&pool, PlayerId(anna))
            .await
            .unwrap()
            .len(),
        1
    );
    let absences: i64 = sqlx::query_scalar("SELECT count(*) FROM absences WHERE player_id = $1")
        .bind(anna)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(absences, 1);

    let mut tx = pool.begin().await.unwrap();
    assert_eq!(
        players::encrypt_plain_names(&mut tx, &key()).await.unwrap(),
        0
    );
    tx.commit().await.unwrap();
    assert_eq!(players::all(&pool, &key()).await.unwrap().len(), 3);
}

fn other_key() -> NameKey {
    // 32 bytes of 8.
    NameKey::from_base64("CAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAg=").unwrap()
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn check_key_reads_a_stored_name_with_the_right_key_only(pool: PgPool) {
    // An empty table: nothing to read, so any key is accepted.
    players::check_key(&pool, &other_key()).await.unwrap();

    let id = players::insert(&pool, &key(), "Zephyrine", Some("Quistorp"), Utc::now())
        .await
        .unwrap();
    players::check_key(&pool, &key()).await.unwrap();
    let error = players::check_key(&pool, &other_key()).await.unwrap_err();
    assert!(
        matches!(error, StoreError::Name { player, .. } if player == id),
        "{error}"
    );
    let text = error.to_string();
    assert!(text.contains(&format!("player {}", id.0)), "{text}");
    assert!(
        !text.contains("Zephyrine") && !text.contains("Quistorp"),
        "{text}"
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn migrate_with_a_wrong_key_stops_and_changes_nothing(pool: PgPool) {
    players::insert(&pool, &key(), "Anna", None, Utc::now())
        .await
        .unwrap();
    // A plain row, as an older version writes it during an update.
    sqlx::query("INSERT INTO players (first_name, created_at) VALUES ('Bram', now())")
        .execute(&pool)
        .await
        .unwrap();

    let error = players::encrypt_after_migrations(&pool, &other_key())
        .await
        .unwrap_err();
    assert!(matches!(error, StoreError::Name { .. }), "{error}");
    let plain: i64 = sqlx::query_scalar("SELECT count(*) FROM players WHERE first_name = 'Bram'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(plain, 1, "the plain row stays plain");

    // With the right key the plain row is encrypted.
    assert_eq!(
        players::encrypt_after_migrations(&pool, &key())
            .await
            .unwrap(),
        1
    );
    let names: Vec<String> = players::all(&pool, &key())
        .await
        .unwrap()
        .into_iter()
        .map(|player| player.first_name)
        .collect();
    assert_eq!(names, ["Anna", "Bram"]);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_rename_by_an_older_version_is_encrypted_over_the_old_name(pool: PgPool) {
    let id = players::insert(&pool, &key(), "Anna", Some("de Vries"), Utc::now())
        .await
        .unwrap();
    // The old version (v0.3.0) renames during the update: it writes both plain columns and leaves
    // the encrypted ones.
    sqlx::query("UPDATE players SET first_name = 'Anne', last_name = NULL WHERE id = $1")
        .bind(id.0)
        .execute(&pool)
        .await
        .unwrap();

    assert_eq!(
        players::encrypt_after_migrations(&pool, &key())
            .await
            .unwrap(),
        1
    );
    let player = players::by_id(&pool, &key(), id).await.unwrap().unwrap();
    assert_eq!(player.first_name, "Anne");
    assert_eq!(player.last_name, None);
    let plain: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM players WHERE first_name IS NOT NULL OR last_name IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(plain, 0);
}
