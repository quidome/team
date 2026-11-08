# Technical spec: Totals on "Attendance"

**Status**: done
**Size**: story
**Change**: [2026-11-08-totals-on-attendance](./)
**Date**: 2026-11-08

## Objective
Each row of "Totals" on "Attendance" also counts the games of the team that are checked and were
played, so the page shows the effort of each player at trainings and games.

## Requirements summary
From story D of the epic's `functional.md` (approved, three spec reviews; S4, S13, S14, S36, S39)
and the coordinator's answers of 2026-11-08:

- **The row**: "of 9 games: 8 there · 1 absent, valid · 0 absent, not valid" below the trainings.
  A row with no game that counts leaves this part out; a row with no training that counts leaves
  the training part out. A player with games and no trainings has a row.
- **Which games count**: every game of the team that is checked on its date now and not "not
  played" (decisions 0031, 0032), also a game from before the release day that the coordinator
  checked by hand, and also a checked game that left the game list later (it was played). A game
  kept only for its answers after another team was chosen does not count: it has no list.
- **Which players count at a game**: a player with the role "plays games" on that date (the list
  of the game) and an answer in effect (`answers_in_effect`): "coming" is there, "not coming" is
  absent. "No answer" does not count (a player with a later role, story C). Extra players never
  count.
- **Valid or not**: the setting of the reason now; an absence without a reason (also one made by
  "Checked") is "absent, not valid", with no tie to the reason named "No reason".
- **A tap on a player** lists the trainings and games the player missed, newest first, on one date
  the training first: "Tue 6 Oct · Ill" (a training, as now), "Game Sat 10 Oct · Ill", "Game Sat
  10 Oct · without a reason".
- **"Left the team"** counts games too (the groups follow `place`, as now).
- **Texts**: "No trainings yet this season." → "No trainings or games yet this season." (no
  training and no game counts); "No player had a role at these trainings." → "No player had a
  role at these events."; "No missed trainings." → "No missed trainings or games."; "Log in to see
  the trainings." → "Log in to see the attendance.".
- **The release**: in the season of the release day (`game_check_start`) the page says "Games
  count once they are checked. The start page asks for games from Sun 8 Nov." Other seasons have
  no such line (S39 g).

## Current state
- `team_core::season_overview` (`contribution/attendance.rs`, 8 inputs with
  `#[allow(clippy::too_many_arguments)]`) counts the trainings of a season: `PlayerTotals` (there,
  absent valid, absent not valid, trainings, `missed: Vec<Missed { date, reason: ReasonId }>`),
  `SeasonOverview` (trainings, team, left, not_opened, first_not_opened). A player gets a row only
  when a training counts; the group comes from `place`.
- `GET /api/attendance` (`api/attendance.rs`) reads the players, `trainings_of` and the reasons,
  and maps the overview to `AttendanceOverview` with `PlayerTotalsView` and `MissedView { date,
  reason, valid }`.
- The games of a season come from `games_of` (`api/season.rs`): `schedule` (past and coming
  `ScheduledGame`s; a game of the team keeps the reason `TeamGame` also when it left the game
  list; `Kept` is only for a game the team no longer plays) and `duties.attendance`
  (`SeasonAttendance { games: checks by source id, answers by source id }`). A season without a
  chosen team gives an empty schedule.
- Core rules of a game: `checked(attendance, date)`, `answers_in_effect(answers, attendance,
  date)`, `plays_games_on(roles, player, date)` (the rule of the list), `ScheduledGame.not_played`.
- `game_attendance::check_start(db)` gives the release day.
- The page `web/src/routes/attendance/+page.svelte` with `ofTrainings`, `countsText` in
  `web/src/lib/trainings.ts`; the missed list is keyed by date.

## Steps

