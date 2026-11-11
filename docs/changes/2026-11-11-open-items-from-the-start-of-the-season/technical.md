# Technical spec: Open items from the start of the season

**Status**: approved
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
- [ ] Step 1: the core rules
- [ ] Step 2: the server
- [ ] Step 3: the front end
- [ ] Step 4: the browser tests
- [ ] Step 5: the look and the checks
- [ ] All checks pass

## Deviations

## Review findings
