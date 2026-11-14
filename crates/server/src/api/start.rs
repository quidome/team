//! `GET /api/start`: the start page (story B of start page and season, and the quieter start
//! page): the open items, the duty games of the last date with duties, and the events of this
//! week and next week with how each looks, in one answer. Each open item comes from the function
//! of its source: the "Game list" card, the list of changes, the page of a game, "Attendance".

use std::collections::BTreeMap;

use axum::Json;
use axum::extract::State;
use serde::Serialize;
use team_core::{
    DutyKind, ENOUGH_PLAYERS, Event, EventKind, EventLook, Game, GameWarnings, HalfCount,
    HomeOrAway, NotCheckedGame, NotCheckedGames, OpenItem, PlacesOf, PlayerId, ReadingState,
    ScheduleReason, Season, duty_games_not_checked, event_look, games_not_checked, next_events,
    open_item_seasons, open_items, open_places_games, schedule_not_entered, team_side,
};
use utoipa::ToSchema;

use super::format::{clock_time, day};
use super::messages::{committee_offered, week_text};
use super::season::{
    EventView, SeasonGames, coming_view, games_of, places_of_season, trainings_of,
};
use crate::app::AppState;
use crate::clock::{current_season, in_netherlands};
use crate::error::AppError;
use crate::{reading, readings};

#[derive(Debug, Serialize, ToSchema)]
pub struct StartAnswer {
    /// Today, the Dutch date, for example "2026-11-10".
    pub today: String,
    /// The header shows the season: in July and August, when the open items also cover the
    /// season before.
    pub show_season: bool,
    /// What still needs the coordinator, in the order of the start page.
    pub open_items: Vec<OpenItemView>,
    /// The games of the team, the home games of other club teams with places of the team, and
    /// the trainings from today until the Sunday of next week, in time order; when there are
    /// none, the first coming one.
    pub next: Vec<StartEvent>,
    /// The messages the start page offers.
    pub messages: MessagesView,
}