### Step 1: The core rules
- **Goal**: the totals count the games; nothing changes for the user yet.
- **Changes**:
  - `contribution/game_attendance.rs`:
    ```rust
    /// A game that counts on "Attendance", with the answers that count now.
    pub struct CountedGame { pub date: NaiveDate, pub answers: Vec<StoredAnswer> }

    /// The games of the team (also gone), checked on their date now, not "not played".
    pub fn counted_games(
        games: &[ScheduledGame],
        checks: &BTreeMap<i64, GameAttendance>,
        answers: &BTreeMap<i64, Vec<StoredAnswer>>,
    ) -> Vec<CountedGame>
    ```
  - `contribution/attendance.rs`:
    ```rust
    pub enum MissedKind { Training, Game }
    pub struct Missed { pub date: NaiveDate, pub kind: MissedKind, pub reason: Option<ReasonId> }
    pub struct GameTotals { pub games: usize, pub there: usize, pub absent_valid: usize, pub absent_not_valid: usize }
    // PlayerTotals: new field `games: GameTotals`; SeasonOverview: new field `games: usize`.
    pub fn season_overview(…, games: &[CountedGame], today: NaiveDate) -> SeasonOverview
    ```
    A player counts at a game when `plays_games_on(roles, player, date)` and the player has an
    answer among `answers`; `Coming` → there; `NotComing` → valid when its reason is valid now,
    else not valid (no reason: not valid). A row when `trainings > 0 || games.games > 0`. The
    missed list newest first; on one date the training before the game.
  - Export the new names in `lib.rs`. The server passes `&[]` for now; `MissedView` gets `kind`
    (`training` / `game`) and `reason: Option<String>` (`valid` false without a reason).
