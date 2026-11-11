//! `/api/games/{season}/{id}`: the page of a game and its duties (story A of duties: the places
//! and drivers; story B: the players on them). A game is a game of the club chosen for its season,
//! with a team, that is in the schedule or kept for its duties or answers; also when it left the game list.
//! Every write: lock the game, read, the core check, write, in one transaction; each answers the
//! page.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use team_core::{
    Answer, AnswerError, Assignment, Candidate, CheckError, Clash, ClashLevel, ClashWhat, Club,
    DutyError, DutyKind, ENOUGH_PLAYERS, Game, GameAttendance, GameCount, GameMoment, GamePlaces,
    GameWarnings, ListRow, MAX_EXTRA_PLAYERS, NotOffered, Offer, Outcome, PlaceKind, PlayerId,
    ReasonId, RequiredDuties, ScheduleReason, ScheduledGame, Season, StoredDuties, StoredGame,
    Team, TeamNumber, answers_in_effect, check_answer, check_assignment, check_drivers,
    check_entered, check_game_checking, check_outcome, check_team, checked, choice_players,
    clashes, club_teams, duty_totals, extra_players, game_count, game_list, game_places,
    game_warnings, in_schedule, kept_games, move_note, offers, pick_team, players_playing_on,
    plays_games_on, required_duties, started, to_settle, worst_clash,
};
use utoipa::ToSchema;

use super::reasons::{AbsenceReason, view as reason_view};
use super::season::{
    DutyKindView, OutcomeView, ScheduleGame, SeasonDuties, SeasonPlaces, duties_of,
};
use crate::api::clubs::TeamView;
use crate::app::AppState;
use crate::clock::{current_season, in_netherlands};
use crate::db::StoreError;
use crate::duties;
use crate::error::AppError;
use crate::games::GameKey;
use crate::{followed_teams, game_attendance, games, players, reasons};

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
    /// The choice of a player for the places of the team: the players with the role "plays
    /// games" on the date (hidden players too), and the players already at the game; sorted by
    /// first name.
    pub players: Vec<ChoiceView>,
    /// Who comes to the game; `null`: not a game of the team.
    pub attendance: Option<GameAttendanceView>,
    /// "Not played" on its date now: its duties and attendance do not count.
    pub not_played: bool,
    /// "Count the drives": the drives of a not played game count from its start time.
    pub drivers_drove: bool,
    /// The start time has passed (without a start time: the game day has begun; Dutch time): the
    /// page shows the row "The drives count" on a not played game with driver places.
    pub started: bool,
}

/// Who comes to a game of the team.
#[derive(Debug, Serialize, ToSchema)]
pub struct GameAttendanceView {
    /// First "no answer", then "coming", then "not coming", each sorted by first name.
    pub rows: Vec<AnswerRow>,
    /// All reasons in their order, also the hidden ones (they still show on old answers).
    pub reasons: Vec<AbsenceReason>,
    pub count: CountView,
    pub extra_players: u8,
    /// The highest number of extra players.
    pub max_extra_players: u8,
    /// "Too few players": fewer than `enough_players` coming, before the start, from 3 days
    /// before the game and earlier once it has an answer (`team_core::game_warnings`, the rule of
    /// the open item).
    pub too_few: bool,
    /// The number of players that is enough (7).
    pub enough_players: usize,
    /// "Answers from before the move": the answers were given for another date than the date of
    /// the game now; until "Mark answers as checked" (`team_core::answers_moved`).
    pub answers_moved: bool,
    /// "Checked" on the date of the game now (`team_core::checked`): the page shows "Checked" with
    /// "Undo", and the list closed behind a tap.
    pub checked: bool,
    /// The game is before the release of "Checked" (`game_check_start`): the list is closed behind
    /// a tap, and the game is never an open item.
    pub before_release: bool,
}

