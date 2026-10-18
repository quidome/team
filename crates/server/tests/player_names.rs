//! The player names are stored encrypted (change 2026-10-18-encrypted-player-names): the database
//! has no plain name and no plain columns, the order and the same-name warning stay, and the
//! removal of the plain columns refuses a player that is not encrypted.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use chrono::Utc;
use common::api::Coordinator;
use serde_json::json;
use sqlx::PgPool;
use sqlx::migrate::Migrator;
use team::cipher::NameKey;
use team::db::StoreError;
use team::players;

fn key() -> NameKey {
    NameKey::from_base64(common::PLAYER_NAME_KEY).unwrap()
}

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

    // The whole row as text, and the encrypted bytes.
    let (row, first_enc, last_enc): (String, Vec<u8>, Option<Vec<u8>>) =
        sqlx::query_as("SELECT players::text, first_name_enc, last_name_enc FROM players")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(!row.contains("Anna") && !row.contains("Vries"), "{row}");
    let last_enc = last_enc.unwrap();
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

/// A database at the migrations before `version`, with the migration files copied to a temporary
/// folder.
async fn migrate_before(pool: &PgPool, version: &str) {
    let folder = tempfile::tempdir().unwrap();
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations");
    for entry in std::fs::read_dir(&source).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();
        if name.as_str() < version {
            std::fs::copy(&path, folder.path().join(&name)).unwrap();
        }
    }
    Migrator::new(folder.path())
        .await
        .unwrap()
        .run(pool)
        .await
        .unwrap();
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_plain_name_columns_are_gone(pool: PgPool) {
    let columns: Vec<(String, String)> = sqlx::query_as(
        "SELECT column_name::text, is_nullable::text FROM information_schema.columns
         WHERE table_name = 'players' ORDER BY ordinal_position",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    let columns: Vec<(&str, &str)> = columns
        .iter()
        .map(|(name, nullable)| (name.as_str(), nullable.as_str()))
        .collect();
    assert_eq!(
        columns,
        [
            ("id", "NO"),
            ("hidden", "NO"),
            ("created_at", "NO"),
            ("first_name_enc", "NO"),
            ("last_name_enc", "YES"),
        ]
    );
}

#[sqlx::test(migrations = false)]
async fn the_removal_refuses_a_player_that_is_not_encrypted(pool: PgPool) {
    migrate_before(&pool, "0008").await;
    // A plain row, as when release 1 was skipped.
    sqlx::query("INSERT INTO players (first_name, created_at) VALUES ('Bram', now())")
        .execute(&pool)
        .await
        .unwrap();

    assert!(team::db::migrate(&pool).await.is_err());
    let plain: i64 = sqlx::query_scalar("SELECT count(*) FROM players WHERE first_name = 'Bram'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(plain, 1, "the row and its plain name are still there");
    let version: i64 = sqlx::query_scalar("SELECT max(version) FROM _sqlx_migrations")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(version, 7);
}
