//! The chosen club and team, one row for each season (table `followed_teams`).

use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use team_core::{Club, ClubNumber, FollowedTeam, Season, Team, TeamNumber};

use crate::db::StoreError;

/// The choice saved for `season`, if any.
pub async fn for_season(
    db: impl PgExecutor<'_>,
    season: Season,
) -> Result<Option<FollowedTeam>, StoreError> {
    let row = sqlx::query_as!(
        Row,
        r#"SELECT season_year AS "season_year!", club_number, club_name, club_town, team_number,
                  team_name
           FROM followed_teams WHERE season_year = $1"#,
        season.start_year(),
    )
    .fetch_optional(db)
    .await?;
    Ok(row.map(Row::into_followed_team))
}

/// The choice of the newest season before `season`, if any. Rows of later seasons (a clock that
/// went back) are never returned, so they cannot hide the current season.
pub async fn newest_before(
    db: impl PgExecutor<'_>,
    season: Season,
) -> Result<Option<FollowedTeam>, StoreError> {
    let row = sqlx::query_as!(
        Row,
        r#"SELECT season_year AS "season_year!", club_number, club_name, club_town, team_number,
                  team_name
           FROM followed_teams WHERE season_year < $1 ORDER BY season_year DESC LIMIT 1"#,
        season.start_year(),
    )
    .fetch_optional(db)
    .await?;
    Ok(row.map(Row::into_followed_team))
}

struct Row {
    season_year: i32,
    club_number: i64,
    club_name: String,
    club_town: Option<String>,
    team_number: Option<i64>,
    team_name: Option<String>,
}

impl Row {
    fn into_followed_team(self) -> FollowedTeam {
        let team = match (self.team_number, self.team_name) {
            (Some(number), Some(name)) => Some(Team {
                number: TeamNumber(number),
                name,
            }),
            _ => None,
        };
        FollowedTeam {
            season: Season::from_start_year(self.season_year),
            club: Club {
                number: ClubNumber(self.club_number),
                name: self.club_name,
                town: self.club_town,
            },
            team,
        }
    }
}

/// Saves the choice for its season. A choice of the same season is replaced; rows of other
/// seasons are not touched. It still writes the text season (decision 0014): the computed
/// `season_year` follows from it.
pub async fn save(
    db: impl PgExecutor<'_>,
    choice: &FollowedTeam,
    chosen_at: DateTime<Utc>,
) -> Result<(), StoreError> {
    sqlx::query!(
        "INSERT INTO followed_teams
             (season, club_number, club_name, club_town, team_number, team_name, chosen_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (season) DO UPDATE SET
             club_number = EXCLUDED.club_number,
             club_name = EXCLUDED.club_name,
             club_town = EXCLUDED.club_town,
             team_number = EXCLUDED.team_number,
             team_name = EXCLUDED.team_name,
             chosen_at = EXCLUDED.chosen_at",
        choice.season.to_string(),
        choice.club.number.0,
        choice.club.name,
        choice.club.town,
        choice.team.as_ref().map(|team| team.number.0),
        choice.team.as_ref().map(|team| team.name.as_str()),
        chosen_at,
    )
    .execute(db)
    .await?;
    Ok(())
}
