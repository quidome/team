# Technical spec: Checked for a duty game

**Status**: approved
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
- [ ] Step 1: the core rules
- [ ] Step 2: the data and the routes
- [ ] Step 3: the row on the page
- [ ] Step 4: the check of the look and the checks
- [ ] All checks pass

## Deviations

## Review findings
