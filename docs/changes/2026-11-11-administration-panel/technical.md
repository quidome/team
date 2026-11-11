# Technical spec: The administration panel

**Status**: in progress
**Size**: story
**Change**: docs/changes/2026-11-11-administration-panel/
**Date**: 2026-11-11

## Objective
Show every mark of a training and a game in one panel "Administration" at the top of the page,
one row each with its state (○ / ✓) and one action, so a state is never mistaken for a button.

## Requirements summary
From the epic's `functional.md`, story 1 (`docs/changes/2026-11-11-checked-for-every-event/`):
- One panel "Administration" at the top of the page of a training and of a game (below the
  heading), with one row for each mark that can be set now; no panel without such a mark.
- A row: the state as a fact ("Not checked", "Checked", …), ✓ when set and ○ when not, an optional
  line below, and one button: the action when not set, "Undo" when set. The button never repeats
  the state.
- Rows of a game, in this order: "Not played" (when set) · Undo, with "Its duties and attendance do
  not count."; "No drive counts" / "The drives count" (a not played game, from its start time, with
  driver places) · "Count the drives" / Undo, never orange; "Schedule not entered" / "Schedule
  entered" (a home game of the club, while required or entered, also before the game) · "Mark as
  entered" (orange) / Undo; "Not checked" / "Checked" (a team game from its start time) · "Mark as
  checked" (orange) / Undo, with the count below, on an away game also "Drivers: Anna, Bram", after
  the tap the message "2 players with no answer are absent.", and before the tap on a game without
  any answer "Nobody has an answer yet: marking as checked makes all N players absent.".
- Rows of a training: "Cancelled" (when set) · Undo; "Not checked" / "Checked" (today or earlier,
  as now) · "Mark as checked" (orange) / Undo, with "2 absent" / "All there" below.
- Outside the panel: "Mark as not played" (last button on the page of a game), "Cancel training"
  (with "Change" and "Delete training"), the note of a moved game with "Mark answers as checked".
- Nothing that a mark does changes.

## Current state
- `web/src/routes/game/[season]/[id]/+page.svelte`: "Not played" panel with "Undo" and the switch
  "The drivers drove" (`data-testid="not-played"`, `drivers-drove`) at the top; "Checked ✓" panel
  with "Undo check" (`checked`, `settled`); the text and switch "Schedule entered" inside the card
  `schedule` (`schedule-state`); the buttons "Checked" (`mark-checked`) and "Not played"
  (`mark-not-played`) at the bottom.
- `web/src/routes/training/[date]/+page.svelte`: "Cancelled" with "Undo cancel"; "Checked" with
  "Undo check" (`checked`) or the button "Checked" at the top; "Cancel training" at the bottom.
- `web/src/lib/components/GameAttendance.svelte`: the note of a moved game with "Answers checked".
- The page of a game has the count (`attendance.count`), the answers and the places (with the
  drivers and their players); the training page has the attendance.

## Steps

### Step 1: The components and the page of a training
- **Goal**: the panel exists, and the page of a training uses it.
- **Changes**: `web/src/lib/components/MarkRow.svelte` (inputs: `state`, `done`, `line?`
  (snippet), `action`, `next?`, `busy`, `onclick`, `testid`; ✓ / ○ `aria-hidden`; attributes
  `data-done` and `data-next`; the button at least 44 px; orange only with `next`),
  `web/src/lib/components/AdminPanel.svelte` (the grey box "Administration", rows as children).
  The page of a training: the rows "Cancelled" and "Checked" in the panel below the heading;
  "Undo check" and "Undo cancel" become "Undo"; "Checked" becomes "Mark as checked".
- **Tests**: `web/e2e/trainings.e2e.ts` follows the new words and `data-testid`s; a new test: the
  panel at the top of a training, "Not checked" with an orange "Mark as checked", after the tap
  "Checked" with "Undo" and no orange; no panel on a training after today.

### Step 2: The page of a game
- **Goal**: the marks of a game are rows of the panel.
- **Changes**: the rows "Not played", "The drives count", "Schedule entered" and "Checked" in the
  panel below the heading, in this order; the switch "Schedule entered" and the text
  `schedule-state` leave the card of the places; the switch "The drivers drove" becomes the row;
  "Undo check" becomes "Undo"; the last button "Not played" becomes "Mark as not played"; the
  "Checked" button at the bottom goes (the row replaces it).
- **Tests**: `checked.e2e.ts`, `not-played.e2e.ts` and `duties.e2e.ts` follow; new tests: the
  order of the rows on a not played away game; "Mark as entered" on a home game of the club before
  the game; no panel on a coming game without marks.

### Step 3: The lines below "Not checked" and the note of a moved game
- **Goal**: the warning without answers, the drivers, and the last word.
- **Changes**: on a team game, below "Not checked": the count; on an away game also "Drivers:
  Anna, Bram" (the players on the driver places, short names as on the page); "Nobody has an
  answer yet: marking as checked makes all N players absent." when the count has no coming, not
  coming, there or absent and no extra player (N = no answer). "Answers checked" becomes "Mark
  answers as checked" in `GameAttendance.svelte`.
- **Tests**: `game-attendance.e2e.ts` follows; new tests: the warning (the answer of the page
  replaced with a game without answers), the drivers line on an away game.

### Step 4: The check of the look
- **Goal**: readable and calm in light and dark mode, on a phone.
- **Changes**: screenshots at 390 × 844 in light and dark mode of a training, a team game, a duty
  game and a not played away game; fix what is not readable. The panel fits 320 px without
  horizontal scroll and passes the accessibility check (in the new tests).
- **Tests**: `just check`.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| The panel at the top of both pages, below the heading | One place for the state of the event; easy to find on a phone (idea 2026-11-08: "Checked" at the bottom was hard to find) | no; the rule goes into `principles.md` at Close |
| "Schedule entered" moves from the card of the places into the panel | Every mark in one panel; the places stay in their card | no |
| `data-done` and `data-next` on a row | Tests check the state and the orange without reading words or colours | no |

## Not in scope
- What a mark does; "Checked" of a duty game (story 2); the closed list of old games (story 3).

## Test plan
- **Existing tests that must still pass**: all, with the five browser test files above following
  the new words.
- **New tests**: per step, above. No Rust test: the server does not change.

## Compatibility and performance
- Front end only; no change to the API or the data.

## Progress
- [x] Step 1: the components and the page of a training (2026-11-11)
- [x] Step 2: the page of a game (2026-11-11)
- [x] Step 3: the lines below "Not checked" and the note of a moved game (2026-11-11)
- [x] Step 4: the check of the look (2026-11-11). Screenshots at 390 × 844 in light and dark mode
  of a training, a team game (with the drivers and the warning), a duty game and a not played away
  game: all readable. Contrast on the panel, light / dark: the grey line 5.28 / 6.26, the red
  warning 5.81 / 7.86, the text 15.20 / 13.84 (need 4.5). Nothing changed.
- [x] All checks pass

## Deviations
- 2026-11-11, step 2: the last button "Mark as not played" has the test id `set-not-played`; the
  row "Not played" of the panel has `mark-not-played`, as the design names the rows. Only names.
- 2026-11-11, step 3: the warning has a form for one player too ("…makes the 1 player absent.").
  The drivers line also shows the open places ("Drivers: Anna, 1 open"), so an open place is in
  view when the coordinator checks the game.

## Review findings
