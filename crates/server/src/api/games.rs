//! `/api/games/{season}/{id}`: the page of a game and its duties (story A of duties: the places
//! and drivers; story B: the players on them). A game is a game of the club chosen for its season,
//! with a team, that is in the schedule or kept for its duties; also when it left the game list.
//! Every write: lock the game, read, the core check, write, in one transaction; each answers the
//! page.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use team_core::{
    Assignment, Clash, ClashLevel, ClashWhat, Club, DutyError, DutyKind, Game, GamePlaces,
    NotOffered, Offer, Outcome, PlaceKind, PlayerId, RequiredDuties, ScheduleReason, ScheduledGame,
    Season, StoredDuties, StoredGame, Team, TeamNumber, check_assignment, check_drivers,
    check_entered, check_outcome, check_team, choice_players, clashes, club_teams, duty_totals,
    game_places, in_schedule, kept_games, offers, pick_team, players_playing_on, plays_games_on,
    required_duties, worst_clash,
};
use utoipa::ToSchema;

use super::season::{
    DutyKindView, OutcomeView, ScheduleGame, SeasonDuties, SeasonPlaces, duties_of,
};
use crate::api::clubs::TeamView;
use crate::app::AppState;
use crate::clock::{current_season, today_in_netherlands};
use crate::db::StoreError;
use crate::duties::{self, GameKey};
use crate::error::AppError;
use crate::{followed_teams, games, players};

#[derive(Debug, Serialize, ToSchema)]
pub struct GamePage {
    /// The season of the game, for example "2026-2027".
    pub season: String,
    pub current_season: String,
    /// The team chosen for the game's season.
    pub team: TeamView,
    /// The game, with its duties.
    pub game: ScheduleGame,
    /// The pull-down list of the places: the club teams of the season, sorted by name.
    pub club_teams: Vec<TeamView>,
    /// The game is before today: its places can be corrected ("Someone else", "Nobody").
    pub past: bool,
    /// The choice of a player for the places of the team: the players with the role "plays
    /// games" on the date (hidden players too), and the players already at the game; sorted by
    /// first name.
    pub players: Vec<ChoiceView>,
}

/// A player in the choice of a place.
#[derive(Debug, Serialize, ToSchema)]
pub struct ChoiceView {
    pub id: i64,
    /// The name shown ("Anna", "Anna B.").
    pub name: String,
    pub hidden: bool,
    /// The duties of the player in the game's season, for each kind.
    pub totals: KindTotalsView,
    /// The clashes with other games.
    pub clashes: Vec<ClashView>,
    /// For each place of the team (in the order of the page): can the player be chosen there
    /// (`team_core::offers`, the checks of the saves).
    pub offers: Vec<OfferView>,
}

/// Why a player cannot be chosen for a place.
#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum NotOfferedView {
    /// The player is on this place already.
    OnThisPlace,
    /// The player has another place at the game.
    AtGame,
    /// No role "plays games" on the date.
    NotInTeam,
    /// A clash under 1:30 on another court.
    Clash,
}

/// Whether a player can be chosen for one place of the team.
#[derive(Debug, Serialize, ToSchema)]
pub struct OfferView {
    pub kind: DutyKindView,
    pub number: u8,
    /// As the planned player; `null`: can be chosen.
    pub assign: Option<NotOfferedView>,
    /// As "Someone else" on a past place with a player; `null`: can be chosen (or no correction
    /// is possible there).
    pub correct: Option<NotOfferedView>,
}

impl From<NotOffered> for NotOfferedView {
    fn from(reason: NotOffered) -> Self {
        match reason {
            NotOffered::OnThisPlace => Self::OnThisPlace,
            NotOffered::AtGame => Self::AtGame,
            NotOffered::NotInTeam => Self::NotInTeam,
            NotOffered::Clash => Self::Clash,
        }
    }
}

