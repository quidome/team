# Technical spec: The start page as a landing page

**Status**: approved
**Size**: story
**Change**: [2026-10-25-landing-page](./)
**Date**: 2026-10-25

## Objective
A start page that answers in a few seconds "must I do something, and what comes next?": the open
items, the last training and the events of this week and next week, in one request; the schedule,
the card of the next training, the "Game list" card and the view "Last season" leave it.

## Requirements summary
Story B of the epic [Start page and season](../2026-10-25-start-page-and-season/) (`functional.md`
approved 2026-10-25, story B; epic `technical.md` approved 2026-10-25):

- **With a club and team chosen**, from the top: the team header (team, season, "Change"); "Open items"; "Last"; "Next"; the links "Season · Attendance · Team · Players · Settings".
- **"Open items"**, one row per item, in this order of kinds, oldest first within a kind:
  1. "The last reading of the game list was held back." (a held-back reading waits) and "The games may be out of date since Thu 22 Oct." (3 nights without a good reading, the rule of now); a tap goes to the "Game list" card on "Season".
  2. "4 changes not seen" ("1 change not seen"); a tap opens the list of changes.
  3. "Training day Thu 22 Oct not opened" for one, "3 training days not opened" for more; a tap opens the oldest. The same days as the line on "Attendance".
  4. "Training Thu 22 Oct not checked" for one, "2 trainings not checked" for more; a tap opens the oldest.
  Without open items: "Nothing open."
- **Which seasons and dates**: the current season and, in July and August, the season before; only dates before today (Dutch date); a cancelled training or training day is never an open item; today's training is in "Next".
- **"Last"**: the newest training before today that is not cancelled, in the same seasons as the open items, as on "Season" (for example "Training 19:45 · 2 absent · Checked"), also when checked; a tap opens it. No training before today: no "Last".
- **"Next"**: the games of the team and the training days and trainings from today until the Sunday of next week (on a Sunday: today and the next week), grouped by date, each as on "Season"; no possible duty games; a tap on a training opens it. Nothing in those two weeks: the first coming event after them; nothing at all: "No coming events."
- **Leaving the start page**: the schedule, the card of the next training, the "Game list" card, the line "N changes not seen" under the team name, the view "Last season".
- **Edge cases**: no club chosen: only the choose step, as now. A new season without a team: the choose step, then "Open items", "Last" and "Next" with trainings only. A failure: "The start page could not be loaded. Load the page again." A training day that is no longer a training day and was never opened is no open item (as on "Attendance").

## Current state
- **Start page** `web/src/routes/+page.ts` loads the choice, `GET /api/readings/latest`, `GET /api/schedule` and `GET /api/trainings/next`; `+page.svelte` shows the choose step (`ChooseTeam.svelte`), the team header with the line `unseenLine(schedule.unseen_changes)` (`changes.ts`), the links, the card of the next training (`openNext`, `?from=start`), the "Game list" card (`ReadGameList.svelte`, `refreshSchedule`), `Schedule.svelte`, and in a new season "Last season" with last season's schedule.
- **Readings** `api/readings.rs` `latest`: works out `held_back` (`readings::newest_not_failed`, when held back) and `out_of_date_since` (`nightly::out_of_date`, only with a team; the `finished_at` of the last good reading) inside the handler; the target from `reading::target` (the club of the current choice, or of the newest earlier one with no team).
- **Season** `api/season.rs` `get`: the choice of the season, the games (`games::stored`, the open changes of `api::changes::list` in the current season, `team_core::schedule`), the trainings (weekly trainings, trainings, absences, roles, `season_list`), `season_events`, `EventView` (with `ScheduleGame` from `api/schedule.rs` and `TrainingRow` from `api/trainings.rs`). `api/attendance.rs` and `next_of` in `api/trainings.rs` read the trainings the same way (technical debt: three times).
- **Core**: `training::{not_opened, not_checked, season_list}`, `season_view::{season_events, SeasonEvents, Event, EventKind}`, `schedule::ScheduleReason`, `nightly::out_of_date`, `Season::of`, `Season::first_day`.
- **Front end**: `SeasonEvents.svelte` (week and date headings, game rows, training rows with "Cancel"), `formatDay` (`schedule.ts`), `statusText` and `checkedText` (`trainings.ts`); the list of changes uses `FilterToggle.svelte`, `filters` and `filterGames`.
- **Tests**: `tests/schedule.rs` (9), `tests/trainings.rs` `the_card_is_the_next_training`; browser: `choose.e2e.ts` (the start page after choosing, "the schedule" block of 9 tests, the "Game list" card), `logged-in.e2e.ts` ("an earlier season with no kept games", screenshots), `trainings.e2e.ts` (the card, lines 9 and 132), `season.e2e.ts` (its first test chooses the team through `/choose`).

