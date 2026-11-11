# Technical spec: The administration panel

**Status**: done
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
Reviewed 2026-11-11 by `/review accept` (fresh agent). Read: `change.md`, this file, the epic's
`idea.md` and `functional.md` (story 1, and the spec review), `docs/language.md`, and the diff
`main...administration-panel` (commit 5eb5afc, the working tree was clean). Ran: `npm run build`,
then the whole browser suite with one extra test of my own from the scratchpad (167 passed, the
extra one included). The Deviations were read after the view below was formed.

**Verdict: needs correction** – one fix in the front end (A1); the spec stays as it is. The rest
matches what was meant.

| # | Point of story 1 | Built | Sort |
|---|---|---|---|
| A1 | The warning without answers, and the count below "Not checked" | Stale after a tap in the list | **concerning deviation** |
| A2 | One panel, one row for each mark that can be set now, none without | Yes | faithful |
| A3 | State as a fact with ○ / ✓, one button, "Undo" when set | Yes | faithful |
| A4 | Orange only for "Mark as checked" and "Mark as entered" | Yes | faithful |
| A5 | Count and drivers below "Not checked"; the message after the tap | Separate lines; open places too | faithful adaptation |
| A6 | The warning: "no answer and no extra player" (technical) | Extra players not looked at | neutral drift |
| A7 | Rare marks, the note of a moved game, test ids | As the spec | neutral drift |

- **A1 – The count and the warning in the panel do not follow the taps in the list (concerning).**
  The row "Not checked" of a game reads `page.attendance.count` (`countText`, `noAnswerWarning`
  in `web/src/routes/game/[season]/[id]/+page.svelte`). Only the saves of the page change that
  count. A tap in the list (`GameAttendance.svelte`) changes the count inside that component and
  never reaches the page. Checked by a trace, and by a run on the away game of Sat 10 Oct in the
  e2e database. Before a tap the panel showed "5 no answer · Drivers: 1 open · Nobody has an
  answer yet: marking as checked makes all 5 players absent.". After one tap on "There" the list
  showed "1 there · 4 no answer", and the panel still showed "5 no answer" and the same warning.
  This is the case the warning was made for (spec review S5: old team games with no answers).
  The coordinator starts entering who was there and is told that all 5 players will be made
  absent, which is no longer true. That makes the taps look lost, and the page shows two
  different counts. The same stale count is below "Not checked" on every team game while the
  list is open. Correction: the row reads the count the list shows (for example a callback next
  to `onanswers`), and a browser test makes a tap in the list and then checks the panel. The
  existing warning test mocks the page answer and makes no tap, so it cannot see this. The page
  of a training does not have this problem: its line uses the live marks (`topLine(page, shown)`).
- **A2 – The panel.** It shows below the heading on both pages, only when a row is there: on a
  game when it is "Not played", the drives, the schedule or "Checked"; on a training when it is
  "Cancelled", "Checked" or can be checked. The rows are in the spec's order. A coming game
  without marks has no panel. "Schedule entered" shows before the game, under the same rule as
  before. Checked by reading the diff and the new tests in `not-played`, `duties` and `trainings`
  (they ran and passed).
- **A3 – Rows.** `MarkRow` shows the state, ✓ / ○ (`aria-hidden`) and one button. The button
  never repeats the state: "Mark as checked", "Mark as entered" and "Count the drives", with
  "Undo" when set. "Undo check" and "Undo cancel" are gone. "Schedule entered" left the card of
  the places. "The drivers drove" is no longer a pressed switch. Checked by reading the diff, and
  by a search for `Undo check|Undo cancel|The drivers drove|aria-pressed` in `web/src`. On the
  screen the pressed state is only left in the choices, which are not marks. The old words are
  still in the comments of `web/src/lib/api/client.ts` (lines 634, 644, 727, 741) and in the API
  descriptions in `schema.d.ts` (made from the server's doc comments). These are not on the
  screen; Close can bring them up to date.
- **A4 – Orange.** `next` sets the button variant `default`, which is `--primary`, orange (hue
  40, `web/src/routes/layout.css`). Only "Mark as checked" and "Mark as entered" get `next` while
  the mark is not set. "Count the drives", "Undo", "Mark as not played", "Cancel training" and
  "Mark answers as checked" are outlined. Checked by reading the diff.
- **A5 – The lines below "Not checked".** The count, the drivers and the message after the tap
  ("N players with no answer are absent.") are each on their own line, not one line joined by
  "·" as in the spec's example. The drivers line also shows the open places ("Drivers: 1 open").
  That serves the intent of S1 better: one check also confirms the drive, so an open place has to
  be in view. Faithful adaptation (the same as the deviation of step 3).
- **A6 – The warning and extra players.** The technical spec asks for the warning only "with no
  extra player". The code checks only the players: no coming and no not coming, and at least one
  with no answer. With an extra player and no answers the warning still shows. It is still true
  there (all N players become absent), and it matches the functional spec ("no player has an
  answer, and no player is marked there or absent"). Neutral drift; the technical spec can be
  brought in line at Close. Checked by reading `noAnswerWarning` and `CountView` in
  `web/src/lib/api/schema.d.ts`.
- **A7 – The rest.** "Mark as not played" and "Cancel training" are plain outlined buttons that
  become the rows "Not played" ("Its duties and attendance do not count.") and "Cancelled" with
  "Undo". "Mark answers as checked" stays in its note. The test id `set-not-played` (deviation of
  step 2) changes only names. What each mark does is not changed: the same API calls as before,
  and no server change (`git diff --stat` touches only `web/` and this file). Checked by reading
  the diff.

Not checked: the screenshots in light and dark mode, and the contrast figures of step 4 (I did
not take them again; the accessibility checks in the tests passed).

**Answers (2026-11-11, Implement).**
- A1: fixed. `GameAttendance.svelte` reports the count it shows (`oncount`, next to `onanswers`);
  the row "Not checked" reads that count while the list is open, so the count and the warning
  follow each tap. New browser test `the row "Not checked" follows the taps in the list`
  (`checked.e2e.ts`): a tap on "There" shows "1 there" and removes the warning; a tap back brings
  the warning back.
- A2–A5: as meant; no change.
- A6: not changed: the warning follows the functional spec (no player with an answer); an extra
  player does not change what "Mark as checked" does to the players. Step 3 above said "and no
  extra player"; the build is right, the step text was wider.
- A7: not changed. The old words in comments of `web/src/lib/api/client.ts` and the API
  descriptions (`schema.d.ts`) are updated at Close.
