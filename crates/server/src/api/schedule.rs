//! `GET /api/schedule`: the coordinator's schedule from the stored games (story 3b).

use axum::Json;
use axum::extract::State;
use serde::Serialize;
use team_core::{
    ChoiceState, Club, ScheduleReason, ScheduledGame, Season, Team, choice_state, schedule,
};
use utoipa::ToSchema;

use crate::api::clubs::{ClubView, TeamView};
use crate::app::AppState;
use crate::clock::{current_season, today_in_netherlands};
use crate::error::AppError;
use crate::{followed_teams, games};

#[derive(Debug, Serialize, ToSchema)]
pub struct ScheduleAnswer {
    /// `null`: no club is chosen, or a team is needed and no earlier season had one.
    pub schedule: Option<ScheduleView>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ScheduleView {
    /// The season of the games, for example "2026-2027".
    pub season: String,
    /// True: a new season started and its team is not chosen yet; these are the games of the
    /// team of the newest earlier season.
    pub last_season: bool,
    pub club: ClubView,
    pub team: TeamView,
    /// Today (Dutch date) and later, in time order.
    pub coming: Vec<ScheduleGame>,
    /// Before today, newest date first, in time order within a date.
    pub past: Vec<ScheduleGame>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ScheduleGame {
    /// The source's game id.
    pub id: i64,
    /// Dutch local date, for example "2026-10-31".
    pub date: String,
    /// Dutch local start time, for example "13:00"; `null` when it is not known yet.
    pub start_time: Option<String>,
    /// Full team names, with the club name in front.
    pub home: String,
    pub away: String,
    pub hall: String,
    pub town: String,
    pub reason: ScheduleReasonView,
    /// False: the game is no longer in the game list.
    pub in_list: bool,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleReasonView {
    /// A game of the team, home or away.
    Team,
    /// A home game of another club team: a possible duty game.
    Duty,
}

impl From<ScheduledGame> for ScheduleGame {
    fn from(scheduled: ScheduledGame) -> Self {
        let game = scheduled.game;
        Self {
            id: game.source_id,
            date: game.date.format("%Y-%m-%d").to_string(),
            start_time: game.start_time.map(|time| time.format("%H:%M").to_string()),
            home: game.home.team_name,
            away: game.away.team_name,
            hall: game.place.hall,
            town: game.place.town,
            reason: match scheduled.reason {
                ScheduleReason::TeamGame => ScheduleReasonView::Team,
                ScheduleReason::DutyGame => ScheduleReasonView::Duty,
            },
            in_list: scheduled.in_list,
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/schedule",
    operation_id = "schedule",
    responses(
        (status = 200, description = "The schedule to show, or `null` when there is none", body = ScheduleAnswer),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn get(State(state): State<AppState>) -> Result<Json<ScheduleAnswer>, AppError> {
    let season = current_season(state.clock.as_ref());
    let this_season = followed_teams::for_season(&state.pool, season).await?;
    let earlier = followed_teams::newest_before(&state.pool, season).await?;
    let shown = match choice_state(this_season, earlier) {
        ChoiceState::Chosen { club, team, .. } => Some((season, club, team, false)),
        ChoiceState::TeamNeeded { last, .. } => last.and_then(|last| {
            let team = last.team?;
            Some((last.season, last.club, team, true))
        }),
        ChoiceState::NotChosen => None,
    };
    let schedule = match shown {
        Some((season, club, team, last_season)) => {
            Some(view(&state, season, club, team, last_season).await?)
        }
        None => None,
    };
    Ok(Json(ScheduleAnswer { schedule }))
}

async fn view(
    state: &AppState,
    season: Season,
    club: Club,
    team: Team,
    last_season: bool,
) -> Result<ScheduleView, AppError> {
    let mut db = state
        .pool
        .acquire()
        .await
        .map_err(crate::db::StoreError::from)?;
    let stored = games::stored(&mut db, club.number, season).await?;
    let today = today_in_netherlands(state.clock.as_ref());
    let schedule = schedule(stored, club.number, team.number, today);
    Ok(ScheduleView {
        season: season.to_string(),
        last_season,
        club: club.into(),
        team: team.into(),
        coming: schedule
            .coming
            .into_iter()
            .map(ScheduleGame::from)
            .collect(),
        past: schedule.past.into_iter().map(ScheduleGame::from).collect(),
    })
}
