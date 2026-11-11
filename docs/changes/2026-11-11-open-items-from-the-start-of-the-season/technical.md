# Technical spec: Open items from the start of the season

**Status**: done
**Size**: story
**Change**: docs/changes/2026-11-11-open-items-from-the-start-of-the-season/
**Date**: 2026-11-11

## Objective
Make every past event of the season that is not checked an open item, also a duty game and the
team games before 4 November 2026, so the coordinator never has to search for a gap.

## Requirements summary
From the epic's `functional.md`, story 3 (`docs/changes/2026-11-11-checked-for-every-event/`):
- New open item after "games not checked": "Duty game Sat 31 Oct not checked" for one, "12 duty
  games not checked" for more: the duty games before today that are not checked, not "not played"
  and still in the game list.
- "Games not checked" counts all team games of the season before today (no date 4 November 2026),
  with the same other rules.
- Both cover the seasons of the open items (the current season; in July and August also the
  season before).
- For one game a tap opens it; for more, a tap opens the list of those games below the item
  ("Sat 31 Oct · 14:30 Blue Drakes X14 1 – BV Alphia X14 1"), each a tap to its page, as "schedule
  not entered"; this replaces "to the oldest of them".
- After "Mark as checked" (or "Not played") the game leaves the item; an item without games is not
  shown.
- The list of a game before 4 November is no longer closed behind "Show the list"; "Attendance" no
  longer says "The start page asks for games from Wed 4 Nov.".
- The report's line "N games not checked" follows the same rule; duty games are not in it.

## Current state
- `team_core::games_not_checked(season, games, checks, from_date, today)` and
  `OpenItem::GamesNotChecked { first, count }` (`output/start_page.rs`); `open_items` puts it last.
- `game_attendance::check_start` (`crates/server/src/game_attendance.rs`) reads
  `game_check_start` (migrations 0015, 0016); used by `api/start.rs` (the open item),
  `api/report.rs` (the line), `api/attendance.rs` (`games_from`) and `api/games.rs`
  (`before_release`).
- `OpenItemView::GamesNotChecked { first: NotCheckedGameView, count }` (`api/start.rs`);
  `GameItemView` (season, id, date, start, home, away) for "schedule not entered".
- Story 2: `team_core::check_kind`, `game_checked`, `StoredDuties.checked_for`.
- Front end: `src/lib/start.ts` (`openItemText`, `openItemLink`, `gameItemText`),
  `routes/+page.svelte` (an item of more games opens its list), `routes/game/[season]/[id]/
  +page.svelte` (`before_release` closes the list), `routes/attendance/+page.svelte` and
  `src/lib/trainings.ts` (`gamesFromText`).
- Tests that set the release day: `web/e2e/global-setup.ts`, `crates/server/tests/checked.rs`,
  `attendance_games.rs`, `start.rs`, `report.rs`.

## Steps

### Step 1: The core rules
- **Goal**: both lists of games not checked, and the items that carry them.
- **Changes**: `games_not_checked(season, games, checks, today)` without `from_date`; new
  `duty_games_not_checked(season, games, places: &PlacesOf, stored: &BTreeMap<i64, StoredDuties>,
  today) -> Vec<NotCheckedGame>`: before today, in the game list, not "not played",
  `check_kind == Duty`, not `game_checked`; sorted by date and id. `OpenItem::GamesNotChecked
  { games }` and new `OpenItem::DutyGamesNotChecked { games }` (after it); `open_items` takes both
  lists.
- **Tests**: a team game of September counts; duty games: not checked (in), checked (out), checked
  for another date (in), not played (out), gone (out), coming (out), without places of the team
  (out); the order of the open items.

### Step 2: The server
- **Goal**: the start page, the report, "Attendance" and the page of a game without the release
  day.
