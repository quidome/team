# Technical spec: Checked for a duty game

**Status**: done
**Size**: story
**Change**: docs/changes/2026-11-11-checked-for-a-duty-game/
**Date**: 2026-11-11

## Objective
Let the coordinator mark a duty game as checked, so it no longer asks for attention, with the same
rule of a check for a date as a team game and without touching the attendance or the totals.

## Requirements summary
From the epic's `functional.md`, story 2 (`docs/changes/2026-11-11-checked-for-every-event/`):
- From its start time (without a start time: from the start of the game day) a duty game has the
  row "Not checked ○" / "Checked ✓" with "Mark as checked" (orange) / "Undo" in the panel
  "Administration", below "Schedule entered"; the line below lists the places of the team
  ("Table 1: Anna · Table 2: Cas (was Bram)").
- Before the start time no row; a check is refused: "A game can be checked from its start time.".
- The check holds for the date of the game (decision 0029), and only while the game is a duty game
  (the check of a team game only while it is a team game).
- A duty game marked "Not played" has no row and is never asked to be checked.
- Only the attention changes; the totals and the report do not.
- "Season" shows "· Checked" on a checked duty game.
- A duty game that left the game list after it was played (kept for its duties) can still be
  checked.
- Duty game: a game of another club team in the club's own hall where the team has table or
  referee places (`docs/language.md`).

## Current state
- `team_core::checked(attendance, date)` (`contribution/game_attendance.rs`): `checked_for ==
  Some(date)`.
- `PUT` / `DELETE /api/games/{season}/{id}/checked` (`api/games.rs`, `set_checked`,
  `unset_checked`): `team_game` refuses other games (`not_a_team_game`); the check is
  `game_attendance::check` / `uncheck` in the attendance of the game; `check_game_checking` refuses
  before the start time.
- `duties::lock` (`crates/server/src/duties.rs`): the row lock of a game for its places.
- `SeasonDuties` (`api/season.rs`) loads the duties of a season; `event_view_with` sets
  `ScheduleGame.checked` from `duties.attendance.games`.
- `ScheduleReason::{TeamGame, DutyGame, Kept}` (`events/schedule.rs`); `GamePlaces.schedule.of_team`
  are the table and referee places of the team.
- Migrations up to `0016_game_check_start_from_release_1.sql`.
- The panel of the page of a game: `routes/game/[season]/[id]/+page.svelte` with `MarkRow`.

## Steps

### Step 1: The core rules
- **Goal**: one rule of a check for a date, and the kind of check a game has.
- **Changes**: in `contribution` (for example a module `contribution/mark.rs`):
  `pub fn mark_holds(checked_for: Option<NaiveDate>, date: NaiveDate) -> bool`;
  `team_core::checked` uses it. In `contribution/duties.rs`: `pub enum CheckKind { Team, Duty }`
  and `pub fn check_kind(reason: ScheduleReason, places: Option<&GamePlaces>) ->
  Option<CheckKind>`: `TeamGame` → `Team`; another reason with table or referee places of the
  team (`schedule.of_team` not empty) → `Duty`; else `None`. Flat exports in `team_core`.
- **Tests**: `mark_holds` (same date, another date, none); `check_kind` (team game; duty game
  with a table place of the team; duty game without places of the team; kept game with places of
  the team; kept game with only answers). The existing tests of `checked` pass unchanged.