/// The answer of a tap: kept small, so a tap is quick.
#[derive(Debug, Serialize, ToSchema)]
pub struct AttendanceView {
    /// The row of the player; `null` after a save of the extra players.
    pub row: Option<AnswerRow>,
    pub count: CountView,
    pub extra_players: u8,
    /// As `GameAttendanceView::too_few`, after the tap.
    pub too_few: bool,
    pub enough_players: usize,
    /// As `GameAttendanceView::answers_moved`, after the tap.
    pub answers_moved: bool,
}

/// A player on the list of a game.
#[derive(Debug, Serialize, ToSchema)]
pub struct AnswerRow {
    pub player: i64,
    /// The name shown ("Anna", "Anna B.").
    pub name: String,
    pub hidden: bool,
    /// `null`: no answer.
    pub answer: Option<AnswerView>,
    /// The reason; it stays when the answer goes to "coming" and back.
    pub reason: Option<i64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnswerView {
    Coming,
    NotComing,
}

/// The count of a game; `coming` is the players coming plus the extra players.
#[derive(Debug, Serialize, ToSchema)]
pub struct CountView {
    pub coming: usize,
    pub not_coming: usize,
    pub no_answer: usize,
}

/// The answer of a player.
#[derive(Debug, Deserialize, ToSchema)]
pub struct AnswerInput {
    pub answer: AnswerView,
    pub reason: Option<i64>,
}

/// The number of extra players, 0 to 10.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ExtraPlayersInput {
    pub extra_players: i64,
}

impl From<Answer> for AnswerView {
    fn from(answer: Answer) -> Self {
        match answer {
            Answer::Coming => Self::Coming,
            Answer::NotComing => Self::NotComing,
        }
    }
}

impl From<AnswerView> for Answer {
    fn from(answer: AnswerView) -> Self {
        match answer {
            AnswerView::Coming => Self::Coming,
            AnswerView::NotComing => Self::NotComing,
        }
    }
}

impl From<GameCount> for CountView {
    fn from(count: GameCount) -> Self {
        Self {
            coming: count.coming,
            not_coming: count.not_coming,
            no_answer: count.no_answer,
        }
    }
}

impl From<CheckError> for AppError {
    fn from(error: CheckError) -> Self {
        Self::BadRequest(match error {
            CheckError::TooEarly => "checked_too_early",
        })
    }
}

/// The answer of "Checked": how many players with no answer became absent, and the page.
#[derive(Debug, Serialize, ToSchema)]
pub struct CheckedAnswer {
    /// The players with "no answer" that "Checked" made absent ("2 players with no answer are
    /// absent."); 0 when the game was already checked.
    pub settled: usize,
    pub page: GamePage,
}

impl From<AnswerError> for AppError {
    fn from(error: AnswerError) -> Self {
        Self::BadRequest(match error {
            AnswerError::NotOnList => "not_on_list",
            AnswerError::ReasonHidden => "reason_hidden",
        })
    }
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
    /// As "Someone else" on a place with a player of a game that has started; `null`: can be chosen (or no correction
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
            DutyError::NotStarted => Self::BadRequest("not_started"),
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
    /// Not played on its date now (`team_core::not_played`).
    not_played: bool,
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
        local: chrono::NaiveDateTime,
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
            GameMoment {
                in_list: self.in_list,
                past: self.past(local.date()),
                started: started(&self.game, local),
                not_played: self.not_played,
            },
        )
    }

    /// The closest clash of `player` for a place at this game.
    fn clash(
        &self,
        duties: &SeasonDuties,
        player: PlayerId,
        local: chrono::NaiveDateTime,
    ) -> Option<ClashLevel> {
        let season = self.season_places(duties, local);
        worst_clash(&clashes(
            player,
            &self.game,
            &season.duties(duties),
            &season.team_games,
        ))
    }

    /// The places of the stored games of the season, for the clashes and the totals.
    fn season_places(
        &self,
        duties: &SeasonDuties,
        local: chrono::NaiveDateTime,
    ) -> SeasonPlaces<'_> {
        SeasonPlaces::new(
            &self.stored,
            duties,
            self.club.number,
            self.team.number,
            local,
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
/// that is not in the schedule and not kept for its duties or answers.
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
    let reason = match in_schedule(&found.game, club.number, Some(team.number)) {
        Some(reason) => reason,
        None => {
            let answered = game_attendance::of_season(&mut db, club.number, season)
                .await?
                .answered();
            if !kept_games(&duties, &assignments, &answered).contains(&id) {
                return Err(AppError::NotFound);
            }
            ScheduleReason::Kept
        }
    };
    Ok(Found {
        season,
        club,
        team,
        not_played: found.not_played(),
        game: found.game,
        in_list: found.in_list,
        reason,
        club_teams,
        stored,
    })
}

