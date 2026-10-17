# Technical spec: The overview

**Status**: done
**Size**: story
**Change**: docs/changes/2026-10-17-the-overview
**Date**: 2026-10-17

## Objective
Show on the page "Attendance", for the current or an earlier season, how often each player was at
training and why not, so the coach sees the effort each player gives to the team and the sheet is
no longer needed for attendance.

## Requirements summary
Story C of the epic [Attendance at training](../../2026-10-14-attendance-at-training/technical.md),
from the epic's `functional.md` (approved again 2026-10-16): story 5 (season totals), with the
rules of stories 1–3 that the totals use. Words from `docs/language.md`: player, role, hidden,
training, training day, weekly training, attendance, reason.

1. **Totals.** The page "Attendance" shows, for the chosen season, one row for each player who could be at one or more trainings of the season: the name, "there", "absent, valid", "absent, not valid", and "of N trainings". Sorted by first name. Example: "Avery – 9 there, 1 absent valid, 0 absent not valid, of 10 trainings".
2. **What counts.** For a player, a training counts when it exists (opened or added), is not cancelled, has a date up to and including today, and the date is inside one of the player's roles. A player who joined later has no missed trainings before that date. An absence outside the player's roles is kept but does not count; it counts again when the roles cover it. A reason changed between valid and not valid moves the counts at once (the current setting is used). A cancelled training counts again after "Undo cancel". A training in the future with an absence noted in advance does not count yet. Nothing is stored for the totals (decision 0020).
3. **"Left the team".** Players who had a role earlier in the season and have none today or later in the season show in a separate group "Left the team", with the totals of their trainings. A player after "Change role" is in the team, once.
4. **Days not opened.** When past training days are still "Not opened", the page says so above the totals: "3 training days not opened", with a link to the first (oldest) of them.
5. **Nothing yet.** With no trainings in the season yet: "No trainings yet this season."
6. **The list stays.** The page also lists the training days and trainings of the season, "Coming" above "Past", as built in story B (open, "Cancel" on a day not opened, "Show cancelled", "Add training"); a tap opens the training.
7. **Earlier seasons.** The season runs from 1 July to 30 June (Dutch time). An earlier season can be chosen on the same page, with its totals and its list; its trainings can still be opened, marked, changed and cancelled (story B).
8. **The missed trainings.** A tap on a player shows the trainings the player missed, with the date and the reason: only the absences that count (coordinator, 2026-10-17).
9. Phone first: no sideways scroll at 320 pixels; only the coordinator (login, decision 0007).

## Current state
On `main` (a9242c5), stories A and B are built, closed and on the cluster:
- Migrations up to `0006_trainings` (`players`, `player_roles`, `reasons`, `weekly_trainings`, `trainings` with a unique date and `cancelled_at`, `absences` with the primary key (training, player)).
- Core `players`: `Player { id, first_name, last_name, hidden }`, `PlayerRole { id, player, season, role, from, until }`, `role_on(roles, player, date)`, `place(roles, player, season, today) -> Placement { group: Group::{Team, Left, LastSeason, Other}, role, roles }`, `team(…)` (the page "Team"; sorts by first name lower case, last name lower case, id).
- Core `training`: `WeeklyTraining`, `Training { id, date, start, end, cancelled }`, `training_days`, `season_list(season, weekly, trainings, absences, roles, today) -> SeasonList { coming, past }` with `SeasonRow { date, start, end, training, extra, status: DayStatus::{NotOpened, AllThere, Absent(n), Cancelled} }`; `past` is newest first and holds the dates before today.
- Core `attendance`: `Absence { training, player, reason }`, `players_at` (sorts like `team`), `now_there`, `check_mark`. Core `reasons`: `ReasonId`.
- Server storage: `players::all`, `players::roles_in(db, &[Season])`, `weekly_trainings::for_season`, `trainings::for_season`, `trainings::absences_at(db, &[TrainingId])`, `reasons::all -> Vec<StoredReason { id, name, valid, hidden }>`.
- Server API: `api/trainings.rs` with `list` (`GET /api/trainings?season=`, parses the season inline, 400 `invalid_season`), `list_of(state, season)`, `TrainingList { season, today, season_first_day, season_last_day, coming, past }`; `api/format.rs` (`day`, `clock_time`, `path_date`); `ApiDoc` in `api/mod.rs`; `clock::{current_season, today_in_netherlands}`.
- Front end: `/attendance` (`+page.ts` loads `loadTrainings(fetch)`, always the current season; `+page.svelte` with the list, "Cancel", "Show cancelled" kept in `localStorage`, "Add training"); `/training/[date]` ("Back" and the jump after "Delete training" go to `/attendance`); the start page line "Attendance · Team · Players · Settings"; `lib/trainings.ts` (`statusText`), `lib/schedule.ts` (`formatDay`).
- Tests: `crates/server/tests/trainings.rs` (17 tests, helpers `add_tuesdays`, `insert_training`, `insert_absence`, `reason_id`, `team_with_tuesdays`), the 401/403 route list in `tests/players.rs`, `tests/player_logs.rs`; `web/e2e/trainings.e2e.ts` (serial, clock Mon 12 Oct 2026, runs after `players.e2e.ts` on the database `team_e2e`).