### Step 2: The data and the routes
- **Goal**: a duty check can be stored, read and undone through the existing address.
- **Changes**:
  - `migrations/0017_duty_checks.sql`: table `duty_checks` (the key of a game as the other tables
    of a game, `checked_for date NOT NULL`, `checked_at timestamptz NOT NULL DEFAULT now()`;
    primary key the game). A second check on the same date keeps the first moment; a check on a
    new date replaces the date.
  - `duties.rs`: `check(tx, key, date)` and `uncheck(tx, key)` under `duties::lock`; the duty
    checks of a season loaded with the duties (`SeasonDuties`), by source id.
  - `api/games.rs`: `set_checked` / `unset_checked` choose by `check_kind`: `Team` as now; `Duty`:
    refused before the start time (`checked_too_early`) and on a game marked "Not played"
    (`duty_game_not_played`, "A game that is not played needs no check."), else the duty check,
    answer `{ settled: 0, page }`; `None`: `not_checkable` (replaces `not_a_team_game` on these
    routes). `DELETE` is always allowed for a duty game. The log line names the season, the game and
    "duty"; no names.
  - `GamePage.duty_checked: Option<bool>`: `null` when the game is not a duty game, else whether
    the duty check holds for its date.
  - `api/season.rs` `event_view_with`: `checked` from the duty check for a duty game, from the
    attendance for a team game (by `check_kind`), so "Season" and the start page follow.
  - `just api-types`; after the query changes `just prepare`.
- **Tests**: new `crates/server/tests/duty_checks.rs`: a check from the start time shows on the
  page (`duty_checked`) and "Undo" removes it; refused before the start time, on a not played game,
  and on a game without places of the team (`not_checkable`); after a move the check does not hold;
  after another team is chosen the check has no effect; "Season" shows the game as checked. The
  existing tests of the check of a team game pass (one may need `not_checkable` instead of
  `not_a_team_game`).

### Step 3: The row on the page
- **Goal**: the coordinator sees and sets the duty check.
- **Changes**: the row "Not checked" / "Checked" in the panel of a duty game, below "Schedule
  entered", when `duty_checked !== null`, the game has started and is not "not played"; the line
  below lists the places of the team with the words of the places on the page ("Table 1: Anna ·
  Table 2: Cas (was Bram)", "open" for a place without a player). The same `setChecked` /
  `unsetChecked` calls.
- **Tests**: in `duties.e2e.ts`: a past duty game with a place of the team shows the row with its
  places line; "Mark as checked" → "Checked ✓" with "Undo"; "· Checked" on "Season"; the test
  undoes the check and the place at the end.

### Step 4: The check of the look and the checks
- **Goal**: readable in light and dark mode; all checks pass.
- **Changes**: a screenshot of a duty game in light and dark mode.
- **Tests**: `just check`.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| The duty check in its own table `duty_checks`, owned by duties | `game_attendance` is the attendance of a team game and its check makes players absent; each domain owns its tables (0030) | yes, at Close |
| One rule `mark_holds` for both checks | The two checks cannot drift apart (0029) | part of that record |
| One address `.../checked`, choosing by `check_kind` | One action for every game on the page and in the panel | part of that record |
| `check_kind` decides which check holds | "Only while it is a duty game" is one rule for the page, the routes and "Season" | part of that record |

## Not in scope
- The open item of duty games (story 3); the start page list (story 4); the totals and the report.

## Test plan
- **Existing tests that must still pass**: all; the tests of the check of a team game unchanged
  except a refusal code.
- **New tests**: per step, above.

## Compatibility and performance
- One migration (a new table, forward-only). The API keeps its shape for team games; `GamePage`
  gets one field. One more small query when the duties of a season are loaded.

## Progress
- [x] Step 1: the core rules (2026-11-11)
- [x] Step 2: the data and the routes (2026-11-11)
- [x] Step 3: the row on the page (2026-11-11)
- [x] Step 4: the check of the look and the checks (2026-11-11). Screenshots at 390 × 844 in
  light and dark mode of a past duty game with two places of the team, before and after "Mark as
  checked": readable; after the tap the panel has no orange left (the first picture caught the
  colour animation of the button, `transition-all`, halfway). Nothing changed.
- [x] All checks pass

## Deviations
- 2026-11-11, step 2: the duty check is two columns of `game_duties` (`checked_for`,
  `checked_at`, migration `0017_duty_checks.sql`), not a new table `duty_checks`. `game_duties` is
  the row of a game that duties owns and locks; "Schedule entered" and "The drives count" are
  columns of it too. The same owner, the same lock, one less table; the design's reason (not in
  the attendance of a team game) holds.