impl From<&Offer> for OfferView {
    fn from(offer: &Offer) -> Self {
        Self {
            kind: offer.kind.into(),
            number: offer.number,
            assign: offer.assign.map(Into::into),
            correct: offer.correct.map(Into::into),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TotalView {
    pub done: u32,
    pub planned: u32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct KindTotalsView {
    pub drive: TotalView,
    pub table: TotalView,
    pub referee: TotalView,
}

impl From<team_core::Total> for TotalView {
    fn from(total: team_core::Total) -> Self {
        Self {
            done: total.done,
            planned: total.planned,
        }
    }
}

impl From<team_core::KindTotals> for KindTotalsView {
    fn from(totals: team_core::KindTotals) -> Self {
        Self {
            drive: totals.drive.into(),
            table: totals.table.into(),
            referee: totals.referee.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClashLevelView {
    /// Under 1:30 apart on another court: cannot be chosen.
    Blocked,
    /// Under 2:00 apart on another court, or a start time not known: a note.
    Warning,
}

/// A clash with another game: a duty place there ("also Table 13:00") or a game of the team
/// ("plays 14:30").
#[derive(Debug, Serialize, ToSchema)]
pub struct ClashView {
    pub level: ClashLevelView,
    /// The kind of the duty place there; `null`: the player plays that game.
    pub duty: Option<DutyKindView>,
    /// Dutch local start time of the other game ("13:00"); `null` when not known.
    pub start: Option<String>,
}

impl From<&Clash> for ClashView {
    fn from(clash: &Clash) -> Self {
        Self {
            level: match clash.level {
                ClashLevel::Blocked => ClashLevelView::Blocked,
                ClashLevel::Warning => ClashLevelView::Warning,
            },
            duty: match clash.what {
                ClashWhat::Duty { kind } => Some(kind.into()),
                ClashWhat::Plays => None,
            },
            start: clash.start.map(|start| start.format("%H:%M").to_string()),
        }
    }
}

/// The team of a place; `null`: "—".
#[derive(Debug, Deserialize, ToSchema)]
pub struct PlaceInput {
    pub team: Option<i64>,
}

/// The player to put on a place.
#[derive(Debug, Deserialize, ToSchema)]
pub struct AssignmentInput {
    pub player: i64,
}

/// A correction after the game: as planned, done by `player` ("Someone else"), or not done
/// ("Nobody").
#[derive(Debug, Deserialize, ToSchema)]
pub struct OutcomeInput {
    pub outcome: OutcomeView,
    /// With `done_by` only.
    pub player: Option<i64>,
}

/// The number of drivers; `null`: the proposal.
#[derive(Debug, Deserialize, ToSchema)]
pub struct DriversInput {
    pub drivers: Option<u8>,
}

impl From<DutyError> for AppError {
    fn from(error: DutyError) -> Self {
        match error {
            DutyError::NotAHomeGame => Self::BadRequest("not_a_home_game"),
            DutyError::NoDriversNeeded => Self::BadRequest("no_drivers_needed"),
            DutyError::OutOfRange => Self::BadRequest("out_of_range"),
            DutyError::UnknownTeam => Self::BadRequest("unknown_team"),
            DutyError::PlayerOnPlace => Self::BadRequest("player_on_place"),
            DutyError::NotAPlaceOfTheTeam => Self::BadRequest("not_a_place_of_the_team"),
            DutyError::PlayerAtGame => Self::BadRequest("player_at_game"),
            DutyError::NotInTeam => Self::BadRequest("not_in_team"),
            DutyError::Clash => Self::BadRequest("clash"),
            DutyError::NotPast => Self::BadRequest("not_past"),
            DutyError::NoPlayerOnPlace => Self::BadRequest("no_player_on_place"),
            DutyError::BelowFilled { filled, highest } => Self::Refused {
                status: StatusCode::BAD_REQUEST,
                code: "places_filled",
                details: json!({ "filled": filled, "highest": highest }),
            },
        }
    }
}

/// A game of a season: the choice of that season, the game, and what the page needs.
struct Found {
    season: Season,
    club: Club,
    team: Team,
    game: Game,
    in_list: bool,
    reason: ScheduleReason,
    /// The pull-down list of the places.
    club_teams: Vec<team_core::TeamRef>,
    /// All stored games of the season, for the clashes.
    stored: Vec<StoredGame>,
}

impl Found {
    fn key(&self) -> GameKey {
        GameKey {
            club: self.club.number,
            season: self.season,
            source_id: self.game.source_id,
        }
    }

    fn required(&self) -> RequiredDuties {
        required_duties(&self.game, self.club.number, self.team.number)
    }

    /// The game is before `today`.
    fn past(&self, today: chrono::NaiveDate) -> bool {
        self.game.date < today
    }

    /// The places of the game from what is locked: `stored` and the players `at_game`.
    fn places(
        &self,
        stored: &StoredDuties,
        at_game: &[Assignment],
        duties: &SeasonDuties,
        today: chrono::NaiveDate,
    ) -> Option<GamePlaces> {
        game_places(
            self.required(),
            Some(stored),
            self.team.number,
            &at_game
                .iter()
                .map(|assignment| (assignment.kind, assignment.number))
                .collect(),
            players_playing_on(&duties.roles, self.game.date),
            self.in_list,
            self.past(today),
        )
    }

    /// The closest clash of `player` for a place at this game.
    fn clash(
        &self,
        duties: &SeasonDuties,
        player: PlayerId,
        today: chrono::NaiveDate,
    ) -> Option<ClashLevel> {
        let season = self.season_places(duties, today);
        worst_clash(&clashes(
            player,
            &self.game,
            &season.duties(duties),
            &season.team_games,
        ))
    }

    /// The places of the stored games of the season, for the clashes and the totals.
    fn season_places(&self, duties: &SeasonDuties, today: chrono::NaiveDate) -> SeasonPlaces<'_> {
        SeasonPlaces::new(
            &self.stored,
            duties,
            self.club.number,
            self.team.number,
            today,
        )
    }
}

/// The players a write locked with its game.
struct Locked {
    stored: StoredDuties,
    at_game: Vec<Assignment>,
}

impl Locked {
    /// The places with a player of `kind`.
    fn filled(&self, kind: DutyKind) -> std::collections::BTreeSet<(DutyKind, u8)> {
        self.at_game
            .iter()
            .filter(|assignment| assignment.kind == kind)
            .map(|assignment| (assignment.kind, assignment.number))
            .collect()
    }

    fn on(&self, kind: DutyKind, number: u8) -> Option<&Assignment> {
        self.at_game
            .iter()
            .find(|assignment| (assignment.kind, assignment.number) == (kind, number))
    }
}

/// The game `id` of `season`, from the choice of that season; 404 without a team, or for a game
/// that is not in the schedule and not kept for its duties.
async fn find(state: &AppState, season: &str, id: &str) -> Result<Found, AppError> {
    let season = season
        .parse::<Season>()
        .map_err(|_| AppError::BadRequest("invalid_season"))?;
    let id = id
        .parse::<i64>()
        .map_err(|_| AppError::BadRequest("invalid_game"))?;
    let Some(team_core::FollowedTeam {
        club,
        team: Some(team),
        ..
    }) = followed_teams::for_season(&state.pool, season).await?
    else {
        return Err(AppError::NotFound);
    };
    let mut db = state.pool.acquire().await.map_err(StoreError::from)?;
    let stored = games::stored(&mut db, club.number, season).await?;
    let duties = duties::for_season(&mut db, club.number, season).await?;
    let assignments = duties::assignments_for_season(&mut db, club.number, season).await?;
    let all: Vec<Game> = stored.iter().map(|stored| stored.game.clone()).collect();
    let club_teams = club_teams(&club, &all, &duties);
    let found = stored
        .iter()
        .find(|stored| stored.game.source_id == id)
        .cloned()
        .ok_or(AppError::NotFound)?;
    let reason = in_schedule(&found.game, club.number, Some(team.number))
        .or_else(|| {
            kept_games(&duties, &assignments)
                .contains(&id)
                .then_some(ScheduleReason::Kept)
        })
        .ok_or(AppError::NotFound)?;
    Ok(Found {
        season,
        club,
        team,
        game: found.game,
        in_list: found.in_list,
        reason,
        club_teams,
        stored,
    })
}

/// The page of `found`, with the duties as stored now.
async fn page(state: &AppState, found: Found) -> Result<GamePage, AppError> {
    let today = today_in_netherlands(state.clock.as_ref());
    let duties = duties_of(state, found.club.number, found.season).await?;
    let places = duties.places_of(
        &found.game,
        found.in_list,
        found.club.number,
        found.team.number,
        today,
    );
    let view = places
        .as_ref()
        .map(|places| duties.view(&found.game, places));
    let players = choice(&found, &duties, places.as_ref(), today);
    let past = found.past(today);
    let mut game = ScheduleGame::from(ScheduledGame {
        game: found.game,
        reason: found.reason,
        in_list: found.in_list,
    });
    game.places = view;
    Ok(GamePage {
        season: found.season.to_string(),
        current_season: current_season(state.clock.as_ref()).to_string(),
        team: TeamView::from(found.team),
        game,
        club_teams: found.club_teams.iter().map(TeamView::from).collect(),
        past,
        players,
    })
}

/// The choice of a player for the places of the game of `found`, with the totals of its season
/// (`team_core::duty_totals`) and the clashes.
fn choice(
    found: &Found,
    duties: &SeasonDuties,
    places: Option<&GamePlaces>,
    today: chrono::NaiveDate,
) -> Vec<ChoiceView> {
    let season = found.season_places(duties, today);
    let season_duties = season.duties(duties);
    let totals = duty_totals(&season_duties);
    let at_game = duties.assignments_of(found.game.source_id);
    // `players` is sorted by first name (`players::all`).
    choice_players(&duties.players, &duties.roles, found.game.date, at_game)
        .into_iter()
        .map(|choice| {
            let player = choice.player;
            let clashes = clashes(player.id, &found.game, &season_duties, &season.team_games);
            let offers = places.map_or_else(Vec::new, |places| {
                offers(
                    places,
                    at_game,
                    player.id,
                    choice.plays_on_date,
                    worst_clash(&clashes),
                    found.past(today),
                )
            });
            ChoiceView {
                id: player.id.0,
                name: duties.names.get(&player.id).cloned().unwrap_or_default(),
                hidden: player.hidden,
                totals: totals.get(&player.id).copied().unwrap_or_default().into(),
                clashes: clashes.iter().map(ClashView::from).collect(),
                offers: offers.iter().map(OfferView::from).collect(),
            }
        })
        .collect()
}

/// One write of the duties of `found`: lock the game, check against what is stored, write;
/// a refused or failed write rolls back.
async fn write<F>(state: &AppState, found: &Found, change: F) -> Result<(), AppError>
where
    F: AsyncFnOnce(&mut sqlx::PgConnection, &Locked) -> Result<(), AppError>,
{
    let now = state.clock.now();
    let mut tx = state.pool.begin().await.map_err(StoreError::from)?;
    let stored = duties::lock(&mut tx, found.key(), now).await?;
    let at_game = duties::assignments_of_game(&mut tx, found.key()).await?;
    change(&mut tx, &Locked { stored, at_game }).await?;
    tx.commit().await.map_err(StoreError::from)?;
    tracing::info!(season = %found.season, game = found.game.source_id, "duties saved");
    Ok(())
}

#[utoipa::path(
    get,
    path = "/api/games/{season}/{id}",
    operation_id = "game",
    params(
        ("season" = String, Path, description = "The season, for example 2026-2027"),
        ("id" = String, Path, description = "The source's game id"),
    ),
    responses(
        (status = 200, description = "The game with its duties", body = GamePage),
        (status = 400, description = "`invalid_season`, `invalid_game`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such game of the club chosen for that season"),
    )
)]
pub async fn get(
    State(state): State<AppState>,
    Path((season, id)): Path<(String, String)>,
) -> Result<Json<GamePage>, AppError> {
    let found = find(&state, &season, &id).await?;
    Ok(Json(page(&state, found).await?))
}

/// The place of the path: `referee` or `table`, 1 or 2.
fn place(kind: &str, number: &str) -> Result<(PlaceKind, u8), AppError> {
    let kind = match kind {
        "referee" => PlaceKind::Referee,
        "table" => PlaceKind::Table,
        _ => return Err(AppError::BadRequest("invalid_place")),
    };
    match number {
        "1" => Ok((kind, 1)),
        "2" => Ok((kind, 2)),
        _ => Err(AppError::BadRequest("invalid_place")),
    }
}

#[utoipa::path(
    put,
    path = "/api/games/{season}/{id}/places/{kind}/{number}",
    operation_id = "setPlace",
    params(
        ("season" = String, Path, description = "The season, for example 2026-2027"),
        ("id" = String, Path, description = "The source's game id"),
        ("kind" = String, Path, description = "`referee` or `table`"),
        ("number" = String, Path, description = "1 or 2"),
    ),
    request_body = PlaceInput,
    responses(
        (status = 200, description = "Saved; the game with its duties", body = GamePage),
        (status = 400, description = "`invalid_season`, `invalid_game`, `invalid_place`, `unknown_team`, `not_a_home_game`, `player_on_place`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such game of the club chosen for that season"),
    )
)]
pub async fn set_place(
    State(state): State<AppState>,
    Path((season, id, kind, number)): Path<(String, String, String, String)>,
    Json(input): Json<PlaceInput>,
) -> Result<Json<GamePage>, AppError> {
    let (kind, number) = place(&kind, &number)?;
    let found = find(&state, &season, &id).await?;
    let team = input
        .team
        .map(|team| pick_team(&found.club_teams, TeamNumber(team)))
        .transpose()?;
    let required = found.required();
    let now = state.clock.now();
    write(&state, &found, async |tx, locked| {
        check_team(
            required,
            locked.stored.teams.get(&(kind, number)),
            team.as_ref(),
            locked.on(kind.into(), number).is_some(),
        )?;
        duties::set_team(tx, found.key(), kind, number, team.as_ref(), now).await?;
        Ok(())
    })
    .await?;
    Ok(Json(page(&state, found).await?))
}

async fn entered(
    state: AppState,
    season: String,
    id: String,
    on: bool,
) -> Result<Json<GamePage>, AppError> {
    let found = find(&state, &season, &id).await?;
    check_entered(found.required(), on)?;
    let now = state.clock.now();
    write(&state, &found, async |tx, _| {
        duties::set_entered(tx, found.key(), on, now).await?;
        Ok(())
    })
    .await?;
    Ok(Json(page(&state, found).await?))
}

#[utoipa::path(
    put,
    path = "/api/games/{season}/{id}/schedule-entered",
    operation_id = "setScheduleEntered",
    params(
        ("season" = String, Path, description = "The season, for example 2026-2027"),
        ("id" = String, Path, description = "The source's game id"),
    ),
    responses(
        (status = 200, description = "Schedule entered; the game with its duties", body = GamePage),
        (status = 400, description = "`invalid_season`, `invalid_game`, `not_a_home_game`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such game of the club chosen for that season"),
    )
)]
pub async fn set_entered(
    State(state): State<AppState>,
    Path((season, id)): Path<(String, String)>,
) -> Result<Json<GamePage>, AppError> {
    entered(state, season, id, true).await
}