/// The page of `found`, with the duties as stored now.
async fn page(state: &AppState, found: Found) -> Result<GamePage, AppError> {
    let local = in_netherlands(state.clock.now());
    let duties = duties_of(state, found.club.number, found.season).await?;
    let places = duties.places_of(
        &found.game,
        found.in_list,
        found.not_played,
        found.club.number,
        found.team.number,
        local,
    );
    let view = places
        .as_ref()
        .map(|places| duties.view(&found.game, places));
    let players = choice(&found, &duties, places.as_ref(), local);
    let started = started(&found.game, local);
    let drivers_drove = duties
        .stored
        .get(&found.game.source_id)
        .is_some_and(|stored| stored.drivers_drove);
    let attendance = attendance_of(state, &found, &duties).await?;
    let mut game = ScheduleGame::from(ScheduledGame {
        game: found.game,
        reason: found.reason,
        in_list: found.in_list,
        not_played: found.not_played,
    });
    game.places = view;
    Ok(GamePage {
        season: found.season.to_string(),
        current_season: current_season(state.clock.as_ref()).to_string(),
        team: TeamView::from(found.team),
        game,
        club_teams: found.club_teams.iter().map(TeamView::from).collect(),
        players,
        attendance,
        not_played: found.not_played,
        drivers_drove,
        started,
    })
}

/// The list of the game of `found` as stored now, with what is stored beside it, its warnings now
/// (`team_core::game_warnings`, the rule of the open items), and whether its answers are from
/// before a move (`team_core::answers_moved`, with no period).
async fn stored_list(
    state: &AppState,
    found: &Found,
    duties: &SeasonDuties,
) -> Result<(Vec<ListRow>, GameAttendance, GameWarnings, bool), AppError> {
    let (attendance, answers) = game_attendance::of_game(&state.pool, found.key()).await?;
    // A settled answer counts only while the check holds (story C).
    let answers = answers_in_effect(&answers, &attendance, found.game.date);
    let list = game_list(&duties.players, &duties.roles, &answers, found.game.date);
    let scheduled = ScheduledGame {
        game: found.game.clone(),
        reason: found.reason,
        in_list: found.in_list,
        not_played: found.not_played,
    };
    let moved = move_note(&scheduled, &attendance, !answers.is_empty());
    let warnings = game_warnings(
        &scheduled,
        &list,
        &attendance,
        !answers.is_empty(),
        duties.assignments_of(found.game.source_id),
        in_netherlands(state.clock.now()),
    );
    Ok((list, attendance, warnings, moved))
}

fn answer_row(row: &ListRow, duties: &SeasonDuties) -> AnswerRow {
    AnswerRow {
        player: row.player.0,
        name: duties.names.get(&row.player).cloned().unwrap_or_default(),
        hidden: duties
            .players
            .iter()
            .any(|player| player.id == row.player && player.hidden),
        answer: row.answer.map(Into::into),
        reason: row.reason.map(|reason| reason.0),
    }
}

