# Technical spec: The report

**Status**: done
**Size**: story
**Change**: docs/changes/2026-11-09-the-report
**Date**: 2026-11-09

## Objective
A page "Report" (`/report`) for the current season that shows, for each player, attendance at
trainings and games as percentages and the duties as done + planned, with the details of a player
on a tap and a Dutch text to copy for each column, from the same counts as "Attendance" and
"Duties", read at one moment.

## Requirements summary
From `functional.md` (approved 2026-11-09):

- **Story A, the table.** The link "Report" after "Duties" on the start page ("Season · Attendance
  · Duties · Report · Team · Players · Settings"). Current season only, heading "Report 2026-2027",
  no season choice. Columns: name, Trainings, Games, Drive, Table, Referee.
  - Rows: each player with a training or game that counts, or with the role "plays games" in the
    season, or on a duty place of the season; hidden players too; sorted by first name; the short
    names of "Duties" ("Anna B.").
  - Players with a role today or later are in the table; all others under "Left the team".
  - Trainings and Games: the percentage "there" of what counts, whole number, half up (2 of 3:
    67%; 1 of 8: 13%); a valid absence lowers it. "–" when nothing counts.
  - Drive, Table, Referee: "done + planned" ("1 + 2"). "–" for a player without the role "plays
    games" in the season and without a duty, and for all players when no team is chosen.
  - Above the table, when not 0: "2 training days not opened · 1 game not checked" (the training
    days of "Attendance", the games of the open item "not checked" of the start page). A tap opens
    the oldest; "Back" there returns to "Report".
  - "No trainings, games or duties yet this season." without rows. "The report could not be loaded.
    Load the page again." when loading fails. Fits at 360 pixels with names of 10 characters.
- **Story B, the details.** A tap on a name opens the details under the row; a second tap closes;
  more than one can be open; also under "Left the team".
  - "9 trainings: 6 there (67%) · 2 absent, valid (22%) · 1 absent, not valid (11%)"; the same for
    games. A line where nothing counts is not shown. Each percentage on its own, half up.
  - For each duty kind with a duty: "Table: 2 done · 1 planned", and below it, when not 0:
    "1 done by another player", "1 done for another player", "1 not done".
  - "Nothing counts yet this season." for a player with no line.
  - "Missed trainings and games" opens "Attendance" with the list of that player open; "Back"
    returns to "Report".
- **Story C, copy.** "Copy" under each column; "Copied", or the text with "Select the text and copy
  it." when the browser refuses. Dutch text: heading "Rijden Blue Drakes M16 1, seizoen 2026-2027
  (tot en met 9 nov)" (without a team: "Rijden seizoen 2026-2027 (tot en met 9 nov)"), then one line
  for each player with a role today or later whose cell is not "–": "Anna 90%"; "Anna 3 (+1
  gepland)", "Bram 0". Kinds: Trainingen, Wedstrijden, Rijden, Tafel, Scheidsrechter. No "Copy"
  when no line is left. No valid split, no corrections.

## Current state
- **Attendance counts**: `team_core::season_overview` (`crates/core/src/contribution/attendance.rs`)
  gives `SeasonOverview { trainings, games, team, left, not_opened, first_not_opened }` with a
  `PlayerTotals` for each player with a training or game that counts (there, absent valid, absent
  not valid, `GameTotals`, `missed`), grouped by `place` (the rule of the page "Team"). Used by
  `crates/server/src/api/attendance.rs`.
- **Duty counts**: `team_core::duty_totals` (`crates/core/src/contribution/duties.rs`) over
  `GameDuties` (game, places, assignments, past, started) gives `BTreeMap<PlayerId, KindTotals>`;
  `KindTotals { drive, table, referee }` of `Total { done, planned }`. A place counts when
  `place_counts` (its part counts, and past or a place of the team) or `drove` (a not played game
  whose drivers drove); `who_did` turns an `Assignment { kind, number, planned, outcome }` with
  `Outcome::{AsPlanned, DoneBy(PlayerId), NotDone}` into done, planned or nobody. The rows of
  "Duties" are `duty_rows` (role "plays games" in the season, or in `totals`). Used by
  `crates/server/src/api/duties.rs` with `SeasonPlaces` (`api/season.rs`).
- **Games not checked**: `team_core::games_not_checked` (`crates/core/src/output/start_page.rs`),
  over the schedule, the checks and the release day `game_attendance::check_start`.
