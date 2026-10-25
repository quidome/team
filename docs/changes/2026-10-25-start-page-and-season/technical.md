# Technical spec: Start page and season

**Status**: approved
**Size**: epic
**Change**: [2026-10-25-start-page-and-season](./)
**Date**: 2026-10-25

## Objective
Give each job of the coordinator its own place: a page "Season" with all events of a season
(labels, week headings), a start page with "Open items", "Last" and "Next", and a check on each
past training, so a training that was not looked at stays in view.

## Requirements summary
From `functional.md` (approved 2026-10-25; spec review notes 1–7 answered):

- **A – The page "Season"** (`/season?season=`): all games of the team, all possible duty games and all training days and trainings of a season. A heading per ISO week ("Week 44 · 26 Oct – 1 Nov"), then per date. Labels: What ("Games of the team", "Possible duty games", "Trainings") and When ("Coming", "Past"); labels of different groups narrow, labels of one group add, a group with none on shows all. Opens with "Coming" at each visit (labels are not kept); the labels that are on are always visible. Coming nearest first; with "Past" on, a second part "Past", newest first (today's week can have a heading in both parts). Within a date: known start times in time order, then unknown. Game rows as the schedule now (crossed out while a gone game's change shows); training rows as "Attendance" now, plus "Checked". A season choice (current and earlier seasons with games or trainings); "This season cannot be read." for `?season=abc`. The "Game list" card on the current season. From "Attendance": "Add training" (current season only), "Cancel" on a day not opened, "Show cancelled" (kept on each device). A new season without a team: trainings and "Choose the team to see its games.". Earlier season without kept games or choice: trainings only. "Back" on a training returns where it came from; after "Delete training" to "Season". Empty texts: "No games or trainings in 2026-2027 yet.", "No coming trainings.", and so on.
- **C – Checked trainings**: "Checked" on a training of today or earlier; after the tap "Checked" with "Undo check"; both save at once ("Not saved. Try again." on failure). A training before today, not checked, not cancelled, is an open item. A check stays through marks, "Change" (also to a later date: then it shows "Checked" with "Undo check" and is no open item when its date passes), "Cancel" and "Undo cancel". "Delete training" removes it. At the release every training before today counts as checked. "Season" shows "Checked" on the row. Earlier seasons can be checked but have no open items. Two screens: a check and a mark are both kept; check and undo: the last wins.
- **B – The start page**: the team header; "Open items" in this order: held-back reading, "may be out of date since", "N changes not seen", training days not opened ("Training day Thu 22 Oct not opened" / "3 training days not opened", to the oldest), trainings not checked ("Training Thu 22 Oct not checked" / "2 trainings not checked", to the oldest); "Nothing open."; "Last": the last training before today that is not cancelled, also when checked; "Next": the games of the team and the training days and trainings from today until the Sunday of next week (no possible duty games), else the first coming event, else "No coming events."; the links "Season · Attendance · Team · Players · Settings". Open items cover the current season and in July and August also the season before, dates before today only; cancelled never. No club chosen: only the choose step. A new season without a team: the choose step, "Open items", "Last", "Next" with trainings only. Gone: the schedule, the card of the next training, the "Game list" card, the line of changes, the view "Last season". A failure: "The start page could not be loaded. Load the page again."

## Current state
- **Games**: core `schedule::{in_schedule, schedule}` (team games and club home games of other teams, coming and past, gone games only while `open` has their change); `GET /api/schedule` (`api/schedule.rs`) for the current season, or last season while a team is needed (`schedule_target`); `ScheduleGame` view; changes from `api::changes::list`.
- **Trainings**: core `training::{SeasonRow, SeasonList, season_list, DayStatus}`; `attendance::season_overview` counts the training days not opened from `season_list` (`not_opened`, `first_not_opened`); table `trainings` (`date` unique, `cancelled_at`); `GET /api/trainings?season=` (the list, `TrainingRow`), `GET /api/trainings/next` (the card), `GET /api/trainings/{date}` (`TrainingDay` with `TrainingView`), open, add, change, cancel, uncancel, delete, mark, there; `trainings::seasons` (seasons of roles, weekly trainings and trainings).
- **Readings**: `GET /api/readings/latest` works out `held_back` (`readings::newest_not_failed` when held back) and `out_of_date_since` (`nightly::out_of_date`, only with a team) inside the handler.
- **Choice**: `followed_teams::{for_season, newest_before}`; `choice_state`.
- **Front end**: start page `web/src/routes/+page.svelte` / `+page.ts` (links, training card, line of changes, "Game list" card with `ReadGameList.svelte`, `Schedule.svelte` with `FilterToggle.svelte`, view "Last season"); `/attendance` (season choice, line not opened, totals, list with "Add training", "Cancel", "Show cancelled"); `/training/[date]` ("Back" to `/attendance?season=`). No front-end unit tests: behaviour is tested with Playwright.
- **Patterns**: rules in the core; lists and totals worked out at each request (decision 0020); training days never stored (0018); routes under the protected `/api` router with `utoipa`, types generated; the lock and race test patterns in `docs/architecture.md`; logs with dates and ids only; Dutch dates through `today_in_netherlands`.

