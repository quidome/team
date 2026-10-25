# Technical spec: The page "Season"

**Status**: done
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
- **Not testable in the browser** (clock fixed at 12 October 2026, no new season): week 53 and the clock change (core only). "Choose the team to see its games." is tested in the browser after all: the choose tests end with a club saved without a team (step 3).
- **Checks while working**: `cargo test -p team_core`; `cargo test -p team --test season`, `--test trainings`, `--test attendance`; `cd web && npx playwright test season trainings` (after `just check-web`); `cargo clippy --workspace --all-targets -- -D warnings` at the end of each step; `just check` once before the commit.

## Compatibility and performance
- New page and route; the link "Season" first on the start page. The list of trainings and its actions move from "Attendance" to "Season"; `GET /api/trainings?season=` is removed (only the front end calls it).
- "Show cancelled" keeps its stored setting (same key).
- `GET /api/season`: the games of one club and season (about 200) and the trainings of one season (about 80 rows); no concern.
- Technical debt until story B: the game rows exist in `Schedule.svelte` and in `SeasonEvents.svelte`. Written in `docs/architecture.md` at the close if B is not done by then.

## Progress
- [x] Step 1 – core: the order of events, and `not_opened`
- [x] Step 2 – server: `GET /api/season`
- [x] Step 3 – the page "Season", read only
- [x] Step 4 – the actions on "Season", and "Back"
- [x] Step 5 – "Attendance" without its list
- [x] All checks pass (`just check`, 2026-10-25)

## Deviations
None from `functional.md`. Readings that the design makes exact (no change of behaviour):
- "No games or trainings in 2026-2027 yet." shows only when the season has no events at all; a part with no events under the labels shows its own line.
- "The labels that are on are always visible": toggle buttons with `aria-pressed` at the top of the list.
- "Choose the team to see its games." links to the start page, where the choose step is.
- `?from=start` moves from story B to this story (epic `technical.md` named it in B).

Found while building (2026-10-25):
- Step 3: `Labels` holds arrays (`what: What[]`, `when: When[]`), not `Set`s, because the toggle groups bind to arrays; the behaviour is the same.
- Step 3: `season.e2e.ts` runs in its own Playwright project `season` (after `trainings`), not inside `trainings`: files of one project can run at the same time, and both use one database.
- Step 3: the first test of `season.e2e.ts` chooses Blue Drakes M16 1 again, because `choose.e2e.ts` ends with a club saved without a team; before that it checks "Choose the team to see its games.".
- Step 4: "Back" also returns to "Attendance" when the training was opened from its line of training days not opened (`?from=attendance`), so "Back" goes to the page it came from in every case; a "Change" to another date keeps `from`.
- Step 4: the lint rule `svelte/no-navigation-without-resolve` accepts only a `resolve()` call, and `resolve()` takes no query on a route with a parameter (`/training/[date]`). The three addresses with `?from=` are a resolved path plus the query, with a lint exception and its reason at each place (as in `src/lib/utils.ts`).
- Step 4: "Season" shows "The new games could not be loaded. Load the page again." when the events cannot be loaded after a good reading, as the start page does for its schedule now.
- Step 4: the tests of `trainings.e2e.ts` that used "Back" to return to "Attendance" changed in this step, not in step 5, because "Back" changed here: they open trainings from "Season" and load "Attendance" by its address to check the totals; the delete test expects "Season".
- Step 5: `list_of` is gone too: `GET /api/trainings/next` now has its own small `next_of` (the first coming row), until story B removes the route.
- Step 5: `tests/trainings.rs` reads the training rows of `GET /api/season` through a helper `training_list` with the old shape (season, today, coming, past), so its checks stay as they were; `tests/players.rs` names `GET /api/season` instead of `GET /api/trainings` in its list of routes that need a login.
- Step 5: the test "an earlier season can be chosen, and Back returns to it" is split into one for "Attendance" (season choice) and one for "Season" (season choice, "Add training", "Back"); the part "Show cancelled hides the row, not the totals" left the test of cancelled trainings, because the rows are no longer on "Attendance" (hiding is tested on "Season").
- After the reviews (R3): the page "Season" loads `/api/season` again itself after a good reading (`reload`), not with `invalidate`, so the "Game list" card keeps its result line, as the start page does.
- After the reviews (A5): without a chosen team, "Season" has no "Game list" card (as the start page).
- After the reviews (A1, coordinator): a return to "Season" from a training opened there keeps the labels and the place in the list, once; the spec says labels are not kept "at each visit", and a return from a training is not a new visit. Kept in `sessionStorage` for that one date only.
- After the reviews (R4, coordinator): the season choice offers the seasons with a chosen team (plus trainings, roles, weekly trainings, the current and the chosen season), not every season with a choice.
- After the reviews (R5, coordinator): "Season" shows "No training days or trainings this season. Add a weekly training in Settings." when the season has none, as "Attendance" did.

