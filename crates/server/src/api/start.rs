//! `GET /api/start`: the start page (story B of start page and season): the open items, the last
//! training, and the events of this week and next week, in one answer. Each open item comes from
//! the function of its source: the "Game list" card, the list of changes, "Attendance".

use axum::Json;
use axum::extract::State;
use serde::Serialize;
use team_core::{
    Event, EventKind, OpenItem, ReadingState, last_training, next_events, open_item_seasons,
    open_items, season_events,
};
use utoipa::ToSchema;

use super::format::day;
use super::season::{EventView, event_view, games_of, trainings_of};
use crate::app::AppState;
use crate::clock::{current_season, in_netherlands, today_in_netherlands};
use crate::error::AppError;
use crate::{reading, readings};

#[derive(Debug, Serialize, ToSchema)]
pub struct StartAnswer {
    /// What still needs the coordinator, in the order of the start page.
    pub open_items: Vec<OpenItemView>,
    /// The last training before today that is not cancelled, as an event; `null` when there is
    /// none.
    pub last: Option<EventView>,
    /// The games of the team and the trainings from today until the Sunday of next week, in time
    /// order; when there are none, the first coming one.
    pub next: Vec<EventView>,
}

/// One open item. Dates are Dutch dates, for example "2026-10-22".
#[derive(Debug, Serialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OpenItemView {
    /// A held-back reading of the game list waits for the coordinator.
    ReadingHeldBack,
    /// The game list has not been read well for 3 nights, since the last good reading.
    OutOfDate { since: String },
    /// Changes not seen yet.
    Changes { count: u32 },
    /// Training days before today that were never opened; `first` is the oldest.
    NotOpened { first: String, count: usize },
    /// Trainings before today that are neither cancelled nor checked; `first` is the oldest.
    NotChecked { first: String, count: usize },
}

impl From<OpenItem> for OpenItemView {
    fn from(item: OpenItem) -> Self {
        match item {
            OpenItem::ReadingHeldBack => Self::ReadingHeldBack,
            OpenItem::OutOfDate { since } => Self::OutOfDate { since: day(since) },
            OpenItem::ChangesNotSeen { count } => Self::Changes { count },
            OpenItem::NotOpened { first, count } => Self::NotOpened {
                first: day(first),
                count,
            },
            OpenItem::NotChecked { first, count } => Self::NotChecked {
                first: day(first),
                count,
            },
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/start",
    operation_id = "start",
    responses(
        (status = 200, description = "The open items, the last training and the next events", body = StartAnswer),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn get(State(state): State<AppState>) -> Result<Json<StartAnswer>, AppError> {
    let clock = state.clock.as_ref();
    let today = today_in_netherlands(clock);
    let current = current_season(clock);

    // The game list: the same rule as the "Game list" card. Without any club there is no reading.
    let reading = match reading::target(&state.pool, clock).await? {
        Some(target) => {
            let status = readings::status(&state.pool, clock, &target).await?;
            ReadingState {
                held_back: status.held_back.is_some(),
                out_of_date_since: status
                    .out_of_date_since
                    .map(|moment| in_netherlands(moment).date()),
            }
        }
        None => ReadingState::default(),
    };
    // The games of the current season, and its changes not seen (the list of changes).
    let games = games_of(&state, current, current, today).await?;

    // The trainings of the seasons the open items cover; the current season comes first.
    let seasons = open_item_seasons(today);
    let mut inputs = Vec::with_capacity(seasons.len());
    for &season in &seasons {
        inputs.push(trainings_of(&state, season).await?);
    }
    let lists: Vec<_> = seasons
        .iter()
        .zip(&inputs)
        .map(|(&season, trainings)| trainings.list(season, today))
        .collect();

    let items = open_items(&reading, games.unseen_changes, &lists);
    let last = last_training(&lists)
        .map(|row| {
            event_view(&Event {
                date: row.date,
                start: Some(row.start),
                kind: EventKind::Training(row),
            })
        })
        .transpose()?;
    let events = match lists.first() {
        Some(list) => season_events(&games.schedule, list),
        None => Default::default(),
    };
    let next = next_events(&events, today)
        .into_iter()
        .map(event_view)
        .collect::<Result<_, _>>()?;
    Ok(Json(StartAnswer {
        open_items: items.into_iter().map(OpenItemView::from).collect(),
        last,
        next,
    }))
}