- **Tests**:
  - Core `counted_games`: a game of another club team and a `Kept` game do not count; a check on
    another date does not count; a not played game does not count; a checked game that left the
    game list counts; the answers are those in effect (a settled answer while the check holds).
  - Core `season_overview`: "coming" there; "not coming" with a valid reason, with a reason that
    is not valid, and without a reason; "no answer" does not count; an answer of a player without
    "plays games" on the date (no role, or "only trains") does not count; a player with games and
    no trainings has a row (`trainings == 0`); a player who left counts the games under "Left the
    team"; the missed list order (newest first, training before game on one date) and `reason:
    None` for a game without a reason; `SeasonOverview.games`.
  - The existing core tests of `season_overview` and `crates/server/tests/attendance.rs` get the
    new input and fields (`kind: "training"`); their numbers do not change.

### Step 2: The route
- **Goal**: `GET /api/attendance` gives the games.
- **Changes**: `api/attendance.rs` calls `games_of(&state, season, current, today)` and
  `counted_games` over `schedule.past` and `schedule.coming` (a game of today can be checked) with
  `duties.attendance`, then `season_overview`. New fields: `PlayerTotalsView.games` (`{ games,
  there, absent_valid, absent_not_valid }`), `AttendanceOverview.games` and `games_from` (the day
  of `game_attendance::check_start`, only when it is in the chosen season, else `null`). Read
  only, no lock, no new log line. `just api-types`.
- **Tests** (new file `crates/server/tests/attendance_games.rs`, with the game list harness
  `Reader::chosen` of `tests/common/game_list.rs`, the game of Saturday 10 October 2026, id
  1073147):
  - Before "Checked" no row has games, `games` is 0.
  - After "Checked": Anna ("coming") there, Bram ("not coming", a valid reason "Exam") absent valid, Cas
    ("no answer") absent not valid with the missed `{ "kind": "game", "reason": null, "valid":
    false }`; `games` is 1; after "Undo check" no row has games (`team` is empty).
  - "Not played" takes the game out; "Undo" brings it back.
  - The game moved to 11 October: it does not count.
  - A player with only games and no trainings has a row.
  - `games_from` is the release day in its season (the test sets `game_check_start`), `null` in
    the season before.

### Step 3: The page
- **Goal**: "Attendance" shows the games.
- **Changes**: `web/src/routes/attendance/+page.svelte` and `web/src/lib/trainings.ts`:
  - the row: the name, with "of 10 trainings" and the counts of the trainings only when
    `trainings > 0`; the line "of 3 games: 1 there · 1 absent, valid · 1 absent, not valid"
    (`ofGames`, `countsText`) only when `games.games > 0`;
  - the missed list: a training "Tue 6 Oct · Ill"; a game "Game Sat 10 Oct · Ill" or "Game Sat 10
    Oct · without a reason"; the key `kind-date`;
  - the line "Games count once they are checked. The start page asks for games from Sun 8 Nov."
    when `games_from` is set;
  - the texts "No trainings or games yet this season." (`trainings === 0 && games === 0`), "No
    player had a role at these events.", "No missed trainings or games.", "Log in to see the
    attendance.".
- **Tests**:
  - Browser, `web/e2e/checked.e2e.ts` (project `checked`): check the game of 10 October through
    the API; "Attendance" shows a row with "of 1 game: …"; a tap shows "Game Sat 10 Oct · without
    a reason"; the line "Games count once they are checked. The start page asks for games from Mon
    12 Oct."; fits 320 pixels and passes the accessibility check; undo the check at the end.
  - The tests in `web/e2e/trainings.e2e.ts` that read the changed texts follow them.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| Every checked game that was played counts, also one from before the release day | "Checked" means the admin is complete (coordinator, 2026-11-08) | No: follows decision 0032 |
| A checked game that left the game list counts; a kept game of another team does not | A game whose date has passed was played (as for duties); a game the team no longer plays has no list (coordinator) | No |
| `counted_games` beside the rules of the page of a game, used by `season_overview` | One rule for what counts (decision 0022), in the core (decision 0001) | No |
| An absence without a reason counts as not valid on its own | S36: the reason "No reason" can be renamed, made valid or hidden | No: in the spec |
| `season_overview` gets a 9th input under its existing exception | A small change at an early maturity; noted as debt | No |

## Not in scope
- Extra players in the totals (never counted).
- The report of what each player did.
- Duty totals on "Attendance" (they are on "Duties").
- A quick way to enter games before the release ("All there").
- Another layout of the row.

## Test plan
- **Existing tests that must still pass**: the core tests of `attendance`, `game_attendance`,
  `start_page`; `crates/server/tests/attendance.rs` (with `kind`), `checked.rs`, `games.rs`,
  `trainings.rs`; `web/e2e/trainings.e2e.ts` (with the new texts), `game-attendance.e2e.ts`,
  `checked.e2e.ts`.
- **New tests**: see each step.

## Compatibility and performance
- The released totals of the trainings do not change. The API answer of `/api/attendance` changes
  form: `missed[].reason` becomes nullable and gets `kind`; the only reader is the page.
- One more `games_of` for each visit of "Attendance" (about 10 queries, as "Season"). A few
  requests a week.

## Progress
- [x] Step 1: the core rules
- [x] Step 2: the route
- [x] Step 3: the page
- [x] All checks pass

## Deviations
None from `functional.md`. The coordinator's answers (2026-11-08) settle what the spec left open:
older checked games count; gone checked games count; the words of a missed game.

## Review findings

Build review, 2026-11-08 (fresh agent, `/review build`). Input: the uncommitted diff against
`main` (10 files changed, `web/openapi.json` and `schema.d.ts` staged, `tests/attendance_games.rs`
new), this folder, `docs/architecture.md`, decisions 0001, 0020, 0022, 0027, 0028, 0029, 0031,
0032, `CLAUDE.md`. No extra text in the arguments.

Runs: `cargo test -p team_core` (353 passed), `cargo test -p team --test attendance_games --test
attendance --test checked --test games` (5, 3, 16, 31 passed), `cargo clippy --workspace
--all-targets -- -D warnings` (clean), `cargo fmt --check` (clean), `svelte-check` (0 errors),
`npm run lint` (clean), `just api-types-check` (no diff), `npm run build` and `npx playwright test
--project checked` (the chain up to `checked`, 159 passed, with the new test). Not run:
`lint-ci`, `check-sqlx` (no query changed), the e2e projects after `checked`.

Nothing found against the domain map: `contribution::attendance` now uses
`contribution::duties::plays_games_on` and `contribution::game_attendance` (same domain);
`counted_games` uses `events::schedule` (allowed). `crates/core/tests/domains.rs` passes. The
route is read-only: `games_of` and `api::changes::list` write nothing and take no lock (search
for `insert|update|delete|lock|info!|warn!` in `api/changes.rs::list`; trace of `games_of`).

| # | Finding | Weight | How checked |
|---|---------|--------|-------------|
| R1 | **Who counts at a game is worked out on the side.** `season_overview` filters the games by `plays_games_on(roles, player.id, game.date)` and looks up the player's answer itself, instead of reading the list of the page of a game (`game_list`, which `architecture.md` calls the source of "the only count", `game_count`). Decision 0022 says "nothing is counted on the side", and this spec's own decision row gives "one rule for what counts (decision 0022)" as the reason for `counted_games`. Today both give the same players (trace: `game_list` keeps the players with `plays_games_on` on the date and finds the answer the same way), so no number is wrong now; but a later change to the list (for example a player left out of it) would change the page of a game and not "Attendance". Better: `counted_games` gives the rows of `game_list` (over the players, the roles and `answers_in_effect`) and `season_overview` reads `ListRow { player, answer, reason }`, skipping `answer: None`. That also drops the use of `plays_games_on` from `attendance.rs`. | medium (pattern, no bug now) | trace of `game_list` (`game_attendance.rs` 233–254) against the new loop in `attendance.rs` |
| R2 | **A gone checked game counts by way of `kept_games`, not by its own rule; the route test does not cover it.** `games_of` drops a game that left the game list once its change is seen (and always in an earlier season), unless `kept_games` keeps it; a game is kept when it has a stored answer. A checked game with anyone on its list always has answers (the settled ones), so it stays and counts: the intent ("a checked game that left the game list counts") holds. A checked gone game with no answer at all drops out, which changes only `overview.games` (no player counts at it anyway). Only the core test `only_checked_games_of_the_team_that_were_played_count` covers "gone"; no route test reads a gone checked game after its change is seen. Worth one line in `attendance_games.rs` (game leaves the list, change marked seen, still counts), since the rule rests on another domain's keep rule. | low | trace of `schedule` (`events/schedule.rs` 90–140), `kept_games` (`duties.rs` 144), `SeasonAttendance::answered`; search for a gone game in `tests/attendance_games.rs` (none) |
| R3 | **Small claims in this document do not match the test.** Step 2 says Bram answers "not coming" with "Ill"; the test adds and uses a reason "Exam" (valid). "After 'Undo check' no games" is checked as `team == []`, not `games == 0`. The meaning holds; the words do not. | low | read of `tests/attendance_games.rs` |
| R4 | **A doc comment the diff made false.** `SeasonOverview.trainings` in `crates/core/src/contribution/attendance.rs` still says `0: "No trainings yet this season."`; the page now says that text only when `games` is 0 too ("No trainings or games yet this season."). The new comment on `games` is right. | low | search `No trainings yet` in `crates/`, `web/`, `docs/` |
| R5 | **For Close: living documents now out of date** (not a fault of the build; listed so Close finds them). `docs/architecture.md`: "contribution" (`season_overview` described with trainings only; `counted_games` missing from `game_attendance`), "Trainings and attendance" (`GET /api/attendance` now also reads `games_of` and `check_start`, gives `games` and `games_from`, `missed[].kind`, nullable `reason`). `docs/product/trainings-and-attendance.md` lines 27 and 29 (the old texts "No missed trainings.", "No trainings yet this season.", "No player had a role at these trainings."). | info | search for the old texts and `season_overview` in `docs/` |

- R1, 2026-11-08: fixed: `counted_games` takes the players and the roles and gives the list of the page of each game (`CountedGame.list`, from `game_list` over the answers in effect); `season_overview` reads the rows and skips "no answer", and no longer uses `plays_games_on`. The core tests build their games through `game_list`.
- R2, 2026-11-08: fixed: route test `a_checked_game_that_left_the_game_list_still_counts` (the game leaves the list, its change is seen when shown, and it still counts).
- R3, 2026-11-08: fixed: step 2 names the reason "Exam" and says that no row has games after "Undo check".
- R4, 2026-11-08: fixed: the comment of `SeasonOverview.trainings`.
- R5, 2026-11-08: not fixed here: for Close.

No bug against the spec's intent found: a not played game, a check for another date (moved
game), a game of another club team, a `Kept` game, "no answer", a player without "plays games" on
the date, a player with only games, "Left the team", the missed order and the release line are
each covered by a test that fails when the rule is removed (read of the tests; the route and
browser tests run). `Progress` keeps "All checks pass" open: `just check` was not run whole here.

### Acceptance review

Acceptance review, 2026-11-08 (fresh agent, `/review accept`). Input: story D, the context, the
spec reviews S4, S11, S13, S14, S36 and S39 in the epic's `functional.md`, this folder's
`change.md` and the objective here, the uncommitted diff against `main`. No extra text in the
arguments. Run: `cargo test -p team --test attendance_games` (3 passed). The page itself was not
opened in a browser; the browser test `checked.e2e.ts` was read, not run here (the build review
ran it).

**Verdict: accepted with notes.** Each criterion of story D is met; A1 is a page error in a rare
case that should be fixed (a few characters) before Close.

| Criterion of story D | Built | How checked |
|---|---|---|
| "of 9 games: 8 there · 1 absent, valid · 0 absent, not valid"; no part when no game counts | yes (`ofGames` + `countsText`, only when `games.games > 0`) | read of `+page.svelte`, `trainings.ts`; browser test text |
| An absence without a reason is "absent, not valid", no tie to "No reason" | yes (`reason: None` → not valid; no lookup by name) | read of `season_overview`; route test (Cas) |
| Only checked games that were played; on the list on that date with "there" or "absent"; no extra players | yes (`counted_games`, `plays_games_on`, "no answer" skipped, `extra_players` never read) | read of the diff; core and route tests |
| "Attendance" says that the games start at the release | yes, in the season of the release only (S39 g); see A2 | read; route test `the_release_day_shows_only_in_its_season` |
| A tap lists the missed games with date and reason | yes, "Game Sat 10 Oct · Ill" / "· without a reason" (S39 f); see A1 | read of `missedText`; browser test |
| A player with games and no trainings has a row; texts cover games; "Left the team" counts games | yes; all four texts changed | read; route test (`trainings == 0`); core test |

| # | Finding | Sort | How checked |
|---|---------|------|-------------|
| A1 | **Two missed games on one date break the tap on that player.** The missed list is keyed `${missed.kind}-${missed.date}`. The tool does hold two games of the team on one date (`events/schedule.rs` 138 `within_date` orders them; `duties.md` "Clashes" speaks of another game of the team). A player absent at both, once both are checked, gets two items with the key `game-2026-10-10`. Svelte 5.57.2 throws on a duplicate key, also in a production build (`each.js` 357–363 calls `e.each_key_duplicate`, which throws in `errors.js` 136–147), so the list does not open. Rare, but the spec asks the list to show each missed game. A key that cannot repeat (the index, or a game id in `MissedView`) fixes it. The two lines would also read the same ("Game Sat 10 Oct · Ill"); with a start time they could be told apart, but that is neutral and not asked. The old key (`missed.date`) could not repeat because trainings are unique by date (`migrations/0006_trainings.sql` line 8). | concerning deviation (small fix, no spec change) | trace of the Svelte source in `web/node_modules`; not reproduced by a run |
| A2 | **The release line says when the start page asks, not that games start at the release.** The spec wanted "Attendance says that the games start at the release" (from S11: no earlier games). The coordinator's answer of 2026-11-08 lets every checked game count, also an older one, so "games start at the release" would be false; the line "Games count once they are checked. The start page asks for games from Sun 8 Nov." says what is true. The epic's story D text still carries the old meaning; Close should write the new one in `docs/product/trainings-and-attendance.md`. | faithful adaptation | read of the spec, S11, the log in `change.md`, `gamesFromText` |
| A3 | **A row with only games shows no training part at all**, not "of 0 trainings"; the games are a second line with "of N games:" in front. The spec leaves the layout open; this reads well and keeps one shape for both parts. | faithful adaptation | read of `+page.svelte` |
| A4 | **A player who gets a role later and so has "no answer" on a checked game does not count there** and nothing on "Attendance" says so. This matches story C ("does not count until the coordinator taps that player"; the note is shown at the change of role) and answers S4. | faithful adaptation | read of `season_overview` (answer `None` skipped) against story C |
| A5 | **A checked game that left the game list and has no stored answer at all drops out of the season count** (`games`), see R2. No player counts at such a game, so no row changes; at most "No trainings or games yet this season." could show for a season whose only checked game is such a game, where "No player had a role at these events." would be the same message in other words. | neutral drift | trace of `games_of`/`kept_games` as in R2 |

- A1, 2026-11-08: fixed: the missed list is keyed by position; browser test `a player who missed two games on one date sees both` (a replaced answer with two missed games on one date; it fails with the old key).
- A2, 2026-11-08: no change needed; Close writes the new meaning into `docs/product/trainings-and-attendance.md`.
- A3, A4, 2026-11-08: no change needed: faithful adaptations.
- A5, 2026-11-08: not fixed: no row changes; the same case as R2.

No other differences from the intent found. Deviations in this document ("None") agree once A2 is
read as covered by the coordinator's answers in the log.
