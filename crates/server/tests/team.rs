//! The team page through the API: `GET /api/team`, "Change role", "Remove from the team", "Add to
//! the team" and "Add all" (attendance at training, story A).

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::time::Duration;

use chrono::NaiveDate;
use common::api::Coordinator;
use common::game_list::utc;
use serde_json::{Value, json};
use sqlx::PgPool;
use team::{followed_teams, players};
use team_core::{Club, ClubNumber, FollowedTeam, PlayerId, Role, Season, Team, TeamNumber};

fn names(list: &Value) -> Vec<&str> {
    list.as_array()
        .unwrap()
        .iter()
        .map(|player| player["first_name"].as_str().unwrap())
        .collect()
}

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

/// The (from, until) of each role of `player` in storage.
async fn stored_roles(pool: &PgPool, player: i64) -> Vec<(NaiveDate, Option<NaiveDate>)> {
    players::roles_of(pool, PlayerId(player))
        .await
        .unwrap()
        .iter()
        .map(|role| (role.from, role.until))
        .collect()
}

/// Moves the clock to noon UTC on that day and logs in again.
async fn on(coordinator: &mut Coordinator, year: i32, month: u32, day: u32) {
    coordinator.harness.clock.set(utc(year, month, day, 12, 0));
    coordinator.log_in_again().await;
}

async fn add_tuesdays(coordinator: &Coordinator) {
    coordinator
        .ok(
            "POST",
            "/api/weekly-trainings",
            Some(json!({ "weekday": 2, "start": "19:45", "end": "21:00",
                         "from": "2026-09-01", "until": "2027-05-25" })),
        )
        .await;
}

fn change(role: &str, from: &str) -> Option<Value> {
    Some(json!({ "role": role, "from": from }))
}