#[utoipa::path(
    delete,
    path = "/api/games/{season}/{id}/schedule-entered",
    operation_id = "unsetScheduleEntered",
    params(
        ("season" = String, Path, description = "The season, for example 2026-2027"),
        ("id" = String, Path, description = "The source's game id"),
    ),
    responses(
        (status = 200, description = "Schedule not entered; the game with its duties", body = GamePage),
        (status = 400, description = "`invalid_season`, `invalid_game`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such game of the club chosen for that season"),
    )
)]
pub async fn unset_entered(
    State(state): State<AppState>,
    Path((season, id)): Path<(String, String)>,
) -> Result<Json<GamePage>, AppError> {
    entered(state, season, id, false).await
}

#[utoipa::path(
    put,
    path = "/api/games/{season}/{id}/drivers",
    operation_id = "setDrivers",
    params(
        ("season" = String, Path, description = "The season, for example 2026-2027"),
        ("id" = String, Path, description = "The source's game id"),
    ),
    request_body = DriversInput,
    responses(
        (status = 200, description = "Saved; the game with its duties", body = GamePage),
        (status = 400, description = "`invalid_season`, `invalid_game`, `no_drivers_needed`, `out_of_range`, `places_filled` (with `filled` and `highest`)"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such game of the club chosen for that season"),
    )
)]
pub async fn set_drivers(
    State(state): State<AppState>,
    Path((season, id)): Path<(String, String)>,
    Json(input): Json<DriversInput>,
) -> Result<Json<GamePage>, AppError> {
    let found = find(&state, &season, &id).await?;
    let required = found.required();
    let now = state.clock.now();
    write(&state, &found, async |tx, locked| {
        check_drivers(required, input.drivers, &locked.filled(DutyKind::Drive))?;
        duties::set_drivers(tx, found.key(), input.drivers, now).await?;
        Ok(())
    })
    .await?;
    Ok(Json(page(&state, found).await?))
}