- **Loading**: `trainings_of(state, season)`, `games_of(state, season, current, today)` and
  `duties_of(state, club, season)` in `crates/server/src/api/season.rs` read from the pool, each
  step on its own; `games_of` loads the duties too (`load_duties` on one connection) and the open
  changes (`changes::list`, reads only: `changes::open` and `duties::with_duties` are plain
  `SELECT`s; checked 2026-11-09). Nearly all store functions take `impl PgExecutor` or
  `&mut PgConnection`.
- **Names**: `display_names` of the season's players (`SeasonDuties.names`).
- **Messages**: `crates/core/src/output/messages.rs` makes the Dutch texts (decision 0024), with the
  Dutch month names. The copy with its fallback is inside `web/src/lib/components/MessagePage.svelte`.
- **Back links**: the training page and the game page read `?from=` (`start`, `attendance`,
  `changes`); "Attendance" has "Back" to the start page only and opens a player's list by a tap.

## Steps

### Step 1: The corrections in the duty totals
- **Goal**: count "done by another player", "done for another player" and "not done" in the only
  source of the duty totals, without changing "Duties" or the choice of a game.
- **Changes** (`crates/core/src/contribution/duties.rs`):
  ```rust
  pub struct Total {
      pub done: u32,
      pub planned: u32,
      /// Planned for this player; another player did it ("Someone else").
      pub done_by_other: u32,
      /// This player did it in place of the planned player (also in `done`).
      pub done_for_other: u32,
      /// Planned for this player; nobody did it ("Nobody").
      pub not_done: u32,
  }
  ```
  In `duty_totals`, for a place that counts (`place_counts || drove`, as now) at a started game:
  `AsPlanned` → `done` of the planned player (as now); `DoneBy(b)` with `b != planned` → `done` and
  `done_for_other` of `b`, `done_by_other` of the planned player; `NotDone` → `not_done` of the
  planned player. A game that has not started counts only `planned`, as now.
  `duty_rows`: a player is in the rows when the role "plays games" is in the season, or the totals
  of the player have `done + planned > 0` for some kind (so a player who was only planned and then
  replaced does not get a new row on "Duties"). The 9 `Total { .. }` literals in the tests get
  `..Default::default()`. `TotalView` in `api/games.rs` keeps `done` and `planned` only.
- **Tests**: `duty_totals`: as planned; "Someone else" (B: done 1, done_for_other 1; A:
  done_by_other 1, done 0); "Nobody" (not_done 1, done 0); no correction before the start; none at a
  not played game; a correction at a not played game with "The drivers drove"; none at a place that
  does not count (a coming game that left the game list). `duty_rows`: a replaced player without
  the role "plays games" has no row.

### Step 2: The report in the core
- **Goal**: the rows, the percentages and the copy texts of the report, in the output domain.
- **Changes**: new module `crates/core/src/output/report.rs`, declared in `output/mod.rs` and
  re-exported in `lib.rs`; added to the table of `crates/core/tests/domains.rs` if that lists
  modules.
  ```rust
  pub struct Share { pub counted: usize, pub there: usize,
                     pub absent_valid: usize, pub absent_not_valid: usize }
  pub struct ReportRow<'a> {
      pub player: &'a Player,
      pub left: bool,                 // under "Left the team"
      pub trainings: Option<Share>,   // None: nothing counts ("–")
      pub games: Option<Share>,
      pub duties: Option<KindTotals>, // None: "–" (no role "plays games", no duty, or no team)
  }
  pub fn report_rows<'a>(season: Season, today: NaiveDate, players: &'a [Player],
      roles: &[PlayerRole], overview: &SeasonOverview<'a>,
      duties: Option<&BTreeMap<PlayerId, KindTotals>>) -> Vec<ReportRow<'a>>;
  pub fn percent(part: usize, whole: usize) -> Option<u8>;
  pub enum ReportColumn { Trainings, Games, Drive, Table, Referee }
  pub fn report_copy(column: ReportColumn, rows: &[ReportRow<'_>],
      names: &BTreeMap<PlayerId, String>, club_and_team: Option<(&str, &str)>,
      season: Season, today: NaiveDate) -> Option<String>;
  ```
  - Rows: the players in `overview.team` or `overview.left`, with the role "plays games" in
    `season`, or in `duties`; sorted by `Player::sort_key`.
  - `left`: `place(roles, player, season, today.min(season.last_day())).group != Group::Team`
    (so a player with a duty and no role in the season is under "Left the team").
  - `trainings`: `Some` when the player's `trainings > 0`; `games`: when `games.games > 0`.
  - `duties`: `None` without a team (`duties` is `None`); else `Some` (zeros when absent from the
    map) when the role "plays games" is in the season or the player is in the map.
  - `percent`: `None` for `whole == 0`; else `(200 * part + whole) / (2 * whole)` (half up).
  - `report_copy`: heading `"{Kind} {club} {team}, seizoen {season} (tot en met {d} {mon})"`,
    without club and team when `None`; the kinds "Trainingen", "Wedstrijden", "Rijden", "Tafel",
    "Scheidsrechter"; the short Dutch date ("9 nov") from the month names of `messages.rs` (add a
    short form there if none exists, and share it); one line for each row with `!left` and a value
    in the column: "Anna 90%"; "Anna 3 (+1 gepland)" or "Bram 0". `None` when no line is left.
