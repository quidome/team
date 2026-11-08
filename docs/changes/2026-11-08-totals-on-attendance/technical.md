# Technical spec: Totals on "Attendance"

**Status**: approved
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
  - After "Checked": Anna ("coming") there, Bram ("not coming", "Ill", valid) absent valid, Cas
    ("no answer") absent not valid with the missed `{ "kind": "game", "reason": null, "valid":
    false }`; `games` is 1; after "Undo check" no games.
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
- [ ] Step 1: the core rules
- [ ] Step 2: the route
- [ ] Step 3: the page
- [ ] All checks pass

## Deviations
None from `functional.md`. The coordinator's answers (2026-11-08) settle what the spec left open:
older checked games count; gone checked games count; the words of a missed game.

## Review findings