## Steps

### Step 1: Core `start_page`
- **Goal**: the rules of the start page, without I/O.
- **Changes**: `crates/core/src/start_page.rs` (new), exported in `lib.rs`:
  ```rust
  /// The state of the game list for the open items (Dutch dates).
  pub struct ReadingState { pub held_back: bool, pub out_of_date_since: Option<NaiveDate> }
  pub enum OpenItem {
      ReadingHeldBack,
      OutOfDate { since: NaiveDate },
      ChangesNotSeen { count: u32 },
      NotOpened { first: NaiveDate, count: usize },
      NotChecked { first: NaiveDate, count: usize },
  }
  /// The current season of `today`; in July and August also the season before.
  pub fn open_item_seasons(today: NaiveDate) -> Vec<Season>;
  /// In the order of the spec; a kind without items gives nothing. Over all `lists`: `count` is
  /// the sum, `first` the oldest date (`not_opened`, `not_checked`).
  pub fn open_items(reading: &ReadingState, unseen: u32, lists: &[SeasonList<'_>]) -> Vec<OpenItem>;
  /// The newest row before today with a training that is not cancelled, over all `lists`.
  pub fn last_training<'a>(lists: &'a [SeasonList<'a>]) -> Option<&'a SeasonRow<'a>>;
  /// The games of the team and the training rows from today until the Sunday of next week; when
  /// there are none, the first coming one of those kinds after it. No possible duty games.
  pub fn next_events<'a>(events: &'a SeasonEvents<'a>, today: NaiveDate) -> Vec<&'a Event<'a>>;
  ```
  The end of the window: `today + (6 − today.weekday().num_days_from_monday()) + 7` days.
- **Tests** (core):
  - `open_item_seasons`: 30 June 2027 → 2026; 1 July 2027 → 2027 and 2026; 31 August 2027 → both; 1 September 2027 → 2027.
  - `open_items_come_in_the_order_of_the_spec`
  - `one_and_three_training_days_not_opened` (`count` 1 with its date; `count` 3 with the oldest)
  - `the_days_of_two_seasons_add_up` (one day in each season: `count` 2, the oldest `first`)
  - `nothing_open_gives_no_items`; `a_cancelled_training_is_never_an_open_item`
  - `last_training`: the newest past training also when checked; a cancelled training and a training day not opened are skipped; on 2 July it finds 30 June; `None` without any.
  - `next_events`: Mon 26 Oct 2026 → up to Sun 8 Nov; Sun 25 Oct 2026 → up to Sun 1 Nov; a possible duty game left out; nothing in the window → the first coming event; nothing at all → empty.

### Step 2: Shared reads
- **Goal**: one rule for the state of the game list, and one place for the reads of a season; no change of behaviour.
- **Changes**:
  - `crates/server/src/readings.rs`:
    ```rust
    pub struct ReadingStatus {
        pub latest: Option<Reading>, pub last_good: Option<Reading>,
        pub held_back: Option<Reading>, pub out_of_date_since: Option<DateTime<Utc>>,
    }
    /// The state of the game list of `target`: the held-back reading the coordinator can confirm,
    /// and the moment of the last good reading when the games may be out of date (only with a team).
    pub async fn status(pool: &PgPool, clock: &dyn Clock, target: &Target) -> Result<ReadingStatus, StoreError>;
    ```
    `GET /api/readings/latest` maps it as now.
  - `crates/server/src/api/season.rs`:
    ```rust
    /// The inputs of the training list of `season`; `season_list` borrows from it.
    pub(crate) struct SeasonTrainings { weekly, trainings, absences, roles }
    pub(crate) async fn trainings_of(state: &AppState, season: Season) -> Result<SeasonTrainings, AppError>;
    /// The games of `season` for its own choice (gone games while their change is open in the
    /// current season), the state of the games and the choice.
    pub(crate) async fn games_of(state: &AppState, season: Season, current: Season, today: NaiveDate)
        -> Result<(GamesState, Option<FollowedTeam>, Schedule), AppError>;
    ```
    `GET /api/season` and `GET /api/attendance` use them (and `GET /api/start` in step 3).
- **Tests**: `tests/readings.rs`, `tests/season.rs`, `tests/attendance.rs` unchanged.