## Steps

### Step 1: The counting rule in the core
- **Goal**: the totals, the groups and the days not opened follow from the stored data, in the core.
- **Changes**:
  - `crates/core/src/players.rs`: `impl Player { pub fn sort_key(&self) -> (String, Option<String>, PlayerId) }` (first name lower case, last name lower case, id). `team` and `attendance::players_at` use it.
  - `crates/core/src/attendance.rs` (exported from `lib.rs`):

    ```rust
    /// A training that a player missed and that counts.
    pub struct Missed { pub date: NaiveDate, pub reason: ReasonId }

    /// The totals of one player in a season.
    pub struct PlayerTotals<'a> {
        pub player: &'a Player,
        pub there: usize,
        pub absent_valid: usize,
        pub absent_not_valid: usize,
        /// "of N trainings": the trainings that count for this player.
        pub trainings: usize,
        /// Newest first.
        pub missed: Vec<Missed>,
    }

    pub struct SeasonOverview<'a> {
        /// Trainings of the season up to today, not cancelled. 0 → "No trainings yet this season."
        pub trainings: usize,
        /// `place` gives `Group::Team` on `day`.
        pub team: Vec<PlayerTotals<'a>>,
        /// `place` gives `Group::Left` on `day`.
        pub left: Vec<PlayerTotals<'a>>,
        /// Training days before today without a training, and the oldest of them.
        pub not_opened: usize,
        pub first_not_opened: Option<NaiveDate>,
    }

    pub fn season_overview<'a>(
        season: Season,
        players: &'a [Player],
        roles: &[PlayerRole],        // at least the roles of `season`
        weekly: &[WeeklyTraining],   // the weekly trainings of `season`
        trainings: &[Training],
        absences: &[Absence],
        valid: &[(ReasonId, bool)],
        today: NaiveDate,
    ) -> SeasonOverview<'a>;
    ```

    - Counted trainings: `Season::of(date) == season`, `date <= today`, not cancelled.
    - For each player: `trainings` = the counted trainings with `role_on(roles, player, date)`; the counted absences are the player's absences at those trainings; each is valid or not from `valid` (a reason missing from `valid` counts as not valid; the database never allows it); `there` = `trainings` − counted absences; `missed` = the counted absences, newest first.
    - A player gets a row only when `trainings > 0`. Hidden players are included.
    - Group: `place(roles, player, season, day)` with `day = today.min(season.last_day())`; `Team` → `team`, `Left` → `left` (other groups have no role in the season, so no row).
    - Days not opened: the rows of `season_list(season, weekly, trainings, absences, roles, today).past` with `DayStatus::NotOpened`; `first_not_opened` is the last of them (`past` is newest first).
    - `team` and `left` sorted with `Player::sort_key`.
- **Tests** (season 2026-2027, Tuesday weekly training Tue 1 Sep 2026 – Tue 25 May 2027, today Tue 13 Oct 2026 unless named; School valid, Holiday not valid):
  1. Trainings 1, 8, 15, 22 Sep; a role from 1 Sep; School on 15 Sep → 3 there, 1 valid, 0 not valid, of 4; `missed` = [15 Sep, School].
  2. A role from Tue 3 Nov, today Tue 10 Nov → only 3 and 10 Nov count; no missed trainings before.
  3. A cancelled training with an absence → not counted and not in `missed`; the same data not cancelled → counted.
  4. A training next week with an absence → not counted, not in `missed`.
  5. A training today → counted.
  6. School set to not valid → the count moves from valid to not valid.
  7. An absence after the end of the role → not counted, not in `missed`.
  8. "Only trains" 1 Sep–31 Oct and "plays games" from 1 Nov (today 10 Nov) → one row in `team`, trainings of both roles counted.
  9. A role 1 Sep–30 Sep → in `left` with its totals; 1 Sep–30 Sep and again from 1 Nov (today 10 Nov) → one row in `team`, October not counted.
  10. A role from next week → no row; a hidden player with a role → a row.
  11. Today Thu 15 Jul 2027: a role until 30 Jun 2027 → `team`; a role that ended 31 Mar 2027 → `left`.
  12. Trainings on 1 and 15 Sep only, today 13 Oct → `not_opened` 4 (8, 22, 29 Sep, 6 Oct), `first_not_opened` 8 Sep; today's 13 Oct not counted; a cancelled training day is not "not opened".
  13. No trainings, only a cancelled one, or only one next week → `trainings` 0.
  14. "anna", "Bob", "Anna de Vries" → sorted by first name without case, then last name.
  15. An absence with a reason missing from `valid` → counted as not valid.
  - The existing tests of `players_at`, `team` and `season_list` pass unchanged.

