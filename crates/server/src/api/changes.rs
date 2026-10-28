//! `GET /api/changes` and `POST /api/changes/seen`: the changes the coordinator has not seen yet
//! (story 4, decision 0016).

use std::collections::BTreeMap;

use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use team_core::{
    Change, ChangeKind, ChoiceState, ClubNumber, Difference, Game, ListedChange, Season,
    StoredGame, TeamNumber, choice_state, list_changes,
};
use utoipa::ToSchema;

use crate::api::season::{PlacesView, ScheduleReasonView, duties_of};
use crate::app::AppState;
use crate::changes;
use crate::clock::{current_season, today_in_netherlands};
use crate::db::{self, StoreError};
use crate::error::AppError;
use crate::{duties, followed_teams, games};

#[derive(Debug, Serialize, ToSchema)]
pub struct ChangesAnswer {
    /// Empty unless a team is chosen for the current season. In the order of the games.
    pub changes: Vec<ChangeView>,
    /// The season of the changes (the current season), for the page of a game; `null` without a
    /// chosen team.
    pub season: Option<String>,
}

/// A change from the game as the coordinator last saw it to the game now.
#[derive(Debug, Serialize, ToSchema)]
pub struct ChangeView {
    /// The source's game id.
    pub id: i64,
    /// Send it back with "seen": a change that a reading updated meanwhile stays.
    pub version: i64,
    pub kind: ChangeKindView,
    /// Every difference of a `moved`, `start_time_known`, `home_and_away` or `team_changed`
    /// change; empty for the other kinds.
    pub differences: Vec<DifferenceView>,
    /// Now, or as last seen when the game is no longer in the schedule.
    pub reason: ScheduleReasonView,
    /// The game as last seen; `null` for a new game.
    pub before: Option<ChangeGame>,
    /// The game now; `null` for a game no longer in the game list.
    pub after: Option<ChangeGame>,
    /// The duties of the game, as on "Season" ("has duties"); `null` when it has none.
    pub places: Option<PlacesView>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKindView {
    New,
    /// Another date, start time or place.
    Moved,
    StartTimeKnown,
    HomeAndAway,
    TeamChanged,
    Gone,
    CameBack,
    /// Now in the schedule; `reason` says why.
    IntoSchedule,
    /// No longer in the schedule; `reason` says what it was.
    OutOfSchedule,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DifferenceView {
    Date,
    /// Another start time, or a start time that is no longer known.
    StartTime,
    StartTimeKnown,
    Place,
    HomeAndAway,
    Team,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ChangeGame {
    /// Dutch local date, for example "2026-10-31".
    pub date: String,
    /// Dutch local start time, for example "13:00"; `null` when it is not known.
    pub start_time: Option<String>,
    pub home: String,
    pub away: String,
    pub hall: String,
    pub town: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SeenRequest {
    pub changes: Vec<SeenChange>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SeenChange {
    pub id: i64,
    pub version: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SeenAnswer {
    /// The number of changes not seen yet after this.
    pub open: u32,
}

impl From<&Game> for ChangeGame {
    fn from(game: &Game) -> Self {
        Self {
            date: game.date.format("%Y-%m-%d").to_string(),
            start_time: game.start_time.map(|time| time.format("%H:%M").to_string()),
            home: game.home.team_name.clone(),
            away: game.away.team_name.clone(),
            hall: game.place.hall.clone(),
            town: game.place.town.clone(),
        }
    }
}

fn view(listed: ListedChange, version: i64) -> Option<ChangeView> {
    let kind = match listed.change.kind()? {
        ChangeKind::New => ChangeKindView::New,
        ChangeKind::Moved => ChangeKindView::Moved,
        ChangeKind::StartTimeKnown => ChangeKindView::StartTimeKnown,
        ChangeKind::HomeAndAway => ChangeKindView::HomeAndAway,
        ChangeKind::TeamChanged => ChangeKindView::TeamChanged,
        ChangeKind::Gone => ChangeKindView::Gone,
        ChangeKind::CameBack => ChangeKindView::CameBack,
        ChangeKind::IntoSchedule => ChangeKindView::IntoSchedule,
        ChangeKind::OutOfSchedule => ChangeKindView::OutOfSchedule,
    };
    let differences = match &listed.change {
        Change::Changed(differences) => differences
            .iter()
            .map(|difference| match difference {
                Difference::Date => DifferenceView::Date,
                Difference::StartTime => DifferenceView::StartTime,
                Difference::StartTimeKnown => DifferenceView::StartTimeKnown,
                Difference::Place => DifferenceView::Place,
                Difference::HomeAndAway => DifferenceView::HomeAndAway,
                Difference::Team => DifferenceView::Team,
            })
            .collect(),
        _ => Vec::new(),
    };
    Some(ChangeView {
        id: listed.source_id,
        version,
        kind,
        differences,
        reason: listed.reason.into(),
        before: listed.last_seen.as_ref().map(|seen| (&seen.game).into()),
        after: listed.now.in_list.then(|| (&listed.now.game).into()),
        places: None,
    })
}

/// The club, team and season of the current choice, when a team is chosen.
async fn chosen(state: &AppState) -> Result<Option<(ClubNumber, TeamNumber, Season)>, AppError> {
    let season = current_season(state.clock.as_ref());
    let this_season = followed_teams::for_season(&state.pool, season).await?;
    let earlier = followed_teams::newest_before(&state.pool, season).await?;
    Ok(match choice_state(this_season, earlier) {
        ChoiceState::Chosen { club, team, .. } => Some((club.number, team.number, season)),
        _ => None,
    })
}

/// The changes to show for `club`, `team` and `season`, with `stored` the stored games of that
/// club and season.
pub async fn list(
    state: &AppState,
    db: &mut PgConnection,
    club: ClubNumber,
    team: TeamNumber,
    season: Season,
    stored: &[StoredGame],
) -> Result<Vec<ChangeView>, StoreError> {
    let (open, versions): (Vec<_>, Vec<_>) = changes::open(&mut *db, club, season)
        .await?
        .into_iter()
        .unzip();
    let version_of: BTreeMap<i64, i64> = open
        .iter()
        .map(|open| open.source_id)
        .zip(versions)
        .collect();
    let with_duties = duties::with_duties(&mut *db, club, season, team).await?;
    let today = today_in_netherlands(state.clock.as_ref());
    Ok(list_changes(open, stored, club, team, today, &with_duties)
        .into_iter()
        .filter_map(|listed| {
            let version = *version_of.get(&listed.source_id)?;
            view(listed, version)
        })
        .collect())
}

async fn list_loaded(
    state: &AppState,
    db: &mut PgConnection,
    club: ClubNumber,
    team: TeamNumber,
    season: Season,
) -> Result<Vec<ChangeView>, StoreError> {
    let stored = games::stored(&mut *db, club, season).await?;
    list(state, db, club, team, season, &stored).await
}

#[utoipa::path(
    get,
    path = "/api/changes",
    operation_id = "changes",
    responses(
        (status = 200, description = "The changes not seen yet", body = ChangesAnswer),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn get(State(state): State<AppState>) -> Result<Json<ChangesAnswer>, AppError> {
    let Some((club, team, season)) = chosen(&state).await? else {
        return Ok(Json(ChangesAnswer {
            changes: Vec::new(),
            season: None,
        }));
    };
    let mut db = state.pool.acquire().await.map_err(StoreError::from)?;
    let stored = games::stored(&mut db, club, season).await?;
    let mut changes = list(&state, &mut db, club, team, season, &stored).await?;
    drop(db);
    // "has duties": the duties of each game, from the same rule as "Season" (decision 0022).
    let duties = duties_of(&state, club, season).await?;
    let today = today_in_netherlands(state.clock.as_ref());
    for change in &mut changes {
        if let Some(game) = stored.iter().find(|game| game.game.source_id == change.id) {
            change.places = duties
                .places_of(&game.game, game.in_list, club, team, today)
                .map(|places| duties.view(&game.game, &places));
        }
    }
    Ok(Json(ChangesAnswer {
        changes,
        season: Some(season.to_string()),
    }))
}

#[utoipa::path(
    post,
    path = "/api/changes/seen",
    operation_id = "markSeen",
    request_body = SeenRequest,
    responses(
        (status = 200, description = "Marked; a change with another version stays", body = SeenAnswer),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn seen(
    State(state): State<AppState>,
    Json(body): Json<SeenRequest>,
) -> Result<Json<SeenAnswer>, AppError> {
    let Some((club, team, season)) = chosen(&state).await? else {
        return Ok(Json(SeenAnswer { open: 0 }));
    };
    // Under the reading lock: a reading never runs in between, and the two never lock the same
    // rows in another order.
    let mut tx = state.pool.begin().await.map_err(StoreError::from)?;
    db::lock_readings(&mut tx).await?;
    let seen: Vec<(i64, i64)> = body
        .changes
        .iter()
        .map(|change| (change.id, change.version))
        .collect();
    changes::mark_seen(&mut tx, club, season, &seen).await?;
    let open = list_loaded(&state, &mut tx, club, team, season)
        .await?
        .len();
    tx.commit().await.map_err(StoreError::from)?;
    let open = u32::try_from(open).map_err(|_| StoreError::Invalid(format!("{open} changes")))?;
    Ok(Json(SeenAnswer { open }))
}