## Review findings
Build review, 2026-10-25, `/review build` (fresh agent). Read: the diff against `main` (committed
and not, with the new files), `change.md`, this spec, the epic's `functional.md` and
`technical.md`, `docs/architecture.md`, the decision records, `CLAUDE.md` and `AGENTS.md`. Runs:
`cargo test -p team_core` (185 passed); `cargo test -p team --test season --test trainings --test
attendance --test player_logs --test players` (all passed); `cargo fmt --check`, `cargo clippy
--workspace --all-targets -- -D warnings`, `cargo sqlx prepare --workspace --check` (all clean);
`openapi` written to a scratch file is the same as `web/openapi.json`. Browser tests not run: the
builder had a Playwright run on the same ports during the review. `crates/server/tests/season.rs`
changed on disk during the review (13:26), not by the reviewer; the findings are on the version
read before that time. No other text was given in the arguments.

No finding against the rules: the order of events is in the core (`season_view.rs`, no I/O,
decision 0001); everything is worked out at each request (0020); the route is under the protected
router and in `players.rs`'s login list (0007); no `unwrap`/`expect` outside tests; no player
names in the new route (`player_logs.rs`). `not_opened` moved out of `season_overview` with the
same result (`tests/attendance.rs` unchanged and passing; the new core test fails if `coming` were
read). `season.rs` builds on `ScheduleGame`, `row_view` and `api::changes::list` as the design
said; nothing is rebuilt except the game rows of `SeasonEvents.svelte`, which is the debt named
under "Compatibility".

**R1 – The test of the "Game list" card cannot fail on its claim (claim, step 4).** "the 'Game
list' card reads, and the new games show at once" checks `last-read`, a first game in "Coming",
and no `season-load-error`. The games were already in the list before the tap (the first test
chose the team, which reads the game list), and the fake source gives the same games again, so
the test also passes when `onRead={reload}` is removed. Nothing shows that the list changed
without a page load. A check that can fail: let the fake source (or `page.route` on
`/api/season`) give one game more or less after the tap, and expect that row to show or go
without a reload. Checked by trace: `season.e2e.ts` (the test and the first test), the season
page (`ReadGameList ... onRead={reload}`).
  - 2026-10-25: Fixed: the test answers `/api/season` with a renamed home team after the tap, and checks that the new name shows; a run with `onRead={reload}` removed (and the front end built) fails on it.

**R2 – "Back" to "Attendance" is built but not tested (claim, sibling of step 4).** The deviation
says "Back" returns to "Attendance" with `?from=attendance`, and that "Change" to another date
keeps `from`. `trainings.e2e.ts` only checks that the address has `?from=attendance`; no test
taps "Back" there, and no test moves a training opened with `?from=` to another date. A mistake in
`backOf` or in the query kept by "Change" would pass every check. Checked by a search
(`from=attendance` in `web/e2e`: one hit, line 500) and a trace of `training/[date]/+page.svelte`
(`backOf`, the `goto` after "Change").
  - 2026-10-25: Fixed: the test "Back returns to the page the training was opened from" taps "Back" after the link on "Attendance" (`?from=attendance`), and moves a training opened with `?from=start` to another date with "Change" (the address keeps `from`; "Back" goes to the start page).

**R3 – The step 4 design said `invalidate`; the build reloads by hand, and this is not under
Deviations (docs).** The spec: "its `onRead` calls `invalidate` for `/api/season`". The page
keeps its own `answer` and calls `loadSeason` again, with a comment why (a new load would build
the card again and lose its result line). The choice is sound and the behaviour is the same, but
it is a difference from the approved design without a line under Deviations. Checked by trace:
step 4 above, `routes/season/+page.svelte` (`reload`).
  - 2026-10-25: Fixed: written under **Deviations** (the page loads `/api/season` again itself, so the card keeps its result line).

