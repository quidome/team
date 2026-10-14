# Technical spec: The schedule

**Status**: done
**Size**: story
**Change**: docs/changes/2026-10-12-the-schedule
**Date**: 2026-10-12

## Objective
Show the coordinator's schedule on the start page, from the games that story 3a stores, so the
coordinator can stop keeping the schedule in the sheet.

## Requirements summary
Story 3b of the epic [Games from the game list](../2026-10-10-games-from-game-list/technical.md),
from functional story 3 ("See the schedule") and the parts of functional story 1 that the
schedule needs. Story 4 (the list of changes) has its own design.

1. The schedule holds all games of the team (home and away) and the home games of other club teams ("possible duty games"). It holds no away games of other club teams.
2. A quick filter with three choices: "All", "Games of the team", "Possible duty games". One tap changes it. (Story 4 uses the same filter on the list of changes.)
3. Coming games first, in time order, grouped by date. Past games of this season open on request.
4. Games of the team look clearly different from games of other club teams.
5. Each game shows day, date, start time, home team, away team, hall and town. A start time of 00:00 in the game list shows "time not known yet". No results are shown; "0 - 0" is never a result.
6. A game that is no longer in the game list shows crossed out until its change is marked as seen (story 4); after that it is hidden. Until story 4, it stays crossed out (coordinator, 2026-10-12).
7. The screen shows when the last good reading was, and when the last reading failed (the "Game list" card of story 3a).
8. No games yet: the screen says that the game list has no games for this season yet, and when it was last read.
9. Phone first, one hand. Only the coordinator can see and use it (decision 0007).
10. After the coordinator chooses the team, the tool reads the game list and shows the games (functional story 1, behaviour; coordinator, 2026-10-12, review finding A4 of story 3a).
11. In a new season, until the team is chosen again, the tool shows last season's games and says that a new choice is needed (functional story 1; coordinator, 2026-10-12, A4).
12. Earlier seasons show the newest hall and town name of a place (coordinator, 2026-10-12, review finding R5 of story 3a). No code change: places are one table.

Decisions that apply: 0001 (rules in a core without I/O), 0002 (static Svelte front end, generated
API types), 0007 (every `/api` route needs a session; debug-only bypass), 0012 (the choice for
each season), 0014 (seasons as start years; Dutch local dates and times), 0015 (readings are
recorded results).

## Current state
- Core (`crates/core`): `schedule::in_schedule(game, club, team) -> Option<ScheduleReason>` (`TeamGame`, `DutyGame`); `comparison::StoredGame { game, in_list }`; `team_choice::choice_state(this_season, earlier)` gives `Chosen`, `TeamNeeded { club, reason, last }` or `NotChosen`; `Game` with `date`, `start_time: Option<NaiveTime>` (`None` = not known), `home`/`away` `Side { team, team_name, club }` (`team_name` is the full name, for example "Blue Drakes M16 1"), `place: Place { number, hall, town, court }`.
- Server (`crates/server`): `games::stored(db, club, season) -> Vec<StoredGame>` (joins `places`, sorted by source id); `followed_teams::{for_season, newest_before}`; `api/selection.rs` builds the `Selection` view from `choice_state`; `clock::{Clock, SystemClock, today_in_netherlands, current_season}`; `config.rs` has `dev_auth_bypass` behind `#[cfg(debug_assertions)]` with a warning when set in a release build; `main.rs` builds `AppState` with `clock: Arc::new(SystemClock)`.
- Front end (`web/`): the start page (`src/routes/+page.svelte`) shows, with a team chosen, the team card with "Change", the "Game list" card (`ReadGameList.svelte`, inside `{#key data.readings}`), and a card "Your schedule comes here" with a link to `/sample`. `/sample` (`src/routes/sample/+page.svelte`, data in `src/lib/sample/games.ts`) has the look to keep: date headings, a team game with `border-l-team bg-team-soft` and the badge "Team", the filter as a `ToggleGroup`, a crossed-out game with the badge "no longer in the game list". `ChooseTeam.svelte` calls `saveSelection` and then `onSaved()`.
- Tests: router tests of readings use `crates/server/tests/common/game_list.rs` (harness `chosen`, `answer`, `games_with`, `read_good`, the test clock `set`); browser tests in `web/e2e/` with projects `choose` (serial, one fresh database), `logged-in`, `logged-out`, and the fake source `web/e2e/fake-source.mjs`. The sample page has tests in `logged-in.e2e.ts` and `logged-out.e2e.ts`; `/sample` is a path in `crates/server/tests/web.rs` and in a test in `crates/server/src/auth/mod.rs`.
- Real data (2026-2027, Blue Drakes, club 37, M16 1 = 184), with today 12 October 2026: 37 games in the schedule; 23 coming (5 of the team, 18 possible duty games) on 12 dates; 14 past (5 of the team). First coming date Sat 31 October: X12 1 13:00, X14 1 14:30, M16 1 16:00, MSE 1 17:30, all Bulwijk, Woerden. No coming schedule game has 00:00.