## Architecture (shared by all stories)
- **Core `training.rs`**: `Training` gets `checked: bool`. New `not_opened(list) -> Vec<NaiveDate>` (past rows `NotOpened`, oldest first), used by `season_overview` and the open items; `not_checked(list) -> Vec<NaiveDate>` (past rows with a training that is neither cancelled nor checked, oldest first); `check_checking(training: Option<&Training>, today) -> Result<(), TrainingError>` with new variants `NotOpened` and `NotYet` (date after today and not checked).
- **Core `season_view.rs`** (new): `EventKind<'a> { Game(&ScheduledGame), Training(&SeasonRow) }`, `Event { date, start: Option<NaiveTime>, week: IsoWeek, kind }`, `SeasonEvents { coming, past }`, `season_events(games: &Schedule, trainings: &SeasonList) -> SeasonEvents`. Coming nearest date first, past newest date first; within a date: known start times in time order, then unknown; at the same time a training before a game; games by source id.
- **Core `start_page.rs`** (new): `ReadingState { held_back: bool, out_of_date_since: Option<NaiveDate> }`; `OpenItem { ReadingHeldBack, OutOfDate { since }, ChangesNotSeen { count }, NotOpened { first, count }, NotChecked { first, count } }`; `open_item_seasons(today) -> Vec<Season>` (current; in July and August also the one before); `open_items(reading, unseen, lists) -> Vec<OpenItem>` in the order of the spec; `last_training(lists) -> Option<&SeasonRow>` (newest past row with a training, not cancelled); `next_events(events, today) -> Vec<&Event>` (team games and training rows up to the Sunday of next week; else the first coming one).
- **Migration `0010_trainings_checked.sql`**: `ALTER TABLE trainings ADD COLUMN checked_at timestamptz;` and `UPDATE trainings SET checked_at = now() WHERE date < (now() AT TIME ZONE 'Europe/Amsterdam')::date;`. sqlx runs it once, in one transaction; the old app ignores the column.
- **Server**:
  - `GET /api/season?season=` (`api/season.rs`): `SeasonAnswer { season, current_season, today, seasons, games: shown | team_needed | none, club, team, coming: [EventView], past: [EventView] }`, `EventView { date, week, week_first_day, week_last_day, start_time, game?: ScheduleGame, training?: TrainingRow }`. Games from the choice of that season (`followed_teams::for_season`); in the current season gone games show while their change is open (`api::changes::list`), in an earlier season gone games are hidden. Seasons: `trainings::seasons` ∪ `followed_teams::seasons` (new) ∪ the current season. 400 `invalid_season`.
  - `PUT` / `DELETE /api/trainings/{date}/checked`: one `UPDATE trainings SET checked_at = … WHERE date = $1 RETURNING …`, after `check_checking` on the stored training; answers the training page `TrainingDay`, as the other actions on a training (changed in the design of story C, 2026-10-25); 400 `not_opened`, `not_yet`, `invalid_date`. `TrainingRow` and `TrainingView` get `checked`.
  - `GET /api/start` (`api/start.rs`): `StartAnswer { open_items: [OpenItemView], last?: TrainingRow, next: [EventView] }`; `OpenItemView` tagged by `kind`: `reading_held_back`, `out_of_date { since }`, `changes { count }`, `not_opened { first, count }`, `not_checked { first, count }`. All or nothing.
  - `readings::state(db, clock, target) -> ReadingState`: the rule of `held_back` and `out_of_date_since`, called by `GET /api/readings/latest` and `GET /api/start`.
  - Removed: `GET /api/trainings?season=` (A), `GET /api/schedule` and `GET /api/trainings/next` (B).
