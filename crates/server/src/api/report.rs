//! `GET /api/report`: the report of the current season (decision 0033). For each player the
//! attendance at trainings and games (as on "Attendance") and the duties with the corrections (as
//! on "Duties"), the training days not opened and the games not checked, and the Dutch texts to
//! copy. All reads are in one read-only transaction, so every number is of one moment. Names
//! never go to the log.

use axum::Json;
use axum::extract::State;
use serde::Serialize;
use team_core::{
    KindTotals, Player, ReportColumn, ReportRow, Share, Total, display_names, duty_totals,
    games_not_checked, report_copy, report_rows,
};
use utoipa::ToSchema;

use super::attendance::overview_of;
use super::format::day;
use super::season::{SeasonPlaces, games_in, trainings_in};
use crate::app::AppState;
use crate::clock::{current_season, in_netherlands, today_in_netherlands};
use crate::db::StoreError;
use crate::error::AppError;
use crate::{game_attendance, games, players, reasons};

/// Trainings or games of a player: what counts, with the percentages (half up).
#[derive(Debug, Serialize, ToSchema)]
pub struct ShareView {
    pub counted: usize,
    pub there: usize,
    pub absent_valid: usize,
    pub absent_not_valid: usize,
    /// "there" of `counted`, for example 67.
    pub percent: u8,
    pub percent_valid: u8,
    pub percent_not_valid: u8,
}

impl From<Share> for ShareView {
    fn from(share: Share) -> Self {
        Self {
            counted: share.counted,
            there: share.there,
            absent_valid: share.absent_valid,
            absent_not_valid: share.absent_not_valid,
            percent: share.percent_there().unwrap_or_default(),
            percent_valid: share.percent_valid().unwrap_or_default(),
            percent_not_valid: share.percent_not_valid().unwrap_or_default(),
        }
    }
}

/// One duty kind of a player, with the corrections after a game.
#[derive(Debug, Serialize, ToSchema)]
pub struct DutyView {
    pub done: u32,
    pub planned: u32,
    /// Planned for this player; another player did it.
    pub done_by_other: u32,
    /// Done by this player for another player (also in `done`).
    pub done_for_other: u32,
    /// Planned for this player; nobody did it.
    pub not_done: u32,
}