/// Who comes to the game of `found`; `None` for a game that is not a game of the team.
async fn attendance_of(
    state: &AppState,
    found: &Found,
    duties: &SeasonDuties,
) -> Result<Option<GameAttendanceView>, AppError> {
    if found.reason != ScheduleReason::TeamGame {
        return Ok(None);
    }
    let (list, attendance, warnings, moved) = stored_list(state, found, duties).await?;
    let start = game_attendance::check_start(&state.pool).await?;
    Ok(Some(GameAttendanceView {
        rows: list.iter().map(|row| answer_row(row, duties)).collect(),
        reasons: reasons::all(&state.pool)
            .await?
            .into_iter()
            .map(reason_view)
            .collect(),
        count: game_count(&list, attendance.extra_players).into(),
        extra_players: attendance.extra_players,
        max_extra_players: MAX_EXTRA_PLAYERS,
        too_few: warnings.too_few_line,
        enough_players: ENOUGH_PLAYERS,
        answers_moved: moved,
        checked: checked(&attendance, found.game.date),
        before_release: found.game.date < start,
    }))
}

/// The answer of a tap, with the row of `player` when the tap was on a row.
async fn tapped(
    state: &AppState,
    found: &Found,
    duties: &SeasonDuties,
    player: Option<PlayerId>,
) -> Result<AttendanceView, AppError> {
    let (list, attendance, warnings, moved) = stored_list(state, found, duties).await?;
    Ok(AttendanceView {
        row: player
            .and_then(|player| list.iter().find(|row| row.player == player))
            .map(|row| answer_row(row, duties)),
        count: game_count(&list, attendance.extra_players).into(),
        extra_players: attendance.extra_players,
        too_few: warnings.too_few_line,
        enough_players: ENOUGH_PLAYERS,
        answers_moved: moved,
    })
}

/// The game of a tap: a game of the team, with what the checks need.
async fn team_game(
    state: &AppState,
    season: &str,
    id: &str,
) -> Result<(Found, SeasonDuties), AppError> {
    let found = find(state, season, id).await?;
    if found.reason != ScheduleReason::TeamGame {
        return Err(AppError::BadRequest("not_a_team_game"));
    }
    let duties = duties_of(state, found.club.number, found.season).await?;
    Ok((found, duties))
}

/// The choice of a player for the places of the game of `found`, with the totals of its season
/// (`team_core::duty_totals`) and the clashes.
fn choice(
    found: &Found,
    duties: &SeasonDuties,
    places: Option<&GamePlaces>,
    local: chrono::NaiveDateTime,
) -> Vec<ChoiceView> {
    let season = found.season_places(duties, local);
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
                    started(&found.game, local),
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
    let local = in_netherlands(state.clock.now());
    let duties = duties_of(&state, found.club.number, found.season).await?;
    let plays = plays_games_on(&duties.roles, player, found.game.date);
    let clash = found.clash(&duties, player, local);
    let now = state.clock.now();
    write(&state, &found, async |tx, locked| {
        let places = found
            .places(&locked.stored, &locked.at_game, &duties, local)
            .ok_or(DutyError::NotAPlaceOfTheTeam)?;
        check_assignment(
            &places,
            kind,
            number,
            &locked.at_game,
            Candidate {
                player,
                plays_on_date: plays,
                clash,
            },
            started(&found.game, local),
        )?;
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
        (status = 400, description = "`invalid_season`, `invalid_game`, `invalid_place`, `invalid_outcome`, `unknown_player`, `not_started`, `no_player_on_place`, `player_at_game`, `not_in_team`"),
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
    let local = in_netherlands(state.clock.now());
    let plays = match new {
        Outcome::DoneBy(player) => {
            let duties = duties_of(&state, found.club.number, found.season).await?;
            plays_games_on(&duties.roles, player, found.game.date)
        }
        Outcome::AsPlanned | Outcome::NotDone => true,
    };
    let started = started(&found.game, local);
    let now = state.clock.now();
    write(&state, &found, async |tx, locked| {
        let outcome = check_outcome(
            started,
            locked.on(kind, number),
            new,
            &locked.at_game,
            plays,
        )?;
        duties::set_outcome(tx, found.key(), kind, number, outcome, now).await?;
        Ok(())
    })
    .await?;
    Ok(Json(page(&state, found).await?))
}

#[utoipa::path(
    put,
    path = "/api/games/{season}/{id}/answers/{player}",
    operation_id = "setAnswer",
    params(
        ("season" = String, Path, description = "The season, for example 2026-2027"),
        ("id" = String, Path, description = "The source's game id"),
        ("player" = i64, Path, description = "The player"),
    ),
    request_body = AnswerInput,
    responses(
        (status = 200, description = "Saved; the last tap wins", body = AttendanceView),
        (status = 400, description = "`invalid_season`, `invalid_game`, `not_a_team_game`, `not_on_list`, `reason_hidden`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such game of the club chosen for that season, or no such reason"),
    )
)]
pub async fn set_answer(
    State(state): State<AppState>,
    Path((season, id, player)): Path<(String, String, i64)>,
    Json(input): Json<AnswerInput>,
) -> Result<Json<AttendanceView>, AppError> {
    let (found, duties) = team_game(&state, &season, &id).await?;
    let player = PlayerId(player);
    let reason = match input.reason {
        Some(id) => {
            let reason = reasons::all(&state.pool)
                .await?
                .into_iter()
                .find(|reason| reason.id == id)
                .ok_or(AppError::NotFound)?;
            Some((ReasonId(reason.id), reason.hidden))
        }
        None => None,
    };
    let (_, answers) = game_attendance::of_game(&state.pool, found.key()).await?;
    let current = answers
        .iter()
        .find(|answer| answer.player == player)
        .and_then(|answer| answer.reason);
    check_answer(player, &duties.roles, found.game.date, reason, current)?;
    game_attendance::answer(
        &state.pool,
        found.key(),
        found.game.date,
        player,
        input.answer.into(),
        reason.map(|(reason, _)| reason),
    )
    .await?;
    tracing::info!(season = %found.season, game = found.game.source_id, player = player.0, "answer saved");
    Ok(Json(tapped(&state, &found, &duties, Some(player)).await?))
}

