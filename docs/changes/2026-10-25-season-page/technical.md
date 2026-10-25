# Technical spec: The page "Season"

**Status**: approved
**Size**: story
**Change**: [2026-10-25-season-page](./)
**Date**: 2026-10-25

## Objective
A page "Season" with all events of a season (games of the team, possible duty games, training
days and trainings) in one list with week and date headings and labels that combine, so any game
or training is found there; the list of trainings with its actions moves here from "Attendance".

## Requirements summary
Story A of the epic [Start page and season](../2026-10-25-start-page-and-season/) (`functional.md`
approved 2026-10-25; epic `technical.md` approved 2026-10-25):

- **Address**: `/season?season=2025-2026`; the current season when left out; a reload keeps it. The start page links to "Season", first in the line of links ("Season · Attendance · Team · Players · Settings").
- **Events**: all games of the team (home and away), all possible duty games (home games of other club teams; away games of other club teams are not shown), all training days and trainings of the season.
- **Headings**: per ISO week, Monday to Sunday, "Week 44 · 26 Oct – 1 Nov"; within it per date, "Sat 31 Oct". A week without events has no heading.
- **Labels**: What ("Games of the team", "Possible duty games", "Trainings") and When ("Coming", "Past"). Labels of different groups narrow the list; labels of one group add to it; a group with no label on shows all of that group. One tap turns a label on or off, without a new load. The page opens with "Coming" on and no What label, at each visit (labels are not kept). The labels that are on are always visible.
- **Order**: the part "Coming" (today and later, Dutch date) nearest first; with "Past" on, the part "Past", newest week and date first. With both on, two parts with their own headings; the week of today can have a heading in both. Within a date: known start times in time order, then unknown.
- **A game row** as the schedule now: start time or "time not known yet", home team, away team, hall and town, the badge "Team" and the team colour for a game of the team; a gone game crossed out with "no longer in the game list" while its change is in the list of changes. A tap on a game does nothing (no page of a game yet).
- **A training row** as "Attendance" now: start time, "Not opened", "All there", "N absent" or "Cancelled", "extra". A tap opens the training (a training day not opened becomes a training).
- **Moved from "Attendance"**: "Add training" (only in the current season), "Cancel" on a training day not opened, "Show cancelled" (on by default, kept on each phone or computer).
- **Season choice**: the current season and the earlier seasons with games or trainings (shown when there is more than one). `?season=abc`: "This season cannot be read." with "Current season".
- **The "Game list" card** at the top of the current season, with its behaviour of now; after a good reading the list shows the new games at once.
- **No team** in the current season (a new season): the trainings, and "Choose the team to see its games." with a link to the start page (where the choose step is). An earlier season with no kept games or no choice: only its trainings.
- **Empty**: a season with no events: "No games or trainings in 2026-2027 yet."; a part with no events under the labels: "No coming trainings.", "No past possible duty games.", "No coming events.", and so on.
- **"Back"** on a training returns to the page it came from: the start page (the card of the next training, which stays until story B) or "Season" of the training's season. After "Delete training": "Season" of that season.
- **"Attendance"** keeps the season choice, the line of training days not opened and "Totals"; the list "Trainings", "Add training", "Cancel" and "Show cancelled" go.
- The start page keeps its schedule, its cards and the view "Last season" until story B.

## Current state
- **Core**:
  - `schedule::schedule(stored, club, team, today, open) -> Schedule { coming, past }` with `ScheduledGame { game, reason, in_list }`; coming in time order, past newest date first, within a date known start times first, then by source id.
  - `training::season_list(season, weekly, trainings, absences, roles, today) -> SeasonList { coming, past }` of `SeasonRow { date, start, end, training, extra, status: DayStatus }`, past newest first.
  - `attendance::season_overview` works out `not_opened` and `first_not_opened` itself from `season_list(...).past` (rows with `DayStatus::NotOpened`).
