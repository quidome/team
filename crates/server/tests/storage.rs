//! Storage of games, places and readings (story 3a), and the season as its start year.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use serde_json::Value;
use sqlx::PgPool;
use team::games;
use team::readings::{self, FailureKind, ReadingResult};
use team::source::Source;
use team_core::{
    Club, ClubNumber, Counts, FollowedTeam, Game, HoldReason, Season, StoredGame, Team, TeamNumber,
};
use url::Url;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

const DRAKES: ClubNumber = ClubNumber(37);

fn season(text: &str) -> Season {
    text.parse().unwrap()
}

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 12, 8, 0, 0).unwrap()
}

/// The real 2026-2027 games of Blue Drakes, through the strict reader.
async fn real_games() -> Vec<Game> {
    let path = format!(
        "{}/tests/data/game-list/games-37-2026-2027.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let body: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;
    let origin = Url::parse("https://team.example.test").unwrap();
    let source = Source::new(Url::parse(&server.uri()).unwrap(), &origin).unwrap();
    source
        .club_games(DRAKES, season("2026-2027"))
        .await
        .unwrap()
}

async fn write(pool: &PgPool, read: &[Game], gone: &[i64]) {
    let mut tx = pool.begin().await.unwrap();
    games::write(&mut tx, DRAKES, season("2026-2027"), read, gone, now())
        .await
        .unwrap();
    tx.commit().await.unwrap();
}

async fn stored(pool: &PgPool, season_text: &str) -> Vec<StoredGame> {
    let mut connection = pool.acquire().await.unwrap();
    games::stored(&mut connection, DRAKES, season(season_text))
        .await
        .unwrap()
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn followed_teams_has_the_season_as_its_start_year(pool: PgPool) {
    let choice = FollowedTeam {
        season: season("2026-2027"),
        club: Club {
            number: DRAKES,
            name: "Blue Drakes".to_owned(),
            town: Some("Woerden".to_owned()),
        },
        team: Some(Team {
            number: TeamNumber(184),
            name: "M16 1".to_owned(),
        }),
    };
    team::followed_teams::save(&pool, &choice, now())
        .await
        .unwrap();

    let (text, year): (String, i32) =
        sqlx::query_as("SELECT season, season_year FROM followed_teams")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((text.as_str(), year), ("2026-2027", 2026));

    let read = team::followed_teams::for_season(&pool, season("2026-2027"))
        .await
        .unwrap();
    assert_eq!(read, Some(choice));
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn written_games_read_back_equal(pool: PgPool) {
    let read = real_games().await;
    write(&pool, &read, &[]).await;

    let back = stored(&pool, "2026-2027").await;
    assert_eq!(back.len(), 71);
    assert!(back.iter().all(|game| game.in_list));
    let mut expected = read.clone();
    expected.sort_by_key(|game| game.source_id);
    let games: Vec<Game> = back.into_iter().map(|stored| stored.game).collect();
    assert_eq!(games, expected);

    let places: i64 = sqlx::query_scalar("SELECT count(*) FROM places")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(places, 27);

    let end_of_summer_time = NaiveDate::from_ymd_opt(2026, 10, 25).unwrap();
    assert!(games.iter().any(|game| game.date == end_of_summer_time
        && game.start_time == NaiveTime::from_hms_opt(9, 30, 0)));
    let not_known: Option<NaiveTime> = sqlx::query_scalar(
        "SELECT start_time FROM games WHERE date = '2027-01-16' AND start_time IS NULL LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(not_known, None);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn a_second_write_marks_the_gone_games_and_keeps_the_first_reading_time(pool: PgPool) {
    let read = real_games().await;
    write(&pool, &read, &[]).await;
    let gone = read[0].source_id;

    let mut tx = pool.begin().await.unwrap();
    let later = now() + chrono::TimeDelta::days(1);
    games::write(
        &mut tx,
        DRAKES,
        season("2026-2027"),
        &read[1..],
        &[gone],
        later,
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let back = stored(&pool, "2026-2027").await;
    assert_eq!(back.len(), 71);
    let not_in_list: Vec<i64> = back
        .iter()
        .filter(|game| !game.in_list)
        .map(|game| game.game.source_id)
        .collect();
    assert_eq!(not_in_list, [gone]);
    let first_read: Vec<DateTime<Utc>> =
        sqlx::query_scalar("SELECT DISTINCT first_read_at FROM games")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(first_read, [now()]);
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn games_of_another_season_are_not_changed(pool: PgPool) {
    let read = real_games().await;
    let last_season = &read[0];
    sqlx::query("INSERT INTO places (number, hall, town) VALUES ($1, 'Old hall', 'Old town')")
        .bind(last_season.place.number.0)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO games
             (club_number, season_year, source_id, game_number, date, start_time,
              home_team_number, home_team_name, home_club_number,
              away_team_number, away_team_name, away_club_number,
              place_number, court, competition, in_list, first_read_at)
         VALUES (37, 2025, $1, 'OLD', '2026-03-01', '11:00', 1, 'Old home', 37, 2, 'Old away', 99,
                 $2, NULL, 'Old competition', true, $3)",
    )
    .bind(last_season.source_id)
    .bind(last_season.place.number.0)
    .bind(now())
    .execute(&pool)
    .await
    .unwrap();

    // The same source id in 2026-2027, and that game not in the list (gone).
    write(&pool, &read, &[]).await;
    write(&pool, &read[1..], &[last_season.source_id]).await;

    let old = stored(&pool, "2025-2026").await;
    assert_eq!(old.len(), 1);
    assert!(old[0].in_list);
    assert_eq!(old[0].game.game_number, "OLD");
    assert_eq!(old[0].game.season, season("2025-2026"));
    assert_eq!(
        old[0].game.date,
        NaiveDate::from_ymd_opt(2026, 3, 1).unwrap()
    );
}

#[sqlx::test(migrator = "team::db::MIGRATOR")]
async fn readings_read_back_equal(pool: PgPool) {
    let this = season("2026-2027");
    let counts = Counts {
        new: 3,
        moved: 1,
        gone: 1,
        ..Counts::default()
    };
    let results = [
        ReadingResult::Good {
            games_read: 71,
            first_reading: true,
            counts: None,
        },
        ReadingResult::Good {
            games_read: 72,
            first_reading: false,
            counts: Some(counts),
        },
        ReadingResult::HeldBack(HoldReason::Empty { before: 72 }),
        ReadingResult::HeldBack(HoldReason::FarFewer {
            before: 72,
            now: 30,
        }),
        ReadingResult::Failed(FailureKind::SourceUnavailable),
        ReadingResult::Failed(FailureKind::SourceFormat),
    ];
    let team = Some(TeamNumber(184));
    let mut ids = Vec::new();
    for (minutes, result) in (0..).zip(&results) {
        let at = now() + chrono::TimeDelta::minutes(minutes);
        ids.push(
            readings::insert(&pool, DRAKES, this, team, at, at, result)
                .await
                .unwrap(),
        );
    }
    for (id, result) in ids.iter().zip(&results) {
        let reading = readings::by_id(&pool, *id).await.unwrap().unwrap();
        assert_eq!(&reading.result, result);
        assert_eq!(
            (reading.club, reading.season, reading.team),
            (DRAKES, this, team)
        );
    }

    let latest = readings::latest(&pool, DRAKES, this)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(latest.id, ids[5]);
    let last_good = readings::last_good(&pool, DRAKES, this)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(last_good.id, ids[1]);
    // The newest that did not fail is the far-fewer hold.
    let not_failed = readings::newest_not_failed(&pool, DRAKES, this)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(not_failed.id, ids[3]);
    // The second good reading started 1 minute after `now`.
    let after = |minutes| {
        readings::good_started_after(
            &pool,
            DRAKES,
            this,
            now() + chrono::TimeDelta::minutes(minutes),
        )
    };
    assert_eq!(
        after(0).await.unwrap().map(|reading| reading.id),
        Some(ids[1])
    );
    assert_eq!(after(1).await.unwrap(), None);
    assert_eq!(
        readings::latest(&pool, DRAKES, season("2025-2026"))
            .await
            .unwrap(),
        None
    );
    assert_eq!(readings::by_id(&pool, 999_999).await.unwrap(), None);
}