**R4 – A season with only a club choice and no team is offered (sibling of `followed_teams::seasons`).**
`followed_teams::seasons` takes every row of `followed_teams`, also one with a club and no team
(the state the choose tests leave behind). In a later season such a season is in the choice and
shows only "No games or trainings in … yet." The spec says "the earlier seasons with games or
trainings"; the design chose "seasons with a choice" on purpose, and a season with a team but no
kept games is the same case (`a_season_with_only_a_choice_is_in_the_seasons` asserts it). Small;
for the accept review to weigh, or `WHERE team_number IS NOT NULL` if a season without a team
should not be offered. Checked by trace: `followed_teams.rs` (`seasons`), `api/season.rs`
(`seasons`); not run.
  - 2026-10-25: Fixed (coordinator: offer only seasons with a team): `followed_teams::seasons` takes only choices with a team; `a_season_with_only_a_choice_is_in_the_seasons` checks that a season with only a club is not offered.

**R5 – The hint to add a weekly training is gone (sibling of step 5).** "Attendance" had, with an
empty list, "No training days or trainings this season. Add a weekly training in Settings." with
a link. That line went with the list, and "Season" shows only "No games or trainings in … yet."
(as the spec says), so a new coordinator no longer gets the way to Settings. Not a rule break;
for the accept review. Checked by the diff of `routes/attendance/+page.svelte` and a trace of
`routes/season/+page.svelte`.
  - 2026-10-25: Fixed (coordinator: put it back on "Season"): "No training days or trainings this season. Add a weekly training in Settings." shows on "Season" when the season has no training days and no trainings; the test of another season checks it.

**R6 – The open button of a training row has no date in its name (a11y, sibling of the move).**
On "Attendance" the row's button read "Tue 6 Oct · 19:45 …"; in `SeasonEvents.svelte` it reads
"Training · 19:45 · Not opened", the same for every Tuesday, because the date moved to the
heading. The "Cancel" button next to it kept its date (`aria-label`). A screen reader that lists
buttons hears many equal names; the axe check does not flag it. Adding the date to an
`aria-label` (as "Cancel" does) would fix it. Checked by trace of both rows; not run with a
screen reader.
  - 2026-10-25: Fixed: each training button has its date for screen readers (a hidden "Tue 13 Oct" in its name).

**R7 – Comments and living documents the diff made false (docs).**
- `crates/server/src/api/trainings.rs` line 1: "the trainings of a season" – the list route is gone.
- `web/src/lib/components/SeasonEvents.svelte` line 3: "training rows as on 'Attendance'" – "Attendance" has no rows now; "as 'Attendance' had".
- For `/close`: `docs/architecture.md` lines 83 and 88 (`GET /api/trainings?season=`), `docs/product/trainings-and-attendance.md` lines 49 and 69 ("choose the season on 'Attendance' and tap the row", "Back … return to 'Attendance'", the route list), and the `CLAUDE.md` list of core rules (no "season view" yet).
Checked by a search (`git grep` for `api/trainings`, `TrainingList`, `list_of`, "on \"Attendance\"").
  - 2026-10-25: Fixed in the code: the first comment of `api/trainings.rs` and the header of `SeasonEvents.svelte`. Not fixed now: `docs/architecture.md`, `docs/product/trainings-and-attendance.md` and the list in `CLAUDE.md` are living documents, updated at `/close`.

Accept review, 2026-10-25, `/review accept` (fresh agent). No extra text came with the arguments.
Read: `change.md`, the epic's `idea.md` and `functional.md` (story A, the facts and the spec
review notes), this spec, the diff against `main` with the new files, and `docs/product/trainings-and-attendance.md`
for R5. I formed my view before reading **Deviations**. Runs: `npm run build`, then
`npx playwright test --project season` (78 passed, with the projects it depends on; this run
may be the Playwright run that the build review met on the same ports). Then the server on its
own port against the database that run left (clock 12 October 2026, dev login), with
`GET /api/season` and a phone-sized browser (390×844). I opened only trainings that were already
opened, and saved nothing. Screenshots removed afterwards.

**Verdict: accepted with notes.** Story A does what the coordinator asked for: one page with every
event of the season, under week and date headings. The labels combine as the spec says. The page
opens on "Coming" at each visit. The list of trainings and its actions moved off "Attendance".
Run: the week headings ("Week 42 · 12 Oct – 18 Oct"), coming nearest first, past newest first,
with "Coming" + "Past" as two parts each with its own headings; "Trainings" + "Past" gives only
the past training rows, newest first. Each acceptance criterion of story A matches by trace and
the browser tests. The items below are the differences, sorted.

