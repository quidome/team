//! `POST /api/readings`, `GET /api/readings/latest` and
//! `POST /api/readings/{id}/confirm-gone`: reading the game list on demand.

use axum::Json;
use axum::extract::{Path, State};
use chrono::{DateTime, Utc};
use serde::Serialize;
use team_core::{Counts, HoldReason, out_of_date};
use utoipa::ToSchema;

use crate::app::AppState;
use crate::clock::in_netherlands;
use crate::error::AppError;
use crate::reading::{self, Mode, ReadingError};
use crate::readings::{self, FailureKind, Reading, ReadingResult};

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResultKind {
    Good,
    Failed,
    HeldBack,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum HoldKind {
    /// The game list has no games.
    Empty,
    /// The game list has fewer than half the games of the last good reading.
    FarFewer,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    /// The game list cannot be reached, or answered with an error.
    SourceUnavailable,
    /// The game list answered with data the tool does not understand.
    SourceFormat,
}

/// How many changes of each kind a reading found, only for games in the schedule.
#[derive(Debug, Serialize, ToSchema)]
pub struct CountsView {
    pub new: u32,
    pub moved: u32,
    pub start_time_known: u32,
    pub home_and_away: u32,
    pub team_changed: u32,
    pub gone: u32,
    pub came_back: u32,
    pub into_schedule: u32,
    pub out_of_schedule: u32,
}

impl From<Counts> for CountsView {
    fn from(counts: Counts) -> Self {
        Self {
            new: counts.new,
            moved: counts.moved,
            start_time_known: counts.start_time_known,
            home_and_away: counts.home_and_away,
            team_changed: counts.team_changed,
            gone: counts.gone,
            came_back: counts.came_back,
            into_schedule: counts.into_schedule,
            out_of_schedule: counts.out_of_schedule,
        }
    }
}

/// Why a reading was held back: `now` games instead of `before`.
#[derive(Debug, Serialize, ToSchema)]
pub struct HoldView {
    pub reason: HoldKind,
    pub before: u32,
    pub now: u32,
}

/// One reading of the game list. A failed or held-back reading is a result, not an HTTP error.
#[derive(Debug, Serialize, ToSchema)]
pub struct ReadingView {
    pub id: i64,
    pub result: ResultKind,
    #[schema(value_type = String, format = DateTime)]
    pub finished_at: DateTime<Utc>,
    /// The number of games read; `null` for a failed reading.
    pub games_read: Option<u32>,
    /// The first reading after a choice of club or team: it counts nothing.
    pub first_reading: bool,
    /// `null` for the first reading, and for a failed or held-back reading.
    pub counts: Option<CountsView>,
    pub hold: Option<HoldView>,
    pub error: Option<ErrorKind>,
}

impl From<Reading> for ReadingView {
    fn from(reading: Reading) -> Self {
        let mut view = ReadingView {
            id: reading.id,
            result: ResultKind::Good,
            finished_at: reading.finished_at,
            games_read: None,
            first_reading: false,
            counts: None,
            hold: None,
            error: None,
        };
        match reading.result {
            ReadingResult::Good {
                games_read,
                first_reading,
                counts,
            } => {
                view.games_read = Some(games_read);
                view.first_reading = first_reading;
                view.counts = counts.map(CountsView::from);
            }
            ReadingResult::HeldBack(reason) => {
                let (reason, before, now) = match reason {
                    HoldReason::Empty { before } => (HoldKind::Empty, before, 0),
                    HoldReason::FarFewer { before, now } => (HoldKind::FarFewer, before, now),
                };
                view.result = ResultKind::HeldBack;
                view.games_read = Some(now);
                view.hold = Some(HoldView {
                    reason,
                    before,
                    now,
                });
            }
            ReadingResult::Failed(kind) => {
                view.result = ResultKind::Failed;
                view.error = Some(match kind {
                    FailureKind::SourceUnavailable => ErrorKind::SourceUnavailable,
                    FailureKind::SourceFormat => ErrorKind::SourceFormat,
                });
            }
        }
        view
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LastGood {
    #[schema(value_type = String, format = DateTime)]
    pub finished_at: DateTime<Utc>,
    pub games_read: u32,
}

/// The newest reading and the last good reading of the current club and season.
#[derive(Debug, Serialize, ToSchema)]
pub struct LatestReadings {
    pub latest: Option<ReadingView>,
    pub last_good: Option<LastGood>,
    /// The held-back reading the coordinator can confirm: the newest reading that did not fail,
    /// when it is held back. A failed reading after it does not hide it.
    pub held_back: Option<ReadingView>,
    /// The time of the last good reading, when 3 nightly run moments passed without a good
    /// reading: the games may be out of date since then. `null` without a chosen team.
    #[schema(value_type = Option<String>, format = DateTime)]
    pub out_of_date_since: Option<DateTime<Utc>>,
}

impl From<ReadingError> for AppError {
    fn from(error: ReadingError) -> Self {
        match error {
            ReadingError::NoChoice => Self::BadRequest("no_choice"),
            ReadingError::NotHeldBack => Self::BadRequest("not_held_back"),
            ReadingError::Store(error) => error.into(),
            ReadingError::Client(error) => Self::Internal(error.into()),
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/readings",
    responses(
        (status = 200, description = "The reading; also a failed or held-back one", body = ReadingView),
        (status = 400, description = "`no_choice`: no club is chosen"),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn read(State(state): State<AppState>) -> Result<Json<ReadingView>, AppError> {
    let reading = reading::read(
        &state.pool,
        &state.source,
        state.clock.as_ref(),
        Mode::Guarded,
    )
    .await?;
    Ok(Json(reading.into()))
}

#[utoipa::path(
    get,
    path = "/api/readings/latest",
    responses(
        (status = 200, description = "The newest and the last good reading of the current club and season; `null`s with no choice", body = LatestReadings),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn latest(State(state): State<AppState>) -> Result<Json<LatestReadings>, AppError> {
    let Some(target) = reading::target(&state.pool, state.clock.as_ref()).await? else {
        return Ok(Json(LatestReadings {
            latest: None,
            last_good: None,
            held_back: None,
            out_of_date_since: None,
        }));
    };
    let latest = readings::latest(&state.pool, target.club, target.season).await?;
    let last_good = readings::last_good(&state.pool, target.club, target.season).await?;
    // Without a team there is no nightly reading, so nothing to be late.
    let now = in_netherlands(state.clock.now());
    let out_of_date_since = last_good
        .as_ref()
        .filter(|reading| {
            target.team.is_some() && out_of_date(now, in_netherlands(reading.started_at))
        })
        .map(|reading| reading.finished_at);
    let last_good = last_good.and_then(|reading| match reading.result {
        ReadingResult::Good { games_read, .. } => Some(LastGood {
            finished_at: reading.finished_at,
            games_read,
        }),
        _ => None,
    });
    let held_back = readings::newest_not_failed(&state.pool, target.club, target.season)
        .await?
        .filter(|reading| matches!(reading.result, ReadingResult::HeldBack(_)));
    Ok(Json(LatestReadings {
        latest: latest.map(ReadingView::from),
        last_good,
        held_back: held_back.map(ReadingView::from),
        out_of_date_since,
    }))
}

#[utoipa::path(
    post,
    path = "/api/readings/{id}/confirm-gone",
    params(("id" = i64, Path, description = "The held-back reading")),
    responses(
        (status = 200, description = "A new reading: with at least as many games as the held-back reading, the missing games count as no longer in the game list; with fewer, it is held back again", body = ReadingView),
        (status = 400, description = "`not_held_back`: not the newest reading of the current club and season that did not fail, or not held back; `no_choice`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such reading"),
    )
)]
pub async fn confirm_gone(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ReadingView>, AppError> {
    readings::by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    let mode = Mode::Confirm { held_id: id };
    let reading = reading::read(&state.pool, &state.source, state.clock.as_ref(), mode).await?;
    Ok(Json(reading.into()))
}
