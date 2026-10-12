//! The stored games of each club and season, and their places (tables `games` and `places`).
//! A reading changes only the games of its own club and season.

use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use sqlx::PgConnection;
use team_core::{ClubNumber, Game, Place, PlaceNumber, Season, Side, StoredGame, TeamNumber};

use crate::db::StoreError;

/// The stored games of `club` and `season`, also the ones no longer in the game list, sorted by
/// source id.
pub async fn stored(
    db: &mut PgConnection,
    club: ClubNumber,
    season: Season,
) -> Result<Vec<StoredGame>, StoreError> {
    let rows = sqlx::query_as!(
        Row,
        "SELECT g.source_id, g.game_number, g.date, g.start_time,
                g.home_team_number, g.home_team_name, g.home_club_number,
                g.away_team_number, g.away_team_name, g.away_club_number,
                g.place_number, p.hall, p.town, g.court, g.competition, g.in_list
         FROM games g JOIN places p ON p.number = g.place_number
         WHERE g.club_number = $1 AND g.season_year = $2
         ORDER BY g.source_id",
        club.0,
        season.start_year(),
    )
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| row.into_stored(season))
        .collect())
}

/// Writes a reading of `club` and `season`: every place of `read` (the newest hall and town win),
/// every game of `read` as in the game list (the time of its first reading stays), and marks the
/// games in `gone` as no longer in the game list.
pub async fn write(
    db: &mut PgConnection,
    club: ClubNumber,
    season: Season,
    read: &[Game],
    gone: &[i64],
    now: DateTime<Utc>,
) -> Result<(), StoreError> {
    let places: BTreeMap<PlaceNumber, &Place> = read
        .iter()
        .map(|game| (game.place.number, &game.place))
        .collect();
    for place in places.values() {
        sqlx::query!(
            "INSERT INTO places (number, hall, town) VALUES ($1, $2, $3)
             ON CONFLICT (number) DO UPDATE SET hall = EXCLUDED.hall, town = EXCLUDED.town",
            place.number.0,
            place.hall,
            place.town,
        )
        .execute(&mut *db)
        .await?;
    }

    for game in read {
        sqlx::query!(
            "INSERT INTO games
                 (club_number, season_year, source_id, game_number, date, start_time,
                  home_team_number, home_team_name, home_club_number,
                  away_team_number, away_team_name, away_club_number,
                  place_number, court, competition, in_list, first_read_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, true, $16)
             ON CONFLICT (club_number, season_year, source_id) DO UPDATE SET
                 game_number = EXCLUDED.game_number,
                 date = EXCLUDED.date,
                 start_time = EXCLUDED.start_time,
                 home_team_number = EXCLUDED.home_team_number,
                 home_team_name = EXCLUDED.home_team_name,
                 home_club_number = EXCLUDED.home_club_number,
                 away_team_number = EXCLUDED.away_team_number,
                 away_team_name = EXCLUDED.away_team_name,
                 away_club_number = EXCLUDED.away_club_number,
                 place_number = EXCLUDED.place_number,
                 court = EXCLUDED.court,
                 competition = EXCLUDED.competition,
                 in_list = true",
            club.0,
            season.start_year(),
            game.source_id,
            game.game_number,
            game.date,
            game.start_time,
            game.home.team.0,
            game.home.team_name,
            game.home.club.0,
            game.away.team.0,
            game.away.team_name,
            game.away.club.0,
            game.place.number.0,
            game.place.court,
            game.competition,
            now,
        )
        .execute(&mut *db)
        .await?;
    }

    sqlx::query!(
        "UPDATE games SET in_list = false
         WHERE club_number = $1 AND season_year = $2 AND source_id = ANY($3)",
        club.0,
        season.start_year(),
        gone,
    )
    .execute(&mut *db)
    .await?;
    Ok(())
}

struct Row {
    source_id: i64,
    game_number: String,
    date: NaiveDate,
    start_time: Option<NaiveTime>,
    home_team_number: i64,
    home_team_name: String,
    home_club_number: i64,
    away_team_number: i64,
    away_team_name: String,
    away_club_number: i64,
    place_number: i64,
    hall: String,
    town: String,
    court: Option<String>,
    competition: String,
    in_list: bool,
}

impl Row {
    fn into_stored(self, season: Season) -> StoredGame {
        StoredGame {
            game: Game {
                source_id: self.source_id,
                game_number: self.game_number,
                season,
                date: self.date,
                start_time: self.start_time,
                home: Side {
                    team: TeamNumber(self.home_team_number),
                    team_name: self.home_team_name,
                    club: ClubNumber(self.home_club_number),
                },
                away: Side {
                    team: TeamNumber(self.away_team_number),
                    team_name: self.away_team_name,
                    club: ClubNumber(self.away_club_number),
                },
                place: Place {
                    number: PlaceNumber(self.place_number),
                    hall: self.hall,
                    town: self.town,
                    court: self.court,
                },
                competition: self.competition,
            },
            in_list: self.in_list,
        }
    }
}