- **Changes**: `api/start.rs`: both lists for each season of the open items (the places of the
  season before in July and August); `OpenItemView::GamesNotChecked { games: Vec<GameItemView> }`
  and `DutyGamesNotChecked { games }`, the details from one map over all games read.
  `api/report.rs`: `games_not_checked` without the release day. `api/attendance.rs`: `games_from`
  goes. `api/games.rs`: `before_release` goes. `game_attendance::check_start` goes. Migration
  `0018_drop_game_check_start.sql`. `just api-types`, `just prepare`.
- **Tests**: `tests/start.rs` (both items with their games; the season before in July),
  `tests/report.rs` (the line counts an old game); the lines that set `game_check_start` go from
  `checked.rs`, `attendance_games.rs`, `start.rs`, `report.rs`.

### Step 3: The front end
- **Goal**: the two items with their lists; the old closed list and the sentence go.
- **Changes**: `src/lib/start.ts`: the words of both items (one game: "Game Sat 31 Oct not
  checked", "Duty game Sat 31 Oct not checked"; more: "3 games not checked", "12 duty games not
  checked"); one game links to its page, more open their list, as "schedule not entered". The page
  of a game: the closed list only on a not played or checked game. "Attendance": the sentence and
  `gamesFromText` go.
- **Tests**: with step 4.

### Step 4: The browser tests
- **Goal**: the tests follow the test data without the release day.
- **Changes**: `web/e2e/global-setup.ts` no longer sets the day; the tests that count the open
  items follow (for example `choose.e2e.ts`); the tests of the closed list of old games and of
  the sentence go; new: an item of more games opens its list, and a tap opens a game.
- **Tests**: the full browser chain.

### Step 5: The look and the checks
- **Goal**: readable; all checks pass.
- **Changes**: a screenshot of the start page with both items and one list opened, light and dark.
- **Tests**: `just check`.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| No start date: every past team game of the season that is not checked is an open item; `game_check_start` dropped | The coordinator entered the older games and wants to find the gaps; nothing to keep up to date; the same rule as trainings | yes, at Close (partly replaces 0032) |
| The items carry their games | The list below an item of more games, as "schedule not entered" | part of that record |
| Duty games not checked as their own item, not in the report's line | A duty game has no attendance; the report counts attendance | part of that record |

## Not in scope
- The list of events of the start page (story 4); checking several games at once; the totals.

## Test plan
- **Existing tests that must still pass**: all, with the changes above.
- **New tests**: per step.

## Compatibility and performance
- One forward-only migration (a table dropped). `GET /api/start`, `GET /api/games/...` and
  `GET /api/attendance` change shape; only the front end reads them, and both ship together.
- In July and August the start page reads the places of the season before once more.
- The first time after the release, the open items can be long (one row for each kind).

## Progress
- [x] Step 1: the core rules (2026-11-11)
- [x] Step 2: the server (2026-11-11)
- [x] Step 3: the front end (2026-11-11)
- [x] Step 4: the browser tests (2026-11-11)
- [x] Step 5: the look and the checks (2026-11-11). Screenshots of the start page at 390 px in
  light and dark mode with "5 games not checked" (its list of 5 games from 12 September opened)
  and "2 duty games not checked": readable, in the order of the spec. The last duty games of Sat 3
  Oct still show above the coming events: story 4 removes them.
- [x] All checks pass

## Deviations
- 2026-11-11, step 1: `open_items` takes the two lists in one struct `NotCheckedGames { games,
  duty_games }`: with a separate argument it had 8 arguments, more than clippy allows (7).
- 2026-11-11, step 3: "Attendance" lost the whole line "Games count once they are checked. The
  start page asks for games from Wed 4 Nov.", not only its second sentence: it was one line from
  one field (`games_from`), shown only in the season of the release (build review R5).

## Review findings

### Accept review (2026-11-11)
`/review accept` by a fresh agent. Read: `change.md`, this spec, the epic's `functional.md`
(story 3, its spec review and answers) and `change.md`, the diff `main..open-items-season`
(commit e03e412; working tree clean). Not read: the epic's `idea.md` and `technical.md`. Run:
`cargo test -p team_core --lib not_checked` (6 passed), `cargo test -p team --test duty_checks
--test start --test report --test checked --test attendance_games` (all passed). Not run: the app
itself (it writes to the local database) and the browser tests; the screenshots of step 5 were not
seen. The view below was formed before reading **Deviations**.

**Verdict: accepted with notes.**

- **A1 – New item "duty games not checked" (criterion 1): faithful.** `duty_games_not_checked`
  (`output/start_page.rs`) takes the past games that are in the game list, not "not played",
  `check_kind == Duty` and not `game_checked`; the words "Duty game Sat 31 Oct not checked" / "N
  duty games not checked" in `web/src/lib/start.ts`; it comes after "games not checked". Checked
  by a trace and by the tests `a_past_duty_game_that_is_not_checked_is_open`,
  `the_duty_games_not_checked_come_after_the_games_not_checked` and
  `duty_checks::a_past_duty_game_that_is_not_checked_is_an_open_item` (run).
- **A2 – "Games not checked" from the start of the season (criterion 2): faithful.** The filter is
  now `date < today`, the other rules stay; the core test includes a game of 2 October. Checked by
  a trace and a run.
- **A3 – The closed list and the sentence go (criterion 3): faithful.** Search
  `games_from|before_release|check_start|gamesFromText` in `crates/` and `web/src`: only the
  module comment of A9 is left. The page of a game closes the list only when not played or checked.
- **A4 – The report's line (criterion 4): faithful.** `api/report.rs` calls `games_not_checked`
  without the release day and never the duty list, so the numbers of the report do not change.
  Checked by a trace and `report::what_does_not_count_yet_is_the_same_as_on_the_start_page` (run).
- **A5 – The seasons of the open items (criterion 5): faithful, one test gap.** `api/start.rs`
  adds both lists for each earlier season, the duty list with `places_of_season` of that season,
  and the details from one map over the past games of all those seasons. `start.rs` tests only the
  games of the team of the season before in July (`a_new_season_without_a_team_has_trainings_only`);
  a duty game of the season before in July is not tested, although the spec's step 2 plans "the
  season before in July" for both items. Checked by a trace; the duty case not checked by a run.
- **A6 – A tap (criterion 6): faithful.** `openItemLink` gives the page of the one game, and
  `null` for more, so the unchanged list of `routes/+page.svelte` (`'games' in item`) opens with
  `gameItemText`, as "schedule not entered"; "to the oldest of them" is gone. The new browser test
  covers the list of "games not checked"; the list of duty games uses the same code and has no
  browser test. Checked by a trace; browser tests not run.
- **A7 – Leaving the item (criterion 7 and the edge case of a moved game): faithful.** After
  "Mark as checked" or "Not played" the duty game leaves the item (run, `duty_checks`); an item
  without games is not pushed (`is_empty` in `open_items`); a game moved after today fails
  `date < today`. Checked by a trace and a run.
- **A8 – `NotCheckedGames` struct (the deviation of step 1): faithful adaptation.** Only the shape
  of an argument; no effect for the coordinator.
- **A9 – Text the change made false: neutral drift, for Close.** The module comment of
  `crates/server/src/game_attendance.rs:1` still names the table `game_check_start`, which
  migration 0018 drops (code, not a living document: fix it in this branch or at Close). The living
  documents still describe the release day: `docs/product/start-page.md` lines 26 and 47,
  `game-attendance.md` lines 50 and 64 (`before_release`), `trainings-and-attendance.md` lines 20,
  26 and 77 (`games_from`), `docs/architecture.md` lines 17, 53, 64, 133 and 165; decision 0032
  needs the planned record that partly replaces it. Checked by a search for `4 Nov`,
  `game_check_start`, `check_start`, `before_release` and `games_from`.
- **A10 – Right after choosing a team, all past games of the season ask at once: neutral drift,
  intended.** `choose.e2e.ts` now expects "N games not checked" on the first start page after the
  first reading, also for games the coordinator never followed in the tool. This is the answer to
  S5 (the whole season the same, to find the gaps) and the edge case "at the first release … one
  row with its list"; it is a consequence for a team chosen in mid-season that the spec does not
  name. Checked by reading the test and the epic's answers.

### Build review (2026-11-11)
`/review build` by a fresh agent. Read: the diff `main..open-items-season` (commit e03e412, working
tree clean), this change folder, the epic's `functional.md` (story 3, spec review and answers),
`docs/architecture.md`, decisions 0032 and 0035 (as quoted in the code and docs), `CLAUDE.md`,
`README.md` (migrations, release). Run: `cargo test -p team_core` (all passed), `cargo test -p
team --test start --test checked --test duty_checks --test report --test attendance_games` (all
passed), `cargo clippy --workspace --all-targets -- -D warnings` (clean), `just api-types-check`
(no diff); `npx playwright test start checked`: see R8. Domains: `output::start_page` now uses
`contribution::duties` (`check_kind`, `game_checked`, `StoredDuties`); output may use all
(`docs/architecture.md`, decision 0027), and `crates/core/tests/domains.rs` passes. No rule rebuilt:
the new rule reuses `check_kind` and `game_checked` of story 2, so the item, "Season" and the page
of a game agree.

- **R1 – Within one date the games are in id order, not time order (low, bug against the
  spec's intent).** `games_not_checked` and the new `duty_games_not_checked` sort by `(date, id)`
  (`crates/core/src/output/start_page.rs`, and again `oldest_first` in `open_items`), while the
  doc comments say "In time order" and the spec asks for the list "as for 'schedule not entered'",
  which is in time order (`schedule_not_entered` and `open_places_games` walk `events.coming`,
  sorted by start time within a date through `within_date` in `events/schedule.rs`). On a
  Saturday with several duty games (the usual case for table and referee places) the rows "Sat 31
  Oct · 14:30 …", "Sat 31 Oct · 10:00 …" can come out of time order. For "games not checked" this
  was harmless while the item showed only the oldest; now that the list is shown it is visible.
  Checked by a trace (the sort keys); not reproduced by a run. Fix: sort by `(date, start_time
  none last, start_time, id)` as `within_date`, or keep the order of `schedule.past` reversed.
- **R2 – The details of the games are looked up by source id alone over two seasons (low).**
  `api/start.rs` builds `past: BTreeMap<i64, Game>` from the past games of the current season and
  (in July and August) the season before, keyed by `source_id` only, and `OpenItemView::of` looks
  each `NotCheckedGame` up with `past.get(&game.id)` while ignoring `game.season`. The key of a
  game in this project is `(club, season, source_id)` (`games::GameKey`, the primary keys from
  migration 0003 on), so the model allows the same id in two seasons. Then one row shows the date
  and teams of the other season's game (the link, built from `game.season` and `game.id`, stays
  right), and the `{#each item.games as game (game.id)}` of `routes/+page.svelte` gets a duplicate
  key, which Svelte refuses at run time. Whether the federation ever repeats an id across seasons
  is not checked. Fix: key the map by `(Season, i64)`; key the Svelte list by `game.season +
  game.id`. Checked by a trace.
- **R3 – The duty games of the season before are not tested (test gap; same as A5).** Step 2
  plans `tests/start.rs` "both items with their games; the season before in July". The July test
  (`a_new_season_without_a_team_has_trainings_only`) only shows `["games_not_checked"]`; the code
  path of `earlier_places = places_of_season(&earlier, now)` with `earlier.duties.stored` has no
  test. By a trace it is right (the same helpers as the current season). Also `the_duty_games_not_
  checked_come_after_the_games_not_checked` passes items that are already sorted, so it cannot
  fail on the merge order of two seasons; the sort over the merged list (in `open_items`) has no
  test with games of two seasons. Checked by a search for `duty_games_not_checked` in
  `crates/server/tests` (only `duty_checks.rs`) and a trace.
- **R4 – The words of the duty item have no test (test gap; see A6).** "Duty game Sat 31 Oct not
  checked" and "N duty games not checked" (`web/src/lib/start.ts`) are in no browser test and
  there are no unit tests of `start.ts`; only the screenshot of step 5 saw "2 duty games not
  checked". The one-game form "Game … not checked" / "Duty game … not checked" (a link, not a
  list) has no browser test either. Checked by a search for `Duty game` and `duty games` in
  `web/e2e` and `web/src`.