#[utoipa::path(
    delete,
    path = "/api/games/{season}/{id}/answers/{player}",
    operation_id = "clearAnswer",
    params(
        ("season" = String, Path, description = "The season, for example 2026-2027"),
        ("id" = String, Path, description = "The source's game id"),
        ("player" = i64, Path, description = "The player"),
    ),
    responses(
        (status = 200, description = "The player is back to \"no answer\"; safe to send twice", body = AttendanceView),
        (status = 400, description = "`invalid_season`, `invalid_game`, `not_a_team_game`, `not_on_list`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such game of the club chosen for that season"),
    )
)]
pub async fn clear_answer(
    State(state): State<AppState>,
    Path((season, id, player)): Path<(String, String, i64)>,
) -> Result<Json<AttendanceView>, AppError> {
    let (found, duties) = team_game(&state, &season, &id).await?;
    let player = PlayerId(player);
    check_answer(player, &duties.roles, found.game.date, None, None)?;
    game_attendance::no_answer(&state.pool, found.key(), player).await?;
    tracing::info!(season = %found.season, game = found.game.source_id, player = player.0, "answer removed");
    Ok(Json(tapped(&state, &found, &duties, Some(player)).await?))
}

#[utoipa::path(
    put,
    path = "/api/games/{season}/{id}/extra-players",
    operation_id = "setExtraPlayers",
    params(
        ("season" = String, Path, description = "The season, for example 2026-2027"),
        ("id" = String, Path, description = "The source's game id"),
    ),
    request_body = ExtraPlayersInput,
    responses(
        (status = 200, description = "Saved; the last tap wins", body = AttendanceView),
        (status = 400, description = "`invalid_season`, `invalid_game`, `not_a_team_game`, `out_of_range`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such game of the club chosen for that season"),
    )
)]
pub async fn set_extra_players(
    State(state): State<AppState>,
    Path((season, id)): Path<(String, String)>,
    Json(input): Json<ExtraPlayersInput>,
) -> Result<Json<AttendanceView>, AppError> {
    let (found, duties) = team_game(&state, &season, &id).await?;
    let number = extra_players(input.extra_players).ok_or(AppError::BadRequest("out_of_range"))?;
    game_attendance::set_extra(&state.pool, found.key(), number).await?;
    tracing::info!(season = %found.season, game = found.game.source_id, "extra players saved");
    Ok(Json(tapped(&state, &found, &duties, None).await?))
}