- 2026-11-11, review fixes: the address of a check takes `?kind=team` or `?kind=duty` (the check
  the page shows); a mismatch with the game now is refused with `game_changed` ("This game
  changed. Load the page again."), so a page that is out of date cannot run the check of the other
  kind (build review R3). `mark_holds` sits in `shared` (`shared/mark.rs`), so "Not played" uses
  it too (R2). One core rule `game_checked` decides whether a game counts as checked (R1).

## Review findings
Reviewed 2026-11-11 by `/review build` (fresh agent). Read: the diff against `main` (branch
`duty-game-checked`, working tree clean), this folder, the epic's `functional.md` (story 2) and
`technical.md`, `docs/architecture.md`, decisions 0029, 0030, 0032, `CLAUDE.md`. Runs:
`cargo test -p team --test duty_checks --test checked` (3 + 16 passed), `cargo test -p team_core`
(368 passed), `cargo clippy --workspace --all-targets -- -D warnings` (clean),
`SQLX_OFFLINE=true cargo check -p team` (ok). Not run: the browser tests and `just check`; the
screenshots of step 4 not checked. Domain map: `contribution::duties` uses `events::schedule`,
`game_attendance` uses `contribution::mark` – allowed (checked by reading the `use` lines).

- **R1 – The rule "which check holds" is written twice in the server.** `api/games.rs` (`page`,
  `duty_checked`) and `api/season.rs` (`event_view_with`) each combine `check_kind` with
  `mark_holds(stored.checked_for, date)` or `checked(attendance, date)` themselves. That is a
  product rule ("each check holds only for its own kind") outside the core, in two copies that can
  drift – the thing the design's decision "`check_kind` decides … one rule for the page, the
  routes and Season" wanted to prevent. Suggest one core function (for example
  `game_checked(kind, attendance, stored, date) -> bool`) used by both. Checked by a trace of both
  call sites.
- **R2 – "One rule `mark_holds`" is not one rule yet.** `events/schedule.rs:32` (`not_played`)
  has the same comparison `not_played_for == Some(game.date)` and cannot call `mark_holds`,
  because events may use only shared (decision 0027) and `mark_holds` sits in `contribution`.
  Decision 0029 covers all marks (answers, "Not played", "Checked"). Either put `mark_holds` in
  `shared` and use it in `not_played` too, or write in the decision record of Close that it covers
  only the two checks. Checked by a search for `== Some(.*date)` in `crates/core/src`.
- **R3 – A page that is out of date can do the other check.** `set_checked` / `unset_checked` pick
  the kind from the state at the moment of the tap. Before this change a non-team game was
  refused (`not_a_team_game`, "Load the page again."). Now: (a) a page opened as a duty game,
  after another team is chosen on another screen so that the game becomes a team game: "Mark as
  checked" runs the team check, which makes every "no answer" absent, without the warning of
  story 1 ("Nobody has an answer yet: …") ever shown; (b) the other way round, a stale team-game
  page's "Mark as checked" / "Undo" silently sets or clears a duty check. Rare (the team of a
  season changes seldom), but (a) writes wrong attendance from one tap. Suggest: the page sends
  the kind it shows (for example `?kind=duty`), and the server refuses a different kind with a
  "Load the page again" code. Checked by a trace of `set_checked`; not reproduced.
- **R4 – "Undo" on a game with no check at all.** `unset_checked` sends every game that is not
  `Team` to `duty_check(false)`, also kind `None` (for example a home game of another club team
  without places of the team): it inserts an empty `game_duties` row under `duties::lock`, sets
  `saved_at` and answers 200. Harmless (an empty row does not keep a game: `kept_games` needs a
  team, drivers or a player, checked by reading it) and it lets a stored duty check whose game
  lost the places of the team be undone. The code comment "always allowed" is right; the design's
  "`DELETE` is always allowed for a duty game" is wider in the code. Note only. Checked by a trace.
