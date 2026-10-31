//! `GET /api/messages/week` and `GET /api/messages/committee`: the message of the week and the
//! committee message (stories C and D of duties), Dutch text for the coordinator to copy into the
//! chat. The start page shows their buttons from the same rules (`week_text`, `committee_text`,
//! decision 0022). The texts hold player names: they never go to the log.

use axum::Json;
use axum::extract::State;
use chrono::NaiveDate;
use serde::Serialize;
use team_core::{MessageGame, PlacesOf, committee_message, full_names, week_message, week_of};
use utoipa::ToSchema;

use super::season::{SeasonGames, games_of, places_of_season};
use crate::app::AppState;
use crate::clock::{current_season, today_in_netherlands};
use crate::error::AppError;

/// A message to copy.
#[derive(Debug, Serialize, ToSchema)]
pub struct MessageView {
    /// The Dutch text; `null` when there is no message (no team chosen, or for the message of the
    /// week: no game of the team from today to Sunday and no table or referee places of the team
    /// in this week).
    pub text: Option<String>,
}

/// The past games of this week and the coming games of `games`, with their duties.
fn message_games<'a>(
    games: &'a SeasonGames,
    places: &'a PlacesOf,
    today: NaiveDate,
) -> Vec<MessageGame<'a>> {
    let (monday, _) = week_of(today);
    games
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
        .collect()
}

/// The message of the week of the season of `games` (the current season), with the duties of
/// `places`; `None` without a chosen team or without anything to say.
pub(crate) fn week_text(
    games: &SeasonGames,
    places: &PlacesOf,
    today: NaiveDate,
) -> Option<String> {
    let (club, team) = (games.club.as_ref()?, games.team.as_ref()?);
    week_message(
        &message_games(games, places, today),
        &games.duties.names,
        club.number,
        team.number,
        today,
    )
}

/// The committee message of the season of `games` (the current season), with the duties of
/// `places`; `None` without a chosen team.
pub(crate) fn committee_text(
    games: &SeasonGames,
    places: &PlacesOf,
    today: NaiveDate,
) -> Option<String> {
    let (club, team) = (games.club.as_ref()?, games.team.as_ref()?);
    Some(committee_message(
        &message_games(games, places, today),
        &full_names(&games.duties.players),
        &club.name,
        &team.name,
        today,
    ))
}

/// The answer of a message route: `text` over the current season, as the start page reads it.
async fn message_view(
    state: &AppState,
    text: fn(&SeasonGames, &PlacesOf, NaiveDate) -> Option<String>,
) -> Result<Json<MessageView>, AppError> {
    let clock = state.clock.as_ref();
    let today = today_in_netherlands(clock);
    let current = current_season(clock);
    let games = games_of(state, current, current, today).await?;
    let places = places_of_season(&games, today);
    Ok(Json(MessageView {
        text: text(&games, &places, today),
    }))
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
    message_view(&state, week_text).await
}

#[utoipa::path(
    get,
    path = "/api/messages/committee",
    operation_id = "committeeMessage",
    responses(
        (status = 200, description = "The committee message, or `null` without a chosen team", body = MessageView),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn committee(State(state): State<AppState>) -> Result<Json<MessageView>, AppError> {
    message_view(&state, committee_text).await
}