### Step 3: `GET /api/start`
- **Goal**: the answer of the start page in one request.
- **Changes**: `crates/server/src/api/start.rs` (new):
  ```rust
  pub struct StartAnswer {
      pub open_items: Vec<OpenItemView>,
      /// The last training before today; `null` when there is none.
      pub last: Option<TrainingRow>,
      /// The events of "Next", in time order.
      pub next: Vec<EventView>,
  }
  #[serde(tag = "kind", rename_all = "snake_case")]
  pub enum OpenItemView {
      ReadingHeldBack, OutOfDate { since: String }, Changes { count: u32 },
      NotOpened { first: String, count: usize }, NotChecked { first: String, count: usize },
  }
  ```
  The handler: `today`, `current`; `reading::target` and, with a target, `readings::status` → `ReadingState` (`held_back.is_some()`, the Dutch date of `out_of_date_since`); without a target no reading items. The changes not seen: with a team chosen in the current season, `api::changes::list(...).len()`, else 0. For each season of `open_item_seasons(today)`: `trainings_of` and `season_list`; then `open_items` and `last_training`. "Next": `games_of(current)` and `trainings_of(current)`, `season_events`, `next_events`; each as `EventView`. All or nothing; 401 without a login. The route in `app.rs` and `api/mod.rs`; `just api-types`. Measure the answer time locally (look again above about 200 ms) and write it under **Progress**.
- **Tests** (`crates/server/tests/start.rs`, new, with `tests/common/game_list.rs`):
  - `without_login_the_start_page_needs_a_login` (401)
  - `a_held_back_reading_is_an_open_item` (as `held_back` of `GET /api/readings/latest`)
  - `out_of_date_is_an_open_item_with_the_dutch_date` (the clock 3 nights on)
  - `changes_not_seen_are_counted_as_the_list_of_changes` (as `GET /api/changes`)
  - `training_days_not_opened_are_those_of_attendance` (as `not_opened` and `first_not_opened` of `GET /api/attendance`)
  - `a_past_training_not_checked_is_an_open_item_until_it_is_checked`
  - `last_is_the_newest_past_training_also_when_checked`
  - `next_has_the_team_games_and_trainings_of_two_weeks` (no possible duty games, in time order)
  - `a_new_season_without_a_team_has_trainings_only` (clock 1 July 2027)
  - `in_july_the_open_items_and_last_cover_the_season_before`
  - `tests/players.rs`: `GET /api/start` needs a login; `tests/player_logs.rs` calls it.

### Step 4: The start page
- **Goal**: the new start page.
- **Changes**:
  - `web/src/lib/components/EventRow.svelte` (new): one row of an event (a game or a training), taken out of `SeasonEvents.svelte`, with the props `event`, `busy`, `onOpen`, `onCancel?` (no "Cancel" without it) and `from?` (added to the training link). `SeasonEvents.svelte` uses it.
  - `web/src/lib/start.ts` (new): `openItemText(item)` ("The last reading of the game list was held back.", "The games may be out of date since Thu 22 Oct.", `unseenLine` moved here from `changes.ts`, "Training day Thu 22 Oct not opened" / "3 training days not opened", "Training Thu 22 Oct not checked" / "2 trainings not checked"); `openItemLink(item)` (`/season#game-list`, `/changes`, `/training/<first>?from=start`).
  - `client.ts`: `loadStart(fetch)` and the types `StartAnswer`, `OpenItemView`.
  - `+page.ts`: the choice; with `chosen` or `team_needed`, `loadStart`; a failure gives `startError`.
  - `+page.svelte`: `not_chosen` the choose step and the links, as now; `team_needed` the choose step, then the three parts and the links; `chosen` the team header with "Change", then "Open items" ("Nothing open." when empty), "Last" (one `EventRow`, `from="start"`), "Next" (grouped by date with `formatDay`, `EventRow` with `from="start"`; "No coming events." when empty), the links. A tap on a training that is not opened opens it first, as on "Season". `startError`: "The start page could not be loaded. Load the page again."
  - "Season": `id="game-list"` on the "Game list" card; on arrival with `#game-list` the page scrolls to it after it is drawn.
- **Tests** (browser): the order of the parts; "Nothing open." (`page.route`); each open item opens its place (the list of changes; `/season#game-list` with the card in view; "Training day … not opened" with "Open training"; "Training … not checked"); "Last" opens the last training, also after "Checked"; "Next": a tap opens the training and "Back" returns to the start page; the load error (`page.route` 500); small phone, accessibility, screenshots. `trainings.e2e.ts` lines 9 and 132 become tests of "Next".