/// One event of the start page: the event as on "Season", and how its row looks.
#[derive(Debug, Serialize, ToSchema)]
pub struct StartEvent {
    pub event: EventView,
    /// `lighter`: nothing is left to do (`team_core::event_look`).
    pub look: EventLookView,
    /// Home or away; only for a game of the team.
    pub side: Option<SideView>,
    /// The other team; only for a game of the team.
    pub opponent: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EventLookView {
    Normal,
    Lighter,
}

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SideView {
    Home,
    Away,
}

/// `view` (of `event`) with its look, and for a game of the team its side and opponent.
fn start_event(
    event: &Event,
    view: EventView,
    places: &PlacesOf,
    games: &SeasonGames,
) -> StartEvent {
    let mut start = StartEvent {
        look: EventLookView::Normal,
        side: None,
        opponent: None,
        event: view,
    };
    let checked = start.event.game.as_ref().is_some_and(|game| game.checked);
    let game_places = match event.kind {
        EventKind::Game(scheduled) => places.get(&scheduled.game.source_id),
        EventKind::Training(..) => None,
    };
    if event_look(event, game_places, checked) == EventLook::Lighter {
        start.look = EventLookView::Lighter;
    }
    if let (EventKind::Game(scheduled), Some(club), Some(team)) =
        (event.kind, &games.club, &games.team)
    {
        if scheduled.reason != ScheduleReason::TeamGame {
            return start;
        }
        let (side, opponent) = team_side(&scheduled.game, club.number, team.number);
        start.side = Some(match side {
            HomeOrAway::Home => SideView::Home,
            HomeOrAway::Away => SideView::Away,
        });
        start.opponent = Some(opponent.to_owned());
    }
    start
}

/// Which buttons the part "Messages" shows.
#[derive(Debug, Serialize, ToSchema)]
pub struct MessagesView {
    /// "Message of the week": the same rule as `GET /api/messages/week`.
    pub week: bool,
    /// "Committee message": this week has table or referee places of the team (decision 0034).
    pub committee: bool,
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
    /// Games of the team in their warning period with fewer than `enough` coming.
    TooFewPlayers { games: Vec<TooFewGameView> },
    /// Coming games of the team with drivers who are not coming.
    DriverNotComing { games: Vec<DriverGameView> },
    /// Games of the team in their warning period with answers from before a move.
    AnswersMoved { games: Vec<GameItemView> },
    /// Coming home games of the club whose schedule is not entered, of the half seasons that are
    /// ready (more than 2 club teams have a competition that starts in it), in time order; with
    /// the count of the second half when one of them is in it (decision 0037).
    ScheduleNotEntered {
        games: Vec<GameItemView>,
        second_half: Option<SecondHalfView>,
    },
    /// Coming games with places of the team without a player, in time order: table and referee
    /// places from today to 4 weeks ahead, driver places from today until the Sunday of next week.
    OpenPlaces { games: Vec<OpenGameView> },
    /// Training days before today that were never opened; `first` is the oldest.
    NotOpened { first: String, count: usize },
    /// Trainings before today that are neither cancelled nor checked; `first` is the oldest.
    NotChecked { first: String, count: usize },
    /// Games of the team before today that are not checked, from the start of the season, oldest
    /// first.
    GamesNotChecked { games: Vec<GameItemView> },
    /// Duty games before today that are not checked, oldest first (decision 0035).
    DutyGamesNotChecked { games: Vec<GameItemView> },
}

/// The club teams known for the second half of the season (a competition that starts in it), of
/// the known and the waiting club teams (games in the first half, none in the second).
#[derive(Debug, Serialize, ToSchema)]
pub struct SecondHalfView {
    pub known: usize,
    pub of: usize,
}

impl From<HalfCount> for SecondHalfView {
    fn from(count: HalfCount) -> Self {
        Self {
            known: count.known,
            of: count.known + count.waiting,
        }
    }
}

/// A game of an open item: enough to show it in the list below the item and to open its page.
#[derive(Debug, Serialize, ToSchema)]
pub struct GameItemView {
    /// The season of the game, for example "2026-2027".
    pub season: String,
    /// The source's game id.
    pub id: i64,
    pub date: String,
    pub start_time: Option<String>,
    pub home: String,
    pub away: String,
}

/// A game of the open item "games with open places", with its open places for each kind.
#[derive(Debug, Serialize, ToSchema)]
pub struct OpenGameView {
    #[serde(flatten)]
    pub game: GameItemView,
    pub referee: usize,
    pub table: usize,
    pub drive: usize,
}

/// A game with too few players: the players coming (with the extra players), without an answer,
/// and the number that is enough (7).
#[derive(Debug, Serialize, ToSchema)]
pub struct TooFewGameView {
    #[serde(flatten)]
    pub game: GameItemView,
    pub coming: usize,
    pub no_answer: usize,
    pub enough: usize,
}

/// A game with drivers who are not coming: their names shown, in the order of the places.
#[derive(Debug, Serialize, ToSchema)]
pub struct DriverGameView {
    #[serde(flatten)]
    pub game: GameItemView,
    pub drivers: Vec<String>,
}

impl GameItemView {
    fn of(season: Season, game: &Game) -> Self {
        Self {
            season: season.to_string(),
            id: game.source_id,
            date: day(game.date),
            start_time: game.start_time.map(clock_time),
            home: game.home.team_name.clone(),
            away: game.away.team_name.clone(),
        }
    }
}

impl OpenItemView {
    /// The view of `item`; the games of an item come from `games` (the games of `season`), the
    /// games not checked from `past` (the past games of the seasons of the open items, by season
    /// and id).
    fn of(
        item: OpenItem,
        season: Season,
        games: &BTreeMap<i64, &Game>,
        past: &BTreeMap<(Season, i64), Game>,
        names: &BTreeMap<PlayerId, String>,
    ) -> Self {
        let not_checked = |list: Vec<NotCheckedGame>| -> Vec<GameItemView> {
            list.iter()
                .filter_map(|game| {
                    Some(GameItemView::of(
                        game.season,
                        past.get(&(game.season, game.id))?,
                    ))
                })
                .collect()
        };
        let item_of = |id: &i64| games.get(id).map(|game| GameItemView::of(season, game));
        match item {
            OpenItem::ReadingHeldBack => Self::ReadingHeldBack,
            OpenItem::OutOfDate { since } => Self::OutOfDate { since: day(since) },
            OpenItem::ChangesNotSeen { count } => Self::Changes { count },
            OpenItem::TooFewPlayers { games: short } => Self::TooFewPlayers {
                games: short
                    .iter()
                    .filter_map(|(id, count)| {
                        Some(TooFewGameView {
                            game: item_of(id)?,
                            coming: count.coming,
                            no_answer: count.no_answer,
                            enough: ENOUGH_PLAYERS,
                        })
                    })
                    .collect(),
            },
            OpenItem::DriverNotComing { games: drivers } => Self::DriverNotComing {
                games: drivers
                    .iter()
                    .filter_map(|(id, players)| {
                        Some(DriverGameView {
                            game: item_of(id)?,
                            drivers: players
                                .iter()
                                .map(|player| names.get(player).cloned().unwrap_or_default())
                                .collect(),
                        })
                    })
                    .collect(),
            },
            OpenItem::AnswersMoved { games: moved } => Self::AnswersMoved {
                games: moved.iter().filter_map(item_of).collect(),
            },
            OpenItem::ScheduleNotEntered {
                games: ids,
                second_half,
            } => Self::ScheduleNotEntered {
                games: ids
                    .iter()
                    .filter_map(|id| games.get(id))
                    .map(|game| GameItemView::of(season, game))
                    .collect(),
                second_half: second_half.map(SecondHalfView::from),
            },
            OpenItem::OpenPlaces { games: open } => Self::OpenPlaces {
                games: open
                    .iter()
                    .filter_map(|open| {
                        let count = |kind| open.open.get(&kind).copied().unwrap_or(0);
                        Some(OpenGameView {
                            game: GameItemView::of(season, games.get(&open.id)?),
                            referee: count(DutyKind::Referee),
                            table: count(DutyKind::Table),
                            drive: count(DutyKind::Drive),
                        })
                    })
                    .collect(),
            },
            OpenItem::NotOpened { first, count } => Self::NotOpened {
                first: day(first),
                count,
            },
            OpenItem::NotChecked { first, count } => Self::NotChecked {
                first: day(first),
                count,
            },
            OpenItem::GamesNotChecked { games } => Self::GamesNotChecked {
                games: not_checked(games),
            },
            OpenItem::DutyGamesNotChecked { games } => Self::DutyGamesNotChecked {
                games: not_checked(games),
            },
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/start",
    operation_id = "start",
    responses(
        (status = 200, description = "The open items and the next events", body = StartAnswer),
        (status = 401, description = "Not logged in"),
    )
)]
pub async fn get(State(state): State<AppState>) -> Result<Json<StartAnswer>, AppError> {
    let clock = state.clock.as_ref();
    let now = in_netherlands(clock.now());
    let today = now.date();
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

    // The duties of the games: the rule of "Season" and of the page of a game (decision 0022).
    let places = places_of_season(&games, now);
    let events = match (lists.first(), inputs.first()) {
        (Some(list), Some(trainings)) => trainings.events(&games.schedule, list),
        _ => Default::default(),
    };
    // The warnings of the coming games of the team: the rule of the page of a game.
    let warnings: Vec<(i64, GameWarnings)> = games
        .schedule
        .coming
        .iter()
        .filter(|scheduled| scheduled.reason == ScheduleReason::TeamGame)
        .map(|scheduled| {
            (
                scheduled.game.source_id,
                games.duties.warnings(scheduled, now),
            )
        })
        .collect();
    // The past games of the team and the past duty games that are not checked, in the seasons of
    // the open items, from the start of each season.
    let mut not_checked = games_not_checked(
        current,
        &games.schedule.past,
        &games.duties.attendance.games,
        today,
    );
    let mut duty_not_checked = duty_games_not_checked(
        current,
        &games.schedule.past,
        &places,
        &games.duties.stored,
        today,
    );
    // By season and id: the key of a game (`GameKey`), so two seasons never mix.
    let mut past: BTreeMap<(Season, i64), Game> = games
        .schedule
        .past
        .iter()
        .map(|scheduled| ((current, scheduled.game.source_id), scheduled.game.clone()))
        .collect();
    for &season in seasons.iter().skip(1) {
        let earlier = games_of(&state, season, current, today).await?;
        let earlier_places = places_of_season(&earlier, now);
        not_checked.extend(games_not_checked(
            season,
            &earlier.schedule.past,
            &earlier.duties.attendance.games,
            today,
        ));
        duty_not_checked.extend(duty_games_not_checked(
            season,
            &earlier.schedule.past,
            &earlier_places,
            &earlier.duties.stored,
            today,
        ));
        past.extend(
            earlier
                .schedule
                .past
                .iter()
                .map(|scheduled| ((season, scheduled.game.source_id), scheduled.game.clone())),
        );
    }
    let items = open_items(
        &reading,
        games.unseen_changes,
        schedule_not_entered(&events, &places, &games.halves),
        open_places_games(&events, &places, today),
        &lists,
        &warnings,
        NotCheckedGames {
            games: &not_checked,
            duty_games: &duty_not_checked,
        },
    );
    let next = next_events(&events, &places, today)
        .into_iter()
        .map(|event| {
            coming_view(event, &places, &games.duties, true, now)
                .map(|view| start_event(event, view, &places, &games))
        })
        .collect::<Result<_, _>>()?;

    let messages = MessagesView {
        week: week_text(&games, &places, today).is_some(),
        committee: committee_offered(&games, &places, today),
    };
    let by_id: BTreeMap<i64, &Game> = games
        .schedule
        .coming
        .iter()
        .map(|scheduled| (scheduled.game.source_id, &scheduled.game))
        .collect();
    Ok(Json(StartAnswer {
        today: day(today),
        show_season: seasons.len() > 1,
        open_items: items
            .into_iter()
            .map(|item| OpenItemView::of(item, current, &by_id, &past, &games.duties.names))
            .collect(),
        next,
        messages,
    }))
}