/// The place of an assignment path: `drive`, `table` or `referee`, and its number (1 to 10).
fn duty_place(kind: &str, number: &str) -> Result<(DutyKind, u8), AppError> {
    let kind = match kind {
        "drive" => DutyKind::Drive,
        "table" => DutyKind::Table,
        "referee" => DutyKind::Referee,
        _ => return Err(AppError::BadRequest("invalid_place")),
    };
    match number.parse::<u8>() {
        Ok(number @ 1..=10) => Ok((kind, number)),
        _ => Err(AppError::BadRequest("invalid_place")),
    }
}

/// The player `id`, when it exists.
async fn player(state: &AppState, id: i64) -> Result<PlayerId, AppError> {
    players::by_id(&state.pool, &state.config.player_name_key, PlayerId(id))
        .await?
        .map(|player| player.id)
        .ok_or(AppError::BadRequest("unknown_player"))
}

#[utoipa::path(
    put,
    path = "/api/games/{season}/{id}/assignments/{kind}/{number}",
    operation_id = "setAssignment",
    params(
        ("season" = String, Path, description = "The season, for example 2026-2027"),
        ("id" = String, Path, description = "The source's game id"),
        ("kind" = String, Path, description = "`drive`, `table` or `referee`"),
        ("number" = String, Path, description = "The place: 1 or 2, or the driver place 1 to 10"),
    ),
    request_body = AssignmentInput,
    responses(
        (status = 200, description = "Saved; the game with its duties", body = GamePage),
        (status = 400, description = "`invalid_season`, `invalid_game`, `invalid_place`, `unknown_player`, `not_a_place_of_the_team`, `player_at_game`, `not_in_team`, `clash`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such game of the club chosen for that season"),
    )
)]
pub async fn set_assignment(
    State(state): State<AppState>,
    Path((season, id, kind, number)): Path<(String, String, String, String)>,
    Json(input): Json<AssignmentInput>,
) -> Result<Json<GamePage>, AppError> {
    let (kind, number) = duty_place(&kind, &number)?;
    let found = find(&state, &season, &id).await?;
    let player = player(&state, input.player).await?;
    let today = today_in_netherlands(state.clock.as_ref());
    let duties = duties_of(&state, found.club.number, found.season).await?;
    let plays = plays_games_on(&duties.roles, player, found.game.date);
    let clash = found.clash(&duties, player, today);
    let now = state.clock.now();
    write(&state, &found, async |tx, locked| {
        let places = found
            .places(&locked.stored, &locked.at_game, &duties, today)
            .ok_or(DutyError::NotAPlaceOfTheTeam)?;
        check_assignment(&places, kind, number, player, &locked.at_game, plays, clash)?;
        // The same player again is no change: a correction after the game stays.
        if locked
            .on(kind, number)
            .is_some_and(|on| on.planned == player)
        {
            return Ok(());
        }
        duties::set_assignment(tx, found.key(), kind, number, player, now).await?;
        Ok(())
    })
    .await?;
    Ok(Json(page(&state, found).await?))
}

