//! The page "Report" through the API (decision 0033): the same numbers as "Attendance" and
//! "Duties", the corrections after a game, what does not count yet, and the texts to copy.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::Harness;
use common::api::Coordinator;
use common::game_list::{M16_1, Reader};
use serde_json::{Value, json};
use sqlx::PgPool;
use team::players;
use team_core::PlayerId;

/// The away game of M16 1 on Saturday 10 October 2026 at 15:00, before the test clock.
const AWAY_10_OCT: i64 = 1_073_147;
/// Home games of the club with table places, on 3 October (past) and 31 October (coming).
const X12_3_OCT: i64 = 1_081_913;
const X12_31_OCT: i64 = 1_081_923;

async fn ok(reader: &Reader, method: &str, path: &str, body: Option<Value>) -> Value {
    let (status, answer) = reader.send(method, path, body).await;
    assert_eq!(status, 200, "{method} {path}: {answer}");
    answer
}

fn game_path(id: i64) -> String {
    format!("/api/games/2026-2027/{id}")
}

async fn add_player(reader: &Reader, first: &str, role: &str) -> i64 {
    let answer = ok(
        reader,
        "POST",
        "/api/players",
        Some(json!({ "first_name": first, "last_name": null,
                     "role": { "role": role, "from": "2026-09-01" } })),
    )
    .await;
    answer["player"]["id"].as_i64().unwrap()
}

async fn add_reason(reader: &Reader, name: &str, valid: bool) -> i64 {
    let answer = ok(
        reader,
        "POST",
        "/api/reasons",
        Some(json!({ "name": name, "valid": valid })),
    )
    .await;
    answer
        .as_array()
        .unwrap()
        .iter()
        .find(|reason| reason["name"] == name)
        .unwrap()["id"]
        .as_i64()
        .unwrap()
}

async fn add_tuesdays(reader: &Reader) {
    ok(
        reader,
        "POST",
        "/api/weekly-trainings",
        Some(json!({ "weekday": 2, "start": "19:45", "end": "21:00",
                     "from": "2026-09-01", "until": "2027-05-25" })),
    )
    .await;
}

async fn table(reader: &Reader, id: i64, number: u8, player: i64) {
    ok(
        reader,
        "PUT",
        &format!("{}/places/table/{number}", game_path(id)),
        Some(json!({ "team": M16_1 })),
    )
    .await;
    ok(
        reader,
        "PUT",
        &format!("{}/assignments/table/{number}", game_path(id)),
        Some(json!({ "player": player })),
    )
    .await;
}

async fn outcome(reader: &Reader, id: i64, number: u8, body: Value) {
    ok(
        reader,
        "PUT",
        &format!("{}/assignments/table/{number}/outcome", game_path(id)),
        Some(body),
    )
    .await;
}