- **Tests**: `percent` (2/3 → 67, 1/8 → 13, 1/2 → 50, 9/9 → 100, 0/0 → None); `report_rows`: the
  union of the three sets; team and left (a role that starts later is in the team; a duty without a
  role is left); "–" for games of a player who only trains; all duties `None` without a team; zeros
  for a player with the role "plays games" and no duty; sort order; hidden players included.
  `report_copy` for each column: heading with and without a team; only `!left`; no "–" lines;
  "(+1 gepland)" only when planned > 0; `None` when no line is left.

### Step 3: The server
- **Goal**: `GET /api/report`, read at one moment.
- **Changes**:
  - `api/season.rs`: `trainings_of(db: &mut PgConnection, season)`, `games_of(state, db, season,
    current, today)` and `duties_of(state, db, club, season)` take the connection; every read in
    them uses it (`followed_teams::for_season`, `games::stored`, `changes::list`, the trainings and
    absences). The callers ("Attendance", "Duties", "Season", the start page, the messages, and any
    other) acquire a connection and pass it; their behaviour does not change.
  - New `crates/server/src/api/report.rs`, routed in `app.rs` under `/api` (needs a session,
    decision 0007), in the OpenAPI document (`main.rs openapi`). The handler:
    `let mut tx = state.pool.begin()`; `SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY`;
    all reads on `&mut *tx` (players, reasons, `trainings_of`, `games_of`, `games::stored` for
    `SeasonPlaces`, `check_start`); then `season_overview` and `counted_games` as in
    `api/attendance.rs`, `duty_totals` as in `api/duties.rs` (only with a team), `report_rows`,
    `games_not_checked` (as the start page, with the release day), and `report_copy` for the five
    columns. The transaction ends with the handler (rollback is fine: nothing was written).
  - The response (types with `ToSchema`):
    ```json
    {
      "season": "2026-2027",
      "incomplete": { "not_opened": 2, "first_not_opened": "2026-10-13",
                      "not_checked": 1,
                      "first_not_checked": { "season": "2026-2027", "id": 4711 } },
      "team": [ { "player": 3, "name": "Bram", "hidden": false,
                  "trainings": { "counted": 9, "there": 6, "absent_valid": 2,
                                 "absent_not_valid": 1, "percent": 67,
                                 "percent_valid": 22, "percent_not_valid": 11 },
                  "games": null,
                  "duties": { "drive": { "done": 0, "planned": 0, "done_by_other": 0,
                                         "done_for_other": 0, "not_done": 0 },
                              "table": { "...": 0 }, "referee": { "...": 0 } } } ],
      "left": [],
      "copies": { "trainings": "Trainingen ...", "games": null, "drive": "Rijden ...",
                  "table": null, "referee": null }
    }
    ```
    `first_not_checked` is the oldest game not checked. All percentages come from `percent`.
  - The TypeScript types: `just api-types` (checked by `api-types-check`).
  - Names never go into a log; errors log the kind only, as the other handlers.
