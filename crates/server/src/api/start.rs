//! `GET /api/start`: the start page (story B of start page and season): the open items, the last
//! training and the last games with places of the team, and the events of this week and next
//! week, in one answer. Each open item comes from the function of its source: the "Game list"
//! card, the list of changes, the page of a game, "Attendance".

use std::collections::BTreeMap;

use axum::Json;
use axum::extract::State;
use serde::Serialize;
use team_core::{
    DutyKind, ENOUGH_PLAYERS, Event, EventKind, Game, GameWarnings, NotCheckedGame, OpenItem,
    PlayerId, ReadingState, ScheduleReason, Season, games_not_checked, last_games, last_training,
    next_events, open_item_seasons, open_items, open_places_games, schedule_not_entered,
};
use utoipa::ToSchema;

use super::format::{clock_time, day};
use super::messages::{committee_text, week_text};
use super::season::{
    EventView, coming_view, event_view, event_view_with, games_of, places_of_season, trainings_of,
};
use crate::app::AppState;
use crate::clock::{current_season, in_netherlands};
use crate::db::StoreError;
use crate::error::AppError;
use crate::{game_attendance, reading, readings};

#[derive(Debug, Serialize, ToSchema)]
pub struct StartAnswer {
    /// What still needs the coordinator, in the order of the start page.
    pub open_items: Vec<OpenItemView>,
    /// The last training before today that is not cancelled, as an event; `null` when there is
    /// none.
    pub last: Option<EventView>,
    /// The games of the team, the home games of other club teams with places of the team, and
    /// the trainings from today until the Sunday of next week, in time order; when there are
    /// none, the first coming one.
    pub next: Vec<EventView>,
    /// The games of the last date before today with places of the team, in time order, in the
    /// seasons of the open items (in July and August also the season before).
    pub last_games: Vec<EventView>,
    /// The season of `last_games`, for their pages; `null` when there are none.
    pub last_games_season: Option<String>,
    /// The messages the start page offers.
    pub messages: MessagesView,
}

/// Which buttons the part "Messages" shows.
#[derive(Debug, Serialize, ToSchema)]
pub struct MessagesView {
    /// "Message of the week": the same rule as `GET /api/messages/week`.
    pub week: bool,
    /// "Committee message": the same rule as `GET /api/messages/committee`.
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
    /// Home games of the club from today to 4 weeks ahead whose schedule is not entered, in time
    /// order.
    ScheduleNotEntered { games: Vec<GameItemView> },
    /// Coming games with places of the team without a player, in time order: table and referee
    /// places from today to 4 weeks ahead, driver places from today until the Sunday of next week.
    OpenPlaces { games: Vec<OpenGameView> },
    /// Training days before today that were never opened; `first` is the oldest.
    NotOpened { first: String, count: usize },
    /// Trainings before today that are neither cancelled nor checked; `first` is the oldest.
    NotChecked { first: String, count: usize },
    /// Games of the team before today, from the day of the release, that are not checked;
    /// `first` is the oldest, for its page.
    GamesNotChecked {
        first: NotCheckedGameView,
        count: usize,
    },
}

/// The oldest game of the open item "games not checked".
#[derive(Debug, Serialize, ToSchema)]
pub struct NotCheckedGameView {
    /// The season of the game, for example "2026-2027".
    pub season: String,
    /// The source's game id.
    pub id: i64,
    pub date: String,
}

impl From<NotCheckedGame> for NotCheckedGameView {
    fn from(game: NotCheckedGame) -> Self {
        Self {
            season: game.season.to_string(),
            id: game.id,
            date: day(game.date),
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
    /// The view of `item`; the games of an item come from `games` (the games of `season`).
    fn of(
        item: OpenItem,
        season: Season,
        games: &BTreeMap<i64, &Game>,
        names: &BTreeMap<PlayerId, String>,
    ) -> Self {
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
            OpenItem::ScheduleNotEntered { games: ids } => Self::ScheduleNotEntered {
                games: ids
                    .iter()
                    .filter_map(|id| games.get(id))
                    .map(|game| GameItemView::of(season, game))
                    .collect(),
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
            OpenItem::GamesNotChecked { first, count } => Self::GamesNotChecked {
                first: first.into(),
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
    // The past games of the team that are not checked, in the seasons of the open items, from
    // the day of the release.
    let from_date = game_attendance::check_start(&state.pool).await?;
    let mut not_checked = games_not_checked(
        current,
        &games.schedule.past,
        &games.duties.attendance.games,
        from_date,
        today,
    );
    for &season in seasons.iter().skip(1) {
        let earlier = games_of(&state, season, current, today).await?;
        not_checked.extend(games_not_checked(
            season,
            &earlier.schedule.past,
            &earlier.duties.attendance.games,
            from_date,
            today,
        ));
    }
    let items = open_items(
        &reading,
        games.unseen_changes,
        schedule_not_entered(&events, &places, today),
        open_places_games(&events, &places, today),
        &lists,
        &warnings,
        &not_checked,
    );
    let last = last_training(&lists)
        .map(|row| {
            // The trainings of the season of the row hold its absences.
            let (_, trainings) = seasons
                .iter()
                .zip(&inputs)
                .find(|(season, _)| **season == Season::of(row.date))
                .ok_or_else(|| StoreError::Invalid(format!("no trainings for {}", row.date)))?;
            let state = trainings.state(row);
            event_view(&Event {
                date: row.date,
                start: Some(row.start),
                kind: EventKind::Training(row, state),
            })
        })
        .transpose()?;
    let next = next_events(&events, &places, today)
        .into_iter()
        .map(|event| coming_view(event, &places, &games.duties, true, now))
        .collect::<Result<_, _>>()?;

    // The last games with places: the current season; in July and August, when it has none
    // yet, the season before.
    let mut last_games_season = None;
    let mut last_game_views: Vec<EventView> = last_games(&events, &places)
        .into_iter()
        .map(|event| event_view_with(event, &places, &games.duties))
        .collect::<Result<_, _>>()?;
    if last_game_views.is_empty() {
        for (&season, trainings) in seasons.iter().zip(&inputs).skip(1) {
            let earlier = games_of(&state, season, current, today).await?;
            let earlier_places = places_of_season(&earlier, now);
            let list = trainings.list(season, today);
            let earlier_events = trainings.events(&earlier.schedule, &list);
            last_game_views = last_games(&earlier_events, &earlier_places)
                .into_iter()
                .map(|event| event_view_with(event, &earlier_places, &earlier.duties))
                .collect::<Result<_, _>>()?;
            if !last_game_views.is_empty() {
                last_games_season = Some(season);
                break;
            }
        }
    } else {
        last_games_season = Some(current);
    }
    let messages = MessagesView {
        week: week_text(&games, &places, today).is_some(),
        committee: committee_text(&games, &places, today).is_some(),
    };
    let by_id: BTreeMap<i64, &Game> = games
        .schedule
        .coming
        .iter()
        .map(|scheduled| (scheduled.game.source_id, &scheduled.game))
        .collect();
    Ok(Json(StartAnswer {
        open_items: items
            .into_iter()
            .map(|item| OpenItemView::of(item, current, &by_id, &games.duties.names))
            .collect(),
        last,
        next,
        last_games: last_game_views,
        last_games_season: last_games_season.map(|season| season.to_string()),
        messages,
    }))
}
