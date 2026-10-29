//! `GET /api/messages/next-game`: the message for the next game (story C of duties), Dutch text
//! for the coordinator to copy into the chat. The start page shows its link from the same rule
//! (`next_game_text`, decision 0022). The text holds player names: it never goes to the log.

use axum::Json;
use axum::extract::State;
use chrono::NaiveDate;
use serde::Serialize;
use team_core::{MessageGame, PlacesOf, next_game_message};
use utoipa::ToSchema;

use super::season::{SeasonGames, games_of, places_of_season};
use crate::app::AppState;
use crate::clock::{current_season, today_in_netherlands};
use crate::error::AppError;

/// A message to copy.
#[derive(Debug, Serialize, ToSchema)]
pub struct MessageView {
    /// The Dutch text; `null` when there is no message (no coming game of the team and no table
    /// or referee places of the team in the coming week).
    pub text: Option<String>,
}

/// The message for the next game of the season of `games` (the current season), with the duties
/// of `places`; `None` without a chosen team or without anything to say.
pub(crate) fn next_game_text(
    games: &SeasonGames,
    places: &PlacesOf,
    today: NaiveDate,
) -> Option<String> {
    let team = games.team.as_ref()?;
    let coming: Vec<MessageGame<'_>> = games
        .schedule
        .coming
        .iter()
        .map(|scheduled| MessageGame {
            scheduled,
            places: places.get(&scheduled.game.source_id),
            assignments: games.duties.assignments_of(scheduled.game.source_id),
        })
        .collect();
    next_game_message(&coming, &games.duties.names, team.number, today)
}

#[utoipa::path(
    get,
    path = "/api/messages/next-game",
    operation_id = "nextGameMessage",
    responses(
        (status = 200, description = "The message for the next game, or `null` when there is none", body = MessageView),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn next_game(State(state): State<AppState>) -> Result<Json<MessageView>, AppError> {
    let clock = state.clock.as_ref();
    let today = today_in_netherlands(clock);
    let current = current_season(clock);
    let games = games_of(&state, current, current, today).await?;
    let places = places_of_season(&games, today);
    Ok(Json(MessageView {
        text: next_game_text(&games, &places, today),
    }))
}
