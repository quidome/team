//! Players, roles and reasons through the API (attendance at training, story A).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::api::Coordinator;
use common::game_list::utc;
use serde_json::{Value, json};
use std::time::Duration;

use chrono::NaiveDate;
use sqlx::PgPool;
use team::players;
use team_core::{PlayerId, Role, Season};

fn names(list: &Value) -> Vec<&str> {
    list.as_array()
        .unwrap()
        .iter()
        .map(|player| player["first_name"].as_str().unwrap())
        .collect()
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_player_with_a_role_is_listed_with_it(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    let answer = coordinator
        .ok(
            "POST",
            "/api/players",
            Some(json!({ "first_name": " Avery ", "last_name": " Jansen ",
                         "role": { "role": "plays_games", "from": "2026-09-01" } })),
        )
        .await;
    assert_eq!(answer["same_name"], false);
    assert_eq!(answer["player"]["first_name"], "Avery");
    assert_eq!(answer["player"]["last_name"], "Jansen");
    assert_eq!(answer["player"]["role"], "plays_games");

    let list = coordinator.ok("GET", "/api/players", None).await;
    assert_eq!(list["season"], "2026-2027");
    assert_eq!(names(&list["players"]), ["Avery"]);
    assert_eq!(
        list["players"][0]["roles"],
        json!([{ "id": list["players"][0]["roles"][0]["id"], "role": "plays_games",
                 "from": "2026-09-01", "until": null }])
    );

    // The same name again is allowed, with a warning.
    let again = coordinator
        .ok(
            "POST",
            "/api/players",
            Some(json!({ "first_name": "avery", "last_name": "JANSEN", "role": null })),
        )
        .await;
    assert_eq!(again["same_name"], true);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_first_name_is_needed(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    let (status, answer) = coordinator
        .send(
            "POST",
            "/api/players",
            Some(json!({ "first_name": "  ", "last_name": "Jansen", "role": null })),
        )
        .await;
    assert_eq!(
        (status, answer),
        (400, json!({ "error": "first_name_required" }))
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_player_without_a_role_is_under_others(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    coordinator
        .ok(
            "POST",
            "/api/players",
            Some(json!({ "first_name": "Casey", "last_name": null, "role": null })),
        )
        .await;
    let list = coordinator.ok("GET", "/api/players", None).await;
    assert_eq!(names(&list["others"]), ["Casey"]);
    assert_eq!(list["others"][0]["role"], Value::Null);
    assert!(list["players"].as_array().unwrap().is_empty());
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn left_the_team_and_undo(pool: PgPool) {
    let mut coordinator = Coordinator::start(pool).await;
    let id = coordinator
        .add_player("Blake", "only_trains", "2026-09-01")
        .await;
    let list = coordinator.ok("GET", "/api/players", None).await;
    let role = list["players"][0]["roles"][0]["id"].as_i64().unwrap();

    coordinator
        .ok(
            "PUT",
            &format!("/api/players/{id}/roles/{role}"),
            Some(json!({ "role": "only_trains", "from": "2026-09-01", "until": "2026-10-15" })),
        )
        .await;
    coordinator.harness.clock.set(utc(2026, 10, 20, 12, 0));
    coordinator.log_in_again().await;
    let list = coordinator.ok("GET", "/api/players", None).await;
    assert_eq!(names(&list["left"]), ["Blake"]);
    assert_eq!(list["left"][0]["role"], "only_trains");
    assert!(list["players"].as_array().unwrap().is_empty());

    coordinator
        .ok(
            "PUT",
            &format!("/api/players/{id}/roles/{role}"),
            Some(json!({ "role": "only_trains", "from": "2026-09-01", "until": null })),
        )
        .await;
    let list = coordinator.ok("GET", "/api/players", None).await;
    assert_eq!(names(&list["players"]), ["Blake"]);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn back_in_the_team_is_a_new_role(pool: PgPool) {
    let mut coordinator = Coordinator::start(pool).await;
    let id = coordinator
        .add_player("Dana", "plays_games", "2026-09-01")
        .await;
    let role = coordinator.ok("GET", "/api/players", None).await["players"][0]["roles"][0]["id"]
        .as_i64()
        .unwrap();
    coordinator
        .ok(
            "PUT",
            &format!("/api/players/{id}/roles/{role}"),
            Some(json!({ "role": "plays_games", "from": "2026-09-01", "until": "2026-10-15" })),
        )
        .await;
    coordinator
        .ok(
            "POST",
            &format!("/api/players/{id}/roles"),
            Some(json!({ "role": "plays_games", "from": "2026-11-01", "until": null })),
        )
        .await;

    coordinator.harness.clock.set(utc(2026, 11, 2, 12, 0));
    coordinator.log_in_again().await;
    let list = coordinator.ok("GET", "/api/players", None).await;
    assert_eq!(names(&list["players"]), ["Dana"]);
    assert_eq!(list["players"][0]["roles"].as_array().unwrap().len(), 2);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn an_overlapping_role_is_refused_and_nothing_is_stored(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    let id = coordinator
        .add_player("Avery", "plays_games", "2026-09-01")
        .await;
    let (status, answer) = coordinator
        .send(
            "POST",
            &format!("/api/players/{id}/roles"),
            Some(json!({ "role": "only_trains", "from": "2027-01-10", "until": null })),
        )
        .await;
    assert_eq!((status, answer), (400, json!({ "error": "roles_overlap" })));
    let list = coordinator.ok("GET", "/api/players", None).await;
    assert_eq!(list["players"][0]["roles"].as_array().unwrap().len(), 1);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn role_errors_have_their_codes(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    let id = coordinator
        .add_player("Avery", "plays_games", "2026-09-01")
        .await;
    let role =
        |from: &str, until: &str| json!({ "role": "plays_games", "from": from, "until": until });
    let path = format!("/api/players/{id}/roles");
    let (_, answer) = coordinator
        .send("POST", &path, Some(role("2027-03-01", "2027-02-01")))
        .await;
    assert_eq!(answer, json!({ "error": "until_before_from" }));
    let (_, answer) = coordinator
        .send("POST", &path, Some(role("2027-03-01", "2027-07-01")))
        .await;
    assert_eq!(answer, json!({ "error": "outside_season" }));
    let (status, _) = coordinator
        .send(
            "POST",
            "/api/players/999999/roles",
            Some(role("2027-03-01", "2027-04-01")),
        )
        .await;
    assert_eq!(status, 404);
    let (status, _) = coordinator
        .send(
            "PUT",
            &format!("/api/players/{id}/roles/999999"),
            Some(role("2027-03-01", "2027-04-01")),
        )
        .await;
    assert_eq!(status, 404);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn two_role_saves_at_the_same_time_never_overlap(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    let answer = coordinator
        .ok(
            "POST",
            "/api/players",
            Some(json!({ "first_name": "Avery", "last_name": null, "role": null })),
        )
        .await;
    let id = answer["player"]["id"].as_i64().unwrap();

    // The first save holds the player's lock and has written its role, but not committed yet.
    let mut first = pool.begin().await.unwrap();
    assert!(players::lock(&mut first, PlayerId(id)).await.unwrap());
    players::insert_role(
        &mut *first,
        PlayerId(id),
        Season::from_start_year(2026),
        Role::PlaysGames,
        NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
        None,
    )
    .await
    .unwrap();

    // The second save waits for the lock, then sees the first role.
    let path = format!("/api/players/{id}/roles");
    let second = coordinator.send(
        "POST",
        &path,
        Some(json!({ "role": "only_trains", "from": "2026-10-01", "until": null })),
    );
    tokio::pin!(second);
    let waited = tokio::time::timeout(Duration::from_millis(300), &mut second).await;
    assert!(waited.is_err(), "the second save did not wait for the lock");
    first.commit().await.unwrap();
    assert_eq!(second.await, (400, json!({ "error": "roles_overlap" })));
    let list = coordinator.ok("GET", "/api/players", None).await;
    assert_eq!(list["players"][0]["roles"].as_array().unwrap().len(), 1);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn rename_and_delete_a_player(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    let id = coordinator
        .add_player("Avery", "plays_games", "2026-09-01")
        .await;
    let answer = coordinator
        .ok(
            "PUT",
            &format!("/api/players/{id}"),
            Some(json!({ "first_name": "Avery", "last_name": "de Vries" })),
        )
        .await;
    assert_eq!(answer["player"]["last_name"], "de Vries");
    assert_eq!(answer["same_name"], false);

    let (status, _) = coordinator
        .send("DELETE", &format!("/api/players/{id}"), None)
        .await;
    assert_eq!(status, 204);
    let list = coordinator.ok("GET", "/api/players", None).await;
    assert!(list["players"].as_array().unwrap().is_empty());
    let (status, _) = coordinator
        .send("DELETE", &format!("/api/players/{id}"), None)
        .await;
    assert_eq!(status, 404);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn reasons_can_be_added_renamed_and_hidden(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    let list = coordinator
        .ok(
            "POST",
            "/api/reasons",
            Some(json!({ "name": " Family ", "valid": false })),
        )
        .await;
    let last = list.as_array().unwrap().last().unwrap();
    assert_eq!(
        (&last["name"], &last["valid"]),
        (&json!("Family"), &json!(false))
    );

    let find = |list: &Value, name: &str| -> Value {
        list.as_array()
            .unwrap()
            .iter()
            .find(|reason| reason["name"] == name)
            .unwrap()
            .clone()
    };
    let work = find(&list, "Work")["id"].as_i64().unwrap();
    let list = coordinator
        .ok(
            "PUT",
            &format!("/api/reasons/{work}"),
            Some(json!({ "name": "work", "valid": false, "hidden": false })),
        )
        .await;
    assert_eq!(find(&list, "work")["id"], work);

    let (status, answer) = coordinator
        .send(
            "PUT",
            &format!("/api/reasons/{work}"),
            Some(json!({ "name": "school", "valid": false, "hidden": false })),
        )
        .await;
    assert_eq!(
        (status, answer),
        (400, json!({ "error": "reason_name_taken" }))
    );

    let holiday = find(&list, "Holiday")["id"].as_i64().unwrap();
    let list = coordinator
        .ok(
            "PUT",
            &format!("/api/reasons/{holiday}"),
            Some(json!({ "name": "Holiday", "valid": false, "hidden": true })),
        )
        .await;
    assert_eq!(find(&list, "Holiday")["hidden"], true);

    let (status, answer) = coordinator
        .send(
            "POST",
            "/api/reasons",
            Some(json!({ "name": " ", "valid": true })),
        )
        .await;
    assert_eq!(
        (status, answer),
        (400, json!({ "error": "reason_name_required" }))
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn every_new_route_needs_a_login_and_changes_need_the_origin(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    let routes = [
        ("GET", "/api/players"),
        ("POST", "/api/players"),
        ("PUT", "/api/players/1"),
        ("DELETE", "/api/players/1"),
        ("POST", "/api/players/1/roles"),
        ("PUT", "/api/players/1/roles/1"),
        ("POST", "/api/players/carry"),
        ("POST", "/api/players/1/role-change"),
        ("POST", "/api/players/1/team-leave"),
        ("GET", "/api/team"),
        ("GET", "/api/reasons"),
        ("POST", "/api/reasons"),
        ("PUT", "/api/reasons/1"),
        ("GET", "/api/weekly-trainings"),
        ("POST", "/api/weekly-trainings"),
        ("PUT", "/api/weekly-trainings/1"),
        ("DELETE", "/api/weekly-trainings/1"),
        ("GET", "/api/weekly-trainings/proposal?weekday=2"),
    ];
    for (method, path) in routes {
        let (status, _) = coordinator
            .send_with(method, path, Some(json!({})), None, true)
            .await;
        assert_eq!(status, 401, "{method} {path}");
        if method != "GET" {
            let (status, _) = coordinator
                .send_with(
                    method,
                    path,
                    Some(json!({})),
                    Some(&coordinator.cookie),
                    false,
                )
                .await;
            assert_eq!(status, 403, "{method} {path}");
        }
    }
}

/// The id of the player's first listed role in `group` of `GET /api/players`.
async fn first_role(coordinator: &Coordinator, group: &str, player: i64) -> i64 {
    coordinator.ok("GET", "/api/players", None).await[group]
        .as_array()
        .unwrap()
        .iter()
        .find(|listed| listed["id"] == player)
        .unwrap()["roles"][0]["id"]
        .as_i64()
        .unwrap()
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn roles_are_saved_only_in_the_current_season(pool: PgPool) {
    let mut coordinator = Coordinator::start(pool).await;
    coordinator.harness.clock.set(utc(2026, 3, 1, 12, 0));
    coordinator.log_in_again().await;
    let avery = coordinator
        .add_player("Avery", "plays_games", "2025-09-02")
        .await;
    let old_role = first_role(&coordinator, "players", avery).await;

    coordinator.harness.clock.set(utc(2026, 10, 12, 12, 0));
    coordinator.log_in_again().await;
    let list = coordinator.ok("GET", "/api/players", None).await;
    assert_eq!(list["season_first_day"], "2026-07-01");
    assert_eq!(list["season_last_day"], "2027-06-30");

    let outside = (400, json!({ "error": "outside_season" }));
    for from in ["2026-06-30", "2027-07-01"] {
        let answer = coordinator
            .send(
                "POST",
                "/api/players",
                Some(json!({ "first_name": "Blake", "last_name": null,
                             "role": { "role": "plays_games", "from": from } })),
            )
            .await;
        assert_eq!(answer, outside, "from {from}");
        let answer = coordinator
            .send(
                "POST",
                &format!("/api/players/{avery}/roles"),
                Some(json!({ "role": "plays_games", "from": from, "until": null })),
            )
            .await;
        assert_eq!(answer, outside, "from {from}");
    }
    // A role of last season is not changed, also not into this season.
    let answer = coordinator
        .send(
            "PUT",
            &format!("/api/players/{avery}/roles/{old_role}"),
            Some(json!({ "role": "plays_games", "from": "2026-09-01", "until": null })),
        )
        .await;
    assert_eq!(answer, outside);

    let list = coordinator.ok("GET", "/api/players", None).await;
    assert_eq!(names(&list["last_season"]), ["Avery"]);
    assert_eq!(list["last_season"][0]["roles"][0]["from"], "2025-09-02");
    assert!(list["others"].as_array().unwrap().is_empty());
}