### Step 5: The old tests of the schedule
- **Goal**: every test of the start page's schedule and "Game list" card tests the same on "Season" or on the new start page.
- **Changes**:
  - `choose.e2e.ts`: after choosing, the start page shows "Open items" and "Next"; "reading the game list again" and "out of date after 3 nights" use the card on `/season` (the latter also checks the open item); "the schedule" block moves to `/season` with the labels ("Games of the team", "Possible duty games", "Past"): the 23 coming games and the first day, "time not known yet" last in its day, the empty lines, the load error after a reading, accessibility and screenshots; the test of the line of changes moves to the open item.
  - `logged-in.e2e.ts`: "an earlier season with no kept games says which season" goes (the view "Last season" leaves; "Season" tests a season with no events); screenshots of the start page again.
  - `season.e2e.ts`: the first test lands on the new start page after choosing.
  - If a test has no place, stop and ask the coordinator before removing it.
- **Tests**: the moved tests pass.

### Step 6: Remove the old parts
- **Goal**: no dead code.
- **Changes**: remove `GET /api/schedule` (`api/schedule.rs`; `ScheduleGame` and `ScheduleReasonView` move to `api/season.rs`), `GET /api/trainings/next` and `next_of`; `Schedule.svelte`, `loadSchedule`, `loadNextTraining`, `noGamesLine`, `groupByDate`; `tests/schedule.rs` (its case "a start time that is not known comes last in its date" moves to `tests/season.rs`; "a team needed and no earlier team" is in `tests/start.rs`), `the_card_is_the_next_training`; `tests/players.rs` without the two routes. `just api-types`.
- **Tests**: `just check`.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| Each open item comes from the function of its source: `not_opened` (as "Attendance"), `not_checked`, `readings::status` (as the "Game list" card), `api::changes::list` (as the list of changes) | One number everywhere; nothing counted on the side | Yes, 0022 at the close of the epic |
| One route `GET /api/start` | One request on the phone; the rules in the core | In 0022 |
| "Last" and the open items use `open_item_seasons` | The record of June stays in view in July and August (coordinator) | In 0022 |
| One `EventRow.svelte` for "Season" and the start page | "Each as on Season" by construction | No |
| The shared reads `trainings_of`, `games_of`, `readings::status` | Removes the debt of three copies; the start page needs the same reads | No |

## Not in scope
- The last game on the start page (with duties); duty items; open items of seasons further back; links from "Attendance" or "Season" to the other pages.

## Test plan
- **Existing tests that must still pass**: all core tests; `tests/readings.rs`, `tests/season.rs`, `tests/attendance.rs`, `tests/changes.rs`, `tests/trainings.rs` (without the card test); the browser tests that do not touch the start page.
- **Changed or removed**: `tests/schedule.rs` (goes, one case to `tests/season.rs`), `the_card_is_the_next_training` (goes), `tests/players.rs`, `tests/player_logs.rs`; `choose.e2e.ts`, `logged-in.e2e.ts`, `trainings.e2e.ts`, `season.e2e.ts` (steps 4 and 5).
- **New tests**: per step, above.
- **Not testable in the browser** (clock fixed at 12 October 2026): July and August, the clock change, a new season without a team (core and server only).
- **Checks while working**: `cargo test -p team_core`; `cargo test -p team --test start --test readings --test season --test changes`; `just check-web`, then `npx playwright test` with the projects that changed; `cargo clippy --workspace --all-targets -- -D warnings` at the end of each step; `just check` once before the commit.

## Compatibility and performance
- The start page changes completely; the schedule and the "Game list" card are on "Season"; the view "Last season" goes (the earlier season is on "Season").
- `GET /api/schedule` and `GET /api/trainings/next` are removed (only the front end calls them).
- `GET /api/start` reads the choice, the readings, the open changes with the stored games, and the trainings of one or two seasons: about 10 queries; measured in step 3.
- Technical debt removed: the game rows twice; the reads of a season's trainings three times; the line about the links of "Attendance" and "Season".

## Progress
- [ ] Step 1 – core `start_page`
- [ ] Step 2 – shared reads
- [ ] Step 3 – `GET /api/start`
- [ ] Step 4 – the start page
- [ ] Step 5 – the old tests of the schedule
- [ ] Step 6 – remove the old parts
- [ ] All checks pass

## Deviations
None from `functional.md`. Readings that the design makes exact (no change of behaviour):
- "A tap goes to the Game list card on Season": `/season#game-list`, scrolled into view.
- The training items, "Last" and "Next" link with `?from=start`, so "Back" returns to the start page.
- "Last" shows the training row as on "Season" without its date in the row text; the date shows as the label of "Last" ("Thu 22 Oct").
- Six steps, not five: moving the browser tests is its own step.

## Review findings