fn row(rows: &Value, player: i64) -> &Value {
    rows.as_array()
        .unwrap()
        .iter()
        .find(|row| row["player"] == player || row["id"] == player)
        .unwrap_or_else(|| panic!("no row for {player} in {rows}"))
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_report_needs_a_login(pool: PgPool) {
    let harness = Harness::start(pool).await;
    assert_eq!(harness.get("/api/report", None).await.status(), 401);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_report_has_the_numbers_of_attendance_and_duties(pool: PgPool) {
    let reader = Reader::chosen(pool.clone()).await;
    reader.read_good().await;
    add_tuesdays(&reader).await;
    let anna = add_player(&reader, "Anna", "plays_games").await;
    let bram = add_player(&reader, "Bram", "plays_games").await;
    let cas = add_player(&reader, "Cas", "only_trains").await;
    let dora = add_player(&reader, "Dora", "plays_games").await;
    // Eva is hidden; Finn left the team on 1 October.
    let eva = add_player(&reader, "Eva", "plays_games").await;
    players::set_hidden(&pool, PlayerId(eva), true)
        .await
        .unwrap();
    let finn = add_player(&reader, "Finn", "plays_games").await;
    ok(
        &reader,
        "POST",
        &format!("/api/players/{finn}/team-leave"),
        Some(json!({ "from": "2026-10-01" })),
    )
    .await;
    let ill = add_reason(&reader, "Exam week", true).await;
    let late = add_reason(&reader, "Overslept", false).await;

    // Three trainings: Bram ill once, Cas late once.
    for date in ["2026-09-22", "2026-09-29", "2026-10-06"] {
        ok(&reader, "PUT", &format!("/api/trainings/{date}/open"), None).await;
    }
    let absence = |date: &str, player: i64| format!("/api/trainings/{date}/absences/{player}");
    ok(
        &reader,
        "PUT",
        &absence("2026-09-29", bram),
        Some(json!({ "reason": ill })),
    )
    .await;
    ok(
        &reader,
        "PUT",
        &absence("2026-10-06", cas),
        Some(json!({ "reason": late })),
    )
    .await;

    // A checked game: Anna there, Bram not coming without a reason.
    let answer = |player: i64| format!("{}/answers/{player}", game_path(AWAY_10_OCT));
    ok(
        &reader,
        "PUT",
        &answer(anna),
        Some(json!({ "answer": "coming", "reason": null })),
    )
    .await;
    ok(
        &reader,
        "PUT",
        &answer(bram),
        Some(json!({ "answer": "not_coming", "reason": ill })),
    )
    .await;
    ok(
        &reader,
        "PUT",
        &format!("{}/checked", game_path(AWAY_10_OCT)),
        None,
    )
    .await;

    // Duties: on 3 October Anna was planned on table 1 and Bram did it; Dora on table 2, nobody
    // did it. On 31 October Anna is planned.
    table(&reader, X12_3_OCT, 1, anna).await;
    table(&reader, X12_3_OCT, 2, dora).await;
    outcome(
        &reader,
        X12_3_OCT,
        1,
        json!({ "outcome": "done_by", "player": bram }),
    )
    .await;
    outcome(&reader, X12_3_OCT, 2, json!({ "outcome": "not_done" })).await;
    table(&reader, X12_31_OCT, 1, anna).await;

    let report = ok(&reader, "GET", "/api/report", None).await;
    let attendance = ok(&reader, "GET", "/api/attendance", None).await;
    let duties = ok(&reader, "GET", "/api/duties", None).await;
    assert_eq!(report["season"], "2026-2027");
    // Finn left: under "Left the team", with the trainings before 1 October.
    assert_eq!(row(&report["left"], finn)["trainings"]["counted"], 2);
    assert_eq!(row(&report["team"], eva)["hidden"], true);

    // The same numbers as "Attendance" for each player.
    for (group, player) in [
        ("team", anna),
        ("team", bram),
        ("team", cas),
        ("team", dora),
        ("team", eva),
        ("left", finn),
    ] {
        let ours = row(&report[group], player);
        let theirs = row(&attendance[group], player);
        let trainings = &ours["trainings"];
        assert_eq!(trainings["counted"], theirs["trainings"]);
        assert_eq!(trainings["there"], theirs["there"]);
        assert_eq!(trainings["absent_valid"], theirs["absent_valid"]);
        assert_eq!(trainings["absent_not_valid"], theirs["absent_not_valid"]);
        let games = &theirs["games"];
        if games["games"] == 0 {
            assert_eq!(ours["games"], Value::Null);
        } else {
            assert_eq!(ours["games"]["counted"], games["games"]);
            assert_eq!(ours["games"]["there"], games["there"]);
            assert_eq!(ours["games"]["absent_valid"], games["absent_valid"]);
            assert_eq!(ours["games"]["absent_not_valid"], games["absent_not_valid"]);
        }
    }
    // The same done and planned as "Duties".
    for (group, player) in [
        ("team", anna),
        ("team", bram),
        ("team", dora),
        ("team", eva),
        ("left", finn),
    ] {
        let ours = &row(&report[group], player)["duties"];
        let theirs = &row(&duties["rows"], player)["totals"];
        for kind in ["drive", "table", "referee"] {
            assert_eq!(ours[kind]["done"], theirs[kind]["done"], "{kind}");
            assert_eq!(ours[kind]["planned"], theirs[kind]["planned"], "{kind}");
        }
    }

    // The numbers themselves.
    let bram_row = row(&report["team"], bram);
    assert_eq!(
        bram_row["trainings"],
        json!({ "counted": 3, "there": 2, "absent_valid": 1, "absent_not_valid": 0,
                "percent": 67, "percent_valid": 33, "percent_not_valid": 0 })
    );
    assert_eq!(bram_row["games"]["percent"], 0);
    assert_eq!(bram_row["games"]["absent_valid"], 1);
    assert_eq!(
        bram_row["duties"]["table"],
        json!({ "done": 1, "planned": 0, "done_by_other": 0, "done_for_other": 1, "not_done": 0 })
    );
    assert_eq!(
        row(&report["team"], dora)["duties"]["table"],
        json!({ "done": 0, "planned": 0, "done_by_other": 0, "done_for_other": 0, "not_done": 1 })
    );
    assert_eq!(
        row(&report["team"], anna)["duties"]["table"],
        json!({ "done": 0, "planned": 1, "done_by_other": 1, "done_for_other": 0, "not_done": 0 })
    );
    // Cas only trains: no games, no duties.
    let cas_row = row(&report["team"], cas);
    assert_eq!(cas_row["games"], Value::Null);
    assert_eq!(cas_row["duties"], Value::Null);
    assert_eq!(cas_row["trainings"]["percent"], 67);

    // The texts to copy, in Dutch, with today (Monday 12 October).
    assert_eq!(
        report["copies"]["trainings"],
        "Trainingen Blue Drakes M16 1, seizoen 2026-2027 (tot en met 12 okt)\n\
         Anna 100%\nBram 67%\nCas 67%\nDora 100%\nEva 100%"
    );
    assert_eq!(
        report["copies"]["table"],
        "Tafel Blue Drakes M16 1, seizoen 2026-2027 (tot en met 12 okt)\n\
         Anna 0 (+1 gepland)\nBram 1\nDora 0\nEva 0"
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn what_does_not_count_yet_is_the_same_as_on_the_start_page(pool: PgPool) {
    let reader = Reader::chosen(pool.clone()).await;
    reader.read_good().await;
    add_tuesdays(&reader).await;
    ok(&reader, "PUT", "/api/trainings/2026-09-15/open", None).await;
    // Games count from this day, so the past away game is not checked.
    sqlx::query("UPDATE game_check_start SET from_date = '2026-10-01'")
        .execute(&pool)
        .await
        .unwrap();

    let report = ok(&reader, "GET", "/api/report", None).await;
    let start = ok(&reader, "GET", "/api/start", None).await;
    let item = |kind: &str| {
        start["open_items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["kind"] == kind)
            .cloned()
            .unwrap()
    };
    let incomplete = &report["incomplete"];
    let not_opened = item("not_opened");
    assert_eq!(incomplete["not_opened"], not_opened["count"]);
    assert_eq!(incomplete["first_not_opened"], not_opened["first"]);
    let not_checked = item("games_not_checked");
    assert_eq!(incomplete["not_checked"], not_checked["count"]);
    assert_eq!(
        incomplete["first_not_checked"],
        json!({ "season": not_checked["first"]["season"], "id": not_checked["first"]["id"] })
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn without_a_team_the_duties_are_unknown(pool: PgPool) {
    // No club and team chosen for the season.
    let coordinator = Coordinator::start(pool).await;
    coordinator
        .ok(
            "POST",
            "/api/weekly-trainings",
            Some(json!({ "weekday": 2, "start": "19:45", "end": "21:00",
                         "from": "2026-09-01", "until": "2027-05-25" })),
        )
        .await;
    let anna = coordinator
        .add_player("Anna", "plays_games", "2026-09-01")
        .await;
    coordinator
        .ok("PUT", "/api/trainings/2026-10-06/open", None)
        .await;

    let report = coordinator.ok("GET", "/api/report", None).await;
    let anna_row = row(&report["team"], anna);
    assert_eq!(anna_row["duties"], Value::Null);
    assert_eq!(anna_row["trainings"]["percent"], 100);
    assert_eq!(report["copies"]["drive"], Value::Null);
    assert_eq!(
        report["copies"]["trainings"],
        "Trainingen seizoen 2026-2027 (tot en met 12 okt)\nAnna 100%"
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn an_empty_season_has_no_rows(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    let report = ok(&reader, "GET", "/api/report", None).await;
    assert_eq!(report["team"], json!([]));
    assert_eq!(report["left"], json!([]));
    assert_eq!(
        report["copies"],
        json!({ "trainings": null, "games": null, "drive": null, "table": null, "referee": null })
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_not_played_game_counts_only_the_drives_with_the_switch(pool: PgPool) {
    let reader = Reader::chosen(pool).await;
    reader.read_good().await;
    let anna = add_player(&reader, "Anna", "plays_games").await;
    let bram = add_player(&reader, "Bram", "plays_games").await;
    // Anna was the driver; Bram drove instead.
    let game = game_path(AWAY_10_OCT);
    ok(
        &reader,
        "PUT",
        &format!("{game}/drivers"),
        Some(json!({ "drivers": 1 })),
    )
    .await;
    ok(
        &reader,
        "PUT",
        &format!("{game}/assignments/drive/1"),
        Some(json!({ "player": anna })),
    )
    .await;
    ok(
        &reader,
        "PUT",
        &format!("{game}/assignments/drive/1/outcome"),
        Some(json!({ "outcome": "done_by", "player": bram })),
    )
    .await;
    ok(&reader, "PUT", &format!("{game}/not-played"), None).await;

    let drive =
        |report: &Value, player: i64| row(&report["team"], player)["duties"]["drive"].clone();
    let none =
        json!({ "done": 0, "planned": 0, "done_by_other": 0, "done_for_other": 0, "not_done": 0 });
    let report = ok(&reader, "GET", "/api/report", None).await;
    assert_eq!(
        (drive(&report, anna), drive(&report, bram)),
        (none.clone(), none)
    );

    // "The drivers drove": the drive and its correction count, as on "Duties".
    ok(&reader, "PUT", &format!("{game}/drivers-drove"), None).await;
    let report = ok(&reader, "GET", "/api/report", None).await;
    assert_eq!(
        drive(&report, anna),
        json!({ "done": 0, "planned": 0, "done_by_other": 1, "done_for_other": 0, "not_done": 0 })
    );
    assert_eq!(
        drive(&report, bram),
        json!({ "done": 1, "planned": 0, "done_by_other": 0, "done_for_other": 1, "not_done": 0 })
    );
    let duties = ok(&reader, "GET", "/api/duties", None).await;
    for player in [anna, bram] {
        let theirs = &row(&duties["rows"], player)["totals"]["drive"];
        let ours = drive(&report, player);
        assert_eq!(
            (&ours["done"], &ours["planned"]),
            (&theirs["done"], &theirs["planned"])
        );
    }
}