#[utoipa::path(
    delete,
    path = "/api/games/{season}/{id}/assignments/{kind}/{number}",
    operation_id = "clearAssignment",
    params(
        ("season" = String, Path, description = "The season, for example 2026-2027"),
        ("id" = String, Path, description = "The source's game id"),
        ("kind" = String, Path, description = "`drive`, `table` or `referee`"),
        ("number" = String, Path, description = "The place: 1 or 2, or the driver place 1 to 10"),
    ),
    responses(
        (status = 200, description = "The place is open; the game with its duties", body = GamePage),
        (status = 400, description = "`invalid_season`, `invalid_game`, `invalid_place`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such game of the club chosen for that season"),
    )
)]
pub async fn clear_assignment(
    State(state): State<AppState>,
    Path((season, id, kind, number)): Path<(String, String, String, String)>,
) -> Result<Json<GamePage>, AppError> {
    let (kind, number) = duty_place(&kind, &number)?;
    let found = find(&state, &season, &id).await?;
    let now = state.clock.now();
    write(&state, &found, async |tx, _| {
        duties::clear_assignment(tx, found.key(), kind, number, now).await?;
        Ok(())
    })
    .await?;
    Ok(Json(page(&state, found).await?))
}

#[utoipa::path(
    put,
    path = "/api/games/{season}/{id}/assignments/{kind}/{number}/outcome",
    operation_id = "setOutcome",
    params(
        ("season" = String, Path, description = "The season, for example 2026-2027"),
        ("id" = String, Path, description = "The source's game id"),
        ("kind" = String, Path, description = "`drive`, `table` or `referee`"),
        ("number" = String, Path, description = "The place: 1 or 2, or the driver place 1 to 10"),
    ),
    request_body = OutcomeInput,
    responses(
        (status = 200, description = "Saved; the game with its duties", body = GamePage),
        (status = 400, description = "`invalid_season`, `invalid_game`, `invalid_place`, `invalid_outcome`, `unknown_player`, `not_past`, `no_player_on_place`, `player_at_game`, `not_in_team`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such game of the club chosen for that season"),
    )
)]
pub async fn set_outcome(
    State(state): State<AppState>,
    Path((season, id, kind, number)): Path<(String, String, String, String)>,
    Json(input): Json<OutcomeInput>,
) -> Result<Json<GamePage>, AppError> {
    let (kind, number) = duty_place(&kind, &number)?;
    let found = find(&state, &season, &id).await?;
    let new = match (input.outcome, input.player) {
        (OutcomeView::AsPlanned, None) => Outcome::AsPlanned,
        (OutcomeView::NotDone, None) => Outcome::NotDone,
        (OutcomeView::DoneBy, Some(id)) => Outcome::DoneBy(player(&state, id).await?),
        _ => return Err(AppError::BadRequest("invalid_outcome")),
    };
    let today = today_in_netherlands(state.clock.as_ref());
    let plays = match new {
        Outcome::DoneBy(player) => {
            let duties = duties_of(&state, found.club.number, found.season).await?;
            plays_games_on(&duties.roles, player, found.game.date)
        }
        Outcome::AsPlanned | Outcome::NotDone => true,
    };
    let past = found.past(today);
    let now = state.clock.now();
    write(&state, &found, async |tx, locked| {
        let outcome = check_outcome(past, locked.on(kind, number), new, &locked.at_game, plays)?;
        duties::set_outcome(tx, found.key(), kind, number, outcome, now).await?;
        Ok(())
    })
    .await?;
    Ok(Json(page(&state, found).await?))
}