- **R5 – Tests that the spec asks for and that are missing or weak.**
  (a) Story 2, criterion 4, "the game becomes a team game": only the branch "no longer a game with
  places of the team" is tested (`a_duty_check_holds_only_for_its_date_and_its_kind`, choosing
  M16_2 removes the places). A duty game that becomes a team game, and its twin, a checked team
  game that becomes a duty game ("Season" must then show it not checked), have no test.
  (b) "A duty game that left the game list after it was played can still be checked" (in the
  requirements summary) has no API test; the unit test with `ScheduleReason::Kept` is another
  case (Kept = no longer in the schedule; a game that left the game list keeps `DutyGame` with
  `in_list` false, see the doc of `ScheduleReason` and `game_places`).
  (c) "A second check changes nothing" asserts only status 200; the claim "a second check on the
  same date keeps the first moment" is not tested, and `checked_at` is read nowhere. The tests
  that exist can fail (each assertion depends on `check_kind` or `mark_holds`; checked by reading
  them).
- **R6 – Comments that mix up "kept" and "left the game list".** The doc of `check_kind` ("also
  one kept for its duties after it left the game list") and the comment in its unit test ("Kept
  for its duties after it left the game list") describe `ScheduleReason::Kept` as a game that left
  the game list; its own doc says "No longer in the schedule (for example home and away swapped)".
  Checked by reading `events/schedule.rs:42`.
- **R7 – Documents the diff made false (for Close).** The epic's `technical.md` (Architecture,
  story 2; Decisions) and the Decisions table of this file still name a table `duty_checks`; the
  migration `0017_duty_checks.sql` names a table that does not exist (it adds two columns to
  `game_duties`, see Deviations). `docs/architecture.md`: the migration list (line 17), the columns
  of `game_duties` (line 145), and "Checked" of story C (line 165, `PUT` / `DELETE .../checked`
  now also take the duty path, `not_a_team_game` replaced by `not_checkable` on these routes). The
  web client keeps `not_a_team_game`, still used by the answer routes (checked by a search for
  `not_a_team_game`). Checked by a search for `duty_checks` and reading architecture.md.

### Acceptance review (`/review accept`, 2026-11-11)
Fresh agent. Read: `change.md`, this spec's objective, the epic's `idea.md` and `functional.md`
(story 2 and the spec review), `docs/language.md`, the diff `main...duty-game-checked` (commit
12e5b17). Ran: `cargo test -p team --test duty_checks --test checked` (19 passed),
`cargo test -p team_core` (all passed). Not run: the browser tests and the app itself. The
**Deviations** were read after the view below was formed.

**Verdict: accepted with notes.**

- **A1 – The row, its place and its line (faithful adaptation).** The row "Not checked" / "Checked"
  with "Mark as checked" (orange, `next`) / "Undo" is in the panel right after "Schedule entered",
  only when `duty_checked` is not null, the game has started and is not "not played"
  (`+page.svelte`, `dutyRow`). The line below uses the words of the places on the page
  (`placesLine` → `playerText`): "Table 1: Anna · Table 2: Cas (was Bram)"; the spec left open
  what an empty place and "Nobody" say: "open" and "not done (was Bram)", the page's own words.
  Checked by a trace (`web/src/lib/duties.ts` `placesLine`, `playerText`) and the e2e test (not run).
- **A2 – Before the start time (matches).** No row (`page.started`); `PUT` refused with
  `checked_too_early` and the spec's text. Checked by a run (`a_duty_check_is_refused_too_early…`).
- **A3 – A check for a date (matches).** One rule `mark_holds` for both checks; after a move the
  check has no effect, back on its date it holds again. Checked by a run
  (`a_duty_check_holds_only_for_its_date_and_its_kind`).
- **A4 – Only while it is a duty game (matches, one neutral drift).** `check_kind` decides on the
  page, the routes and "Season"; a stored check of the other kind has no effect. Drift: the duty
  check is stored for the game, not for the team. When another team is chosen that also has
  places at the same game, the earlier check holds for the places of the new team, which the
  coordinator never looked at. Rare (a new team within a season); the team check behaves the same.
  Checked by a trace (`check_kind`, `duties::set_checked`, key without team).