- **Server**:
  - `api/schedule.rs`: `GET /api/schedule`, `ScheduleGame` with `From<ScheduledGame>`; the open changes from `api::changes::list(state, db, club, team, season, &stored)`.
  - `api/trainings.rs`: `TrainingRow`, `row_view(&SeasonRow)`, `list_of(state, season) -> TrainingList` (used by `GET /api/trainings?season=` and `GET /api/trainings/next`), `SeasonQuery`; `query_season` in `api/format.rs`.
  - `api/attendance.rs`: the seasons are `trainings::seasons` + the current + the chosen season, newest first.
  - `followed_teams::{for_season, newest_before}`; no list of seasons yet.
- **Front end**:
  - Start page: links "Attendance · Team · Players · Settings"; the card of the next training (`openNext`: opens the day when needed, then `goto('/training/<date>')`); `ReadGameList.svelte` with `onRead`.
  - `/attendance`: loads `/api/trainings` and `/api/attendance`; the list with "Add training", "Cancel", "Show cancelled" (kept in `localStorage`).
  - `/training/[date]`: "Back" to `/attendance?season=<season>` (`attendanceOf`); after "Delete training" also there.
  - `Schedule.svelte` (game rows), `schedule.ts` (`formatDay`, `noGamesLine`, `groupByDate`), `trainings.ts` (`statusText`).
  - No front-end unit tests: behaviour is tested with Playwright.
- **Tests**: `tests/trainings.rs` reads `GET /api/trainings` in `the_list_has_one_row_per_date`, `a_date_or_season_that_cannot_be_read_is_refused` and after add, cancel, undo and delete (lines 119, 143, 247, 358, 460, 467, 533). `web/e2e/trainings.e2e.ts` uses `/attendance` for the list and its actions (lines 32–118, 416–450, 468, 507, 556, 566).

## Steps