- **Tests**: new `crates/server/tests/report.rs` (`#[sqlx::test]`):
  - **The comparison test**: a season with trainings, absences (valid and not valid), checked
    games, a not played game with "The drivers drove", duties with "Someone else" and "Nobody";
    for each player, `/api/report` gives the same there, counted and absences as `/api/attendance`,
    and the same done and planned as `/api/duties`.
  - `incomplete` gives the same counts as the open items of `/api/start`.
  - Without a team: all `duties` are `null`; the attendance works.
  - Not logged in: 401.
  - The read-only transaction: the request succeeds (it fails if a read in it writes).
  - New `crates/server/tests/report_logs.rs`: no player name in the log when the report loads.
  - The existing tests of attendance, duties, season, start and messages still pass.

### Step 4: The page "Report"
- **Goal**: the table of story A on the phone.
- **Changes**: `web/src/routes/report/+page.ts` and `+page.svelte` (the load function as on
  "Duties"; a `loadReport` in the API client). The table with five narrow columns, the group "Left
  the team", the line about missing numbers (a tap on a part opens
  `/training/{first_not_opened}?from=report` or `/game/{season}/{id}?from=report`), the empty text
  and the error text. The start page: the link "Report" after "Duties". The training page and the
  game page: `from=report` makes "Back" go to `/report`.
- **Tests**: new `web/e2e/report.e2e.ts`: the link on the start page; the table with example
  numbers, "–" and "Left the team"; the line and "Back" from the training day and the game; the
  empty text; no horizontal scroll at 360 pixels with names of 10 characters.

### Step 5: The details
- **Goal**: story B.
- **Changes**: on `/report`, a tap on a name toggles a set of open players; the details under the
  row with the lines of story B (from the response; nothing calculated in the page). The link
  "Missed trainings and games" goes to `/attendance?player={id}&from=report`. On "Attendance":
  `player` opens the list of that player at load (current season), and `from=report` makes "Back"
  go to `/report`.
- **Tests**: `report.e2e.ts`: open, close, two open at once, a player under "Left the team", the
  three correction lines; `trainings.e2e.ts`: "Attendance" with `?player=&from=report` shows the
  list open, and "Back" goes to the report.

### Step 6: Copy
- **Goal**: story C.
- **Changes**: move the copy with its fallback out of `MessagePage.svelte` into a small component
  (for example `web/src/lib/components/CopyText.svelte`) that `MessagePage.svelte` and `/report`
  both use. "Copy" under each column heading when its text in `copies` is not `null`.
- **Tests**: `report.e2e.ts`: "Copy" puts the text on the clipboard and shows "Copied"; with the
  clipboard refused, the text shows with "Select the text and copy it."; no "Copy" for a column
  without lines. `messages.e2e.ts` still passes.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| The corrections are counted in `duty_totals`, in the same pass as done and planned | One source for each total: they follow the rules of "done" without a second count | 0033 |
| The rows of the report are the union of three sets of players, in `report_rows` (output) | The rule is written in one place; "Attendance" and "Duties" keep their own subsets | 0033 |
| The report reads in one read-only transaction (repeatable read) | All numbers from one moment, so a row never contradicts itself; the store functions already take a connection | 0033 |
| The other pages keep separate reads | They show one kind of number each; the helpers can take a transaction later | 0033 |
| A player with a duty but no role in the season is under "Left the team" | The page "Team" has no group for that player; the player is not in the team today | 0033 |
| The report is in the output domain | It combines contribution, team and events, as the start page does; the texts are made in the core (decision 0024) | no (decision 0027) |

## Not in scope
- Sorting by a column, or any ranking.
- A season choice or another period.
- A list with the date of each missed event or duty inside the report.
- Other changes to "Attendance" or "Duties" than `?player=&from=report`.
- The valid absences in the copied text (a line in `docs/ideas.md`).
- One transaction for the other pages.

## Test plan
- **Existing tests that must still pass**: the unit tests of `contribution/duties.rs` and
  `output/messages.rs`; `crates/core/tests/domains.rs`; the server tests `attendance.rs`,
  `attendance_games.rs`, `duties.rs`, `season.rs`, `start.rs`, `messages.rs`, the log tests; the
  browser tests `trainings.e2e.ts`, `duties.e2e.ts`, `start.e2e.ts`, `messages.e2e.ts`.
- **New tests**: see each step. The comparison test of step 3 is the main proof that the report
  shows the numbers of the other pages.
- **Manual check after the release**: the table on the coordinator's phone with the real names
  (the width is an estimate). Goes into `docs/backlog.md` at close.