impl From<Total> for DutyView {
    fn from(total: Total) -> Self {
        Self {
            done: total.done,
            planned: total.planned,
            done_by_other: total.done_by_other,
            done_for_other: total.done_for_other,
            not_done: total.not_done,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReportDutiesView {
    pub drive: DutyView,
    pub table: DutyView,
    pub referee: DutyView,
}

impl From<KindTotals> for ReportDutiesView {
    fn from(totals: KindTotals) -> Self {
        Self {
            drive: totals.drive.into(),
            table: totals.table.into(),
            referee: totals.referee.into(),
        }
    }
}

/// A row of the report.
#[derive(Debug, Serialize, ToSchema)]
pub struct ReportRowView {
    pub player: i64,
    /// The name shown ("Anna", "Anna B.").
    pub name: String,
    pub hidden: bool,
    /// `null`: no training counts ("–").
    pub trainings: Option<ShareView>,
    /// `null`: no game counts ("–").
    pub games: Option<ShareView>,
    /// `null` ("–"): no team chosen, or no role "plays games" in the season and no duty.
    pub duties: Option<ReportDutiesView>,
}

/// A game not checked, to open from the report.
#[derive(Debug, Serialize, ToSchema)]
pub struct ReportGameRef {
    pub season: String,
    pub id: i64,
}

/// What does not count yet: training days not opened and games not checked.
#[derive(Debug, Serialize, ToSchema)]
pub struct ReportIncomplete {
    pub not_opened: usize,
    /// The oldest training day not opened, for example "2026-10-13".
    pub first_not_opened: Option<String>,
    /// The games of the open item "not checked" of the start page in this season.
    pub not_checked: usize,
    /// The oldest of them.
    pub first_not_checked: Option<ReportGameRef>,
}

/// The texts to copy for each column; `null`: no line, no "Copy".
#[derive(Debug, Serialize, ToSchema)]
pub struct ReportCopies {
    pub trainings: Option<String>,
    pub games: Option<String>,
    pub drive: Option<String>,
    pub table: Option<String>,
    pub referee: Option<String>,
}

/// The page "Report".
#[derive(Debug, Serialize, ToSchema)]
pub struct Report {
    /// The current season, for example "2026-2027".
    pub season: String,
    pub incomplete: ReportIncomplete,
    /// A role today or later in the season; sorted by first name.
    pub team: Vec<ReportRowView>,
    /// "Left the team": everyone else with a number.
    pub left: Vec<ReportRowView>,
    pub copies: ReportCopies,
}

#[utoipa::path(
    get,
    path = "/api/report",
    operation_id = "report",
    responses(
        (status = 200, description = "The report of the current season", body = Report),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn get(State(state): State<AppState>) -> Result<Json<Report>, AppError> {
    let season = current_season(state.clock.as_ref());
    let today = today_in_netherlands(state.clock.as_ref());
    let now = in_netherlands(state.clock.now());

    // One moment for every number (decision 0033).
    let mut tx = state.pool.begin().await.map_err(StoreError::from)?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
        .execute(&mut *tx)
        .await
        .map_err(StoreError::from)?;
    let players = players::all(&mut *tx, &state.config.player_name_key).await?;
    let trainings = trainings_in(&mut tx, season).await?;
    let reasons = reasons::all(&mut *tx).await?;
    let games = games_in(&state, &mut tx, season, season, today).await?;
    let release = game_attendance::check_start(&mut *tx).await?;
    let stored = match &games.club {
        Some(club) if games.team.is_some() => games::stored(&mut tx, club.number, season).await?,
        _ => Vec::new(),
    };
    tx.rollback().await.map_err(StoreError::from)?;

    // Attendance, fed as on "Attendance".
    let overview = overview_of(season, &players, &trainings, &reasons, &games, today);
    let attendance = &games.duties.attendance;

    // Duties, as on "Duties"; none without a team.
    let duties = match (&games.club, &games.team) {
        (Some(club), Some(team)) => {
            let places = SeasonPlaces::new(&stored, &games.duties, club.number, team.number, now);
            Some(duty_totals(&places.duties(&games.duties)))
        }
        _ => None,
    };

    let rows = report_rows(
        season,
        today,
        &players,
        &trainings.roles,
        &overview,
        duties.as_ref(),
    );

    // Games not checked: the rule of the open item of the start page.
    let not_checked = games_not_checked(
        season,
        &games.schedule.past,
        &attendance.games,
        release,
        today,
    );
    let first_not_checked = not_checked.iter().min_by_key(|game| game.date);

    // The names of "Duties"; without a team, of the players with a role in the season.
    let names = if games.team.is_some() {
        games.duties.names.clone()
    } else {
        let of_season: Vec<Player> = players
            .iter()
            .filter(|player| trainings.roles.iter().any(|role| role.player == player.id))
            .cloned()
            .collect();
        display_names(&of_season)
    };
    let names = &names;
    let who = match (&games.club, &games.team) {
        (Some(club), Some(team)) => Some((club.name.as_str(), team.name.as_str())),
        _ => None,
    };
    let copy = |column| report_copy(column, &rows, names, who, season, today);
    let view = |row: &ReportRow<'_>| ReportRowView {
        player: row.player.id.0,
        name: names
            .get(&row.player.id)
            .cloned()
            .unwrap_or_else(|| row.player.first_name.clone()),
        hidden: row.player.hidden,
        trainings: row.trainings.map(Into::into),
        games: row.games.map(Into::into),
        duties: row.duties.map(Into::into),
    };

    Ok(Json(Report {
        season: season.to_string(),
        incomplete: ReportIncomplete {
            not_opened: overview.not_opened,
            first_not_opened: overview.first_not_opened.map(day),
            not_checked: not_checked.len(),
            first_not_checked: first_not_checked.map(|game| ReportGameRef {
                season: game.season.to_string(),
                id: game.id,
            }),
        },
        team: rows.iter().filter(|row| !row.left).map(view).collect(),
        left: rows.iter().filter(|row| row.left).map(view).collect(),
        copies: ReportCopies {
            trainings: copy(ReportColumn::Trainings),
            games: copy(ReportColumn::Games),
            drive: copy(ReportColumn::Drive),
            table: copy(ReportColumn::Table),
            referee: copy(ReportColumn::Referee),
        },
    }))
}