### Step 1: Core – the order of events, and one rule for "not opened"
- **Goal**: games and trainings of a season in one list in the right order with their ISO week; the training days not opened from one function.
- **Changes**:
  - `crates/core/src/training.rs`:
    ```rust
    /// The training days before today that were never opened, oldest first.
    pub fn not_opened(list: &SeasonList<'_>) -> Vec<NaiveDate>
    ```
    the past rows with `DayStatus::NotOpened`, reversed (`past` is newest first).
  - `crates/core/src/attendance.rs`: `season_overview` uses it: `overview.not_opened = days.len(); overview.first_not_opened = days.first().copied();` (same result as now).
  - `crates/core/src/season_view.rs` (new), exported in `lib.rs`:
    ```rust
    pub enum EventKind<'a> { Game(&'a ScheduledGame), Training(&'a SeasonRow<'a>) }
    pub struct Event<'a> {
        pub date: NaiveDate,
        /// `None` only for a game whose start time is not known.
        pub start: Option<NaiveTime>,
        pub kind: EventKind<'a>,
    }
    impl Event<'_> {
        /// The ISO week number, its Monday and its Sunday; `None` is not reached.
        pub fn week(&self) -> Option<(u32, NaiveDate, NaiveDate)>;
    }
    pub struct SeasonEvents<'a> { pub coming: Vec<Event<'a>>, pub past: Vec<Event<'a>> }
    pub fn season_events<'a>(games: &'a Schedule, trainings: &'a SeasonList<'a>) -> SeasonEvents<'a>;
    ```
    `coming` = `games.coming` + `trainings.coming`, sorted by date, then the key within a date; `past` = `games.past` + `trainings.past`, sorted by date newest first, then the same key. The key: (start time not known, start time, training before game, the game's source id). The split on today comes from `schedule` and `season_list` (same `today`), not again. The week: `date.iso_week()`, Monday `NaiveDate::from_isoywd_opt(iso.year(), iso.week(), Weekday::Mon)`, Sunday = Monday + 6 days.
- **Tests** (core unit tests):
  - `a_game_at_14_30_comes_before_a_training_at_19_45`
  - `a_game_without_a_start_time_comes_after_a_training_on_its_date`
  - `at_the_same_time_a_training_comes_before_a_game`
  - `coming_is_nearest_first_and_past_is_newest_first` (a game today is in `coming`)
  - `the_week_of_31_december_2026_is_week_53_from_28_december_to_3_january`
  - `the_week_of_25_october_2026_is_week_43` (the day of the clock change)
  - `not_opened_is_oldest_first_and_leaves_out_today`
  - The existing tests of `season_list`, `season_overview` and `schedule`, and `tests/attendance.rs`, pass unchanged.

### Step 2: Server – `GET /api/season`
- **Goal**: one answer with the events of a season, in order, with their weeks, the seasons to choose and the state of the games.
- **Changes**:
  - `crates/server/src/followed_teams.rs`: `pub async fn seasons(db) -> Result<Vec<Season>, StoreError>` (`SELECT season_year FROM followed_teams`).
  - `crates/server/src/api/trainings.rs`: `row_view` becomes `pub(crate)`.
  - `crates/server/src/api/season.rs` (new):
    ```rust
    #[derive(Serialize, ToSchema)]
    pub struct SeasonAnswer {
        pub season: String, pub current_season: String, pub today: String,
        pub season_first_day: String, pub season_last_day: String,
        /// Newest first: trainings::seasons ∪ followed_teams::seasons ∪ current ∪ chosen.
        pub seasons: Vec<String>,
        pub games: GamesState,
        pub club: Option<ClubView>, pub team: Option<TeamView>,
        pub coming: Vec<EventView>, pub past: Vec<EventView>,
    }
    #[derive(Serialize, ToSchema)] #[serde(rename_all = "snake_case")]
    pub enum GamesState { Shown, TeamNeeded, None }
    #[derive(Serialize, ToSchema)]
    pub struct EventView {
        pub date: String, pub week: u32,
        pub week_first_day: String, pub week_last_day: String,
        /// Exactly one of `game` and `training`.
        pub game: Option<ScheduleGame>, pub training: Option<TrainingRow>,
    }
    ```
    The handler: `query_season` (400 `invalid_season`); `followed_teams::for_season(season)`; with a team: `games::stored`, in the current season `api::changes::list(...)` for the open ids (as `api/schedule.rs`), in an earlier season no open ids; `team_core::schedule`. Without a team: no games, `team_needed` in the current season, else `none`. The trainings as `list_of`: weekly trainings, trainings, absences, roles, `season_list`. Then `season_events`; a `week()` of `None` is `StoreError::Invalid` (500). Separate queries, no transaction, as `/api/attendance`.
  - `app.rs`: `.route("/season", get(season::get))`; the OpenAPI description; `just api-types`.
- **Tests** (`crates/server/tests/season.rs`, new, with `tests/common/game_list.rs`, Blue Drakes M16 1 chosen, and Tuesday trainings):
  - `without_login_the_season_needs_a_login` (401)
  - `the_current_season_has_games_and_trainings_in_order_with_their_weeks` (both kinds; `week`, `week_first_day`, `week_last_day` of one game and one training; the order within a date)
  - `a_gone_game_shows_while_its_change_is_open` (`in_list: false`; gone after "seen")
  - `a_new_season_without_a_team_has_team_needed_and_only_trainings`
  - `an_earlier_season_shows_its_games_from_its_own_choice_and_hides_gone_games` (`?season=2025-2026`)
  - `a_season_with_only_a_choice_is_in_the_seasons`
  - `a_season_that_cannot_be_read_is_refused` (`?season=abc`, 400 `invalid_season`)
  - `tests/player_logs.rs`: `GET /api/season` logs no player name.

### Step 3: The page "Season", read only
- **Goal**: the events with labels, headings, season choice and empty texts; the link on the start page.
- **Changes**:
  - `web/src/lib/season.ts` (new):
    ```ts
    export type What = 'team' | 'duty' | 'training';
    export type When = 'coming' | 'past';
    export type Labels = { what: Set<What>; when: Set<When> };
    export const startLabels = (): Labels => ({ what: new Set(), when: new Set(['coming']) });
    export function whatOf(event: EventView): What;          // game.reason, or 'training'
    export function visible(events: EventView[], labels: Labels, showCancelled: boolean): EventView[];
    export function parts(labels: Labels): When[];           // no When label on: both
    export function emptyLine(labels: Labels, part: When): string;
    export function weekHeading(event: EventView): string;   // "Week 44 · 26 Oct – 1 Nov"
    ```
    `emptyLine`: "No {coming|past} {noun}." with the noun from the What labels: none or all three "events"; only "Games of the team" "games of the team"; only "Possible duty games" "possible duty games"; both kinds of game "games"; only "Trainings" "trainings"; trainings with a kind of game "events". Date words from `formatDay` (`schedule.ts`), statuses from `statusText` (`trainings.ts`).
  - `web/src/lib/components/Labels.svelte` (new): two groups of toggle buttons with `aria-pressed`, at the top of the list; several can be on.
  - `web/src/lib/components/SeasonEvents.svelte` (new): for each part ("Coming", "Past") the week headings (keyed by `week_first_day`), the date headings, the game rows (the look of `Schedule.svelte`) and the training rows (the look of the rows on "Attendance"); a tap on a training row opens it as "Attendance" does now.
  - `web/src/routes/season/+page.ts` and `+page.svelte` (new): load `/api/season` with `?season=`; `invalid_season` gives "This season cannot be read." with "Current season"; the season choice when `seasons.length > 1` (changes the address); the line "Choose the team to see its games." with a link to `/` when `games === 'team_needed'`; "No games or trainings in 2026-2027 yet." when both lists are empty; else the labels (fresh `startLabels()` at each visit) and `SeasonEvents`.
  - Start page: the link "Season" first in the line of links.
- **Tests** (`web/e2e/season.e2e.ts`, new, in the Playwright project `trainings` after `trainings.e2e.ts`; clock 12 October 2026):
  - "Season" is first in the line of links and opens with "Coming" pressed and no What label pressed.
  - The first week heading is "Week 42 · 12 Oct – 18 Oct", with date headings below it.
  - "Possible duty games" shows only games without the badge "Team" and no training rows.
  - "Games of the team" + "Trainings" shows both, and no possible duty games.
  - "Past" with "Coming" on shows the parts "Coming" and "Past"; past events newest first.
  - A label combination with no events shows its line (for example "No past possible duty games."; check first which combination is empty with the fake source).
  - After a visit to "Team" and back, only "Coming" is pressed.
  - A tap on a training row opens the training.
  - The season choice to 2025-2026 changes the address, a reload keeps it, and "Add training" is not offered there.
  - `?season=abc` shows "This season cannot be read." with "Current season".
  - The page fits a small phone, has no console errors, and passes the accessibility check.

### Step 4: The actions on "Season", and "Back"
- **Goal**: the actions of the list of trainings and the "Game list" card on "Season"; "Back" to where the coordinator came from.
- **Changes**:
  - `/season`: "Add training" (when `season === current_season`; the form and its checks as on "Attendance", with `today`, `season_first_day`, `season_last_day`), "Cancel" on a training day not opened, "Show cancelled" (the same `localStorage` key as "Attendance" now, read and written in `try`/`catch`). The "Game list" card (`ReadGameList.svelte`, with `/api/readings/latest` loaded in `+page.ts`) when `season === current_season` and `games === 'shown'`; its `onRead` calls `invalidate` for `/api/season`.
  - `/training/[date]`: "Back" goes to `/` with `?from=start`, else to `/season?season=<season>`; after "Delete training" to `/season?season=<season>`.
  - Start page: the card of the next training opens `/training/<date>?from=start`.
- **Tests** (`season.e2e.ts` and `trainings.e2e.ts`):
  - "Add training" on "Season" adds a training that shows as a row.
  - "Cancel" on a training day not opened; "Show cancelled" off hides it, and the setting stays after a reload.
  - "Read the game list" on the "Game list" card reads, and the list shows the games without a page load.
  - "Back" from a training opened on "Season" returns to "Season" of that season; "Delete training" goes to "Season".
  - `trainings.e2e.ts` line 129 (a tap on the card): "Back" returns to the start page.

### Step 5: "Attendance" without its list
- **Goal**: one list of trainings, on "Season".
- **Changes**:
  - `/attendance`: remove the list "Trainings", "Add training", "Cancel" and "Show cancelled"; `+page.ts` loads only `/api/attendance`.
  - `api/trainings.rs`: remove `list` (`GET /api/trainings`) and `TrainingList`; `list_of` stays for `GET /api/trainings/next` (removed in story B). `app.rs`: `/trainings` keeps only `post`. `just api-types`.
  - `tests/trainings.rs`: read the training rows from `GET /api/season` (a helper `training_rows(&Value)`) in `the_list_has_one_row_per_date` and after add, cancel, undo and delete; `a_date_or_season_that_cannot_be_read_is_refused` uses `/api/season?season=2026`.
  - `web/e2e/trainings.e2e.ts`: the tests of the list and its actions go to `/season`; the totals stay on `/attendance`; "an earlier season can be chosen, and Back returns to it" is split (season choice and "Back" on "Season"; the season choice of the totals on "Attendance").
- **Tests**:
  - "Attendance" has no list "Trainings" and no "Add training", and still the totals and the line of training days not opened.
  - The changed `tests/trainings.rs` and `trainings.e2e.ts` pass; `just check`.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| One route `GET /api/season`; the core puts games and trainings in order and gives the ISO week | The order is a rule with tests; no date arithmetic in the front end; one request | In 0022 at the close of the epic |
| "Season" reuses `ScheduleGame` and `TrainingRow` | The rows are the same as in the schedule and on "Attendance" | No |
| The seasons of "Season" are those of "Attendance" plus the seasons with a choice | A season with only games is shown; "Attendance" does not change | No |
| Gone games: in the current season shown while their change is open, in an earlier season hidden | The rule of the schedule now | No |
| A new `SeasonEvents.svelte` next to `Schedule.svelte` until story B | It needs week headings and training rows; B removes `Schedule.svelte` | No (debt below) |
| `?from=start` comes in story A, not B | The card of the next training opens a training from the start page already in story A | No |

## Not in scope
- A tap on a game (no page of a game yet); duty labels; keeping the labels; open items; removing the schedule, cards and view "Last season" from the start page (story B); "Checked" (story C).

## Test plan
- **Existing tests that must still pass**: core tests of `season_list`, `season_overview`, `schedule`; `tests/attendance.rs` unchanged; `tests/schedule.rs`, `tests/changes.rs`, `tests/readings.rs` unchanged; `tests/trainings.rs` (changed in step 5); `web/e2e/choose.e2e.ts` (the schedule stays on the start page), `changes.e2e.ts`, `logged-in.e2e.ts`, `players.e2e.ts` unchanged; `trainings.e2e.ts` (changed in steps 4 and 5).
- **New tests**: per step, above.
- **Not testable in the browser** (clock fixed at 12 October 2026, no new season): "Choose the team to see its games." (server test only), week 53 and the clock change (core only).
- **Checks while working**: `cargo test -p team_core`; `cargo test -p team --test season`, `--test trainings`, `--test attendance`; `cd web && npx playwright test season trainings` (after `just check-web`); `cargo clippy --workspace --all-targets -- -D warnings` at the end of each step; `just check` once before the commit.

## Compatibility and performance
- New page and route; the link "Season" first on the start page. The list of trainings and its actions move from "Attendance" to "Season"; `GET /api/trainings?season=` is removed (only the front end calls it).
- "Show cancelled" keeps its stored setting (same key).
- `GET /api/season`: the games of one club and season (about 200) and the trainings of one season (about 80 rows); no concern.
- Technical debt until story B: the game rows exist in `Schedule.svelte` and in `SeasonEvents.svelte`. Written in `docs/architecture.md` at the close if B is not done by then.

## Progress
- [ ] Step 1 – core: the order of events, and `not_opened`
- [ ] Step 2 – server: `GET /api/season`
- [ ] Step 3 – the page "Season", read only
- [ ] Step 4 – the actions on "Season", and "Back"
- [ ] Step 5 – "Attendance" without its list
- [ ] All checks pass

## Deviations
None from `functional.md`. Readings that the design makes exact (no change of behaviour):
- "No games or trainings in 2026-2027 yet." shows only when the season has no events at all; a part with no events under the labels shows its own line.
- "The labels that are on are always visible": toggle buttons with `aria-pressed` at the top of the list.
- "Choose the team to see its games." links to the start page, where the choose step is.
- `?from=start` moves from story B to this story (epic `technical.md` named it in B).

## Review findings