**A1 – "Back" (and the phone's back) to "Season" drops the labels and the place in the list
(neutral drift, a point the spec left open).** The behaviour of story A: "To correct a training of
three weeks ago, the coordinator turns on 'Trainings' and 'Past'". After the correction, "Back"
gives "Coming" at the top of the page. To correct a second past training, the coordinator turns
the labels on and scrolls again. This follows the letter: "labels are not kept between visits",
"at each visit". But spec review note 2 worried about a *later* visit that hides events. It did not
worry about coming back from a training opened from this list. Whether that return counts as a new
visit is not decided anywhere. Story B's "Last" covers the common case (last week's training, one
tap from the start page), so this does not block. For the coordinator: should a return from a
training opened on "Season" keep its labels (and scroll)? Checked by a run: "Trainings" + "Past"
on, tap the row of Tue 22 Sep, then "Back" and, a second time, `history.back()`. Both gave
`/season?season=2026-2027` with only "Coming" pressed and scroll 0. The cause is a trace:
`afterNavigate` sets `startLabels()` on every navigation to the page.
  - 2026-10-25: Fixed (coordinator: keep the labels on a return): a training opened from "Season" keeps the labels and the place for one return ("Back", the browser's back, or after "Delete training"); a new visit still opens with "Coming". Written under **Deviations**; the test "a return from a training keeps the labels".

**A2 – The labels are at the top of the list, not fixed on the screen (faithful adaptation).**
"The labels that are on are always visible at the top of the list": the design read this as
"placed at the top" (Deviations). Scrolled down to November, the labels and the part heading are
out of view. Only the date heading stays fixed. This serves the intent of note 2: the labels do
not carry over to another visit, and the coordinator set them on this visit. Checked by a run:
after a scroll of 2500 px the labels are at −2141 px; screenshot of the scrolled list.
  - 2026-10-25: No change needed.

**A3 – The hint "Add a weekly training in Settings" is gone (neutral drift; agrees with R5).**
`docs/product/trainings-and-attendance.md` line 24 promises it for a season with no training days.
Functional "Consistency" says "the rules of trainings stay", but story A gives only "No games or
trainings in … yet.". A season with games and no weekly training shows no hint at all. The
coordinator meets this case at the start of each season. Advice: put the line back on "Season"
when the season has no training rows (a small fix), or remove it from the product doc at `/close`
by a choice the coordinator makes. Checked by a search (`Add a weekly` in `docs/product`, `web/src`:
only the product doc) and the old `attendance/+page.svelte` (`git show HEAD:`, line 380).
  - 2026-10-25: Fixed: see R5.

**A4 – "Back" also returns to "Attendance" (faithful adaptation).** The spec names "the start page
or 'Season'"; the build adds `?from=attendance` for the line of training days not opened that
stays on "Attendance". That is the intent ("the page it came from") in the one case the spec did
not list. Checked by trace (`backOf` in `training/[date]/+page.svelte`); not run (see R2).
  - 2026-10-25: No change needed; now also run (see R2).

**A5 – No "Game list" card on "Season" without a team (faithful adaptation).** The card shows only
when `games === 'shown'`. With no team chosen there are no games for the card to read, and
"Choose the team to see its games." is the next step. Not written under Deviations. Checked by
trace (`season/+page.ts`).
  - 2026-10-25: Written under **Deviations**.

**A6 – An earlier season with only a club choice is offered and shows an empty page (neutral
drift; agrees with R4).** The spec: "the earlier seasons with games or trainings". The build:
also seasons with a choice, a team, or a club alone. This is rare in real use, and harmless (an
empty line). Checked by trace (`api/season.rs`, `seasons`); not run.
  - 2026-10-25: Fixed: see R4.

**A7 – The page on a phone: the first event is low on the first screen (neutral drift, no
action).** At 390×844 the "Game list" card, "Add training", the labels and "Show cancelled" come
first, and the first event row starts about 660 px down. The spec puts the card at the top, and
the job of this page is a rare look-up, so this matches. Worth a look when story B makes "Season"
the place for the card. Checked by a run (screenshot at the top of `/season`).
  - 2026-10-25: No change needed now; the order of the start page and "Season" is looked at again in story B.

No concerning deviation. A1 and A3 are for the coordinator. A3 also belongs on the list for
`/close`, next to R7.