#[utoipa::path(
    put,
    path = "/api/games/{season}/{id}/answers-checked",
    operation_id = "setAnswersChecked",
    params(
        ("season" = String, Path, description = "The season, for example 2026-2027"),
        ("id" = String, Path, description = "The source's game id"),
    ),
    responses(
        (status = 200, description = "Saved: the answers hold for the date of the game now", body = AttendanceView),
        (status = 400, description = "`invalid_season`, `invalid_game`, `not_a_team_game`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such game of the club chosen for that season"),
    )
)]
pub async fn set_answers_checked(
    State(state): State<AppState>,
    Path((season, id)): Path<(String, String)>,
) -> Result<Json<AttendanceView>, AppError> {
    let (found, duties) = team_game(&state, &season, &id).await?;
    game_attendance::answers_checked(&state.pool, found.key(), found.game.date).await?;
    tracing::info!(season = %found.season, game = found.game.source_id, "answers checked");
    Ok(Json(tapped(&state, &found, &duties, None).await?))
}

#[utoipa::path(
    put,
    path = "/api/games/{season}/{id}/not-played",
    operation_id = "setNotPlayed",
    params(
        ("season" = String, Path, description = "The season, for example 2026-2027"),
        ("id" = String, Path, description = "The source's game id"),
    ),
    responses(
        (status = 200, description = "Not played on its date now; the game page", body = GamePage),
        (status = 400, description = "`invalid_season`, `invalid_game`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such game of the club chosen for that season"),
    )
)]
pub async fn set_not_played(
    State(state): State<AppState>,
    Path((season, id)): Path<(String, String)>,
) -> Result<Json<GamePage>, AppError> {
    not_played(state, season, id, true).await
}

#[utoipa::path(
    delete,
    path = "/api/games/{season}/{id}/not-played",
    operation_id = "unsetNotPlayed",
    params(
        ("season" = String, Path, description = "The season, for example 2026-2027"),
        ("id" = String, Path, description = "The source's game id"),
    ),
    responses(
        (status = 200, description = "\"Undo\": the game counts again; the game page", body = GamePage),
        (status = 400, description = "`invalid_season`, `invalid_game`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such game of the club chosen for that season"),
    )
)]
pub async fn unset_not_played(
    State(state): State<AppState>,
    Path((season, id)): Path<(String, String)>,
) -> Result<Json<GamePage>, AppError> {
    not_played(state, season, id, false).await
}

/// "Not played" (`on`) or "Undo" on any game whose page opens (story E, D3). One write, no lock:
/// the last tap wins. The page is read again after the write, with the new mark.
async fn not_played(
    state: AppState,
    season: String,
    id: String,
    on: bool,
) -> Result<Json<GamePage>, AppError> {
    let found = find(&state, &season, &id).await?;
    let mut db = state.pool.acquire().await.map_err(StoreError::from)?;
    if on {
        games::set_not_played(&mut db, found.key()).await?;
    } else {
        games::unset_not_played(&mut db, found.key()).await?;
    }
    drop(db);
    let found = find(&state, &season, &id).await?;
    Ok(Json(page(&state, found).await?))
}

#[utoipa::path(
    put,
    path = "/api/games/{season}/{id}/drivers-drove",
    operation_id = "setDriversDrove",
    params(
        ("season" = String, Path, description = "The season, for example 2026-2027"),
        ("id" = String, Path, description = "The source's game id"),
    ),
    responses(
        (status = 200, description = "\"Count the drives\": the drives count; the game page", body = GamePage),
        (status = 400, description = "`invalid_season`, `invalid_game`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such game of the club chosen for that season"),
    )
)]
pub async fn set_drivers_drove(
    State(state): State<AppState>,
    Path((season, id)): Path<(String, String)>,
) -> Result<Json<GamePage>, AppError> {
    drivers_drove(state, season, id, true).await
}