## Steps

### Step 1: Core rule
- **Goal**: which stored games show, in which order, and coming or past; testable without I/O.
- **Changes** (`crates/core/src/schedule.rs`, exported from `lib.rs`):

  ```rust
  /// A game in the schedule, with why it is there.
  pub struct ScheduledGame { pub game: Game, pub reason: ScheduleReason, pub in_list: bool }

  pub struct Schedule { pub coming: Vec<ScheduledGame>, pub past: Vec<ScheduledGame> }

  /// Keeps the stored games that `in_schedule` accepts. Coming: date >= today, in time order.
  /// Past: date < today, newest date first, in time order within a date.
  pub fn schedule(stored: Vec<StoredGame>, club: ClubNumber, team: TeamNumber, today: NaiveDate) -> Schedule;
  ```

  Rules:
  - `team` is not optional: a schedule always has a team (the current one, or last season's).
  - Within a date: known start times first, in time order; "not known" after them; then by source id.
  - A game on `today` is coming.
  - A game with `in_list = false` stays in, with `in_list: false`.
- **Tests** (unit tests in `schedule.rs`, and one real-data test in `crates/server/tests/source.rs`):
  - An away game of another club team is not in the result.
  - A game today is coming; a game yesterday is past.
  - Coming: by date, then start time; a not-known time after the known times of its date; equal times by source id.
  - Past: newest date first, in time order within a date.
  - A game with `in_list = false` is in the result with `in_list: false`.
  - Real data, today 12 October 2026: 23 coming (5 `TeamGame`, 18 `DutyGame`) and 14 past; the first coming game is X12 1 at 13:00 on 31 October 2026.

### Step 2: Fixed clock for development
- **Goal**: browser tests that do not depend on the real date.
- **Changes**:
  - `config.rs`: `#[cfg(debug_assertions)] pub dev_now: Option<DateTime<Utc>>` from `DEV_NOW` (RFC 3339, for example `2026-10-12T10:00:00Z`). A value that is not a valid time is a config error that names `DEV_NOW`. In a release build the setting is ignored with a warning, as `DEV_AUTH_BYPASS`.
  - `clock.rs`: `ShiftedClock`: starts at the given moment and then runs with the real time (a clock that stands still would give readings the same start time; decision 0015 needs a later start to win).
  - `main.rs`: `ShiftedClock` when `dev_now` is set, else `SystemClock`; a warning in the log that the clock is shifted.
  - `web/playwright.config.ts`: `DEV_NOW=2026-10-12T10:00:00Z` for the server.
  - `.env.example`: the setting, commented out.
- **Tests**:
  - `DEV_NOW=2026-10-12T10:00:00Z` gives a clock that starts at that time and does not go back.
  - A value that is not a time is a config error that names `DEV_NOW`.
  - A release build ignores `DEV_NOW` (the same pattern as the `DEV_AUTH_BYPASS` test).
  - All existing browser tests pass with the fixed date.

### Step 3: The API
- **Goal**: one read-only route that gives the schedule to show.
- **Changes** (new `crates/server/src/api/schedule.rs`, on the protected `/api` router, in `ApiDoc`; `just api-types`):
  - `GET /api/schedule`. Which schedule, from `choice_state` (as in `api/selection.rs`):

    | State of the choice | Schedule |
    |---|---|
    | Team chosen | current season, chosen club and team, `last_season: false` |
    | Team needed, earlier choice with a team | that season, club and team, `last_season: true` |
    | Nothing chosen, or team needed with no earlier team | `null` |

  - Games from `games::stored(club, season)`; today from `clock::today_in_netherlands`; then `team_core::schedule`.

    ```text
    ScheduleAnswer = { schedule: ScheduleView | null }
    ScheduleView   = { season: "2026-2027", last_season: boolean, club: ClubView, team: TeamView,
                       coming: ScheduleGame[], past: ScheduleGame[] }
    ScheduleGame   = { id: number (source id), date: "2026-10-17", start_time: "12:15" | null,
                       home: string, away: string, hall: string, town: string,
                       reason: "team" | "duty", in_list: boolean }
    ```

  - Errors: 401 without a session; storage errors 500 `internal`. No source read, so no 502.
- **Tests** (router tests, new `crates/server/tests/schedule.rs`, `game_list` harness and test clock):
  - Without login: 401.
  - Nothing chosen: `{ schedule: null }`.
  - Team chosen, no reading yet: current season, empty lists.
  - After a first reading (clock 12 October 2026): 23 coming and 14 past; `reason` "team" on the 5 coming M16 1 games; no away game of another club team; season "2026-2027", `last_season: false`.
  - A game with 00:00 in the source has `start_time: null`; other times are "HH:MM".
  - A schedule game removed from the source in the next reading is in the answer with `in_list: false`.
  - New season (clock 1 July 2027, no choice for 2027-2028): the 2026-2027 schedule of M16 1, `last_season: true`, all 37 games past.
  - New season with the club saved for an empty season (`no_games_yet`) and an earlier team: last season's schedule.
  - Team needed with no earlier team: `{ schedule: null }`.

### Step 4: The schedule on the start page
- **Goal**: the coordinator sees the real schedule; the sample goes.
- **Changes**:
  - New `web/src/lib/components/Schedule.svelte` (the look of the sample page) and `web/src/lib/schedule.ts` (filter, grouping by date, the date heading "Sat 31 Oct"); `client.ts` helper `loadSchedule`.
  - Filter "All", "Games of the team", "Possible duty games" on `reason`; always one choice.
  - Team game: coloured edge and badge "Team". `start_time: null`: "time not known yet". `in_list: false`: crossed out with the badge "no longer in the game list".
  - "Show past games" / "Hide past games", only when there are past games.
  - No coming and no past games: "The game list has no games for this season yet." Only past games: "No coming games."
  - Start page, team chosen: a short team header (name, season, "Change"), the "Game list" card, then the schedule. The card "Your schedule comes here" goes.
  - After a good reading from the card, the page gets `GET /api/schedule` again and replaces only the schedule (`ReadGameList` gets an `onRead` callback). No `invalidateAll`: it would build the card again and lose its result line.
  - Remove `web/src/routes/sample/`, `web/src/lib/sample/`, the `sample page` tests in `logged-in.e2e.ts` and the sample test in `logged-out.e2e.ts`; the screenshot test keeps `/`. `/sample` → `/choose` in `crates/server/tests/web.rs` and the test in `crates/server/src/auth/mod.rs`.
  - `web/e2e/fake-source.mjs`: the X12 1 game of 31 October 2026 gets time 00:00.
- **Tests** (Playwright, project `choose`, after the tap on "Read the game list" that exists today):
  - 23 games; the first date heading is "Sat 31 Oct".
  - 5 games with the team look and the badge "Team".
  - Filter: "Games of the team" 5, "Possible duty games" 18, "All" 23.
  - "Show past games" shows 14 more games; "Hide past games" hides them.
  - "time not known yet" shows for X12 1 on 31 October.
  - A second tap gives "No changes" and the schedule stays.
  - Accessibility check, no console errors, no horizontal scroll at 320 px; light and dark screenshots of `/`.

### Step 5: Read after the choice, and last season
- **Goal**: choosing a team shows the games at once; a new season keeps last season visible.
- **Changes**:
  - `ChooseTeam.svelte`: when the saved selection is `chosen`, call `readNow()`, then `onSaved()`. Buttons disabled while it runs. A failed reading is recorded (decision 0015) and shows on the card after the page loads. When the reading request itself fails (for example 500), the save stays done and `onSaved()` still runs.
  - Start page, state "team needed": the choose step first; below it, when `GET /api/schedule` gives `last_season: true`, the heading "Last season · 2025-2026", the team name, and the schedule with past games open, without the "Game list" card.
  - `web/e2e/choose.e2e.ts`: the tests after the choice expect the schedule without a tap; "the coordinator reads the game list" expects "No changes" (the save did the first reading) and merges with "a second reading of the same game list has no changes".
- **Tests** (Playwright, project `choose`):
  - Choosing M16 1 shows, without a tap, 23 games with "Sat 31 Oct" first, and "Last read: …".
  - The changed tests in `choose.e2e.ts` pass in their order.
  - The last-season view is covered by the router tests of step 3 (the browser run has one fixed date).

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| The server sends all schedule games split into coming and past; the screen filters (option A) | One tap on the filter needs no request; "today" is the server's Dutch date, so a wrong phone clock moves nothing; about 40 games a season (coordinator) | no |
| A game that left the game list stays crossed out until story 4 adds "seen" | No change is hidden before the coordinator has seen it (coordinator) | no |
| A save with a team starts a reading at once, from the screen | Functional story 1: "chooses M16 1, the tool reads the game list and shows the games" (coordinator, A4) | no |
| In a new season, last season's schedule shows under the choose step, read-only | Functional story 1 (coordinator, A4) | no |
| Earlier seasons show the newest hall and town name | One table of places; rare case (coordinator, R5); written in `architecture.md` at Close | no |
| `DEV_NOW`: a shifted clock, only in debug builds | Browser tests must not depend on the real date; same pattern as `DEV_AUTH_BYPASS` (decision 0007) (coordinator) | no (`architecture.md` at Close) |
| Closure `review build + accept` | New route, clock and choose flow (build); the first daily screen against functional story 3 (accept) | no |

## Not in scope
- The line "N changes not seen", hiding a crossed-out game after "seen", the list of changes and its filter (story 4).
- `filter` and `period` parameters on the API.
- Remembering the filter between visits.
- A read action in the last-season view (a reading always reads the current season).
- Browser tests for the last-season view and the crossed-out look (router tests cover them; the fake source has one fixed answer).
- The nightly reading (story 5).

## Test plan
- **Existing tests that must still pass**: all of stories 1, 2 and 3a (core, router, storage, login, web), and the browser projects `choose`, `logged-in`, `logged-out`, with the changes named in steps 4 and 5.
- **New tests**: per step, above. Order of value: the core order rules, the "which schedule" router cases, the browser flow after the choice.
- **Checks**: `just check`. No new query, so `.sqlx/` does not change; `just api-types` for the new route.

## Compatibility and performance
- No migration, no new production setting, no cluster change. `DEV_NOW` exists only in debug builds.
- The start page makes one more request (`GET /api/schedule`): one query of the stored games of one club and season (about 70–120 rows).
- `/sample` is gone; the path now gives the front end's start page (the router sends unknown paths to `index.html`).
- Saving the choice with a team now also reads the game list: one more request to the source for each save.

## Progress
- [x] Step 1: core rule (2026-10-12)
- [x] Step 2: fixed clock for development (2026-10-12)
- [x] Step 3: the API (2026-10-12; `api-types-check` passes once the new types are committed)
- [x] Step 4: the schedule on the start page (2026-10-12)
- [x] Step 5: read after the choice, and last season (2026-10-12)
- [x] All checks pass (2026-10-12; `api-types-check` passes once the new types are committed)

## Deviations
From the epic's technical spec (none from the functional spec):
- `GET /api/schedule` has no `filter` and `period` parameters (the epic's API outline has them); the screen filters and the server splits coming and past (coordinator, 2026-10-12).

The design reads the functional spec more strictly than written, within its words:
- A game today is a coming game for the whole day.
- Within a date, "time not known yet" comes after the known start times.
- Past games show newest date first.
- A game no longer in the game list stays crossed out until story 4 adds "seen".
- The reading after a choice shows no result line; the schedule is the visible result.

Small choices in the build (step 2, 2026-10-12):
- The check that a release build ignores `DEV_NOW` is in `scripts/smoke-test.sh` on the real
  image, where `DEV_AUTH_BYPASS` is checked: the container with the bypass also gets `DEV_NOW`, and
  its log must say "DEV_NOW is set but ignored". `smoke-test.sh` is an image input, so a pull
  request with this change also builds the image.
- `ShiftedClock` exists only in debug builds, as `DEV_NOW` itself.

Small choices in the build (step 3, 2026-10-12):
- `GET /api/schedule` has the OpenAPI operation name `schedule`: the handler is `schedule::get`,
  and the name `get` was already taken by `GET /api/selection`, which mixed up the generated
  front-end types.

Small choices in the build (step 4, 2026-10-12):
- With no games and no good reading yet, the schedule says "The game list has not been read yet."
  instead of "The game list has no games for this season yet.": the second would be wrong before
  the first reading. Step 5 makes this state short (the choice starts the first reading).
- The schedule's screenshots (light and dark) are taken in `choose.e2e.ts` while M16 1 is chosen;
  the screenshot test in `logged-in.e2e.ts` keeps `/`, which by then shows the choose step.
- Found and fixed: after a save on `/choose`, the start page could show the old team. SvelteKit
  preloads the start page's data when the pointer passes over the "Back" link, before the save,
  and `goto('/')` used that old data (seen in a Playwright trace). `/choose` now navigates with
  `invalidateAll: true`. The bug was already in story 2; the browser test "the coordinator changes
  the team later" found it once the start page made one more request.

Small choices in the build (step 5, 2026-10-12):
- When the reading request after a save fails as a request (not a recorded failed reading), the
  choose step says nothing: the choice is saved, and the "Game list" card offers the read action.
- The last-season view was checked once by hand (2026-10-12): a scratch database, the fake source,
  M16 1 chosen and read at `DEV_NOW` 12 October 2026, then the start page at `DEV_NOW` 1 July 2027.
  It showed the choose step, "Last season · 2026-2027", Blue Drakes M16 1, and 37 past games, open,
  newest date first. Above the open past games it also says "No coming games." (true, kept).

## Review findings

### Acceptance review (2026-10-12)
By `/review accept` (fresh agent). Input: mode `accept` and this change folder; no other
arguments. Read: `change.md`, this file, the epic's `functional.md` (stories 1 and 3) and
`idea.md`, `docs/vision.md`, `docs/language.md`, and the diff of commit 4497315. Not read:
`docs/changes/done/`. Run: `cargo test -p team_core schedule` (14 passed). Looked at the browser
test screenshot `web/test-results/look/schedule-light.png` (390 px wide, 12 October 2026, fake
source). Router tests (`tests/schedule.rs`) and Playwright were not run (no database started).
The **Deviations** are in the same file as the steps, so they were read before this view was
complete; each point below was checked against the functional spec on its own.

**Story 3 (See the schedule)**

| Criterion or open point | Built | Sort | How checked |
|---|---|---|---|
| Team games home and away, home games of other club teams, no away games of other club teams | `in_schedule` + `schedule` in core | matches | read `crates/core/src/schedule.rs`; unit tests run |
| Quick filter, three choices, one tap | `ToggleGroup`, always one choice, filters coming and past on the screen | matches; see A1 | read `Schedule.svelte`, `schedule.ts` |
| Coming first, time order, grouped by date; past on request | Server splits on the Dutch date; "Show past games" | matches | read; screenshot |
| Past games newest date first; a game today is coming; unknown time last within its date | as listed under Deviations | faithful adaptation | read core rule and tests |
| Team games look clearly different | coloured edge, tinted card, badge "Team" | matches | screenshot: the team games stand out among 18 duty games |
| Day, date, start time, home, away, hall and town | all shown; "Sat 31 Oct" heading | matches | screenshot |
| 00:00 shows "time not known yet"; no results | `start_time: null`; the view has no result field | matches | read `api/schedule.rs` (no score in `ScheduleGame`) |
| A game no longer in the game list crossed out, hidden after "seen" | crossed out with a badge; stays until story 4 | faithful adaptation (coordinator's answer, recorded) | read |
| Last good reading and a failed reading on the screen | the "Game list" card above the schedule | matches | read `+page.svelte`, `ReadGameList.svelte` |
| No games yet: says so, and when last read | "has no games for this season yet" plus "Last read" on the card; before any reading "has not been read yet" | faithful adaptation | read |
| Phone, one hand | 320 px test, the first coming game is in the first screen at 390 × 800 | matches | screenshot (top ≈ 300 px is header and card) |
| Only the coordinator | route on the protected `/api` router | matches | read `app.rs`, `api/mod.rs` |
| "The first thing shown is the next games" | team header and the "Game list" card come first, then the games | neutral drift | screenshot: next games still above the fold |

**Story 1 (the parts this change builds)**

| Criterion | Built | Sort | How checked |
|---|---|---|---|
| Chooses M16 1, the tool reads the game list and shows the games | the save calls `readNow()`, then the start page loads; a failed request leaves the choice saved | matches | read `ChooseTeam.svelte` |
| After a change, the games shown follow the new choice | `/choose` navigates with `invalidateAll`; the reading runs before it | matches | read; the preload bug fix is a faithful repair of story 2 |
| New season: last season's games, and says a new choice is needed | choose step first, then "Last season · 2026-2027", team name, past games open | matches | read `+page.svelte`, `api/schedule.rs` match on `choice_state` |

**Findings**

- **A1 – concerning deviation (small, no spec change needed).** A filter with no games shows an empty area with no words. `Schedule.svelte` decides "No coming games." on the unfiltered `schedule.coming`, then renders the filtered groups; when the filter leaves nothing, nothing is shown between the filter and "Show past games" (the same for the open past games). This is a real case, not a corner: the youth teams' games stop at the end of the calendar year (functional Context), so from mid-December until the second half appears, "Games of the team" is blank — the screenshot's data already shows no M16 1 game after Sat 12 Dec. A blank list after one tap reads as "the games are lost", against "keep it correct". Say it in words (for example "No coming games of the team."), and add a test. Checked: trace of `Schedule.svelte` (`schedule.coming.length === 0` vs `coming`), and the screenshot.
  - 2026-10-12: fixed. `Schedule.svelte` decides on the filtered lists and shows a line from `noGamesLine` in `schedule.ts` ("No coming games of the team.", "No past possible duty games.", …), for the coming and the open past games. Test: browser test "a filter that leaves no games says so" (the schedule answer without coming team games, by `page.route`).
- **A2 – neutral drift.** The start page shows the team header and the "Game list" card above the schedule; the spec's behaviour shows "Last read" and then the games. At 390 × 800 the first coming games are still in the first screen, so the intent holds. Watch it when story 4 adds the line of unseen changes. Checked: screenshot.
  - 2026-10-12: not fixed: the first coming games are in the first screen at 390 × 800, so the intent holds. For the design of story 4, which adds the line of unseen changes to this page.
- **A3 – neutral drift.** In the last-season view the empty text would be "The game list has no games for this season yet." under the heading "Last season". It shows only if the earlier season has a team but no stored games (for example a choice saved in story 2, before readings were stored). Rare; the words should not say "this season" there. Checked: trace (`hasReading` is always true in that view); not run.
  - 2026-10-12: fixed. The last-season view with no games says "The tool has no games of 2025-2026." (its own season). Test: browser test "an earlier season with no kept games says which season" (`page.route`).
- **A4 – faithful adaptation.** The reading after a choice shows no result line; the schedule is the visible result, and story 4 says the first reading lists nothing as new. Checked: read.
  - 2026-10-12: no change needed.
- The deviation from the epic's API outline (no `filter` and `period` parameters) is not visible to the coordinator: faithful.

**Verdict: accepted with notes.** The schedule is what story 3 meant and the choose flow now does
what story 1's behaviour says. A1 should be fixed before release (a few lines and a test in the
build, not a spec question); A2 and A3 are for awareness.

### Build review (2026-10-12)
By `/review build` (fresh agent). Input: mode `build` and this change folder; no other
arguments. Read: commit 4497315 against 3728d7b, and the uncommitted changes in the work tree
during the review (the fix of A1 and A3 in `Schedule.svelte`, `schedule.ts` and two browser
tests), this change folder, `docs/architecture.md`, decisions 0001, 0007, 0012, 0015, `CLAUDE.md`.
Run (on the work tree with the uncommitted fix): `cargo test -p team_core schedule` (14 passed),
`cargo test -p team --test schedule` (9 passed), `--test source` (22 passed, 1 ignored),
`cargo test -p team --lib` (19 passed), `cargo clippy --all-targets -D warnings` and
`cargo clippy -p team --release -D warnings` (clean, so the `cfg(debug_assertions)` split
compiles both ways), `just api-types-check` (no diff), `svelte-check` and `npm run lint` (clean),
`npm run build` + Playwright (19 passed, including the two new tests). Not run: `just images` /
`just smoke`, so the release image's handling of `DEV_NOW` was not seen.

No finding blocks Close. The core rule, the route, the `DEV_NOW` split and the choose flow follow
the project's rules; the A1/A3 fix covers both lists (coming and past) and the last-season view.

- **R1 – logic in the wrong place (low).** Which schedule shows is a product rule (functional
  story 1: in a new season, last season's games until the team is chosen again), but it is
  decided in the handler `api/schedule.rs::get` (the `match` on `choice_state`: `TeamNeeded` with
  an earlier team → that season with `last_season: true`, else `null`). The architecture says
  "new rules go in `crates/core` first; `crates/server` only moves data in and out" (decision
  0001). It is covered only by router tests, not by a core unit test. A side effect of the rule
  as written: only the *newest* earlier choice is looked at, so if that one has a club and no team
  (a club saved with no games yet) while an older season had a team, no schedule shows. That
  matches the design's table, so it is not a bug, but a core function (for example
  `team_choice::schedule_target(state) -> Option<(Season, Club, Team, bool)>`) would make it a
  tested rule. Checked: read of `api/schedule.rs` against `architecture.md` "Patterns to follow".
  - 2026-10-12: fixed. `team_core::schedule_target(state) -> Option<ScheduleTarget>` (in `team_choice.rs`) is the rule; the handler only loads the choice and the games. The behaviour is unchanged (only the newest earlier choice is looked at, as designed). Tests: core `the_schedule_is_of_the_chosen_team`, `while_a_team_is_needed_the_schedule_is_of_last_season`, `without_a_team_in_this_or_the_last_season_there_is_no_schedule`; the 9 router tests still pass.
- **R2 – a failed schedule reload looks like a failed reading (low).** `ReadGameList.run` awaits
  `onRead()` inside its `try`. When `GET /api/schedule` fails after a good reading,
  the `catch` sets `error` and the card shows "Something went wrong. Please try again." (or the
  API message) right under the result line of a reading that succeeded, and the old schedule
  stays. The twin path "These games are really gone" (`confirmGone`) goes through the same `run`,
  so it has the same effect. Catch the reload on its own (in `refreshSchedule` or around
  `onRead`). Checked: trace of `ReadGameList.svelte` and `+page.svelte`; not run.
  - 2026-10-12: fixed. `refreshSchedule` on the start page catches its own failure and shows "The new schedule could not be loaded. Load the page again." above the old schedule; the card shows only the reading. This covers "These games are really gone" too (same callback). Test: browser test "a schedule that cannot be loaded after a reading does not look like a failed reading".
- **R3 – the change of team is not tested against the schedule (low, test gap).** "The
  coordinator changes the team later" checks only the team name after the save. Nothing (browser
  or router) checks that the schedule then follows M16 2 (its team games marked, M16 1's games
  as duty games or gone), although the `invalidateAll` fix of step 4 exists for exactly that
  case. One assertion on `[data-of-team="true"]` after the change would close it. Checked: read
  of `choose.e2e.ts` and `tests/schedule.rs` (search for a second team number: none).
  - 2026-10-12: fixed. "The coordinator changes the team later" now checks that the 5 team games are games of Blue Drakes M16 2 and none of M16 1.
- **R4 – possible flake in the smoke test (low).** `scripts/smoke-test.sh` runs with
  `set -o pipefail` and checks `docker logs team-smoke-bypass 2>&1 | grep -q 'DEV_NOW is set but
  ignored'`. `grep -q` exits at the first match; if `docker logs` is still writing later lines,
  it gets SIGPIPE, the pipeline fails and the script reports that the release build does not
  ignore `DEV_NOW`. Capture the log first (`logs=$(docker logs … 2>&1)`) or use
  `grep -q … <(docker logs …)`. Checked: read of the script; not run (no image build).
  - 2026-10-12: fixed. The script reads the log into a variable first, then `grep -q` on it. `just images` and `just smoke` pass ("ok   the release build ignores DEV_NOW").
- **R5 – sticky date headings under the header in the installed app (low, not checked on a
  device).** The date headings stick at `top-14` (3.5 rem), but the header is `h-14` plus
  `pt-[env(safe-area-inset-top)]`, and the app has `viewport-fit=cover` and a manifest with
  `"display": "standalone"`. Where the top inset is not 0, a sticky heading slides partly under
  the header. Checked: search of `sticky\|h-14` in `+layout.svelte`, `viewport` in `app.html`.
  - 2026-10-12: fixed. The date headings stick at `calc(3.5rem + env(safe-area-inset-top))`, the height of the header with its top inset. Not checked on a device (no test can set the inset); the browser tests and screenshots are unchanged where the inset is 0.
- **R6 – documents the change makes false (for `/close`).** Checked: `git grep -i sample` and a
  read of `architecture.md`, `README.md`.
  - `docs/architecture.md`: the debt line "The sample page `/sample` … is temporary"; the debt
    line "Places are shared between seasons … Decide in story 3b" (decided, R5 of story 3a); the
    layer table (core `schedule` now also `schedule`; server `clock.rs` `ShiftedClock`, `api/`
    `schedule`; web `Schedule.svelte`, `schedule.ts`); `DEV_NOW` next to `DEV_AUTH_BYPASS` and in
    the test patterns (browser tests run on 12 October 2026); saving a choice with a team now also
    reads the game list.
  - `docs/product/read-the-game-list.md` line 25: "The schedule itself is not shown yet; the
    sample page shows how it will look."
  - `docs/product/choose-club-and-team.md` line 24: saving a team now also starts a reading.
  - `README.md` settings table: `DEV_NOW` (debug builds only) is missing next to
    `DEV_AUTH_BYPASS`.
  - The epic's technical.md row 3b still says stage `implement`.
  - 2026-10-12: not fixed: for `/close` (architecture, product documents, README, the epic's row and progress).

Checked and found right: `today` is the server's Dutch date, so a wrong phone clock moves
nothing; a game today is coming; unknown times sort last within a date in both lists; the route is
on the protected router (401 test); no new query, so `.sqlx/` is unchanged (as claimed); the
`/choose` preload fix is the only `goto` in `web/src`, so there is no twin; the `DEV_NOW` field,
parser and `ShiftedClock` exist only in debug builds and the release build warns; no secrets or
tokens in the new log lines.
