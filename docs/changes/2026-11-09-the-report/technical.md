# Technical spec: The report

**Status**: approved
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
- [ ] Step 1: the corrections in the duty totals
- [ ] Step 2: the report in the core
- [ ] Step 3: the server
- [ ] Step 4: the page "Report"
- [ ] Step 5: the details
- [ ] Step 6: copy
- [ ] All checks pass

## Deviations
- None. One wording of `functional.md` (story B: the corrections count "only on the places of the
  team") was changed with the coordinator at design (2026-11-09) to "with the same rule as done":
  that rule also counts the past places of an earlier team, as the duty totals do, so the
  corrections always explain the "done" number.

## Review findings
