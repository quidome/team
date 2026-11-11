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
| 3 | Open items from the start of the season | `games_not_checked` without start date, `game_check_start` removed (migration), `duty_games_not_checked` and its open item, the list of an item of more games, the report's line, the old closed list and the sentence on "Attendance"; decision record (partly replaces 0032) | 2 | done |
| 4 | Only today and later on the start page | `last_games` removed from core, API and page; `event_look` without "before today"; product documents | 3 | done |

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
- [x] Story 3: open items from the start of the season (2026-11-11, `changes/done/2026-11-11-open-items-from-the-start-of-the-season/`, decision 0036)
- [x] Story 4: only today and later on the start page (2026-11-11, `changes/done/2026-11-11-only-today-and-later/`)

## Deviations
None. Design reads the spec as written.

## Review findings
Acceptance review of the epic, 2026-11-11, by `/review accept` (fresh agent). Input: the mode
and the change folder only. Read: `functional.md`, `idea.md`, `change.md`, this file (the
objective first, **Deviations** after forming a view), `docs/language.md`, the product way of
working; the result as the diff `aa3b628..HEAD` (v0.11.2 to v0.15.0, the four stories, code only).
Not read: `docs/changes/done/`. Run: `cargo test -p team --test duty_checks --test start --test
checked --test report --test not_played` (58 passed) and `npx playwright test start checked duties
trainings not-played game-attendance` (169 passed). The app was not opened by hand.

**Verdict: accepted with notes.** The four stories do what the coordinator meant: every mark is
one row of one panel, a duty game has "Checked" for its date and its kind, every past event of the
season that is not checked asks once, and the start page lists only today and later. A1 is one
small fix of the front end (a line in `backlog.md`, not a spec correction); A2 to A5 are notes.

| Criterion | Matches the intent? | Checked by |
|---|---|---|
| S1 one panel, ○ / ✓, one button, "Undo", no repeated words | yes | reading `AdminPanel.svelte`, `MarkRow.svelte`, both pages; e2e `checked`, `trainings` |
| S1 the lines below "Not checked" (attendance; count and drivers on an away game; the message after) | yes | reading `+page.svelte` of a game, `driversLine`; e2e `checked.e2e.ts:104` |
| S1 "Schedule entered", "The drives count", the rare marks, "Mark answers as checked" as a note | yes | reading the pages, `GameAttendance.svelte` |
| S1 orange only for the next step | yes, except A1 | reading `next=` of every `MarkRow`; e2e `not-played.e2e.ts:101` |
| S1 the warning of a team game without answers | yes | reading `noAnswerWarning`; e2e `checked.e2e.ts:75` |
| S2 "Checked" of a duty game: from the start time, the places on the line, for its date, only for its kind, not when not played, only the attention | yes | reading `check_kind`, `game_checked`, `duty_check`, `same_kind`; run `duty_checks.rs` |
| S2 "· Checked" on "Season" | yes, see A3 | reading `StartEventRow.svelte`; e2e `duties.e2e.ts:720` |
| S3 duty games not checked, games not checked from the start of the season, the seasons of the open items, the list of more games, the report's line | yes | reading `games_not_checked`, `duty_games_not_checked`, `api/start.rs`, `api/report.rs`; search `before_release\|game_check_start\|games_from` in `crates`, `web/src` (none left but migration 0018 and comments); run `start.rs`, `report.rs`, `duty_checks.rs` |
| S4 no past event in the list; today and later as now | yes | reading `routes/+page.svelte`, `event_look`; run `duty_checks.rs` (`the_list_of_the_start_page_has_no_past_event`) |

### Findings
- **A1 – Orange "Mark as checked" on a not played team game. Concerning deviation (small).** On
  a team game marked "Not played", from its start time, the panel shows "Not played ✓" and also
  "Not checked ○" with an orange "Mark as checked" (`checkedRow = !!page.attendance &&
  page.started`, `next={!checked}` in `web/src/routes/game/[season]/[id]/+page.svelte`; the e2e
  test `not-played.e2e.ts:101` expects the row `mark-checked` there). But such a game is never an
  open item (`games_not_checked` drops `not_played`), and its attendance does not count, so
  nothing needs the coordinator: against "orange only for the next step the coordinator must do;
  a panel without orange means the administration of the event is done" (story 1). The twin case
  is built the other way: a not played duty game has no "Checked" row at all (story 2). Story 1
  allows both rows in one panel ("Not played" and "Checked"), so the row may stay; only the orange
  is wrong. Suggest: `next={!checked && !page.not_played}`, and the e2e test asks `data-next`
  `false` there. Checked by reading the page and the test; not run in a browser by hand.
  - 2026-11-11: fixed: `next={!checked && !page.not_played}`; `not-played.e2e.ts` now asks `data-next` `false` on `mark-checked`.
- **A2 – Living documents with the old words. Neutral drift (docs).** The button words changed,
  the documents still use some old ones: "Undo check" in
  `docs/product/trainings-and-attendance.md` (lines 49, 50, 77) and
  `docs/product/game-attendance.md:65`; the refusal `not_a_team_game` for `PUT .../checked` in
  `game-attendance.md:65` (the code answers `not_checkable`, `duty_game_not_played`,
  `game_changed`); "The drivers drove" as the name of the mark in `docs/product/report.md:20`,
  `docs/product/duties.md:34` and `docs/architecture.md:63`; "Answers checked" in
  `docs/architecture.md:167`. Checked by a search for `Undo check|"Answers checked"|drivers
  drove|not_a_team_game` in `docs/product`, `docs/architecture.md` and the code.
  - 2026-11-11: fixed in the Close: "Undo", "The drives count", "Mark answers as checked", and the three refusals in `game-attendance.md`.
- **A3 – "Checked" of a duty game on "Season" without the dot. Faithful adaptation.** The spec
  says "· Checked", as a team game. A row of another club team has no hall line to put it after,
  so "Checked" stands on its own line below the teams, as "Not played" did already
  (`StartEventRow.svelte`). Checked by reading.
  - 2026-11-11: not fixed: accepted as it is.
- **A4 – The whole sentence on "Attendance" went. Neutral drift.** Story 3 asked that "Attendance"
  no longer says "The start page asks for games from Wed 4 Nov."; the build removed the whole
  line, also its first half "Games count once they are checked." (`gamesFromText`, removed from
  `web/src/lib/trainings.ts`). The rule still holds, and the open item now finds every game, so the
  hint is less needed; but it was the one place on "Attendance" that said why a played game is
  not in the totals yet. The coordinator may want it back. Checked by the diff of
  `routes/attendance/+page.svelte`.
  - 2026-11-11: fixed: "Attendance" shows "Games count once they are checked." again above the totals, without the date; `checked.e2e.ts` asks for it; `trainings-and-attendance.md` describes it.
- **A5 – A team game with places of its own team. Note, as intended.** Spec review S2 asked which
  check covers the table or referee places of the team at its own game ("This team plays this
  game."); the answers of Discover left this open. The build answers it: such a game is a team game
  (`check_kind` gives `Team` for `ScheduleReason::TeamGame`), so one "Mark as checked" of its
  attendance covers it, and it never gets a duty check or a duty open item. It fits the club rule
  in the word list (a team never has those places at its own game), but the line below "Not
  checked" shows the drivers only, not those places. Rare; known limit. Checked by reading
  `check_kind` and the page.
  - 2026-11-11: not fixed: a note, accepted by the coordinator (rare, and the club rule says the team never has those places).

**Deviations** of Design ("None") read after the view above: none of A1 to A5 is listed there; the
deviations of the stories are in their own folders, not read (`docs/changes/done/`).