## Compatibility and performance
- New page and new route; no migration, no data change. "Duties" and the choice of a game show the
  same as before (step 1 keeps `duty_rows` and `TotalView` as they are for the user).
- The report loads one season of trainings, games and duties, as "Attendance" and "Duties" do
  together: small. The transaction is read-only and short.

## Progress
- [x] Step 1: the corrections in the duty totals
- [x] Step 2: the report in the core
- [x] Step 3: the server
- [x] Step 4: the page "Report"
- [x] Step 5: the details
- [x] Step 6: copy
- [x] All checks pass

## Deviations
- Design: no deviation. One wording of `functional.md` (story B: the corrections count "only on the places of the
  team") was changed with the coordinator at design (2026-11-09) to "with the same rule as done":
  that rule also counts the past places of an earlier team, as the duty totals do, so the
  corrections always explain the "done" number.

- Step 3 (2026-11-09): the loading helpers keep their names and signatures (`trainings_of`,
  `games_of`, `duties_of`, 16 callers) and each gets a variant on a connection: `trainings_in(db,
  season)`, `games_in(state, db, ...)`, and `load_duties` made `pub(crate)`. The old names acquire a
  connection and call the new ones, so the callers did not change. Same behaviour as designed, fewer
  changed lines. `trainings_of` now reads its four queries on one connection instead of the pool;
  the results are the same.
- Step 3: without a chosen team the names are the short names (`display_names`) of the players with
  a role in the season, since `SeasonDuties.names` is empty then.
- Step 5: the test of "Attendance" opened from the report (`?player=&from=report`) is in
  `web/e2e/report.e2e.ts`, not in `trainings.e2e.ts`: it follows the link from the details, and the
  report tests run last, after all data exists.
- Step 6: the copy is shared as a function (`web/src/lib/copy.ts`, `copyText`), not as a
  component: on the report the buttons are in the column headings and the fallback text shows below
  the table, so one component with a button and a text does not fit both pages. The message pages
  use the same function and show the same words ("Copied", "Select the text and copy it.").
- Step 6: "Copy" is only in the table of the team: the texts hold only the players in the team.
- Review R6/A3: the details show a duty kind that has only a correction ("Table: 0 done · 0
  planned" with "1 not done"): story B lists kinds "with at least one duty"; a correction is a duty
  that was planned for the player, so it is shown.
- Review fixes (2026-11-09): `has_duty_row` (core) and `overview_of` (server) are new shared
  functions, not in the design; they put the rule of the duty rows and the feed of the attendance
  totals in one place each (R4, R5).

## Review findings

### Build review (2026-11-09, `/review build`, fresh agent)
Read: the diff against `main` (committed and not), this change folder, `docs/architecture.md`,
decisions 0022, 0023, 0024, 0027, 0031, 0033, `CLAUDE.md`. Ran: `cargo test -p team_core` (354 + 7
pass), `cargo test -p team --test report|report_logs|duties|attendance|start` (all pass), `cargo
clippy --workspace --all-targets -D warnings` (clean), `npx playwright test --project=report` (169
pass, with all the projects it depends on). Then a server on the e2e database after that run (GET
requests only) and the page at 360 pixels in a browser.

Overall: the structure follows the decisions. The corrections are in `duty_totals`. The report is
in `output` and uses only allowed domains (checked by reading the `use` lines and by running
`tests/domains.rs`). Every read of the handler is on the transaction: a search for `state.pool`
in `games_in`, `trainings_in`, `load_duties` and `api::changes::list` found none. `Duties` and the
choice of a game are unchanged, because `duty_rows` uses `has_duty` and `TotalView` keeps done and
planned only (traced). No blocking bug was found. The findings are about tests that cannot fail and
two places where a rule now lives twice.

- **R1 – the 360-pixel test cannot fail, and the headings run together (minor, the page).** The
  table is `table-fixed w-full` and the name is `truncate`, so the document never gets wider than
  the screen, whatever the content. `expectNoHorizontalScroll(page, 360)` always passes, and
  `toBeVisible` on "Wilhelmina" was checked at 390 pixels, before the resize. Measured at 360
  pixels with the e2e data: no name is cut off ("Wilhelmina" is 91 of 91 pixels, flush against
  the Trainings column). But the headings overflow their cells: "Trainings" is 57 pixels in a
  47-pixel cell, so the screen reads "TrainingsGames". "Drive", "Table" and "Referee" with "Copy"
  are 48 pixels in 47-pixel cells, and "Referee" touches the border. A cell "12 + 3" is 48 of 47
  pixels. Checked by a run (Playwright, `scrollWidth` against `clientWidth`, and a screenshot). A
  test that compares `scrollWidth` with `clientWidth` of the cells and of the name button would catch
  this. The manual check on the phone that the test plan names stays needed.
  - 2026-11-09 fixed: the headings may break with a hyphen (`hyphens-auto`, 11 px) and the cells are 13 px with less padding. New browser test "every heading and cell fits its column on a phone": at 360 px, with "Wilhelmina" and "12 + 3" in all duty cells, no heading, cell or name has `scrollWidth > clientWidth`; it measures the cells, so it fails when a heading runs over.
- **R2 – the branch "game not checked" never runs in the browser tests (test gap).**
  `global-setup.ts` sets the release day to the day of the test clock (12 October), so
  `games_not_checked` (range `from_date..today`) is always empty. In
  `report.e2e.ts`, "what does not count yet …" skips the game link, the URL with `?from=report` and
  "Back" from a game. That test also returns early when both counts are 0. The game page's
  `from=report` has no browser test at all. Checked by search (`game_check_start` in `web/e2e/`)
  and by a run: `/api/report` after the full run gives `not_checked: 0`. The server test
  `what_does_not_count_yet_is_the_same_as_on_the_start_page` does cover the count.
  - 2026-11-09 fixed: new browser test "1 game not checked" replaces the answer of `/api/report` with a real past game of the team, follows the link to the game page and "Back" to the report.
- **R3 – some tests in the plan are missing, and the progress says done (claims).**
  - Step 3, the comparison test: the plan names "a not played game with 'The drivers drove'". The
    test has none. The test also does not compare `absent_valid` of games (line 203 compares only
    `absent_not_valid`), and it has no hidden player and no player under "Left the team".
  - Step 4: "the empty text" ("No trainings, games or duties yet this season.") has no browser test
    (search: `report-empty` in `web/e2e/`). The server's empty answer is tested.
  - Step 5: "the three correction lines" depend on data. After the full run every duty is `0 + 0`
    (run), so the details test checks no correction line in the page. The lines are only checked by
    `details()` in the test, which copies the page's logic, so a wrong rule in both still passes.
  - "The read-only transaction" has no test of its own. A write in a helper would fail every
    report test, which is enough for that half of the claim. Nothing tests the "one moment" half:
    removing the `SET TRANSACTION` line breaks no test. That is acceptable, but it should not be
    claimed as tested.
  - 2026-11-09 fixed, except one moment: the comparison test now has a valid absence at a game (compared too), a hidden player and a player who left (compared in their groups); a new server test covers a not played game with "The drivers drove" and "Someone else" on the drive. New browser tests show the empty text, the error text and the three correction lines (with a replaced answer of `/api/report`). Not tested: that all numbers come from one moment; a test would need two connections racing inside one request. The transaction is in the code, and a write in it fails at once (read-only).
- **R4 – the attendance part of the handler is a copy of `api/attendance.rs` (rule in two
  places).** `report::get` rebuilds the reason list, the chain of past and coming games,
  `counted_games` and `season_overview`, line for line as `attendance::get` does. The core
  functions are shared, but the server code that feeds them is not. If "Attendance" changes what it
  passes in (for example a filter of the games), the report drifts silently, which is the first
  warning in `functional.md`. The comparison test would catch it only for the cases it builds (see
  R3). Better: one helper in the server that both handlers call, taking the loaded data. Found by
  reading the two handlers side by side.
  - 2026-11-09 fixed: `api/attendance.rs` has `overview_of` (reasons, counted games and `season_overview`), used by "Attendance" and the report, so the feed is in one place.
- **R5 – "plays games in the season" and "in the duty rows" are written twice (twin case).**
  `report_rows` repeats the role test of `duty_rows` (`role.season == season && role.role ==
  PlaysGames`). For the duties it uses `duties.get(..)` (any entry), where `duty_rows` now uses
  `has_duty`. So a player whose only duty entry is a correction (planned, then replaced or
  "Nobody"), with no role "plays games" in the season and no attendance, gets a report row with
  "0 + 0" and none on "Duties". `functional.md` ("the rows of 'Duties'") does not allow that, but
  decision 0033 ("the players in the duty totals") does. It is rare, because `check_assignment`
  needs the role on the date, so only a role removed later gives it. Traced, not run. Choose one
  rule, and use a shared predicate from `contribution::duties` so the two cannot drift.
  - 2026-11-09 fixed: `team_core::has_duty_row` is the one rule (role "plays games" in the season, or a duty done or planned); `duty_rows` and `report_rows` both use it. A player with only a correction now has no duty numbers in the report, as on "Duties" (unit test `a_player_with_only_a_correction_has_no_row_as_on_duties`).
- **R6 – the details show a duty kind that has only a correction (spec reading).** `dutyLines`
  shows a kind when done + planned + any correction > 0. Dora in `tests/report.rs` would show
  "Table: 0 done · 0 planned" with "1 not done" under it. Story B says "for each duty kind with at
  least one duty of the player". This reading explains the number, which the spec wants, but it is
  a choice and is not under Deviations. The page also does this sum itself, while its header says
  "the page calculates nothing". Found by reading.
  - 2026-11-09 not changed, written under Deviations: a kind with only a correction is shown, because the correction is the reason to look ("1 not done").
- **R7 – documents the diff makes false (for `/close`).** `docs/architecture.md`: the line of start
  page links (line 18), the output modules (line 31: no `report`), `Total`/`duty_totals` without
  the corrections (line 63), `MessagePage.svelte` as the copy (now `src/lib/copy.ts`), no
  `/api/report` and no `/report`. `CLAUDE.md`, Architecture: `output` and the list of `api/` files
  lack the report. Decision 0033 says "the loading helpers of `api/season.rs` take a connection".
  In fact they got `*_in` variants (Deviations, step 3), and its Consequences name only the `*_of`
  helpers. The record stays as it is, but `architecture.md` should name the `*_in` helpers. Found
  by search (`grep` on `docs/architecture.md` for `output`, `duty_totals`, `MessagePage`).
  - 2026-11-09 for `/close`: `docs/architecture.md` (output `report`, `has_duty_row`, the corrections in `duty_totals`, the `*_in` helpers, `overview_of`, `copy.ts`, the route and the page) and the architecture part of `CLAUDE.md`.

### Acceptance review (2026-11-09, `/review accept`, fresh agent)
Read: `change.md`, `idea.md`, `functional.md` (with the spec review and the answers), the diff
against `main` (committed and not), then the result. I formed my view before I read
**Deviations**. Ran: `cargo test -p team --test report --test report_logs` (6 pass); `npx
playwright test --project=report` with the projects it depends on (169 pass when run alone. A
first run failed one test in `trainings.e2e.ts` because another session ran the same suite at the
same time on the shared `team_e2e` database. This is not a fault of the change). Then a server of
my own on `team_e2e` (port 4300, GET requests and a Copy only) and the page in a browser at 360 ×
740 pixels.

**Verdict: accepted with notes.** The page does what was meant. It shows one table of what each
player gave, without ranking. The numbers come from the same counts as "Attendance" and "Duties",
read at one moment. The details explain a low percentage and a difference in duties. Each column
can be copied as Dutch text. A1 and A2 are small faults of the page on the phone, the main screen.
They should be fixed in the build before `/close`. They do not need the spec to change.

For each criterion (checked by reading the code, plus the run named):
- Story A. The link after "Duties" and the heading "Report 2026-2027" match (e2e run). Rows are the
  union of attendance, the role "plays games" and duty entries. "Left the team" uses `place`, so
  "today or later" works as on "Team" (`report_rows`, unit test with a role from December). Half
  up: `(200·part + whole) / (2·whole)` gives 67 for 2 of 3 and 13 for 1 of 8 (unit test). "–" and
  "0 + 0" follow the rule, and every duty cell is "–" without a team (server test). The line
  "N training days not opened · N games not checked" uses the same functions as the start page,
  and its links carry `?from=report` (traced). The empty and the error texts are as specified.
  Width: see A1.
- Story B. The details open and close, several can be open, and they work under "Left the team"
  (e2e). The lines and the separate percentages are as specified. The corrections are counted in
  `duty_totals` in the same pass as "done". So they count only at games and places where "done"
  counts, a place set back to its planned player gives no correction (`player !=
  assignment.planned`), and "The drivers drove" is honoured (unit tests). "Missed trainings and
  games" opens "Attendance" with the player open, and "Back" returns (e2e).
- Story C. The heading is in Dutch, with the team or without it, and "tot en met" the date of the
  tool's clock (`today_in_netherlands`). The text holds only the team, leaves out "–" cells, and
  shows "(+N gepland)" only when the planned number is not 0. A column with no lines has no
  "Copy". The fallback is "Select the text and copy it." with `select-all` (unit, server and e2e
  tests).

- **A1 – at 360 pixels the column headings run together (concerning deviation, small).** The
  criterion "fits on a phone 360 pixels wide without scrolling to the side, with names of up to 10
  characters" holds as written: the document is 360 pixels wide, and "Wilhelmina" is not cut off
  (measured: `scrollWidth` against `clientWidth`). But the intent was a table that is quick to
  read on the phone. "Trainings" is 53 pixels wide in a 47-pixel column, so the screen shows
  "TrainingsGames". "Referee" and the last column touch the border of the card (screenshot). The
  numbers can be read, the headings of the first two columns cannot. Same as R1. The fix is in the
  page, for example a shorter heading or a narrower name column. The spec does not need to change.
  - 2026-11-09 fixed with R1.
- **A2 – "Copied" shows below the table, far from the "Copy" that was tapped (concerning
  deviation, small).** Story C: "A tap … shows 'Copied'". The "Copy" buttons are in the column
  headings at the top. "Copied" and, when the browser refuses, the text to select are placed under
  the whole table of the team. With the 10 players of the e2e data at 360 × 740, "Copied" was at
  y = 725 of 740 (measured after a tap on "Copy" of Referee). A team of 12 or more pushes it off the
  screen. The coordinator then sees no answer to the tap, or does not see the text to copy by hand.
  Deviation step 6 explains why the place differs from the message pages, but not that the
  feedback can leave the screen. A short state on the button itself ("Copied"), or the fallback
  shown in view, would keep the intent.
  - 2026-11-09 fixed: "Copied" and the text to select now show above the table, under the line about missing numbers, close to the buttons in the headings. Browser test "the answer to \"Copy\" shows above the table".
- **A3 – a duty kind with only a correction is shown in the details (faithful adaptation).**
  "Table: 0 done · 0 planned" with "1 not done" under it. Story B says "for each duty kind with at
  least one duty". The purpose of the details is to explain the numbers, and hiding the only
  correction would hide what the coordinator wants to explain. Same as R6. It belongs under
  Deviations.
  - 2026-11-09 not changed, written under Deviations (see R6).
- **A4 – "games not checked" counts only the current season (faithful adaptation).** The start
  page also counts earlier seasons (`api/start.rs`, the loop over `seasons.iter().skip(1)`). The
  report is about the current season, so counting its own games is the right reading of "the games
  that are the open item 'not checked'". The server test compares with the start page only when
  both have games of one season. Checked by reading.
  - 2026-11-09 not changed: accepted as the reading for a report of one season.
- **A5 – a player with only a correction can get a row that "Duties" does not have (neutral
  drift).** Same as R5: `report_rows` uses "any entry in the duty totals", and `duty_rows` uses
  `has_duty`. Such a player can exist only after a role is removed later (an assignment needs the
  role "plays games" on the date). The coordinator would see a row "0 + 0" for this player, and no
  row on "Duties". Traced, not run.
  - 2026-11-09 fixed with R5.
- **A6 – two players with the same first name and no last name have the same name (neutral drift,
  rule from before this change).** The e2e data shows "Yara" twice, and "Sky" and "sky", in the
  table and in the copied text. That follows the short names of "Duties" (`display_names` adds an
  initial only when there is a last name), as the spec asks. In a chat text, two lines "Yara 83%"
  and "Yara 100%" cannot be told apart. This is a candidate for `docs/ideas.md`, not for this
  change. Checked by a run (`/api/report` on the e2e data).
  - 2026-11-09 not changed in this change: the rule of the short names is the rule of "Duties"; a line in `docs/ideas.md`.
- **The Deviations of the build** (connection variants, the names without a team, the test of
  "Attendance" in `report.e2e.ts`, copy as a function, "Copy" only in the table of the team) are
  faithful adaptations. None changes what the coordinator sees, except the place of "Copied" (A2).