- **A5 – "Not played" (matches on the page; neutral drift on the rows).** No row and the check is
  refused (`duty_game_not_played`, run). But a check made before "Mark as not played" stays stored,
  so "Season" shows "· Not played · Checked" on that duty game, and the start page row of a duty
  game (the part "last duty games") prints both labels with no separator: "Not playedChecked"
  (`web/src/lib/components/StartEventRow.svelte`, the `{:else if game.not_played || game.checked}`
  branch for games that are not team games). Before this change only a kept game could reach that
  branch with a check; now any checked duty game can. The spec says a not played duty game "has no
  'Checked' row and never asks for attention"; it does not ask, but the label says it is checked.
  Suggest: `checked` false on a not played duty game in `event_view_with`, or a separator. Small;
  the start page part goes in story 4. Checked by a trace, not run.
- **A6 – Only the attention (matches).** No count reads the duty check: `checked_for` of
  `StoredDuties` is read only by `mark_holds` on the page and in `event_view_with`; `duty_totals`
  and `api/report.rs` are not in the diff. Checked by a search (`checked_for` in `crates/`).
- **A7 – "Season" shows "· Checked" (matches).** `event_view_with` sets `checked` by `check_kind`.
  Checked by a run (`checked_on_season` in `duty_checks.rs`); the e2e test covers the label (not
  run). Neutral drift: the start page rows follow too (a checked duty game shows "Checked" in the
  last duty games); the spec names only "Season", and the epic wants the same everywhere.
- **A8 – A duty game that left the game list (matches).** Its reason stays `DutyGame`
  (`in_schedule` reads the teams, not `in_list`) and `game_places` keeps the places of a past game
  out of the list, so it has the check. A game kept only because home and away swapped (`Kept`)
  with places of the team gets the duty check too. Checked by a trace and the core test of
  `check_kind`; no server test of a game out of the list.
- **A9 – Two screens (matches).** The write is unconditional under the row lock: the last check or
  undo wins. Checked by a trace (`duty_check`, `duties::set_checked`).
- **A10 – The deviation (faithful adaptation).** Two columns of `game_duties` instead of a table
  `duty_checks`: the same owner and the same lock as "Schedule entered" and "Count the drives";
  nothing the coordinator sees changes. The decision record of the design table is still due at
  Close and should name the columns, not the table.

**Answers (2026-11-11, Implement).**
- A5 / R1: fixed. One core rule `team_core::game_checked` (the check of its own kind holds for its
  date; a not played duty game is never checked), used by the page (`duty_checked`) and by
  "Season" (`event_view_with`), so the start page follows. The start page row of a game that is
  not a team game has a separator between "Not played" and "Checked". New core test
  `a_game_is_checked_only_by_the_check_of_its_kind`; new server test
  `a_checked_duty_game_marked_not_played_is_not_checked`.
- R2: fixed. `mark_holds` moved to `shared/mark.rs`; `events::schedule::not_played` uses it.
- R3: fixed. `?kind=` on `PUT` and `DELETE .../checked`, refused with `game_changed` on a mismatch;
  the page sends `team` or `duty`. New server test
  `a_check_from_a_page_of_another_kind_is_refused`.
- R4: not fixed (a note): an "Undo" on a game without a check writes an empty row of
  `game_duties`, with no effect.
- R5: fixed. New server tests `a_duty_game_that_becomes_a_team_game_loses_its_duty_check`,
  `a_duty_game_kept_for_its_duties_can_be_checked`, `a_second_check_keeps_the_first_moment`.
- R6: fixed. The comments of `check_kind` and its test name "kept" as "no longer in the schedule".
- R7, A10: at Close (the decision record and `docs/architecture.md` name the two columns of
  `game_duties`).
- A4, A7: not changed (notes): the duty check belongs to the game, as the check of a team game;
  the start page shows "Checked" by the same rule as "Season".