- **R5 – "Games count once they are checked." went too (neutral, not a recorded deviation).**
  The spec only drops "The start page asks for games from Wed 4 Nov."; the build removed the whole
  line of "Attendance", also its first sentence, which told why an unchecked game is not in the
  totals. It was only shown in the season of the release, so other seasons never had it. Either
  keep the first sentence or write the removal under **Deviations**. Checked by the diff of
  `web/src/routes/attendance/+page.svelte` and `web/src/lib/trainings.ts`.
- **R6 – The migration drops a table that the running version still reads (low, informational).**
  `0018_drop_game_check_start.sql` runs in the init container of the new pod (`README.md`,
  "migration (init container)"); until the old pod stops, its `/api/start`, `/api/report`,
  `/api/attendance` and `/api/games/...` fail on the missing table. The window is short, there is
  one coordinator, and 0008 did the same (dropped columns), so this follows precedent; dropping the
  table one release later would avoid it. Checked by reading `README.md` and the migration.
- **R7 – Comments and documents the diff made false (for Close; same as A9).** Code:
  `crates/server/src/game_attendance.rs:1` still names the table `game_check_start`. Living
  documents still describe the release day: `docs/product/start-page.md` (item 8, "to the oldest
  of them", and line 47), `docs/product/game-attendance.md` (lines 50, 64: `before_release`),
  `docs/product/trainings-and-attendance.md` (lines 20, 26, 77: `games_from`),
  `docs/architecture.md` (lines 17, 53, 64, 133, 165, 170: `check_start`, "the release day",
  `global-setup.ts` sets it); decision 0032 ("The day of the release") needs the planned record.
  No instruction file (`CLAUDE.md`, `AGENTS.md`) is affected. Checked by a search for
  `check_start|game_check_start|before_release|games_from|4 Nov|release day`.
- **R8 – The claims of the progress: true as far as run.** "All checks pass": the Rust tests,
  clippy and the API types are confirmed by a run (above); `npx playwright test start checked` ran
  169 browser tests (the filter matched more files than the two), all passed in 3.6 minutes. Not
  run: `svelte-check`, the lint, `check-sqlx`, the images. The tests that changed can fail: the
  `checked.rs` test asserts the count 5 and the presence and absence of the game after each mark,
  and the duty test asserts an empty item before the place is given. The weak spots are R3 and R4.

**Summary**: no rule in the wrong place, no forbidden use between domains, no rule rebuilt. Two
small bugs to fix before Close (R1 order within a date, R2 key without the season), two test gaps
(R3, R4), one unrecorded removal (R5), and the text for Close (R7).

**Answers (2026-11-11, Implement).**
- R1: fixed. Both lists are in time order within a date (date, start time with an unknown time
  last, id; `in_time_order`), and `open_items` sorts by date only, stably, so that order stays over
  the seasons. New core test `the_games_not_checked_of_one_date_are_in_time_order`.
- R2: fixed. The details of a game not checked are looked up by season and id (the key of a game).
- R3 / A5: fixed. New server test
  `in_july_the_duty_games_of_the_season_before_are_an_open_item` (`tests/duty_checks.rs`), also
  that from September the season before is no longer covered.
- R4 / A5: fixed. New browser test `the duty games not checked have their own words`
  (`start.e2e.ts`): one game with its link, more games with their list.
- R5: not changed; written under Deviations.
- R6: not changed (a note): during a release the old version can fail for a moment after the
  migration drops the table, as with migration 0008.
- R7 / A9: the comment of `crates/server/src/game_attendance.rs` is fixed; the living documents
  and the decision record follow at Close.
- R8, A1–A4, A6–A8, A10: as meant; no change.
