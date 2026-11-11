# Technical spec: Checked for every event

**Status**: approved
**Size**: epic
**Change**: docs/changes/2026-11-11-checked-for-every-event/
**Date**: 2026-11-11

## Objective
Give every event one end in the administration ("Checked"), ask for every past event that is not
checked, and show every mark in one form, in four stories that can each be released alone.

## Requirements summary
From `functional.md` (approved 2026-11-11):
1. **One administration panel for each event**: every mark of a training and a game in one
   panel, one row each, the state as a fact (○ / ✓) and one button ("Mark as …", "Count the
   drives", "Undo"); orange only for the next step; rare marks ("Cancel training", "Mark as not
   played") a plain button until set; on an away game the drivers on the line below "Not
   checked"; a team game without any answer warns before "Mark as checked". Nothing that a mark
   does changes.
2. **"Checked" for a duty game**: from its start time, for its date (decision 0029), only while it
   is a duty game; not for a duty game marked "Not played"; changes only the attention; "·
   Checked" on "Season".
3. **Every past event that is not checked is an open item**: a new item "duty games not checked";
   "games not checked" from the start of the season (no date 4 November 2026); both over the
   seasons of the open items; an item of more games opens their list; the closed list of old games
   and the sentence on "Attendance" go; the report's line follows.
4. **The start page lists only today and later**: the last duty games go.

## Current state
- Marks: `routes/training/[date]/+page.svelte` ("Checked" / "Undo check", "Cancel training" /
  "Undo cancel"), `routes/game/[season]/[id]/+page.svelte` ("Checked ✓" panel with "Undo check",
  "Not played" with "Undo", the switch "The drivers drove", the text and switch "Schedule
  entered"), `lib/components/GameAttendance.svelte` ("Answers checked", the closed list of games
  before the release).
- The check of a team game: `game_attendance.checked_for` (decisions 0029, 0032), `PUT` / `DELETE
  /api/games/{season}/{id}/checked` in `api/games.rs`, `team_core::checked`; "Checked" makes "no
  answer" absent (`game_answers.settled`).
- The open item: `team_core::games_not_checked` with the release day from `game_check_start`
  (migrations 0015, 0016), read by `api/start.rs` and `api/report.rs`.
- The start page: `last_games` in `output/start_page.rs`, `StartAnswer.last_games` in
  `api/start.rs`, the part `last-games` in `routes/+page.svelte`.

## Architecture (all stories)
- **Story 1** is front end only: new components `AdminPanel.svelte` (the panel) and `MarkRow.svelte`
  (one row: state, ✓ or ○, an optional line, one button; an input `next` gives the orange), used by
  the pages of a training and a game. No server change.
- **Story 2**: two columns of `game_duties` (the date checked for, the moment; planned as a new table `duty_checks`, see the story), owned by
  `duties.rs` (decision 0030), written under the row lock of a game in duties (decision 0023). The
  rule "a mark holds for a date" (decision 0029) becomes one function in `contribution` that the
  check of a team game and of a duty game both use. `PUT` / `DELETE .../checked` decide by the kind
  of the game: a team game as now, a duty game only the duty check. Each check is read only for its
  own kind of game.
- **Story 3**: `games_not_checked` loses its start date; `game_check_start` goes (forward-only
  migration); a new rule `duty_games_not_checked` and open item after "games not checked"; the
  items of more games carry their games for the list; the report's line follows the same rule.
- **Story 4**: `last_games`, `StartAnswer.last_games` and its part on the page go; `event_look`
  loses "a date before today".
- The domain map does not change.

## Stories
| # | Story | Scope | Depends on | Status |
|---|---|---|---|---|
| 1 | The administration panel | `AdminPanel.svelte`, `MarkRow.svelte`; the pages of a training and a game; the warning of a team game without answers; the drivers below "Not checked"; button words; browser tests | – | done |
| 2 | Checked for a duty game | two columns of `game_duties` (migration 0017; was planned as a table `duty_checks`), the shared rule of a check for a date, `.../checked` for both kinds, the duty game page and "Season"; decision record | 1 (the panel row) | done |
| 3 | Open items from the start of the season | `games_not_checked` without start date, `game_check_start` removed (migration), `duty_games_not_checked` and its open item, the list of an item of more games, the report's line, the old closed list and the sentence on "Attendance"; decision record (partly replaces 0032) | 2 | to design |
| 4 | Only today and later on the start page | `last_games` removed from core, API and page; `event_look` without "before today"; product documents | 3 | to design |

Each story gets its own change folder (`docs/changes/YYYY-MM-DD-<name>/`, type `feature`, stage
`design`) and its own `/design`, with its own steps, tests and closure line. Each can be released
alone, in this order.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| The check of a duty game with the duties of the game (built as two columns of `game_duties`) | `game_attendance` is the attendance of a team game, and its check makes players absent; each domain owns its tables (0030) | yes, at Close of story 2 |
| One rule "a mark holds for a date" for both checks | The two checks cannot drift apart (0029) | part of the record of story 2 |
| One address `.../checked`, deciding by the kind of the game | The page and the panel call one action for every game; the server knows the kind | part of the record of story 2 |
| No start date for "games not checked"; `game_check_start` removed | The same rule as for trainings (the seasons of the open items); nothing to keep up to date | yes, at Close of story 3 (partly replaces 0032) |
| One panel component with an input `next` for the orange | Every page follows one rule for the marks | no; the rule goes into `principles.md` at Close of story 1 |

## Not in scope
- "Someone else" before the game; checking several games at once; the totals and the numbers of
  the report; "Season" in the quiet look.

## Test plan
- **Existing tests that must still pass**: all, except where a story changes them (button words in
  story 1; the open items and the last duty games in stories 3 and 4).
- **New tests**: in the design of each story.

## Compatibility and performance
- Story 2 and story 3 each add a migration (forward-only). Story 3 makes the first open items long
  once; one row per kind.
- No change to what counts.

## Progress
- [x] Story 1: the administration panel (2026-11-11, `changes/done/2026-11-11-administration-panel/`)
- [x] Story 2: Checked for a duty game (2026-11-11, `changes/done/2026-11-11-checked-for-a-duty-game/`, decision 0035)
- [ ] Story 3: open items from the start of the season
- [ ] Story 4: only today and later on the start page

## Deviations
None. Design reads the spec as written.

## Review findings