#[utoipa::path(
    delete,
    path = "/api/games/{season}/{id}/drivers-drove",
    operation_id = "unsetDriversDrove",
    params(
        ("season" = String, Path, description = "The season, for example 2026-2027"),
        ("id" = String, Path, description = "The source's game id"),
    ),
    responses(
        (status = 200, description = "\"Undo\" of \"The drives count\"; the game page", body = GamePage),
        (status = 400, description = "`invalid_season`, `invalid_game`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such game of the club chosen for that season"),
    )
)]
pub async fn unset_drivers_drove(
    State(state): State<AppState>,
    Path((season, id)): Path<(String, String)>,
) -> Result<Json<GamePage>, AppError> {
    drivers_drove(state, season, id, false).await
}

/// "Count the drives" (`on`) or its "Undo", under the row lock of the game. Accepted on any game
/// whose page opens: it has effect only on a not played game from its start time (story E).
async fn drivers_drove(
    state: AppState,
    season: String,
    id: String,
    on: bool,
) -> Result<Json<GamePage>, AppError> {
    let found = find(&state, &season, &id).await?;
    let now = state.clock.now();
    write(&state, &found, async |tx, _| {
        duties::set_drivers_drove(tx, found.key(), on, now).await?;
        Ok(())
    })
    .await?;
    Ok(Json(page(&state, found).await?))
}

#[utoipa::path(
    put,
    path = "/api/games/{season}/{id}/checked",
    operation_id = "setChecked",
    params(
        ("season" = String, Path, description = "The season, for example 2026-2027"),
        ("id" = String, Path, description = "The source's game id"),
    ),
    responses(
        (status = 200, description = "Checked: each \"no answer\" is absent; how many, and the game page", body = CheckedAnswer),
        (status = 400, description = "`invalid_season`, `invalid_game`, `not_a_team_game`, `checked_too_early`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such game of the club chosen for that season"),
    )
)]
pub async fn set_checked(
    State(state): State<AppState>,
    Path((season, id)): Path<(String, String)>,
) -> Result<Json<CheckedAnswer>, AppError> {
    let (found, duties) = team_game(&state, &season, &id).await?;
    check_game_checking(&found.game, in_netherlands(state.clock.now()))?;
    let date = found.game.date;
    let mut tx = state.pool.begin().await.map_err(StoreError::from)?;
    // The list is read under the lock of the game, so an "Undo" of the check on another screen is seen.
    let settled = game_attendance::check(&mut tx, found.key(), date, |attendance, answers| {
        let answers = answers_in_effect(answers, attendance, date);
        to_settle(&game_list(&duties.players, &duties.roles, &answers, date))
    })
    .await?;
    tx.commit().await.map_err(StoreError::from)?;
    tracing::info!(season = %found.season, game = found.game.source_id, settled, "game checked");
    Ok(Json(CheckedAnswer {
        settled,
        page: page(&state, found).await?,
    }))
}

#[utoipa::path(
    delete,
    path = "/api/games/{season}/{id}/checked",
    operation_id = "unsetChecked",
    params(
        ("season" = String, Path, description = "The season, for example 2026-2027"),
        ("id" = String, Path, description = "The source's game id"),
    ),
    responses(
        (status = 200, description = "\"Undo\" of a check: the settled players have no answer again; the game page", body = GamePage),
        (status = 400, description = "`invalid_season`, `invalid_game`, `not_a_team_game`"),
        (status = 401, description = "Not logged in"),
        (status = 404, description = "No such game of the club chosen for that season"),
    )
)]
pub async fn unset_checked(
    State(state): State<AppState>,
    Path((season, id)): Path<(String, String)>,
) -> Result<Json<GamePage>, AppError> {
    let (found, _) = team_game(&state, &season, &id).await?;
    let mut tx = state.pool.begin().await.map_err(StoreError::from)?;
    game_attendance::uncheck(&mut tx, found.key()).await?;
    tx.commit().await.map_err(StoreError::from)?;
    tracing::info!(season = %found.season, game = found.game.source_id, "game check undone");
    Ok(Json(page(&state, found).await?))
}