/// Makes every `operation` ("INSERT", "UPDATE" or "DELETE") on `player_roles` fail from now on,
/// in this test's own database: a save that writes more than once must then store nothing.
async fn refuse_role_writes(pool: &PgPool, operation: &str) {
    sqlx::query(
        "CREATE FUNCTION refuse_role_write() RETURNS trigger LANGUAGE plpgsql AS \
         $$ BEGIN RAISE EXCEPTION 'refused by the test'; END $$",
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(&format!(
        "CREATE TRIGGER refuse_role_write BEFORE {operation} ON player_roles \
         FOR EACH ROW EXECUTE FUNCTION refuse_role_write()"
    ))
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_player_added_with_a_role_is_in_the_team(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    coordinator
        .add_player("Avery", "plays_games", "2026-09-01")
        .await;
    let page = coordinator.ok("GET", "/api/team", None).await;
    assert_eq!(page["season"], "2026-2027");
    assert_eq!(page["today"], "2026-10-12");
    assert_eq!(page["season_first_day"], "2026-07-01");
    assert_eq!(page["season_last_day"], "2027-06-30");
    assert_eq!(page["proposed_from"], "2026-09-01");
    assert_eq!(names(&page["members"]), ["Avery"]);
    assert_eq!(page["members"][0]["role"], "plays_games");
    assert_eq!(page["members"][0]["roles"][0]["from"], "2026-09-01");
    assert!(page["candidates"].as_array().unwrap().is_empty());
    assert!(page["left"].as_array().unwrap().is_empty());
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_team_name_is_the_chosen_team(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    let page = coordinator.ok("GET", "/api/team", None).await;
    assert_eq!(page["team_name"], Value::Null);

    let mut choice = FollowedTeam {
        season: Season::from_start_year(2026),
        club: Club {
            number: ClubNumber(1234),
            name: "Example Club".to_owned(),
            town: None,
        },
        team: None,
    };
    // The club is chosen, the team is not.
    followed_teams::save(&pool, &choice, utc(2026, 10, 1, 12, 0))
        .await
        .unwrap();
    let page = coordinator.ok("GET", "/api/team", None).await;
    assert_eq!(page["team_name"], Value::Null);

    choice.team = Some(Team {
        number: TeamNumber(7),
        name: "U14-1".to_owned(),
    });
    followed_teams::save(&pool, &choice, utc(2026, 10, 1, 12, 0))
        .await
        .unwrap();
    let page = coordinator.ok("GET", "/api/team", None).await;
    assert_eq!(page["team_name"], "U14-1");
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn change_role_ends_the_role_the_day_before(pool: PgPool) {
    let mut coordinator = Coordinator::start(pool).await;
    on(&mut coordinator, 2026, 11, 2).await;
    let id = coordinator
        .add_player("Avery", "plays_games", "2026-09-01")
        .await;
    let page = coordinator
        .ok(
            "POST",
            &format!("/api/players/{id}/role-change"),
            change("only_trains", "2026-11-01"),
        )
        .await;
    assert_eq!(names(&page["members"]), ["Avery"]);
    let avery = &page["members"][0];
    assert_eq!(avery["role"], "only_trains");
    let roles: Vec<(&Value, &Value, &Value)> = avery["roles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|role| (&role["role"], &role["from"], &role["until"]))
        .collect();
    assert_eq!(
        roles,
        [
            (
                &json!("plays_games"),
                &json!("2026-09-01"),
                &json!("2026-10-31")
            ),
            (&json!("only_trains"), &json!("2026-11-01"), &Value::Null),
        ]
    );
    assert!(page["left"].as_array().unwrap().is_empty());
    assert_eq!(page, coordinator.ok("GET", "/api/team", None).await);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_refused_change_of_role_changes_nothing(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    let id = coordinator
        .add_player("Avery", "plays_games", "2026-09-01")
        .await;
    // A later role from Sun 15 Nov, written past the checks: the new role from Sun 1 Nov would
    // overlap it.
    players::insert_role(
        &pool,
        PlayerId(id),
        Season::from_start_year(2026),
        Role::OnlyTrains,
        date(2026, 11, 15),
        None,
    )
    .await
    .unwrap();
    let before = stored_roles(&pool, id).await;
    let answer = coordinator
        .send(
            "POST",
            &format!("/api/players/{id}/role-change"),
            change("only_trains", "2026-11-01"),
        )
        .await;
    assert_eq!(answer, (400, json!({ "error": "roles_overlap" })));
    assert_eq!(stored_roles(&pool, id).await, before);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn change_role_errors_have_their_codes(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    let id = coordinator
        .add_player("Avery", "plays_games", "2026-09-01")
        .await;
    let path = format!("/api/players/{id}/role-change");
    let answer = coordinator
        .send("POST", &path, change("only_trains", "2026-09-01"))
        .await;
    assert_eq!(
        answer,
        (400, json!({ "error": "change_before_role_start" }))
    );
    // Before the role: it starts later.
    let answer = coordinator
        .send("POST", &path, change("only_trains", "2026-08-15"))
        .await;
    assert_eq!(
        answer,
        (
            400,
            json!({ "error": "role_starts_later", "date": "2026-09-01" })
        )
    );
    // A player with no role in the season.
    let casey = coordinator
        .ok(
            "POST",
            "/api/players",
            Some(json!({ "first_name": "Casey", "last_name": null, "role": null })),
        )
        .await["player"]["id"]
        .as_i64()
        .unwrap();
    let answer = coordinator
        .send(
            "POST",
            &format!("/api/players/{casey}/role-change"),
            change("only_trains", "2026-10-12"),
        )
        .await;
    assert_eq!(answer, (400, json!({ "error": "not_in_team" })));
    let answer = coordinator
        .send("POST", &path, change("plays_games", "2026-10-12"))
        .await;
    assert_eq!(answer, (400, json!({ "error": "same_role" })));
    // A role that starts later: the answer names its start.
    let later = coordinator
        .add_player("Blake", "only_trains", "2026-11-02")
        .await;
    let answer = coordinator
        .send(
            "POST",
            &format!("/api/players/{later}/role-change"),
            change("plays_games", "2026-10-12"),
        )
        .await;
    assert_eq!(
        answer,
        (
            400,
            json!({ "error": "role_starts_later", "date": "2026-11-02" })
        )
    );
    let (status, _) = coordinator
        .send(
            "POST",
            "/api/players/999999/role-change",
            change("only_trains", "2026-11-01"),
        )
        .await;
    assert_eq!(status, 404);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn change_role_saves_both_or_neither(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    let id = coordinator
        .add_player("Avery", "plays_games", "2026-09-01")
        .await;
    // The end of the current role is written, then the insert of the new role fails.
    refuse_role_writes(&pool, "INSERT").await;
    let (status, _) = coordinator
        .send(
            "POST",
            &format!("/api/players/{id}/role-change"),
            change("only_trains", "2026-10-01"),
        )
        .await;
    assert_eq!(status, 500);
    assert_eq!(stored_roles(&pool, id).await, [(date(2026, 9, 1), None)]);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn change_role_waits_for_the_players_lock(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    let id = coordinator
        .add_player("Avery", "plays_games", "2026-09-01")
        .await;

    // Another save holds the player's lock and has ended the role, but not committed yet.
    let mut first = pool.begin().await.unwrap();
    assert!(players::lock(&mut first, PlayerId(id)).await.unwrap());
    let role = players::roles_of(&mut *first, PlayerId(id)).await.unwrap()[0].id;
    players::update_role(
        &mut *first,
        PlayerId(id),
        role,
        Season::from_start_year(2026),
        Role::PlaysGames,
        date(2026, 9, 1),
        Some(date(2026, 10, 15)),
    )
    .await
    .unwrap();

    // "Change role" on Sun 1 Nov waits, then sees the ended role: no role on that date.
    let path = format!("/api/players/{id}/role-change");
    let second = coordinator.send("POST", &path, change("only_trains", "2026-11-01"));
    tokio::pin!(second);
    let waited = tokio::time::timeout(Duration::from_millis(300), &mut second).await;
    assert!(waited.is_err(), "the change did not wait for the lock");
    first.commit().await.unwrap();
    assert_eq!(second.await, (400, json!({ "error": "not_in_team" })));
    assert_eq!(
        stored_roles(&pool, id).await,
        [(date(2026, 9, 1), Some(date(2026, 10, 15)))]
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn remove_from_the_team_and_add_again(pool: PgPool) {
    let mut coordinator = Coordinator::start(pool).await;
    let id = coordinator
        .add_player("Blake", "only_trains", "2026-09-01")
        .await;
    let page = coordinator
        .ok(
            "POST",
            &format!("/api/players/{id}/team-leave"),
            Some(json!({ "from": "2026-10-16" })),
        )
        .await;
    // Not in the team from Fri 16 Oct, so still in the team on Mon 12 Oct.
    assert_eq!(names(&page["team"]["members"]), ["Blake"]);
    assert_eq!(
        page["team"]["members"][0]["roles"][0]["until"],
        "2026-10-15"
    );

    on(&mut coordinator, 2026, 10, 20).await;
    let page = coordinator.ok("GET", "/api/team", None).await;
    assert_eq!(names(&page["left"]), ["Blake"]);
    assert!(page["members"].as_array().unwrap().is_empty());
    assert!(page["candidates"].as_array().unwrap().is_empty());

    // "Add to the team": a new role.
    let page = coordinator
        .ok(
            "POST",
            &format!("/api/players/{id}/roles"),
            Some(json!({ "role": "plays_games", "from": "2026-11-01", "until": null })),
        )
        .await;
    assert_eq!(names(&page["members"]), ["Blake"]);
    assert_eq!(page["members"][0]["role"], "plays_games");
    assert_eq!(page["members"][0]["roles"].as_array().unwrap().len(), 2);
    assert!(page["left"].as_array().unwrap().is_empty());
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn remove_from_the_team_needs_a_role_on_that_date(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    let id = coordinator
        .add_player("Blake", "only_trains", "2026-09-01")
        .await;
    let path = format!("/api/players/{id}/team-leave");
    let answer = coordinator
        .send("POST", &path, Some(json!({ "from": "2027-07-01" })))
        .await;
    assert_eq!(answer, (400, json!({ "error": "outside_season" })));
    // A role that already ends on the day before: nothing to end or remove.
    coordinator
        .ok("POST", &path, Some(json!({ "from": "2026-10-12" })))
        .await;
    let answer = coordinator
        .send("POST", &path, Some(json!({ "from": "2026-10-12" })))
        .await;
    assert_eq!(answer, (400, json!({ "error": "not_in_team" })));
    let casey = coordinator
        .ok(
            "POST",
            "/api/players",
            Some(json!({ "first_name": "Casey", "last_name": null, "role": null })),
        )
        .await["player"]["id"]
        .as_i64()
        .unwrap();
    let answer = coordinator
        .send(
            "POST",
            &format!("/api/players/{casey}/team-leave"),
            Some(json!({ "from": "2026-10-12" })),
        )
        .await;
    assert_eq!(answer, (400, json!({ "error": "not_in_team" })));
    let (status, _) = coordinator
        .send(
            "POST",
            "/api/players/999999/team-leave",
            Some(json!({ "from": "2026-10-15" })),
        )
        .await;
    assert_eq!(status, 404);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_hidden_player_in_the_team_stays_in_the_team(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    let avery = coordinator
        .add_player("Avery", "plays_games", "2026-09-01")
        .await;
    coordinator
        .ok(
            "POST",
            "/api/players",
            Some(json!({ "first_name": "Casey", "last_name": null, "role": null })),
        )
        .await;
    players::set_hidden(&pool, PlayerId(avery), true)
        .await
        .unwrap();
    let page = coordinator.ok("GET", "/api/team", None).await;
    assert_eq!(names(&page["members"]), ["Avery"]);
    assert_eq!(names(&page["candidates"]), ["Casey"]);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn add_all_brings_last_seasons_team_back(pool: PgPool) {
    let mut coordinator = Coordinator::start(pool.clone()).await;
    // In the season 2025-2026: three players stay until the end, one leaves in March.
    on(&mut coordinator, 2026, 3, 1).await;
    let avery = coordinator
        .add_player("Avery", "plays_games", "2025-09-02")
        .await;
    let blake = coordinator
        .add_player("Blake", "only_trains", "2025-09-02")
        .await;
    let casey = coordinator
        .add_player("Casey", "plays_games", "2025-09-02")
        .await;
    let dana = coordinator
        .add_player("Dana", "plays_games", "2025-09-02")
        .await;
    coordinator
        .ok(
            "POST",
            &format!("/api/players/{casey}/team-leave"),
            Some(json!({ "from": "2026-03-11" })),
        )
        .await;
    // Dana left the club.
    players::set_hidden(&pool, PlayerId(dana), true)
        .await
        .unwrap();

    // Summer: the new season 2026-2027, its first training on Tue 1 September.
    on(&mut coordinator, 2026, 8, 15).await;
    add_tuesdays(&coordinator).await;
    let page = coordinator.ok("GET", "/api/team", None).await;
    assert_eq!(page["season"], "2026-2027");
    assert_eq!(page["proposed_from"], "2026-09-01");
    assert_eq!(names(&page["last_season"]), ["Avery", "Blake"]);
    assert_eq!(page["last_season"][1]["role"], "only_trains");
    assert_eq!(names(&page["candidates"]), ["Avery", "Blake", "Casey"]);
    assert!(page["members"].as_array().unwrap().is_empty());

    let page = coordinator
        .ok(
            "POST",
            "/api/players/carry",
            Some(json!({ "players": [avery, blake, casey, dana] })),
        )
        .await;
    assert_eq!(names(&page["members"]), ["Avery", "Blake"]);
    assert_eq!(page["members"][1]["role"], "only_trains");
    assert_eq!(page["members"][0]["roles"][0]["from"], "2026-09-01");
    assert!(page["last_season"].as_array().unwrap().is_empty());
    assert_eq!(names(&page["candidates"]), ["Casey"]);

    // Again: nothing more.
    let page = coordinator
        .ok(
            "POST",
            "/api/players/carry",
            Some(json!({ "players": [avery, blake] })),
        )
        .await;
    assert_eq!(page["members"][0]["roles"].as_array().unwrap().len(), 1);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn after_add_all_in_summer_the_players_stay_in_the_team_and_can_leave(pool: PgPool) {
    let mut coordinator = Coordinator::start(pool).await;
    on(&mut coordinator, 2026, 3, 1).await;
    let avery = coordinator
        .add_player("Avery", "plays_games", "2025-09-02")
        .await;

    on(&mut coordinator, 2026, 8, 15).await;
    add_tuesdays(&coordinator).await;
    coordinator
        .ok(
            "POST",
            "/api/players/carry",
            Some(json!({ "players": [avery] })),
        )
        .await;

    // After the first training date the open role of last season has ended.
    on(&mut coordinator, 2026, 9, 2).await;
    let page = coordinator.ok("GET", "/api/team", None).await;
    assert_eq!(names(&page["members"]), ["Avery"]);
    assert!(page["left"].as_array().unwrap().is_empty());

    coordinator
        .ok(
            "POST",
            &format!("/api/players/{avery}/team-leave"),
            Some(json!({ "from": "2026-10-01" })),
        )
        .await;
    on(&mut coordinator, 2026, 10, 2).await;
    let page = coordinator.ok("GET", "/api/team", None).await;
    assert_eq!(names(&page["left"]), ["Avery"]);
    assert_eq!(page["left"][0]["roles"][0]["until"], "2026-09-30");
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn last_seasons_players_join_during_the_season(pool: PgPool) {
    let mut coordinator = Coordinator::start(pool).await;
    on(&mut coordinator, 2026, 3, 1).await;
    let avery = coordinator
        .add_player("Avery", "plays_games", "2025-09-02")
        .await;
    let blake = coordinator
        .add_player("Blake", "only_trains", "2025-09-02")
        .await;

    on(&mut coordinator, 2026, 10, 12).await;
    // "Add all" for Avery: from 1 September (step 8 gives "Add all" a date).
    let page = coordinator
        .ok(
            "POST",
            "/api/players/carry",
            Some(json!({ "players": [avery] })),
        )
        .await;
    assert_eq!(names(&page["members"]), ["Avery"]);
    assert_eq!(page["members"][0]["roles"][0]["from"], "2026-09-01");
    assert!(page["left"].as_array().unwrap().is_empty());
    // One of last season's team has a role: "Players of last season" goes; Blake is still the
    // first choice of "Add player".
    assert!(page["last_season"].as_array().unwrap().is_empty());
    assert_eq!(names(&page["candidates"]), ["Blake"]);

    // "Add player" from the list for Blake, with another role; the answer is the team page.
    let page = coordinator
        .ok(
            "POST",
            &format!("/api/players/{blake}/roles"),
            Some(json!({ "role": "plays_games", "from": "2026-10-12", "until": null })),
        )
        .await;
    assert_eq!(names(&page["members"]), ["Avery", "Blake"]);
    assert_eq!(page["members"][1]["role"], "plays_games");
    assert_eq!(page["members"][1]["roles"].as_array().unwrap().len(), 1);
    assert!(page["candidates"].as_array().unwrap().is_empty());
    assert_eq!(page, coordinator.ok("GET", "/api/team", None).await);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_proposed_start_is_the_first_of_september(pool: PgPool) {
    let mut coordinator = Coordinator::start(pool).await;
    // No weekly training is needed.
    let page = coordinator.ok("GET", "/api/team", None).await;
    assert_eq!(page["proposed_from"], "2026-09-01");
    on(&mut coordinator, 2026, 8, 15).await;
    let page = coordinator.ok("GET", "/api/team", None).await;
    assert_eq!(page["proposed_from"], "2026-09-01");
    on(&mut coordinator, 2027, 1, 5).await;
    let page = coordinator.ok("GET", "/api/team", None).await;
    assert_eq!(page["proposed_from"], "2026-09-01");
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn last_seasons_team_stays_until_one_of_them_has_a_role(pool: PgPool) {
    let mut coordinator = Coordinator::start(pool).await;
    on(&mut coordinator, 2026, 3, 1).await;
    let avery = coordinator
        .add_player("Avery", "plays_games", "2025-09-02")
        .await;
    coordinator
        .add_player("Blake", "only_trains", "2025-09-02")
        .await;

    on(&mut coordinator, 2026, 10, 12).await;
    // A newcomer does not make the group go.
    let page = coordinator
        .ok(
            "POST",
            "/api/players",
            Some(json!({ "first_name": "Casey", "last_name": null,
                         "role": { "role": "plays_games", "from": "2026-09-01" } })),
        )
        .await;
    assert_eq!(page["player"]["first_name"], "Casey");
    let page = coordinator.ok("GET", "/api/team", None).await;
    assert_eq!(names(&page["last_season"]), ["Avery", "Blake"]);

    let page = coordinator
        .ok(
            "POST",
            &format!("/api/players/{avery}/roles"),
            Some(json!({ "role": "plays_games", "from": "2026-09-01", "until": null })),
        )
        .await;
    assert!(page["last_season"].as_array().unwrap().is_empty());
    assert_eq!(names(&page["candidates"]), ["Blake"]);
}

fn leave(from: &str, removes: &[i64]) -> Option<Value> {
    Some(json!({ "from": from, "removes": removes }))
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn remove_from_today_puts_the_player_under_left_at_once(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    let id = coordinator
        .add_player("Avery", "only_trains", "2026-09-01")
        .await;
    let answer = coordinator
        .ok(
            "POST",
            &format!("/api/players/{id}/team-leave"),
            leave("2026-10-12", &[]),
        )
        .await;
    assert_eq!(
        stored_roles(&pool, id).await,
        [(date(2026, 9, 1), Some(date(2026, 10, 11)))]
    );
    assert!(answer["team"]["members"].as_array().unwrap().is_empty());
    assert_eq!(names(&answer["team"]["left"]), ["Avery"]);
    // What "Undo" sends back: the roles of the season from before.
    assert_eq!(
        answer["undo"],
        json!({ "from": "2026-10-12",
                "roles": [{ "role": "only_trains", "from": "2026-09-01", "until": null }] })
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn remove_with_a_later_role_asks_first(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    let id = coordinator
        .add_player("Avery", "only_trains", "2026-09-01")
        .await;
    // A planned "Change role" from Sun 1 Nov.
    coordinator
        .ok(
            "POST",
            &format!("/api/players/{id}/role-change"),
            change("plays_games", "2026-11-01"),
        )
        .await;
    let later = players::roles_of(&pool, PlayerId(id)).await.unwrap()[1]
        .id
        .0;
    let before = stored_roles(&pool, id).await;
    let path = format!("/api/players/{id}/team-leave");
    let answer = coordinator
        .send("POST", &path, leave("2026-10-12", &[]))
        .await;
    assert_eq!(
        answer,
        (
            409,
            json!({ "error": "confirm_removal",
                    "roles": [{ "id": later, "role": "plays_games",
                                "from": "2026-11-01", "until": null }] })
        )
    );
    assert_eq!(stored_roles(&pool, id).await, before);

    // Confirmed: the role ends on Sun 11 Oct, the later role goes.
    let answer = coordinator
        .ok("POST", &path, leave("2026-10-12", &[later]))
        .await;
    assert_eq!(
        stored_roles(&pool, id).await,
        [(date(2026, 9, 1), Some(date(2026, 10, 11)))]
    );
    assert_eq!(names(&answer["team"]["left"]), ["Avery"]);
    assert_eq!(answer["undo"]["roles"].as_array().unwrap().len(), 2);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn remove_after_add_all_leaves_nothing_in_the_season(pool: PgPool) {
    let mut coordinator = Coordinator::start(pool.clone()).await;
    on(&mut coordinator, 2026, 3, 1).await;
    let avery = coordinator
        .add_player("Avery", "plays_games", "2025-09-02")
        .await;
    on(&mut coordinator, 2026, 8, 15).await;
    coordinator
        .ok(
            "POST",
            "/api/players/carry",
            Some(json!({ "players": [avery] })),
        )
        .await;
    let role = players::roles_of(&pool, PlayerId(avery)).await.unwrap()[1]
        .id
        .0;
    // Avery did not come back: "Not in the team from" today.
    let answer = coordinator
        .ok(
            "POST",
            &format!("/api/players/{avery}/team-leave"),
            leave("2026-08-15", &[role]),
        )
        .await;
    assert_eq!(stored_roles(&pool, avery).await, [(date(2025, 9, 2), None)]);
    let team = &answer["team"];
    assert!(team["members"].as_array().unwrap().is_empty());
    assert!(team["left"].as_array().unwrap().is_empty());
    assert_eq!(names(&team["candidates"]), ["Avery"]);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn the_last_day_today_is_shown(pool: PgPool) {
    let coordinator = Coordinator::start(pool).await;
    let id = coordinator
        .add_player("Avery", "plays_games", "2026-09-01")
        .await;
    let page = coordinator.ok("GET", "/api/team", None).await;
    assert_eq!(page["members"][0]["last_day_today"], false);
    // "Tomorrow": not in the team from Tue 13 Oct, so today is the last day.
    let answer = coordinator
        .ok(
            "POST",
            &format!("/api/players/{id}/team-leave"),
            leave("2026-10-13", &[]),
        )
        .await;
    assert_eq!(names(&answer["team"]["members"]), ["Avery"]);
    assert_eq!(answer["team"]["members"][0]["last_day_today"], true);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn remove_from_the_team_waits_for_the_players_lock(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    let id = coordinator
        .add_player("Avery", "plays_games", "2026-09-01")
        .await;

    // Another save holds the player's lock and has ended the role, but not committed yet.
    let mut first = pool.begin().await.unwrap();
    assert!(players::lock(&mut first, PlayerId(id)).await.unwrap());
    let role = players::roles_of(&mut *first, PlayerId(id)).await.unwrap()[0].id;
    players::update_role(
        &mut *first,
        PlayerId(id),
        role,
        Season::from_start_year(2026),
        Role::PlaysGames,
        date(2026, 9, 1),
        Some(date(2026, 10, 15)),
    )
    .await
    .unwrap();

    // The removal from Sun 1 Nov waits, then sees the ended role: nothing to end or remove.
    let path = format!("/api/players/{id}/team-leave");
    let second = coordinator.send("POST", &path, leave("2026-11-01", &[]));
    tokio::pin!(second);
    let waited = tokio::time::timeout(Duration::from_millis(300), &mut second).await;
    assert!(waited.is_err(), "the removal did not wait for the lock");
    first.commit().await.unwrap();
    assert_eq!(second.await, (400, json!({ "error": "not_in_team" })));
    assert_eq!(
        stored_roles(&pool, id).await,
        [(date(2026, 9, 1), Some(date(2026, 10, 15)))]
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn remove_saves_all_or_nothing(pool: PgPool) {
    let coordinator = Coordinator::start(pool.clone()).await;
    let id = coordinator
        .add_player("Avery", "only_trains", "2026-09-01")
        .await;
    coordinator
        .ok(
            "POST",
            &format!("/api/players/{id}/role-change"),
            change("plays_games", "2026-11-01"),
        )
        .await;
    let later = players::roles_of(&pool, PlayerId(id)).await.unwrap()[1]
        .id
        .0;
    let before = stored_roles(&pool, id).await;
    // The end of the current role is written, then the delete of the later role fails.
    refuse_role_writes(&pool, "DELETE").await;
    let (status, _) = coordinator
        .send(
            "POST",
            &format!("/api/players/{id}/team-leave"),
            leave("2026-10-12", &[later]),
        )
        .await;
    assert_eq!(status, 500);
    assert_eq!(stored_roles(&pool, id).await, before);
}
