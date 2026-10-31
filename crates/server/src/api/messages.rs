//! `GET /api/messages/week`: the message of the week (story C of duties), Dutch text for the
//! coordinator to copy into the chat. The start page shows its button from the same rule
//! (`week_text`, decision 0022). The text holds player names: it never goes to the log.

use axum::Json;
use axum::extract::State;
use chrono::NaiveDate;
use serde::Serialize;
use team_core::{MessageGame, PlacesOf, week_message, week_of};
use utoipa::ToSchema;

use super::season::{SeasonGames, games_of, places_of_season};
use crate::app::AppState;
use crate::clock::{current_season, today_in_netherlands};
use crate::error::AppError;

/// A message to copy.
#[derive(Debug, Serialize, ToSchema)]
pub struct MessageView {
    /// The Dutch text; `null` when there is no message (no game of the team from today to Sunday
    /// and no table or referee places of the team in this week).
    pub text: Option<String>,
}

/// The message of the week of the season of `games` (the current season), with the duties of
/// `places`; `None` without a chosen team or without anything to say.
pub(crate) fn week_text(
    games: &SeasonGames,
    places: &PlacesOf,
    today: NaiveDate,
) -> Option<String> {
    let (club, team) = (games.club.as_ref()?, games.team.as_ref()?);
    let (monday, _) = week_of(today);
    let week: Vec<MessageGame<'_>> = games
        .schedule
        .past
        .iter()
        .filter(|scheduled| scheduled.game.date >= monday)
        .chain(&games.schedule.coming)
        .map(|scheduled| MessageGame {
            scheduled,
            places: places.get(&scheduled.game.source_id),
            assignments: games.duties.assignments_of(scheduled.game.source_id),
        })
        .collect();
    week_message(&week, &games.duties.names, club.number, team.number, today)
}

#[utoipa::path(
    get,
    path = "/api/messages/week",
    operation_id = "weekMessage",
    responses(
        (status = 200, description = "The message of the week, or `null` when there is none", body = MessageView),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn week(State(state): State<AppState>) -> Result<Json<MessageView>, AppError> {
    let clock = state.clock.as_ref();
    let today = today_in_netherlands(clock);
    let current = current_season(clock);
    let games = games_of(&state, current, current, today).await?;
    let places = places_of_season(&games, today);
    Ok(Json(MessageView {
        text: week_text(&games, &places, today),
    }))
}