- **Front end**: `/season` (`+page.ts` loads `/api/season` and, for the current season, `/api/readings/latest`), `Labels.svelte` (several on at once, groups), `SeasonEvents.svelte` (week and date headings, the two parts); "Attendance" without its list; `/training/[date]` with "Checked" / "Undo check" and "Back" by `?from=start`, else `/season?season=`; the start page rewritten on `/api/start`.

| Option | For | Against |
|--------|-----|---------|
| One route `/api/season` with the events in order (chosen) | The order of mixed games and trainings is a rule in the core, with tests; one request | A new answer; `/api/trainings?season=` goes |
| The front end merges `/api/schedule` and `/api/trainings` | No new route | The order rule in TypeScript; two requests; the schedule route needs a season |
| The ISO week from the server (chosen) | No date arithmetic in the front end | One field more |
| One route `/api/start` (chosen) | One request on the phone; the open items from one core function | About 8 queries; fine for one coordinator |
| The start page calls 4 routes | No new route | 4 requests; the rules of open items split over the front end |
| `checked_at timestamptz` (chosen) | Keeps the moment; `NULL` = not checked | – |
| `checked boolean` | Simple | Loses the moment for no gain |

## Stories
| # | Story | Type | Scope | Depends on | Closure | Release | Status |
|---|-------|------|-------|------------|---------|---------|--------|
| A | [The page "Season"](../done/2026-10-25-season-page/) | feature | Core `season_events` (order within a date first, risk 3) and `not_opened` out of `season_overview`. `followed_teams::seasons`. `GET /api/season`. Page `/season` with `Labels`, `SeasonEvents`, the season choice, the "Game list" card, "Add training", "Cancel", "Show cancelled". "Attendance" without its list; "Back" on a training to "Season"; `GET /api/trainings?season=` removed; the link "Season" first on the start page (the start page keeps its schedule and cards until B); `?from=start` on the card of the next training (moved here from B, 2026-10-25). | – | review build + accept | 1 | done (2026-10-25) |
| C | [Checked trainings](../2026-10-25-checked-trainings/) | feature | Migration `0010_trainings_checked` first (risk 2). `Training.checked`, `check_checking`, `not_checked`. `PUT` / `DELETE /api/trainings/{date}/checked`. "Checked" / "Undo check" on the training page; "Checked" on the row on "Season". | A | review build | 1 with A, or 2 | implement |
| B | The start page as a landing page | feature | Core `start_page`. `readings::state`. `GET /api/start`. The start page rewritten (open items, "Last", "Next", links, the choose step of a new season); the schedule, cards, line of changes and view "Last season" removed; `GET /api/schedule` and `GET /api/trainings/next` removed; `?from=start` on the training links of the new start page (the parameter itself comes in A). Measure the answer time of `/api/start` (risk 4). | A, C | review build + accept | 2 or 3 | not started |

Within each story: the core with unit tests, then the route with its tests, then the page with its
browser tests; each step can be merged alone. Each story gets its own change folder (type
`feature`, stage `design`) and its own `/design`, which writes its steps and exact tests from this
spec and `functional.md`.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| Each open item is worked out at each request with the same core function as its source (training days not opened as on "Attendance", the reading state as on the "Game list" card, the changes as the list of changes); only the check of a training is stored | One number everywhere; nothing counted on the side (as decision 0020) | Yes, 0022 at the close of the epic |
| "Season" and the start page each get one route; the core puts the events in order and gives the ISO week | Rules in the core with tests; one request on the phone | In 0022 |
| The games of a season come from the choice of that season | One rule for this and earlier seasons; a club no longer chosen stays out of view | In 0022 |
| `trainings.checked_at`, set for every past training at the release by the migration | "Checked" is new; old trainings must not become open items | In 0022 |
| "Last" uses the same seasons as the open items | In July the last training of June stays one tap away (coordinator, 2026-10-25) | No |

## Not in scope
- Duties (their open items, the label group "Duties", the page of a game, the last game); attendance at games; open items of seasons further back; labels kept on a device; a list of past events on the start page; notifications; a tap on a game; the overview for the parents.

