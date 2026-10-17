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

/// The history of a player: (season, role, from, until, correctable) of each role, newest first.
async fn history(coordinator: &Coordinator, id: i64) -> Vec<(String, String, String, Value, bool)> {
    coordinator
        .ok("GET", &format!("/api/players/{id}"), None)
        .await["history"]
        .as_array()
        .unwrap()
        .iter()
        .map(|role| {
            (
                role["season"].as_str().unwrap().to_owned(),
                role["role"].as_str().unwrap().to_owned(),
                role["from"].as_str().unwrap().to_owned(),
                role["until"].clone(),
                role["correctable"].as_bool().unwrap(),
            )
        })
        .collect()
}

/// The id of the newest role in the history of the player.
async fn newest_role(coordinator: &Coordinator, id: i64) -> i64 {
    coordinator
        .ok("GET", &format!("/api/players/{id}"), None)
        .await["history"][0]["id"]
        .as_i64()
        .unwrap()
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn all_players_are_listed_with_hidden_and_the_role_today(pool: PgPool) {
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
    assert_eq!(answer["player"]["hidden"], false);
    assert_eq!(answer["player"]["history"][0]["role"], "plays_games");
    // A first name only; a role that starts later; a hidden player.
    coordinator
        .ok(
            "POST",
            "/api/players",
            Some(json!({ "first_name": "casey", "last_name": null, "role": null })),
        )
        .await;
    coordinator
        .add_player("Blake", "only_trains", "2026-11-02")
        .await;
    let dana = coordinator
        .add_player("Dana", "only_trains", "2026-09-01")
        .await;
    coordinator
        .ok(
            "PUT",
            &format!("/api/players/{dana}"),
            Some(json!({ "first_name": "Dana", "last_name": null, "hidden": true })),
        )
        .await;

    let list = coordinator.ok("GET", "/api/players", None).await;
    assert_eq!(list["season"], "2026-2027");
    let rows: Vec<(&Value, &Value, &Value)> = list["players"]
        .as_array()
        .unwrap()
        .iter()
        .map(|player| {
            (
                &player["first_name"],
                &player["hidden"],
                &player["role_today"],
            )
        })
        .collect();
    assert_eq!(
        rows,
        [
            (&json!("Avery"), &json!(false), &json!("plays_games")),
            (&json!("Blake"), &json!(false), &Value::Null),
            (&json!("casey"), &json!(false), &Value::Null),
            (&json!("Dana"), &json!(true), &json!("only_trains")),
        ]
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
    let id = coordinator
        .add_player("Avery", "plays_games", "2026-09-01")
        .await;
    let answer = coordinator
        .send(
            "PUT",
            &format!("/api/players/{id}"),
            Some(json!({ "first_name": " ", "last_name": null, "hidden": true })),
        )
        .await;
    assert_eq!(answer, (400, json!({ "error": "first_name_required" })));
    // Nothing is saved, also not "Hidden".
    let player = coordinator
        .ok("GET", &format!("/api/players/{id}"), None)
        .await;
    assert_eq!(
        (&player["first_name"], &player["hidden"]),
        (&json!("Avery"), &json!(false))
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_history_has_all_seasons_newest_first(pool: PgPool) {
    let mut coordinator = Coordinator::start(pool).await;
    coordinator.harness.clock.set(utc(2026, 3, 1, 12, 0));
    coordinator.log_in_again().await;
    let id = coordinator
        .add_player("Avery", "only_trains", "2025-09-02")
        .await;

    coordinator.harness.clock.set(utc(2026, 11, 2, 12, 0));
    coordinator.log_in_again().await;
    coordinator
        .ok(
            "POST",
            &format!("/api/players/{id}/roles"),
            Some(json!({ "role": "only_trains", "from": "2026-09-01", "until": null })),
        )
        .await;
    coordinator
        .ok(
            "POST",
            &format!("/api/players/{id}/role-change"),
            Some(json!({ "role": "plays_games", "from": "2026-11-01" })),
        )
        .await;
    let player = coordinator
        .ok("GET", &format!("/api/players/{id}"), None)
        .await;
    assert_eq!(player["season"], "2026-2027");
    assert_eq!(player["season_first_day"], "2026-07-01");
    assert_eq!(player["season_last_day"], "2027-06-30");
    let text = |text: &str| text.to_owned();
    assert_eq!(
        history(&coordinator, id).await,
        [
            (
                text("2026-2027"),
                text("plays_games"),
                text("2026-11-01"),
                Value::Null,
                true
            ),
            (
                text("2026-2027"),
                text("only_trains"),
                text("2026-09-01"),
                json!("2026-10-31"),
                true
            ),
            (
                text("2025-2026"),
                text("only_trains"),
                text("2025-09-02"),
                Value::Null,
                false
            ),
        ]
    );
    let (status, _) = coordinator.send("GET", "/api/players/999999", None).await;
    assert_eq!(status, 404);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn correct_changes_a_role_of_this_season(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    let id = coordinator
        .add_player("Blake", "only_trains", "2026-09-01")
        .await;
    let role = newest_role(&coordinator, id).await;
    let player = coordinator
        .ok(
            "PUT",
            &format!("/api/players/{id}/roles/{role}"),
            Some(json!({ "role": "plays_games", "from": "2026-09-08", "until": "2027-03-31" })),
        )
        .await;
    assert_eq!(player["history"][0]["role"], "plays_games");
    assert_eq!(player["history"][0]["from"], "2026-09-08");
    assert_eq!(player["history"][0]["until"], "2027-03-31");

    // "Correct" back to a continuing role.
    coordinator
        .ok(
            "PUT",
            &format!("/api/players/{id}/roles/{role}"),
            Some(json!({ "role": "plays_games", "from": "2026-09-08", "until": null })),
        )
        .await;
    let team = coordinator.ok("GET", "/api/team", None).await;
    assert_eq!(names(&team["members"]), ["Blake"]);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_correction_that_overlaps_is_refused_and_nothing_is_stored(pool: PgPool) {
    let mut coordinator = Coordinator::start(pool).await;
    coordinator.harness.clock.set(utc(2026, 11, 2, 12, 0));
    coordinator.log_in_again().await;
    let id = coordinator
        .add_player("Avery", "plays_games", "2026-09-01")
        .await;
    coordinator
        .ok(
            "POST",
            &format!("/api/players/{id}/role-change"),
            Some(json!({ "role": "only_trains", "from": "2026-11-01" })),
        )
        .await;
    let before = history(&coordinator, id).await;
    // The first role (until Sat 31 Oct) corrected to end on Sun 15 Nov.
    let first = coordinator
        .ok("GET", &format!("/api/players/{id}"), None)
        .await["history"][1]["id"]
        .as_i64()
        .unwrap();
    let answer = coordinator
        .send(
            "PUT",
            &format!("/api/players/{id}/roles/{first}"),
            Some(json!({ "role": "plays_games", "from": "2026-09-01", "until": "2026-11-15" })),
        )
        .await;
    assert_eq!(answer, (400, json!({ "error": "roles_overlap" })));
    // A new role on top of the current one.
    let answer = coordinator
        .send(
            "POST",
            &format!("/api/players/{id}/roles"),
            Some(json!({ "role": "only_trains", "from": "2027-01-10", "until": null })),
        )
        .await;
    assert_eq!(answer, (400, json!({ "error": "roles_overlap" })));
    assert_eq!(history(&coordinator, id).await, before);
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
    assert_eq!(history(&coordinator, id).await.len(), 1);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn hide_rename_and_delete_a_player(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    let id = coordinator
        .add_player("Avery", "plays_games", "2026-09-01")
        .await;
    let answer = coordinator
        .ok(
            "PUT",
            &format!("/api/players/{id}"),
            Some(json!({ "first_name": "Avery", "last_name": "de Vries", "hidden": true })),
        )
        .await;
    assert_eq!(answer["player"]["last_name"], "de Vries");
    assert_eq!(answer["player"]["hidden"], true);
    assert_eq!(answer["same_name"], false);
    // Hidden, and still in the team.
    let list = coordinator.ok("GET", "/api/players", None).await;
    assert_eq!(list["players"][0]["hidden"], true);
    let team = coordinator.ok("GET", "/api/team", None).await;
    assert_eq!(names(&team["members"]), ["Avery"]);
    let (status, _) = coordinator
        .send(
            "PUT",
            "/api/players/999999",
            Some(json!({ "first_name": "Avery", "last_name": null, "hidden": false })),
        )
        .await;
    assert_eq!(status, 404);

    let (status, _) = coordinator
        .send("DELETE", &format!("/api/players/{id}"), None)
        .await;
    assert_eq!(status, 204);
    let list = coordinator.ok("GET", "/api/players", None).await;
    assert!(list["players"].as_array().unwrap().is_empty());
    assert!(
        players::roles_of(&pool, PlayerId(id))
            .await
            .unwrap()
            .is_empty()
    );
    let (status, _) = coordinator
        .send("DELETE", &format!("/api/players/{id}"), None)
        .await;
    assert_eq!(status, 404);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn roles_are_saved_only_in_the_current_season(pool: PgPool) {
    let mut coordinator = Coordinator::start(pool).await;
    coordinator.harness.clock.set(utc(2026, 3, 1, 12, 0));
    coordinator.log_in_again().await;
    let avery = coordinator
        .add_player("Avery", "plays_games", "2025-09-02")
        .await;
    let old_role = newest_role(&coordinator, avery).await;

    coordinator.harness.clock.set(utc(2026, 10, 12, 12, 0));
    coordinator.log_in_again().await;
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
    // "Correct" of a role of last season is refused, also into this season.
    for until in [json!("2026-03-31"), Value::Null] {
        let answer = coordinator
            .send(
                "PUT",
                &format!("/api/players/{avery}/roles/{old_role}"),
                Some(json!({ "role": "plays_games", "from": "2025-09-02", "until": until })),
            )
            .await;
        assert_eq!(answer, outside);
    }
    let answer = coordinator
        .send(
            "PUT",
            &format!("/api/players/{avery}/roles/{old_role}"),
            Some(json!({ "role": "plays_games", "from": "2026-09-01", "until": null })),
        )
        .await;
    assert_eq!(answer, outside);

    let history = history(&coordinator, avery).await;
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].2, "2025-09-02");
    let team = coordinator.ok("GET", "/api/team", None).await;
    assert_eq!(names(&team["last_season"]), ["Avery"]);
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
        ("GET", "/api/players/1"),
        ("PUT", "/api/players/1"),
        ("DELETE", "/api/players/1"),
        ("POST", "/api/players/1/roles"),
        ("PUT", "/api/players/1/roles/1"),
        ("POST", "/api/players/carry"),
        ("POST", "/api/players/1/role-change"),
        ("POST", "/api/players/1/team-leave"),
        ("POST", "/api/players/1/team-undo"),
        ("GET", "/api/team"),
        ("GET", "/api/attendance"),
        ("GET", "/api/trainings"),
        ("GET", "/api/trainings/next"),
        ("GET", "/api/trainings/2026-10-13"),
        ("POST", "/api/trainings"),
        ("PUT", "/api/trainings/2026-10-13"),
        ("DELETE", "/api/trainings/2026-10-13"),
        ("PUT", "/api/trainings/2026-10-13/open"),
        ("POST", "/api/trainings/2026-10-13/cancel"),
        ("POST", "/api/trainings/2026-10-13/uncancel"),
        ("PUT", "/api/trainings/2026-10-13/absences/1"),
        ("DELETE", "/api/trainings/2026-10-13/absences/1"),
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