### Step 2: The route `GET /api/attendance`
- **Goal**: the overview of a season, and the seasons to choose from, over the API.
- **Changes**:
  - `api/format.rs`: `pub(crate) fn query_season(text: Option<String>, clock: &dyn Clock) -> Result<Season, AppError>` (the current season when left out; 400 `invalid_season` when it cannot be read). `trainings::list` uses it.
  - `crates/server/src/trainings.rs`: `pub async fn seasons(db) -> Result<Vec<Season>, StoreError>`: the distinct seasons of `player_roles.season_year`, `weekly_trainings.season_year`, and of `trainings.date` (start year = the year, minus 1 when the month is before July: the rule of `Season::of`).
  - `crates/server/src/api/attendance.rs` (new), on the protected router and in `ApiDoc`:

    ```rust
    pub struct AttendanceOverview {
        pub season: String,            // "2026-2027"
        pub current_season: String,
        /// Newest first: the seasons with data, the current season and the chosen season.
        pub seasons: Vec<String>,
        pub trainings: usize,
        pub not_opened: usize,
        pub first_not_opened: Option<String>,
        pub team: Vec<PlayerTotalsView>,
        pub left: Vec<PlayerTotalsView>,
    }
    pub struct PlayerTotalsView {
        pub id: i64, pub first_name: String, pub last_name: Option<String>,
        pub there: usize, pub absent_valid: usize, pub absent_not_valid: usize, pub trainings: usize,
        pub missed: Vec<MissedView>,
    }
    pub struct MissedView { pub date: String, pub reason: String, pub valid: bool }
    ```

    `GET /api/attendance?season=` reads the players, the roles of the season, the weekly trainings, the trainings, their absences and the reasons (separate queries, as `list_of`; no transaction), calls `season_overview`, and maps reason ids to name and valid. It writes nothing to the log.
  - `just prepare`, `just api-types`.
