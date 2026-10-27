//! The reader of the game list source, against a fake source (wiremock) with real answers from
//! `tests/data/game-list/`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeSet;
use std::time::Duration;

use chrono::NaiveDate;
use serde_json::{Value, json};
use team::source::{Source, SourceError};
use team_core::{
    Club, ClubNumber, ScheduleReason, ScheduledGame, Season, StoredGame, TeamNumber, in_schedule,
    own_teams, schedule,
};
use url::Url;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const DRAKES: ClubNumber = ClubNumber(37);

fn data(name: &str) -> Value {
    let path = format!("{}/tests/data/game-list/{name}", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(&path).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn season(text: &str) -> Season {
    text.parse().unwrap()
}

fn origin() -> Url {
    Url::parse("https://team.example.test").unwrap()
}

fn drakes() -> Club {
    Club {
        number: DRAKES,
        name: "Blue Drakes".to_owned(),
        town: Some("Woerden".to_owned()),
    }
}

async fn source_answering(answer: ResponseTemplate) -> (MockServer, Source) {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(answer)
        .mount(&server)
        .await;
    let source = Source::new(Url::parse(&server.uri()).unwrap(), &origin()).unwrap();
    (server, source)
}

async fn source_with(body: Value) -> (MockServer, Source) {
    source_answering(ResponseTemplate::new(200).set_body_json(body)).await
}

/// This season's games with one field of the first game changed (or removed when `None`).
fn games_with_first(field: &str, value: Option<Value>) -> Value {
    let mut body = data("games-37-2026-2027.json");
    let game = body["wedstrijden"][0].as_object_mut().unwrap();
    match value {
        Some(value) => game.insert(field.to_owned(), value),
        None => game.remove(field),
    };
    body
}

async fn format_error_of(body: Value) -> (String, String) {
    let (_server, source) = source_with(body).await;
    match source.club_games(DRAKES, season("2026-2027")).await {
        Err(SourceError::Format { place, reason }) => (place, reason),
        other => panic!("expected a format error, got {other:?}"),
    }
}

#[tokio::test]
async fn reads_the_club_list() {
    let (_server, source) = source_with(data("clubs.json")).await;
    let clubs = source.clubs().await.unwrap();
    assert_eq!(clubs.len(), 316);
    assert!(clubs.contains(&drakes()));
}

#[tokio::test]
async fn a_club_with_a_missing_or_empty_town_has_no_town() {
    let (_server, source) = source_with(data("clubs.json")).await;
    let clubs = source.clubs().await.unwrap();
    let town_of = |number| {
        clubs
            .iter()
            .find(|club| club.number == ClubNumber(number))
            .map(|club| club.town.clone())
    };
    assert_eq!(town_of(1164), Some(None), "Day 1 has plaats null");
    assert_eq!(town_of(1160), Some(None), "Green Wolves has plaats \"\"");
    assert_eq!(clubs.iter().filter(|club| club.town.is_none()).count(), 29);
}

#[tokio::test]
async fn reads_this_season_of_blue_drakes() {
    let (_server, source) = source_with(data("games-37-2026-2027.json")).await;
    let games = source
        .club_games(DRAKES, season("2026-2027"))
        .await
        .unwrap();
    assert_eq!(games.len(), 71);

    let teams: Vec<String> = own_teams(&drakes(), &games)
        .into_iter()
        .map(|team| team.name)
        .collect();
    assert_eq!(
        teams,
        ["M16 1", "M16 2", "M18 1", "MSE 1", "X12 1", "X14 1"]
    );

    let mut not_known: Vec<NaiveDate> = games
        .iter()
        .filter(|game| game.start_time.is_none())
        .map(|game| game.date)
        .collect();
    not_known.sort();
    assert_eq!(
        not_known,
        [
            NaiveDate::from_ymd_opt(2027, 1, 16).unwrap(),
            NaiveDate::from_ymd_opt(2027, 2, 13).unwrap()
        ]
    );
    assert_eq!(
        games
            .iter()
            .filter(|game| game.place.court.is_none())
            .count(),
        2
    );
}

#[tokio::test]
async fn the_schedule_of_m16_1_has_its_games_and_the_club_home_games() {
    let (_server, source) = source_with(data("games-37-2026-2027.json")).await;
    let games = source
        .club_games(DRAKES, season("2026-2027"))
        .await
        .unwrap();
    let reasons: Vec<Option<ScheduleReason>> = games
        .iter()
        .map(|game| in_schedule(game, DRAKES, Some(TeamNumber(184))))
        .collect();
    let count = |reason| reasons.iter().filter(|r| **r == reason).count();
    assert_eq!(count(Some(ScheduleReason::TeamGame)), 10);
    assert_eq!(count(Some(ScheduleReason::DutyGame)), 27);
    assert_eq!(count(None), 34);
}

#[tokio::test]
async fn on_12_october_2026_the_schedule_of_m16_1_has_23_coming_and_14_past_games() {
    let (_server, source) = source_with(data("games-37-2026-2027.json")).await;
    let games = source
        .club_games(DRAKES, season("2026-2027"))
        .await
        .unwrap();
    let stored = games
        .into_iter()
        .map(|game| StoredGame {
            game,
            in_list: true,
        })
        .collect();
    let today = NaiveDate::from_ymd_opt(2026, 10, 12).unwrap();
    let schedule = schedule(
        stored,
        DRAKES,
        TeamNumber(184),
        today,
        &BTreeSet::new(),
        &BTreeSet::new(),
    );

    let team_games = |games: &[ScheduledGame]| {
        games
            .iter()
            .filter(|scheduled| scheduled.reason == ScheduleReason::TeamGame)
            .count()
    };
    assert_eq!(schedule.coming.len(), 23);
    assert_eq!(team_games(&schedule.coming), 5);
    assert_eq!(schedule.past.len(), 14);
    assert_eq!(team_games(&schedule.past), 5);

    let first = &schedule.coming[0].game;
    assert_eq!(first.date, NaiveDate::from_ymd_opt(2026, 10, 31).unwrap());
    assert_eq!(first.start_time.unwrap().to_string(), "13:00:00");
    assert_eq!(first.home.team_name, "Blue Drakes X12 1");
}

#[tokio::test]
async fn reads_one_game_with_all_its_fields() {
    let (_server, source) = source_with(data("games-37-2026-2027.json")).await;
    let games = source
        .club_games(DRAKES, season("2026-2027"))
        .await
        .unwrap();
    let game = games
        .iter()
        .find(|game| game.source_id == 1_073_597)
        .unwrap();
    assert_eq!(game.game_number, "499858");
    assert_eq!(game.season, season("2026-2027"));
    assert_eq!(game.date, NaiveDate::from_ymd_opt(2026, 9, 12).unwrap());
    assert_eq!(
        game.start_time.map(|time| time.to_string()),
        Some("11:15:00".to_owned())
    );
    assert_eq!(game.home.team_name, "Green Giants M16 2");
    assert_eq!(game.home.club, ClubNumber(21));
    assert_eq!(game.away.team_name, "Blue Drakes M16 2");
    assert_eq!(game.away.club, DRAKES);
    assert_eq!(game.place.number.0, 43);
    assert_eq!(game.place.hall, "Korf De");
    assert_eq!(game.place.town, "Leusden");
    assert_eq!(game.place.court.as_deref(), Some("Veld 3"));
    assert!(game.competition.starts_with("Mannen U16"));
}

#[tokio::test]
async fn reads_last_season_of_blue_drakes() {
    let (_server, source) = source_with(data("games-37-2025-2026.json")).await;
    let games = source
        .club_games(DRAKES, season("2025-2026"))
        .await
        .unwrap();
    assert_eq!(games.len(), 118);
    let teams: Vec<String> = own_teams(&drakes(), &games)
        .into_iter()
        .map(|team| team.name)
        .collect();
    assert!(teams.contains(&"VSE 1".to_owned()));
    assert!(!teams.contains(&"M16 2".to_owned()));
}

#[tokio::test]
async fn an_empty_season_gives_no_games_and_no_error() {
    let (_server, source) = source_with(data("games-empty.json")).await;
    let games = source
        .club_games(DRAKES, season("2030-2031"))
        .await
        .unwrap();
    assert!(games.is_empty());
}

#[tokio::test]
async fn sends_the_user_agent_season_and_club() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/db/json/wedstrijd.pl"))
        .and(query_param("seizoen", "2026-2027"))
        .and(query_param("clb_ID", "37"))
        .and(header(
            "user-agent",
            format!(
                "team-coordinator/{} (+https://team.example.test)",
                env!("CARGO_PKG_VERSION")
            )
            .as_str(),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(data("games-empty.json")))
        .expect(1)
        .mount(&server)
        .await;
    let source = Source::new(Url::parse(&server.uri()).unwrap(), &origin()).unwrap();
    let games = source.club_games(DRAKES, season("2026-2027")).await;
    assert!(games.is_ok(), "{games:?}");
}

#[tokio::test]
async fn reads_the_club_list_from_its_address() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/db/json/club.pl"))
        .respond_with(ResponseTemplate::new(200).set_body_json(data("clubs.json")))
        .expect(1)
        .mount(&server)
        .await;
    let source = Source::new(Url::parse(&server.uri()).unwrap(), &origin()).unwrap();
    assert!(source.clubs().await.is_ok());
}

#[tokio::test]
async fn a_missing_start_time_names_the_game_and_field() {
    let (place, reason) = format_error_of(games_with_first("tijd", None)).await;
    assert_eq!(place, "game 1073597, field tijd");
    assert_eq!(reason, "missing");
}

#[tokio::test]
async fn a_start_time_that_is_not_a_time_is_refused() {
    let (place, _) = format_error_of(games_with_first("tijd", Some(json!("abc")))).await;
    assert_eq!(place, "game 1073597, field tijd");
}

#[tokio::test]
async fn a_team_number_as_text_is_refused() {
    let (place, _) = format_error_of(games_with_first("thuis_ploeg_id", Some(json!("1688")))).await;
    assert_eq!(place, "game 1073597, field thuis_ploeg_id");
}

#[tokio::test]
async fn a_game_of_another_season_is_refused() {
    let (place, reason) =
        format_error_of(games_with_first("seizoen", Some(json!("2025-2026")))).await;
    assert_eq!(place, "game 1073597, field seizoen");
    assert_eq!(reason, "is 2025-2026, asked for 2026-2027");
}

#[tokio::test]
async fn a_date_that_is_not_a_date_is_refused() {
    let (place, _) = format_error_of(games_with_first("datum", Some(json!("soon")))).await;
    assert_eq!(place, "game 1073597, field datum");
}

#[tokio::test]
async fn a_game_without_an_id_is_named_by_its_position() {
    let (place, reason) = format_error_of(games_with_first("id", None)).await;
    assert_eq!(place, "game 1 in the list, field id");
    assert_eq!(reason, "missing");
}

#[tokio::test]
async fn games_that_are_not_a_list_are_refused() {
    let (place, _) = format_error_of(json!({ "wedstrijden": {} })).await;
    assert_eq!(place, "field wedstrijden");
}

#[tokio::test]
async fn an_answer_that_is_not_json_is_refused() {
    let (_server, source) =
        source_answering(ResponseTemplate::new(200).set_body_string("<html>busy</html>")).await;
    let result = source.club_games(DRAKES, season("2026-2027")).await;
    assert!(
        matches!(result, Err(SourceError::Format { ref place, .. }) if place == "answer"),
        "{result:?}"
    );
}

#[tokio::test]
async fn a_club_with_an_empty_name_is_refused() {
    let mut body = data("clubs.json");
    body["clubs"][0]["naam"] = json!("");
    let (_server, source) = source_with(body).await;
    match source.clubs().await {
        Err(SourceError::Format { place, reason }) => {
            assert_eq!(place, "club 1067, field naam");
            assert_eq!(reason, "is empty");
        }
        other => panic!("expected a format error, got {other:?}"),
    }
}

#[tokio::test]
async fn an_error_answer_gives_the_status() {
    let (_server, source) = source_answering(ResponseTemplate::new(500)).await;
    let result = source.club_games(DRAKES, season("2026-2027")).await;
    assert!(
        matches!(result, Err(SourceError::Status(status)) if status.as_u16() == 500),
        "{result:?}"
    );
}

#[tokio::test]
async fn a_slow_answer_cannot_be_reached() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(data("games-empty.json"))
                .set_delay(Duration::from_millis(500)),
        )
        .mount(&server)
        .await;
    let source = Source::with_timeout(
        Url::parse(&server.uri()).unwrap(),
        &origin(),
        Duration::from_millis(50),
    )
    .unwrap();
    let result = source.club_games(DRAKES, season("2026-2027")).await;
    assert!(
        matches!(result, Err(SourceError::Unreachable(_))),
        "{result:?}"
    );
}

#[tokio::test]
async fn no_server_cannot_be_reached() {
    // Port 9 (discard) on the local machine: nothing listens there in the tests.
    let source = Source::new(Url::parse("http://127.0.0.1:9").unwrap(), &origin()).unwrap();
    let result = source.clubs().await;
    assert!(
        matches!(result, Err(SourceError::Unreachable(_))),
        "{result:?}"
    );
}

/// Reads the real source. Run by hand: `cargo test -p team --test source -- --ignored`.
#[tokio::test]
#[ignore = "reads the live source"]
async fn the_live_source_still_has_the_expected_format() {
    let source = Source::new(Url::parse("https://db.basketball.nl").unwrap(), &origin()).unwrap();
    let clubs = source.clubs().await.unwrap();
    let drakes = clubs.iter().find(|club| club.number == DRAKES).unwrap();
    assert_eq!(drakes.name, "Blue Drakes");
    let games = source
        .club_games(DRAKES, season("2026-2027"))
        .await
        .unwrap();
    assert!(!own_teams(drakes, &games).is_empty());
}