## Test plan
- **Existing tests that change or must still pass**:
  - `crates/server/tests/schedule.rs`: moves to `tests/season.rs` (A); the cases of last season while a team is needed become `?season=2025-2026` (B).
  - `tests/trainings.rs`: the list cases go to `/api/season` (A); open, mark, cancel, change, delete must still pass.
  - `tests/attendance.rs` must pass unchanged (proves that `not_opened` moved without a change); `tests/readings.rs` must pass after `readings::state`.
  - `tests/player_logs.rs` gets the new routes.
  - Core tests of `season_list`, `season_overview`, `schedule` must pass.
  - `web/e2e/choose.e2e.ts`: the "Schedule" block moves to `season.e2e.ts` (A) and goes from the start page (B); the "Game list" card tests click it on "Season".
  - `web/e2e/trainings.e2e.ts`: list, "Cancel", "Show cancelled", "Add training" and "Back" go to "Season" (A); the card tests become "Next" (B).
  - `logged-in.e2e.ts`, `changes.e2e.ts`: the line of changes becomes an open item; screenshots again (B).
- **New tests**:
  - A: `season_events` (game 14:30 before training 19:45; unknown time after a training; training before a game at the same time; today is coming; week 53 for 28 Dec 2026 and 3 Jan 2027); `tests/season.rs` (401; both kinds in order with weeks; `team_needed`; `?season=2025-2026` with gone games hidden; `invalid_season`; a season with only a choice in `seasons`); `season.e2e.ts` (link, "Coming" at open, label combinations, two parts, empty texts, "Coming" again after another page, tap and "Back", "Add training", "Cancel", "Show cancelled", the "Game list" card, small phone, console, accessibility).
  - C: `check_checking` (`NotOpened`, `NotYet`, today allowed); `not_checked`; the migration (yesterday checked, today not; if running only the first 9 migrations is not easy, the statement on its own, written down); routes (PUT, DELETE, `not_opened`, `not_yet`, "Change" to next week keeps it, "Delete training" removes it); the race test: an open transaction on another connection inserting an absence does not block a check, and both are stored after the commit (must fail if the check locks the absences); browser ("Checked" and reload, failure with `page.route`, no "Checked" next week, "Checked" on the row).
  - B: `open_items` (order; one and three not opened; cancelled gives nothing; empty); `open_item_seasons` (30 June 2027, 1 July, 31 August, 1 September); `last_training` (checked included, cancelled skipped, 30 June found on 2 July, none); `next_events` (Mon 26 Oct 2026 to Sun 8 Nov; Sun 25 Oct 2026 to Sun 1 Nov; duty games left out; first coming; none); `tests/start.rs` (401; held back as `/api/readings/latest`; changes as `/api/changes`; not opened as `/api/attendance`; a new season without a team); browser (order of the parts, "Nothing open.", taps of each open item, "Last" after "Checked", screenshots, small phone, accessibility).
- **Browser tests**: `season.e2e.ts` in the Playwright project `trainings` (after the player tests; one database, in order), so the trainings it needs exist; the clock stays at 12 October 2026 (`DEV_NOW`): July and August and the clock change are tested in the core and server only.
- **Checks**: `just check`.

## Compatibility and performance
- The schedule moves from the start page to "Season"; the list of trainings from "Attendance" to "Season"; the start page is rewritten (B). The routes `GET /api/trainings?season=`, `GET /api/schedule` and `GET /api/trainings/next` are removed; only the front end calls them.
- The migration adds one column and sets it for the past trainings; the old app can run on it.
- `/api/start` does about 8 queries per load; B measures the answer time and writes it here (look again above about 200 ms locally).
- No player names in the new routes.

## Progress
- [x] Story A – the page "Season" (done 2026-10-25; folder in `changes/done/`)
- [ ] Story C – checked trainings
- [ ] Story B – the start page as a landing page
- [ ] Epic review accept

## Deviations
None. One point was fixed in `functional.md` with the coordinator instead (2026-10-25): "Last" (story B) looks in the same seasons as the open items, so in July and August it also finds the last training of the season before.

Readings of the spec that the design makes exact (no change of behaviour):
- "Back" returns to the page it came from: the training links of the start page carry `?from=start`; every other way in returns to "Season" of the training's season.
- "The labels that are on are always visible": the labels are toggles at the top of the list, in their on state.

## Review findings