- **Tests** (`crates/server/tests/attendance.rs`, new; clock Mon 12 Oct 2026):
  1. `the_overview_of_the_current_season`: two players, trainings with absences, a reason changed to not valid with `PUT /api/reasons/{id}` → the new counts, `team` and `left`, `missed` with reason name and valid, `not_opened`, `first_not_opened`, `current_season`.
  2. `an_earlier_season_can_be_chosen`: roles and a training in 2025-2026 (through storage) → `?season=2025-2026` has its totals; `seasons` = ["2026-2027", "2025-2026"]; a season without data in the address is in `seasons`.
  3. `the_seasons_follow_the_date_rule`: a training on Tue 30 Jun 2026 → 2025-2026; on Wed 1 Jul 2026 → 2026-2027.
  4. `a_season_that_cannot_be_read_is_refused`: `?season=abc` → 400 `invalid_season` (the list's test stays).
  5. `no_trainings_yet`: roles and no trainings → `trainings` 0, `team` empty.
  6. `GET /api/attendance` in the route list of the 401/403 test (`tests/players.rs`).
  7. The new route in `tests/player_logs.rs` (no player names in the logs).

### Step 3: The choice of season
- **Goal**: the coordinator chooses an earlier season on "Attendance" and comes back to it from a training.
- **Changes**:
  - `web/src/lib/api/client.ts`: `loadTrainings(fetch, season?)`; new `loadAttendance(fetch, season?)`.
  - `web/src/routes/attendance/+page.ts`: reads `?season=` from the address and loads the list and the overview at the same time. A season that cannot be read shows "This season cannot be read." with a link to the current season.
  - `web/src/routes/attendance/+page.svelte`: the choice of season (a select box with `seasons`, newest first; a new choice goes to `/attendance?season=…`); the title shows the chosen season; "Add training" only when `season == current_season`.
  - `web/src/routes/training/[date]/+page.svelte`: "Back" and the jump after "Delete training" go to `/attendance?season=<the training's season>` (`TrainingDay.season`).
  - `web/e2e/database.ts`: a helper that runs SQL on `team_e2e` with `psql`, for data in an earlier season (the screens cannot add it there).
- **Tests** (browser, a new `test.describe` block at the end of `web/e2e/trainings.e2e.ts`, so it runs after its data and in a known order):
  - A training and a role in 2025-2026 made with the SQL helper. Choose 2025-2026 → the address has `?season=2025-2026`; its row shows; "Add training" is not there. Open its training; "Back" goes to `?season=2025-2026`.
  - `/attendance?season=abc` shows "This season cannot be read."

### Step 4: The totals on the page
- **Goal**: the totals, "Left the team", the days not opened and the missed trainings on "Attendance".
- **Changes** (`web/src/routes/attendance/+page.svelte`, top down: title and season, the line about days not opened, the totals, the list):
  - "3 training days not opened" ("1 training day not opened") with a link to `/training/<first_not_opened>` (that page shows "Not opened" with "Open training"); not shown when 0.
  - "No trainings yet this season." when `trainings` is 0; else the totals of `team`, then the heading "Left the team" with `left` (not shown when empty).
  - Each player row is a button with `aria-expanded`: line 1 the name and "of 10 trainings"; line 2 "9 there · 1 absent, valid · 0 absent, not valid". A tap shows the missed trainings under the row ("Tue 6 Oct · Ill", newest first), or "No missed trainings."; a second tap closes it.
  - After "Cancel" on a row, the list and the overview are loaded again.
- **Tests** (browser, same block):
  1. Open four past Tuesdays from the list and mark one absence → "of 4 trainings" with the right counts; "N training days not opened" goes down after each, and its link opens the oldest day.
  2. A tap on a player shows "Tue 6 Oct · Ill"; a second tap closes it.
  3. "Cancel training" on an opened training with an absence → the counts go down; "Undo cancel" → back.
  4. "Show cancelled" off hides cancelled rows; the totals stay the same.
  5. "No trainings yet this season." for a season with roles and no trainings (SQL helper).
  6. No sideways scroll at 320 pixels with an open missed list; `expectAccessible`.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| The totals are worked out at each request in the core (`season_overview`), with the counting rule of decision 0020 | Totals always follow the data (role changes, cancels, reasons changed between valid and not valid); about 15 players and 40 trainings a season | no: decision 0020 already says it |
| A separate route `GET /api/attendance`; the list route stays | The card and the training page do not pay for totals; each route tested alone; the epic's plan | no |
| The groups reuse `place` on today, or on 30 June for an earlier season | One rule for the page "Team" and the totals; an earlier season does not put every player under "Left the team" | no |
| The days not opened reuse `season_list` | The line and the list cannot disagree | no |
| The choice of season offers the seasons with a role, a weekly training or a training, plus the current and the chosen season | No empty seasons in the choice | no |
| The season is in the address (`?season=`) | A reload and "Back" from a training keep the season | no |
| The start page keeps "Attendance · Team · Players · Settings"; "Attendance" gets no links to Team, Players and Settings | The start page will become one page of past and coming events (idea of 2026-10-15, extended 2026-10-17); links would move twice (coordinator, 2026-10-17) | no; see Deviations |
| The browser tests of story C go at the end of `trainings.e2e.ts` | Test files run in alphabetical order on one database; a new `attendance.e2e.ts` would run first and its Tuesday weekly training would break the files after it | no |

## Not in scope
- Everything under "Not in scope" in the epic's `functional.md` (trends, charts, recent weeks, messages, attendance at games).
- A change of the start page (coordinator, 2026-10-17).
- Absences noted in advance in the missed list (coordinator, 2026-10-17).
- A read transaction around the reads of `GET /api/attendance` (one coordinator; the next load is right again).
- A page for each player; an export of the totals.
- A change of story B's routes or answer types (only the season parameter of `loadTrainings` in the front-end client).

## Test plan
- **Existing tests that must still pass**: all of `just check`; in particular the core tests of `players_at`, `team` and `season_list`, `crates/server/tests/trainings.rs` (with `a_date_or_season_that_cannot_be_read_is_refused`), the 401/403 route list in `tests/players.rs`, `tests/player_logs.rs`, and `web/e2e/trainings.e2e.ts`.
- **New tests**: per step, see above (core 15, server 7, browser 8).
- **Checks**: `just check` at each step; `just prepare` after the storage query; `just api-types` after the route.
- **By hand**: after the release, the coordinator compares the totals of the trainings since 1 September on the phone with the sheet.

## Compatibility and performance
- No migration and no stored data. The start page, the card, the training page (apart from "Back") and the routes of story B do not change.
- A training of an earlier season is reachable from the screen after step 3 (story B accept review A2; the backlog asks for this before 1 July 2027).
- Load: one more request per visit of "Attendance"; about 15 players, 40 trainings and a few hundred absences a season.
- Risk: the totals may differ from the coordinator's sheet (for example a day the sheet marks and the tool has as not opened). The line "N training days not opened" helps to explain a difference; the check by hand finds the rest.

## Progress
- [x] Step 1: the counting rule in the core (2026-10-17: `Player::sort_key`, `season_overview`, 15 core tests)
- [x] Step 2: the route `GET /api/attendance` (2026-10-17: `query_season`, `trainings::seasons`, `api/attendance.rs`; 5 tests in `tests/attendance.rs`, the route in the 401 test and the log test)
- [x] Step 3: the choice of season (2026-10-17: `loadTrainings` and `loadAttendance` with a season, `?season=` on `/attendance` with the choice and "This season cannot be read.", "Add training" only in the current season, "Back" and "Delete training" return to the training's season; 2 browser tests, and the URL after "Delete training" in the existing test)
- [x] Step 4: the totals on the page (2026-10-17: the line of days not opened with its link, "No trainings yet this season.", the totals and "Left the team" with the missed trainings under a player; 4 browser tests)
- [x] All checks pass (`just check`, 2026-10-17: 64 browser tests)

## Deviations
- 2026-10-17, during /design, the coordinator's choice: the epic's Interface says "the start page links to "Attendance"; "Attendance" links to "Team", "Players" and the settings". Story C keeps the start page line "Attendance · Team · Players · Settings" (built in story B) and adds no links to "Attendance". Reason: the start page will become one page with the last games, the last training and the coming events (`docs/ideas.md`, 2026-10-15, extended 2026-10-17); moving the links now would move them twice.

- 2026-10-17, step 1: `season_overview` takes 8 inputs, one more than clippy allows by default; it has `#[allow(clippy::too_many_arguments)]` instead of a struct that only groups them (the engineer's choice). Core test 12 uses trainings on 1, 15 and a cancelled 29 Sep (3 not opened), and then all six past Tuesdays (0), so it also covers "a cancelled training day is not not opened".
- 2026-10-17, step 2: server test 2 makes its earlier season by moving the clock to Thu 15 Jul 2027 (the September data is then in 2026-2027, an earlier season), not by roles written in 2025-2026 through storage; it tests the same answer with less hand-written data. Server test 3 uses Mon 30 Jun 2025 and Tue 1 Jul 2025, so a wrong rule would lose the season 2024-2025.
- 2026-10-17, step 3: no SQL helper in `web/e2e/database.ts`. CI runs the browser tests outside the Nix shell, where `psql` may be missing. The browser tests do not need data in an earlier season: a season in the address is in the choice even without data, and the training page of any date (Tue 26 May 2026) has its season. The server tests cover an earlier season with data. Step 4's "No trainings yet this season." uses 2025-2026 in the same way.
- 2026-10-17, step 3: the links with a season use `resolve('/attendance?season=…')` (SvelteKit accepts a query in `resolve`), so the lint rule `svelte/no-navigation-without-resolve` holds; the training page types its helper as `ResolvedPathname`.
- 2026-10-17, step 4: the page has two headings of its own, "Totals" and "Trainings" (above "Add training" and the list); "Coming" and "Past" became sub-headings of "Trainings". The spec names no headings; they keep the long page readable on a phone and for a screen reader.
- 2026-10-17, step 4: the browser tests run on the data of the tests before them, so the numbers differ from the plan: 11 days not opened (the Tuesdays of September and the Thursdays from Thu 3 Sep, added by the last test of story B), and "of 5 trainings" after four Tuesdays, because Tue 6 Oct already counted. The check at 320 pixels with an open missed list is in the test of the missed list. The text "1 training day not opened" has no browser test (`notOpenedText`).

The design reads the spec more strictly within its words:
- "Each player who could be at one or more trainings of the season": a player with a role on the date of at least one training that counts (up to today, not cancelled). A newcomer from next week has no row yet. Hidden players have rows.
- "Left the team" ("none today or later in the season") for an earlier season: measured on its last day, 30 June.
- "Past training days still Not opened": training days before today, the same as "Past" in the list; today's training day is in "Coming".
- "The trainings the player missed": only the absences that count (coordinator, 2026-10-17).
- "An earlier season can be chosen": the seasons with a role, a weekly training or a training, plus the current season.
- "Back" on a training page returns to the season of that training (needed for an earlier season; the spec does not say where "Back" goes).
- "Add training" stays only for the current season (story B, accept review A5).

### Spec review of the deviations (2026-10-17)
Independent reviewer, fresh context. Input: the deviations above, judged against the epic's
`functional.md` (stories 1–5, Interface), `docs/vision.md`, `docs/language.md`, `docs/ideas.md`,
`docs/product/trainings-and-attendance.md` and `docs/product/players-and-team.md`. Only the
listed differences are judged; the rest of this spec is not. Question each time: the coach opens
"Attendance" to see each player's effort and to finish the record; does the difference serve that?

| # | Deviation | Verdict | Why (and how checked) |
|---|-----------|---------|-----------------------|
| D1 | Start page keeps "Attendance · Team · Players · Settings"; no links on "Attendance" | serves – note | The coordinator's own choice, tied to the start page idea (checked: `docs/ideas.md`, 2026-10-15 line extended 2026-10-17; the diff of that file is not committed yet). Cost to the user: from "Attendance" to "Team" is "Back" (a search of `web/src/routes/attendance/+page.svelte` for `href`: `Back` goes to `/`) and one more tap, not a dead end. Note: the epic's Interface line "Attendance links to Team, Players and the settings" is now false; at /close the epic spec or the product doc must not keep a promise the screens do not keep (`docs/product/trainings-and-attendance.md` already describes the start page line, so the living doc is right). |
| D2 | A row only for a player with a role on the date of at least one counted training; hidden players have rows | serves | "Could be at one or more trainings" read as "was in the team at a training that counts" avoids "0 there, of 0 trainings" rows that tell the coach nothing. Hidden players with rows follows story 1 ("the history, the team and the totals stay"). Side effect, acceptable: a newcomer added today shows on "Attendance" only once today's training is opened; "Team" shows the player at once. Not checked in code (design only). |
| D3 | "Left the team" of an earlier season measured on 30 June | serves | The literal "none today or later" puts every player of an earlier season under "Left the team", which would tell the coach nothing. The last day of the season keeps the meaning of the group: players who left during that season. Same rule as "Team" (`place`) in the current season (checked: `docs/product/players-and-team.md`, line 28). |
| D4 | "Not opened" counts training days before today; today's day is under "Coming" | serves | Matches the list the coach sees (same rows, "Past" before today), so the line and the list cannot disagree. Flagging today's training before it happens would be noise; the start page card already shows today's training until midnight (product doc, "The card on the start page"). A day forgotten tonight shows tomorrow. |
| D5 | The missed list shows only absences that count | serves | "The trainings the player missed" reads naturally as past, not cancelled, inside a role; then the list adds up to "absent, valid" + "absent, not valid" and the coach can check each count against it. Absences noted in advance stay visible on each training. Coordinator's choice, written in `change.md`. |
| D6 | Seasons offered: with a role, a weekly training or a training, plus the current (and the chosen) season | serves | A season with only games and no attendance data has nothing to show here; empty choices would be noise. The current season is always there, so a new season in July is not lost. |
| D7 | "Back" on a training returns to that training's season | serves | Without it, every training of an earlier season (story 2: "the last training of May found in July") sends the coach back to the current season and a new choice; with it, entering a run of old trainings stays a loop of two taps. Fills a gap the spec left open. |
| D8 | "Add training" only for the current season | serves – note | Earlier decision (story B accept review A5), already in the product doc ("Errors and limits"); not new in story C. Note: story 2 says trainings of an earlier season can still be "opened, entered, changed and cancelled". A forgotten **extra** training of last season (one that is not a training day) can then not be entered at all: "Change" cannot move a training across seasons. Rare (the context calls extra trainings rare, and July is after the last training of May), so not a blocker; the epic spec should say this limit in words so it is a choice, not a gap. |

**Verdict: the deviations serve the user; none conflicts with the vision.** Two notes for /close:
the epic's Interface line about the links from "Attendance" (D1), and the limit on "Add
training" for an earlier season written into the epic spec (D8).

## Review findings

### Build review (2026-10-17)
Independent reviewer, fresh context. Input: `/review build docs/changes/2026-10-17-the-overview`
(no other text given). Read: the diff `main..feat/the-overview` (4 commits, nothing uncommitted
apart from untracked local files), this change folder, `docs/architecture.md`, decisions 0001,
0014, 0018, 0020, `CLAUDE.md`, `AGENTS.md`, `docs/language.md`. Runs: `cargo test -p team_core
attendance` (18 passed), `cargo test -p team --test attendance` (5 passed), `--test player_logs`
(1 passed), `just api-types-check` (no diff), `just check-web` (passes). The browser tests and
`check-sqlx` were not run.

Overall: the counting rule sits in the core and reuses `role_on`, `place` and `season_list`, as the
design says; the server only moves data; the route is behind the login and in the 401/403 and log
tests; no `unwrap`/`expect` outside tests; no names in logs. No bug against the spec's intent was
found in the counting. The findings are small.

| # | Finding | Weight | How checked |
|---|---------|--------|-------------|
| R1 | **The season rule is written a second time, in SQL.** `trainings::seasons` (`crates/server/src/trainings.rs`) works out the season of a training date in the query (`EXTRACT(YEAR …) - CASE WHEN EXTRACT(MONTH …) < 7 …`). The rule "a season starts on 1 July" is a product rule in `team_core::Season::of` (decision 0001: rules in the core, the server only moves data; decision 0014: the conversion of seasons is in one place in the core). Server test 3 pins the two copies together today, but a change of `FIRST_MONTH` would not reach the SQL. Simpler and inside the rules: select the distinct training dates (or `season_year` of roles and weekly trainings plus the dates) and map the dates with `Season::of` in Rust. | low (no wrong answer now) | Read of the diff; search `grep -rn "EXTRACT" crates/server/src` (only this query); read of `crates/core/src/season.rs` and decisions 0001, 0014 |
| R2 | **Page state outlives a change of season.** SvelteKit keeps the page component when only `?season=` changes, which is why `list` and `overview` became `$derived`; but `adding`, `error` and `expanded` stay `$state` and are not reset. So: open "Add training", then choose an earlier season → the form stays open in the earlier season (the `{#if adding}` branch comes before `{:else if current}`); a "Save" there is refused by the server (`outside_season`, `api/trainings.rs` checks against the current season), so no wrong data, but the page offers what D8 says it does not. An error line of one season also stays on the next. Reset `adding` and `error` when `data` changes (or hide the form when not `current`). | low | Trace of `web/src/routes/attendance/+page.svelte` (`chooseSeason` → `goto` on the same route; `{#if adding} … {:else if current}`); server side read in `crates/server/src/api/trainings.rs` (`add` uses `current_season`). Not run in a browser |
| R3 | **"Totals" can be an empty heading.** When the season has trainings (`trainings > 0`) but no player has a role on any of their dates (for example trainings opened before players were added, or only on dates outside every role), the page shows the heading "Totals" with an empty list and no line, while the spec's "No trainings yet this season." does not apply. Small and rare; a line such as "No players in the team at these trainings." or hiding the heading would avoid a page that looks broken. Not a spec rule, so for the coordinator to decide. | low | Trace: `season_overview` skips rows with `trainings == 0`; the page renders `totals(overview.team, …)` when `overview.trainings !== 0` |
| R4 | **A browser test that cannot fail.** In "a cancelled training does not count …", the check "Show cancelled off hides the row, not the totals" (`of 4 trainings` after the switch) cannot fail: the switch only filters rows on the page (`shown()`), the totals are not loaded again and do not read it. Harmless; it documents intent rather than testing it. | info | Read of `web/e2e/trainings.e2e.ts` and `shown()` in `+page.svelte` |
| R5 | **Claims in this spec.** True as far as checked: 15 core tests named as in the plan (18 pass with the older attendance tests); 5 server tests plus the route in `tests/players.rs` and `tests/player_logs.rs`; 2 + 4 browser tests (6, not the planned 8: the 320-pixel check is folded into the missed-list test and the SQL helper was dropped, both written under Deviations); `query_season` used by both routes; API types up to date. The planned "1 training day not opened" text has no test (said under Deviations). `MissedView.valid` is sent but not used by the page (the spec asks only date and reason); harmless. Not checked: "64 browser tests" and `just check` as a whole. | info | Runs listed above; count of tests in the diff |
| R6 | **Docs the diff made false (for /close).** `docs/architecture.md`: the known-limits lines "The start page links to … (story C … decides)" and "A training of an earlier season is opened only by its address until story C …" are now settled; the "Trainings and attendance" section lacks `GET /api/attendance`, `season_overview`, `trainings::seasons`, `query_season` and `Player::sort_key`. `docs/product/trainings-and-attendance.md`: "Earlier seasons … For now such a training is opened by its address" is false now, and the server address list lacks `GET /api/attendance`; "Back" on a training now returns to that training's season. Decision 0020's consequence "until story C adds the choice of season" is history (records are not edited). Code comments touched by the diff are correct. | info (/close) | Search of `docs/architecture.md` and `docs/product/trainings-and-attendance.md` for `story C`, `address`, `/api/` |

`#[allow(clippy::too_many_arguments)]` on `season_overview` (Deviations, step 1) is the only such
allow in the crates (`grep -rn too_many_arguments crates`); acceptable for one pure function, noted
only so a second one is a choice.

**Verdict: passes the build review.** No finding blocks Close; R1 and R2 are small fixes worth
doing before Close, R3 is a question for the coordinator, R6 is work for /close.

Answers of the builder:
- R1 – 2026-10-17: fixed. `trainings::seasons` reads the `season_year` of roles and weekly trainings and the distinct training dates, and maps the dates with `Season::of` in Rust; no season rule in SQL. Server test `the_seasons_follow_the_date_rule` passes unchanged; `just prepare` again.
- R2 – 2026-10-17: fixed. `afterNavigate` on `/attendance` closes "Add training" and clears the error line and the open missed lists, so a change of season (by the choice or by the browser's back) starts clean. Browser test "an earlier season can be chosen, and "Back" returns to it" now opens the form with an error ("There is already a training on this date.") in 2026-2027 and goes back to 2025-2026: no form, no error. Without the reset the test fails (checked by removing it and running all browser tests).
- R3 – 2026-10-17: fixed (the coordinator's choice: a line). When trainings count but no player had a role at any of them, "Totals" says "No player had a role at these trainings." and, in the current season, "Add players in Team." with a link. Browser test "trainings at which no player had a role" (the answer of `/api/attendance` replaced: such a season needs trainings before any role).
- R4 – 2026-10-17: not changed. The check guards against a later change that would make "Show cancelled" filter the totals as well; it documents the rule of story 2 ("hides the row") on the screen.
- R5 – 2026-10-17: nothing to fix. `MissedView.valid` stays in the answer; the page may mark lines later without an API change.
- R6 – 2026-10-17: for /close.

### Accept review (2026-10-17)
Independent reviewer, fresh context. Input: `/review accept docs/changes/2026-10-17-the-overview`
(no other text given). This folder has no `functional.md` or `idea.md`; as `change.md` says, the
functional spec is the epic's (`../2026-10-14-attendance-at-training/functional.md`, story 5, the
parts of stories 1–3 the totals use, and the Interface line of "Attendance"), with the epic's
`idea.md`. Read: those, `change.md`, the objective and requirements of this spec, and the diff
`main...feat/the-overview`; the view below was formed before reading **Deviations** and the build
review. Runs: `cargo test -p team_core attendance` (18 passed), `cargo test -p team --test
attendance` (5 passed), `npx playwright test` (64 passed, among them the 6 of "the overview").
Not run: the server against the dev database (`serve` starts the nightly reading, which writes),
so the page was seen only through the browser tests. This review covers story C; the epic's own
`review accept` of the whole result is a separate run.

The intent of the idea ("afterwards, the coach looks at the season totals to see who comes
regularly and who misses trainings without a valid reason") and of story 5 is met: the totals are
worked out from the stored data at each request, so they follow role changes, cancels, and
reasons changed between valid and not valid; the line of days not opened tells the coach when the
totals are not complete; and an earlier season can be chosen and its trainings reached.

| # | Point (criterion or open point) | Built | Sort | How checked |
|---|---------------------------------|-------|------|-------------|
| A1 | Story 5: one row for each player who could be at one or more trainings, with "there", "absent, valid", "absent, not valid", "of N trainings", sorted by first name | A row for a player with a role on the date of at least one training that counts; hidden players too; two lines per row ("Name … of 10 trainings" / "9 there · 1 absent, valid · 0 absent, not valid") instead of the one line of the Behaviour example | faithful adaptation | Trace of `season_overview` (`crates/core/src/attendance.rs`, `trainings == 0` → no row); core tests `a_row_needs_a_training_that_counts_and_hidden_players_have_rows`, `the_rows_are_sorted_by_first_name_then_last_name`; browser test "the totals change while past trainings are entered" |
| A2 | Only trainings up to and including today; only at dates inside a role; a cancelled training does not count; a reason changed moves the counts; absences outside a role kept, not counted | All five in the core rule; nothing stored for the totals | faithful | Read of `season_overview`; core tests 2–7 and 15 (run); server test `the_overview_of_the_current_season` (reason changed with `PUT /api/reasons/{id}`); browser test of cancel and "Undo cancel" |
| A3 | "Left the team" with the totals of their trainings; a player after "Change role" once in the team | `place` on today (on 30 June for an earlier season): the same rule as the page "Team" | faithful adaptation (the 30 June reading for an earlier season is what keeps the group meaningful) | Core tests `a_player_after_change_role_is_counted_once_in_the_team`, `a_player_who_left_is_in_left_the_team_and_one_who_came_back_is_in_the_team`, `an_earlier_season_groups_on_its_last_day`. "Left the team" on the page is not in a browser test (the group is rendered by the same snippet as the team) |
| A4 | "3 training days not opened" above the totals, with a link to the first of them | Training days before today with no training, from the same `season_list` as the list; the link goes to the oldest day's training page, which shows "Not opened" and asks for "Open training" before the day counts | faithful adaptation: one more tap than a row of the list, but it keeps Bruce's warning ("the coordinator must see clearly that a day is Not opened before opening it") and the link cannot open a holiday by mistake | Browser test (11 → 10 → … → 7, link to Tue 1 Sep, "Open training"); `web/src/routes/training/[date]/+page.svelte` lines ~252–255 |
| A5 | "No trainings yet this season." | When no training up to today is left after cancels | faithful | Core test `no_training_counts_yet`, server test `no_trainings_yet`, browser test on 2025-2026 |
| A6 | A tap on a player shows the missed trainings, with date and reason | Inline under the row, newest first, "No missed trainings." when none; only absences that count (coordinator's choice) so the list adds up to the two absent counts; valid / not valid is not marked per line (the counts show it) | faithful | Browser test "a tap on a player shows the trainings it missed" (also 320 px, accessibility) |
| A7 | An earlier season can be chosen on the same page | A select box (shown only with more than one season), the season in the address; "Back" and "Delete training" on a training return to the training's season; "Add training" only in the current season | faithful adaptation (the "Back" rule fills a gap the spec left open); the limit on "Add training" is neutral drift already noted as D8 | Browser test "an earlier season can be chosen, and "Back" returns to it"; server test `an_earlier_season_can_be_chosen`. An earlier season **with data** is only covered by the server test, not on the screen (Deviations, step 3) |
| A8 | The list of training days and trainings stays ("Coming" above "Past", cancel, "Show cancelled", "Add training") | Unchanged, under a new heading "Trainings", below the totals; after "Cancel" on a row the totals load again | faithful adaptation. On a phone the totals (about 14 two-line rows) now push the list one screen down; entering past trainings goes through the "not opened" link or a scroll. The weekly path (the card on the start page) is not touched, so this does not cost the hall use | Read of `+page.svelte`; existing browser tests of story B pass |
| A9 | Interface: "Attendance" links to "Team", "Players" and the settings | Not built; the start page keeps its line | neutral drift, the coordinator's own choice (D1); the epic's Interface line must not stay as a promise after /close | `change.md` log; search of `+page.svelte` for `href` (only "Back" to `/` and the training links) |
| A10 | Phone first, no sideways scroll at 320 px; only the coordinator | Checked at 320 px with an open missed list; the route is behind the login and in the 401/403 test | faithful | Browser tests (run); `tests/players.rs` route list |
| A11 | Page state after a change of season (build review R2) | An open "Add training" form and an error line stay when another season is chosen; the server refuses a save outside the current season | concerning only in a small way: the screen offers in an earlier season what the design says it does not; no wrong data. Worth the small fix of R2 before Close | Trace (as R2); not run |
| A12 | Trainings exist but no player had a role at any of them (build review R3) | The heading "Totals" with nothing under it | neutral drift; rare (players come before past trainings in the setup the spec describes) and for the coordinator to decide | Trace of `season_overview` and the `{#if overview.trainings === 0}` branch |

Differences from my view found in **Deviations**: none that I had not seen; D2–D7 match what I
found built. The deviations that change what the coordinator sees (D1, D8) are the coordinator's
choices and are already notes for /close.

**Verdict: accepted with notes.** What was built is what was meant: the coach can see each
player's effort over the trainings that count, can trust that the counts follow later changes,
and is told when days are still not opened. Notes: A11 (fix with R2 before Close), A12 (a
question for the coordinator), A9 and D8 (the epic spec's Interface and the "Add training" limit
put into words at /close), and the check by hand against the sheet after the release, which the
test plan already names.

Answers of the builder:
- A11 – 2026-10-17: fixed with R2.
- A12 – 2026-10-17: fixed with R3 (the coordinator's choice: a line with a link to Team).
- A3, A7 – 2026-10-17: not changed. "Left the team" and an earlier season with data are covered in the core and server tests; the browser tests cannot make data in an earlier season (Deviations, step 3), and "Left the team" uses the same snippet as the team.
- A9, D8 – 2026-10-17: for /close (the epic's Interface line about links from "Attendance"; the limit of "Add training" in words).
