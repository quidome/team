# Technical spec: The list of changes

**Status**: done
**Size**: story
**Change**: docs/changes/2026-10-13-the-list-of-changes
**Date**: 2026-10-13

## Objective
Keep each change to a coming game in the schedule until the coordinator has seen it, so the
coordinator knows when a drive, a jury or a referee duty may need action.

## Requirements summary
Story 4 of the epic [Games from the game list](../2026-10-10-games-from-game-list/technical.md),
from functional story 4 ("See what changed") and the part of functional story 1 about a change of
club or team. The coordinator's view (2026-10-13): this is a list of things that may need action.
When a game of the team changes, or a game where players may have a duty, the coordinator must
know, so the players can still do their drive or duty. Changes to past games need no action.

1. The tool keeps a change for each game in the schedule (all games of the team, and the home games of other club teams, "possible duty games"), for these kinds:
   - a new game;
   - moved: new date, new start time, or new place (old and new value shown);
   - the start time became known;
   - home and away swapped (old and new home team shown);
   - a team of the game changed (old and new team shown);
   - no longer in the game list;
   - came back (a game that left the game list while in the schedule, and is in it again).
2. A change that moves a game into or out of the schedule says why: "now a home game of another club team", "now a game of the team", "no longer a home game of another club team", "no longer a game of the team". Also for a game that left the game list while outside the schedule and comes back into it (coordinator, 2026-10-13, A3 of story 3a).
3. One change for each game: from what the coordinator last saw to what is true now. A game that changes twice shows the first old value and the newest value. A game that is again as last seen shows no change (coordinator, 2026-10-13).
4. Only games of today or later: a change is kept and counted only when the game's date, as last seen or now, is today (Dutch date) or later (coordinator, 2026-10-13; see Deviations).
5. Each change stays until the coordinator marks it as seen. One change, or all changes on the screen at once.
6. 5 or more new possible duty games show as one summary line ("18 new games") that opens (coordinator, 2026-10-13). New games of the team always show one by one, so they stand out (spec review of the deviations, 2026-10-13).
7. Changes to games of the team look clearly different from changes to other club teams' games (the same team look as the schedule).
8. The quick filter of the schedule (all, games of the team, possible duty games) also applies to the list.
9. The first reading after a choice records no changes (already true for the counts).
10. A change of club or team removes the unseen changes of the old choice, and the new choice starts with no changes.
11. The start page shows one short line with the number of changes not seen yet, as a link to the list; nothing when all are seen. The next games stay in the first screen on a phone (390 × 800; A2 of story 3b).
12. A game that is no longer in the game list shows crossed out in the schedule until its change is seen; after that the schedule hides it.
13. Only the coordinator can see and use it (decision 0007).

Decisions that apply: 0001 (rules in a core without I/O), 0002 (static Svelte front end,
generated API types), 0004 (forward-only migrations, compile-time checked queries), 0005 (one
reading at a time by an advisory lock), 0007 (every `/api` route needs a session), 0012 (the
choice for each season), 0014 (seasons as start years; Dutch local dates and times), 0015
(readings are recorded results; first reading).

## Current state
- Core (`crates/core/src/comparison.rs`): `StoredGame { game, in_list }`; `Change` (`New`, `Gone`, `CameBack`, `IntoSchedule(ScheduleReason)`, `OutOfSchedule`, `Changed(Vec<Difference>)`); `Difference` (`Date`, `StartTime`, `StartTimeKnown`, `Place`, `HomeAndAway`, `Team`); `compare(stored, read, club, team: Option<TeamNumber>) -> Vec<GameChange>` with `GameChange { source_id, change, before, after }`; `count` (the result line; a `Changed` counts by its first difference: moved, start time known, swapped, team changed); `no_longer_in_list`; `same_number_pairs`. Today `compare` gives `CameBack` for every stored game with `in_list = false` that is read again and is in the schedule, also when it was outside the schedule when it left (A3).
- Core (`schedule.rs`): `in_schedule(game, club, team) -> Option<ScheduleReason>` (`TeamGame`, `DutyGame`); `schedule(stored, club, team, today) -> Schedule { coming, past }` keeps games with `in_list = false` (crossed out). `guard.rs`: `is_first_reading`.
- Server: `reading::read` (decision 0015) reads the source, then in one transaction takes `pg_advisory_xact_lock(READING_LOCK)`, applies the guard, calls `compare` and `count` (not on a first reading), and writes games (`games::write`) and the reading (`readings::insert`). The changes are dropped after counting. `api/selection.rs::save` calls `followed_teams::save(&pool, …)` (an upsert of the season's row) with no transaction and no lock. `api/schedule.rs` builds the schedule from `schedule_target`, `games::stored` and `team_core::schedule`.
- Tables (`migrations/0003_games.sql`): `places`, `games` (key `club_number, season_year, source_id`; `in_list`), `readings` (with `counts` as JSON). No table for changes.
- Front end: the start page (`web/src/routes/+page.svelte`) with the team header, the "Game list" card (`ReadGameList.svelte`, calls `onRead` after a good reading, which loads `GET /api/schedule` again), and `Schedule.svelte` (filter in `web/src/lib/schedule.ts`: `filters`, `filterGames`, `noGamesLine`, `formatDay`; team look `border-l-team bg-team-soft` and the badge "Team"; gone look: crossed out with the badge "no longer in the game list"). The words for kinds of change are in `web/src/lib/reading.ts` (`kinds`).
- Tests: core unit tests for every kind in `comparison.rs`; router tests of readings with the harness `crates/server/tests/common/game_list.rs` (Blue Drakes M16 1 chosen; the fake source's answer is set by the test; `read_good`; the test clock); `tests/schedule.rs`; browser tests in `web/e2e/` with a fake source that has one fixed answer, so other states come from replaced API answers (`page.route`).

## Steps

### Step 1: Core rules
- **Goal**: the rule for one change, the update of the open changes after a reading, and the schedule that hides seen gone games; all without I/O.
- **Changes** (`crates/core/src/comparison.rs`, exported from `lib.rs`):

  ```rust
  /// The change from the game as last seen to the game now; `None`: nothing to show.
  /// `last_seen: None` = the game was not in the tool before. Only games of `today` or later
  /// (as last seen or now) give a change.
  pub fn change_between(
      last_seen: Option<&StoredGame>,
      now: &StoredGame,
      club: ClubNumber,
      team: Option<TeamNumber>,
      today: NaiveDate,
  ) -> Option<Change>;
  ```

  A game is *visible* when it is in the game list and `in_schedule` gives a reason.

  | Last seen | Now | Change |
  |---|---|---|
  | not in the tool | visible | `New` |
  | visible | visible | `Changed(differences)`; nothing when there is no difference |
  | visible | no longer in the game list | `Gone` |
  | visible | in the game list, outside the schedule | `OutOfSchedule` |
  | no longer in the game list, in the schedule | visible | `CameBack` |
  | outside the schedule (in the game list or not) | visible | `IntoSchedule(reason now)` (A3) |
  | not visible | not visible | nothing |

  Before the table: when the date as last seen (if any) and the date now are both before `today`, nothing.

  - `compare(stored, read, club, team, today)`: gets `today`; for each game it calls `change_between` with the stored game as last seen and the game after the reading as now (a read game is `in_list: true`; a stored game not read is the same game with `in_list: false`). So the result line and the list use one rule. `count` does not change.

  ```rust
  /// An unseen change: the game as the coordinator last saw it (`None`: a new game).
  pub struct OpenChange { pub source_id: i64, pub last_seen: Option<StoredGame> }

  pub enum ChangeUpdate {
      /// A new open change; `last_seen` is the stored game before this reading.
      Insert { source_id: i64, last_seen: Option<StoredGame> },
      /// The game changed in this reading and still differs from last seen: keep `last_seen`,
      /// raise the version.
      Touch { source_id: i64 },
      /// Nothing to show any more (as last seen again, or a past game).
      Remove { source_id: i64 },
  }

  /// For each game of the club and season: last seen = the copy in its open change, or else the
  /// stored game before this reading; now = the game after this reading.
  pub fn update_open_changes(
      open: &[OpenChange],
      stored: &[StoredGame],
      read: &[Game],
      club: ClubNumber,
      team: TeamNumber,
      today: NaiveDate,
  ) -> Vec<ChangeUpdate>;
  ```

  - `Touch` only when the game after the reading differs from the stored game before it (`StoredGame` equality). A reading that does not change a game keeps its version, so a "seen" from the screen still matches.
  - Sorted by source id.
  - `schedule(stored, club, team, today, open: &BTreeSet<i64>)`: a game with `in_list = false` is kept only when `open` has its source id; else it is left out.
- **Tests** (unit tests in `comparison.rs` and `schedule.rs`):
  - One test for each row of the table above; the existing kind tests keep passing through `compare`.
  - A3: a stored game outside the schedule with `in_list = false`, read again as a duty game, gives `IntoSchedule(DutyGame)`, not `CameBack`. The existing test `a_game_that_came_back_out_of_the_schedule_gives_nothing` still passes.
  - Only coming games: a moved game of yesterday gives nothing; a game moved from yesterday to next week gives `Changed([Date])`; a game of today gives a change; a new game of yesterday gives nothing and is not counted.
  - `update_open_changes`: a game that moves twice keeps the first old value (`Touch`, `last_seen` unchanged); a game that moves back gives `Remove`; a new game that is gone again before it is seen gives `Remove`; a game that a reading does not change gives no update; a game with an open change whose date as last seen and date now are both before today gives `Remove`; a game moved from next week to yesterday keeps its change (the date as last seen is coming).
  - `schedule`: a gone game with an open change is in the result with `in_list: false`; without one it is not.

### Step 2: Storage and the reading
- **Goal**: every reading keeps the open changes in the same transaction as the games; a change of club or team removes them.
- **Changes**:
  - `migrations/0004_changes.sql`:

    ```sql
    -- One row for each game with a change the coordinator has not seen yet. "Seen" deletes it.
    CREATE TABLE changes (
        club_number  bigint NOT NULL,
        season_year  integer NOT NULL,
        source_id    bigint NOT NULL,
        last_seen    jsonb,                  -- the game as the coordinator last saw it; NULL = new game
        version      integer NOT NULL DEFAULT 1,
        updated_at   timestamptz NOT NULL,
        PRIMARY KEY (club_number, season_year, source_id),
        FOREIGN KEY (club_number, season_year, source_id) REFERENCES games
    );
    ```

    The primary key makes "one change for each game" a rule of the database.
  - New `crates/server/src/changes.rs`:
    - `last_seen` as JSON: a server struct with every field of `Game` (with the hall and town names as they were) and `in_list`, converted to and from `StoredGame` (the core has no JSON library, decision 0001). A value that cannot be read is `StoreError::Invalid`.
    - `open_for_update(tx, club, season) -> Vec<OpenChange>` (`SELECT … FOR UPDATE`).
    - `apply(tx, club, season, &[ChangeUpdate], now)`: `Insert` inserts with version 1; `Touch` raises the version and sets `updated_at`; `Remove` deletes.
    - `remove_for(tx, club, season)` and `remove_all(tx)`.
    - `open(db, club, season) -> Vec<(OpenChange, version)>` and `open_ids(db, club, season) -> BTreeSet<i64>` for the routes (step 3).
  - `reading.rs`, inside the transaction, after the lock and the guard:
    1. `games::stored` (as now), then `changes::open_for_update`.
    2. `compare(…, today)` and `count` as now, with today from `clock::today_in_netherlands`.
    3. First reading after a choice: `changes::remove_for(club, season)`, no updates.
    4. No team chosen (`team: None`): no updates.
    5. Else `update_open_changes(…)`.
    6. `games::write` first (a new change must point to a stored game), then `changes::apply`.
  - `api/selection.rs::save` and `followed_teams::save`: one transaction that takes `pg_advisory_xact_lock(READING_LOCK)` (moved to a shared place, for example `db.rs`), reads the season's row, saves, and calls `changes::remove_all` when there was no row for this season or the club or the team is different. A save of the same club and team removes nothing. The lock makes a reading that is still running finish first; the reading after the save is then a first reading.
  - `just prepare`; commit `.sqlx/`.
- **Tests** (router tests with the `game_list` harness, and `#[sqlx::test]` storage tests):
  - A reading after a first reading, with one coming M16 1 game moved in the source, stores one open change; the same answer read again changes nothing (same version).
  - A second move of that game before "seen" keeps the first old value and raises the version; a move back removes the row.
  - The first reading after a choice stores no change.
  - A change of team removes all open changes; a save of the same team removes none. (The test "change of team removes unseen changes and records none", moved from story 2.)
  - A new season's first choice removes the changes of last season.
  - A failure halfway (for example a game whose place breaks the write) leaves no change rows and no games changed.
  - The last-seen copy keeps the old hall name after the source gives the place a new name.
  - A move of a past game stores nothing and does not count in the result line.
  - The "really gone" reading stores `Gone` changes for those games.

### Step 3: The API
- **Goal**: the routes that the screens need.
- **Changes** (new `crates/server/src/api/changes.rs`, on the protected `/api` router, in `ApiDoc`; `just api-types`):

  ```text
  GET /api/changes → { changes: ChangeView[] }
    // The open changes of the chosen club, season and team; empty unless a team is chosen this
    // season. Changes whose game (as last seen and now) is before today are left out.
  ChangeView = { id: number (source id), version: number,
                 kind: "new" | "moved" | "start_time_known" | "home_and_away" | "team_changed"
                     | "gone" | "came_back" | "into_schedule" | "out_of_schedule",
                 differences: ("date" | "start_time" | "place" | "home_and_away" | "team")[],
                 reason: "team" | "duty",     // now if in the schedule, else as last seen
                 before: ChangeGame | null,   // as last seen; null for a new game
                 after:  ChangeGame | null }  // now; null for a game no longer in the game list
  ChangeGame = { date: "2026-10-31", start_time: "13:00" | null, home, away, hall, town }

  POST /api/changes/seen { changes: [{ id, version }] } → { open: number }
    // Deletes the open changes with the same version; others stay. `open`: the number left.
  ```

  - The kind comes from `change_between(last_seen, game now, …)` for each row; a row that gives nothing is left out.
  - `kind` of a `Changed`: the same order as `count` (moved, start time known, home and away, team changed); `differences` lists all.
  - Order: by date and start time (now, or as last seen for a gone game); not known start times last in their date; then source id.
  - `GET /api/schedule`: adds `unseen_changes: number` (the number of rows `GET /api/changes` would give); `team_core::schedule` gets `changes::open_ids`. The last-season view gives 0.
  - Errors: 401 without a session, 403 without the right `Origin` on the `POST`, 500 `internal` for storage errors. An unknown or old id in "seen" is not an error.
- **Tests** (router tests, new `crates/server/tests/changes.rs`):
  - Without login: 401 on both routes.
  - Nothing chosen: empty list.
  - One test for each kind through real readings with the harness: `kind`, `reason`, `before`, `after`, `differences` (for example a moved game: `before.start_time` "16:00", `after.start_time` "17:30").
  - Into and out of the schedule: an away game of another club team that becomes a home game gives `into_schedule` with `reason: "duty"`.
  - "Seen" with the right version deletes the change and answers `open: 0`; with an old version it stays.
  - Schedule: `unseen_changes` is 1 after one change; a gone game shows with `in_list: false` before "seen" and is not in the answer after it.

### Step 4: The screens
- **Goal**: the coordinator sees what changed and marks it as seen.
- **Changes**:
  - Start page (team chosen): one line "1 change not seen" / "N changes not seen" as a link to `/changes`, in the team header, only when `unseen_changes > 0`. It takes at most one line, so the first coming games stay in the first screen at 390 × 800.
  - New page `web/src/routes/changes/+page.svelte` (+ `+page.ts` loading `GET /api/changes`), with words in `web/src/lib/changes.ts` and the kinds' words shared with `reading.ts`:
    - "Back" to the start page; the heading "Changes".
    - The same filter as the schedule (`filters`, on `reason`).
    - Each change: the game (home – away), the words of the kind, old and new values for each difference (for example "moved: Sat 31 Oct 16:00 → Sun 1 Nov 13:30", "place: Bulwijk, Woerden → Sporthal Zuid, Woerden"), "was Sat 31 Oct 14:00" for a gone game, the reason words for into or out of the schedule, and a "Seen" button. Team changes have the team look and the badge "Team".
    - 5 or more new possible duty games (after the filter): one line "N new games" that opens to the single new games, each with "Seen". New games of the team are never in the summary; they show one by one with the team look.
    - "Mark all as seen" (filter "All") or "Mark these N as seen" (another filter): sends every change that the filter shows, with its version, also the changes inside a closed summary line.
    - After "seen", the page loads the list again; changes that stayed (a reading changed them meanwhile) show their new values.
    - No changes: "No changes not seen." (or the filter's words, as `noGamesLine`).
  - The start page loads `GET /api/schedule` again after a good reading (as now), so the line follows.
- **Tests** (Playwright, phone-sized screen; the change states by `page.route` on `GET /api/changes` and `GET /api/schedule`):
  - With `unseen_changes: 3` the start page shows "3 changes not seen", and the first coming game is in the first screen at 390 × 800; with 0 there is no line.
  - `/changes` shows a moved team game with the team look and "16:00 → 17:30"; "Seen" sends `{ id, version }` and the change goes.
  - The filter "Possible duty games" leaves only duty changes.
  - 5 new possible duty games show one line "5 new games" that opens to 5 games; 4 show 4 single lines; 5 new games of the team show 5 single lines with the team look.
  - "Mark all as seen" sends all changes, also those in a closed summary line; with the filter "Games of the team" the button says "Mark these N as seen" and sends only those N.
  - Accessibility check, no console errors, no horizontal scroll at 320 px; light and dark screenshots of `/changes`.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| An open change keeps the game as the coordinator last saw it; the kind is worked out from that copy and the game now | "From what was last seen to what is true now" needs the last-seen state; merging two changes and "moved back" need no extra rules; nothing stored can disagree with the game; the duties change will build on it | yes, 0016 (at Close) |
| "Seen" deletes the open change; no history | The spec asks for no history (YAGNI) | in 0016 |
| A version number on each open change; "seen" deletes only the same version | A change that a reading updated after the screen loaded is not marked as seen unseen | in 0016 |
| The save of a choice takes the reading lock and removes the open changes when club or team differ | A reading that is still running cannot write changes of the old team after the removal | in 0016 |
| Only changes to games of today or later | Past games need no action (coordinator, 2026-10-13) | no (deviation, below) |
| A3: a game that left the list outside the schedule and comes back into it is "now in the schedule" | The coordinator never saw it in the schedule (coordinator, 2026-10-13) | no |
| Summary line from 5 new possible duty games; new games of the team always one by one | A normal week has 0–2 new games; the second half of the season about 18 (coordinator, 2026-10-13) | no |
| The list is its own page; the start page has one line | The next games stay in the first screen (A2 of story 3b; coordinator, 2026-10-13) | no |
| Closure `review build + accept` | New table, the save of the choice takes the reading lock, two new routes (build); the daily "what must I act on" screen against functional story 4 (accept) | no |

## Not in scope
- Duties, and which duty a change affects (the duties change; the functional spec says it then marks changes to games with duties as more important).
- Messages to players about a change (the messages change), and notifications on the phone.
- A history of seen changes.
- The nightly reading (story 5).
- Dropping a game of today from the list once its start time has passed (coordinator, 2026-10-13: today or later is fine).

## Test plan
- **Existing tests that must still pass**: all core, router, storage, login and web tests of stories 1, 2, 3a and 3b, and the browser projects `choose`, `logged-in`, `logged-out`. Tests that call `compare` or `schedule` get the new `today` and `open` arguments.
- **New tests**: per step, above. Order of value: the core rule table and `update_open_changes`; the router tests of a reading that keeps, merges and removes changes; the change of team; then the screens.
- **Checks**: `just check`. New queries, so `just prepare` and `.sqlx/`; `just api-types` for the new routes.

## Compatibility and performance
- One new migration (`0004_changes`), only adds a table: the released app runs on it (decision 0004).
- A reading does one more query (the open changes, a few rows) and a few writes. The start page's request is the same; `/changes` is one query of a few rows plus the stored games.
- The save of the choice now waits for a running reading (a few seconds at most).
- The result line no longer counts changes to past games, and A3 moves a few cases from "came back" to "now in the schedule".

## Progress
- [x] Step 1: core rules (2026-10-13)
- [x] Step 2: storage and the reading (2026-10-13)
- [x] Step 3: the API (2026-10-13; `api-types-check` passes once the new types are committed)
- [x] Step 4: the screens (2026-10-13)
- [x] All checks pass (2026-10-13)

## Deviations
From the functional spec (agreed by the coordinator, 2026-10-13):
- Only changes to games of today or later are kept and counted. The functional spec says "for the games in the schedule" with no date limit; the schedule also holds past games. The coordinator: changes to past games need no action. This also applies to the result line of a reading (functional story 2: its counts match the list of changes).

From the epic's technical spec:
- One route `POST /api/changes/seen` with a list of changes for one or all, instead of `POST /api/changes/{id}/seen` and `POST /api/changes/seen`; `GET /api/changes` has no `filter` parameter (the screen filters, as the schedule).
- The `changes` table keeps the game as last seen and a version, not the kind, old and new values and a seen time (see Decisions).

The design reads the functional spec more strictly than written, within its words:
- The summary line counts all unseen new games (after the filter), not the new games of one reading. In real use these are the same.
- A game that is again as last seen shows no change (coordinator, 2026-10-13).
- A reading while no team is chosen (a new season before the team is chosen) keeps no changes: there is no schedule of the current season then.
- "Mark all as seen" marks the changes the filter shows (with another filter than "All" the button says "Mark these N as seen"), also those inside a closed summary line.
- A gone game whose change is not kept (a past game, or gone in a first reading) is hidden in the schedule at once.

### Spec review of the deviations (2026-10-13)
Reviewed by `/review spec … deviations` (fresh agent). Input: mode `spec`, this change folder,
`deviations`; no other arguments. This folder has no `functional.md`; the functional spec is
story 4 (and story 1, 2, 3 where they touch it) of
[the epic's functional spec](../2026-10-10-games-from-game-list/functional.md). Read: that file
with its `idea.md`, `change.md` of this folder, `docs/vision.md`, `docs/language.md`, and
`docs/product/` (`schedule.md`, `read-the-game-list.md`, `choose-club-and-team.md`). Only the
items under **Deviations** are judged. All checks are by reading (a trace through the spec
texts); nothing was run.

| Deviation | What the coordinator is doing here | Verdict | Note |
|---|---|---|---|
| Only changes to games of today or later (also in the result line) | Opening the list to find what may still need a drive or a duty arranged. | serves | Past games need no action, and the rule takes "as last seen **or** now", so a postponed game (last seen yesterday, now next week) still shows as moved, which is the case that matters most. The result line follows the same rule, so story 2's "counts match the list" holds. Changes that age out drop from the count without a tap; that is right for this screen. |
| One route for "seen" (one or all), no filter parameter | Marking what was read. | serves | No difference for the coordinator. The version check means a change that a reading altered after the page loaded stays open: this protects "until I have looked at it". |
| The table keeps the game as last seen and a version, no history | (none; storage) | serves | Story 4 asks for no history. Gives "from what I last saw to what is true now" directly. |
| The summary line counts all unseen new games after the filter, not one reading's | Opening the list in January after the second half of the season appeared. | serves – note | Counting all unseen new games is fine (a few weeks without looking also collapses into one line, which still opens). But with the filter on "All", the team's own new games (the second half of the team's season) sit in the same "18 new games" line as the other club teams' games, while story 4 asks that a change to a game of the team is clearly visible next to the others, and those are the new games the coordinator must plan drives for. Show how many of them are games of the team in the line (for example "18 new games, 7 of the team"), or keep new games of the team out of the summary. Design decides which. |
| A game that is again as last seen shows no change | Not being bothered by a move that was undone before they looked. | serves | Literally what story 4's edge case asks ("from what the coordinator last saw to what is true now"). The same holds for a new game that is gone again before it is seen. |
| A reading while no team is chosen keeps no changes | (nothing; a new season before the team is chosen) | serves | No effect the coordinator can see: there is no read action then, and choosing the team gives a first reading, which records nothing anyway (story 1). |
| "Mark all as seen" marks the changes on the screen, after the filter | Clearing what they just read, often with the filter on "Games of the team". | serves – note | Marking hidden changes as seen would break "stays visible until I mark it", so after the filter is the right reading. But the label "Mark all as seen" then promises more than it does, and the start-page line still shows changes afterwards. Name what it marks when a filter is on (for example "Mark these 4 as seen"). Also say the changes inside a closed summary line are included (the build steps imply it). |
| A gone game whose change is not kept is hidden in the schedule at once | Looking through past games, or at a new team's schedule. | serves – note | For the coordinator now this is right: nothing to act on. Two things for later, not for this change: (a) a past game that leaves the game list vanishes from "past games" without any notice; when duties and duty totals arrive (vision core value 2, "anyone can check it"), a duty done at such a game must not silently disappear, so the duties change must decide what a gone past game means for its duties. (b) The schedule doc says a gone game stays crossed out "until the list of changes can mark it as seen"; at Close, also say it disappears when the game is past. |

**For the build (not a user question):** step 1's test "a game with an open change whose date is
now before today gives `Remove`" reads stricter than the rule ("nothing only when the date as last
seen and the date now are both before today"). For a game moved from next week to yesterday the
two give different answers. Make the test and the rule say the same.

**Verdict:** every deviation serves the coordinator, and none conflicts with the vision. Two notes
for Design before the build: the summary line hides the team's own new games among the others,
and "Mark all as seen" with a filter on needs a label that says what it marks.

Answers (2026-10-13, design):
- Summary line: fixed. New games of the team are never in the summary; only 5 or more new possible duty games show as one line (requirement 6, step 4 and its tests).
- "Mark all as seen" with a filter: fixed. With a filter other than "All" the button says "Mark these N as seen"; it also marks the changes inside a closed summary line (step 4 and its tests).
- A gone past game hidden at once: no change now. (a) is for the duties change (what a gone past game means for its duties and the duty total); (b) is for `/close` (`docs/product/schedule.md`: a gone game also disappears once it is past).
- Step 1 test against the rule: fixed. The test now removes a change only when the date as last seen and the date now are both before today, and keeps a game moved from next week to yesterday.

Small choices in the build (step 1, 2026-10-13):
- `ChangeUpdate::Insert` holds `last_seen` as `Option<Box<StoredGame>>`: clippy refuses an enum whose variants differ much in size.
- Until step 3 stores the changes, `GET /api/schedule` passes every game no longer in the game list as "open", so these games stay crossed out as before.
- The reading tests used a duty game of 19 September 2026 (`DUTY_GAME`) for "gone" and "new". With the rule "only games of today or later" these are past games and no longer count, so the tests now use the coming duty game X12 1 of 31 October 2026 (id 1081923).

Small choices in the build (step 2, 2026-10-13):
- The server's `chrono` gets its `serde` feature, so the game as last seen is stored with real date and time values (`"date": "2026-12-12"`, `"start_time": "14:30:00"`). No new library.
- The reading lock moved from `reading.rs` to `db::lock_readings`, which the reading and the save of the choice both call. `followed_teams::for_season`, `newest_before` and `save` take any executor, so the save can run inside the transaction.
- The router tests of step 2 are in the new `crates/server/tests/changes.rs`; step 3 adds the API tests there.

Small choices in the build (step 3, 2026-10-13):
- The kind of a change for the result line and the list is one core function, `Change::kind` (`ChangeKind`); `count` uses it. The reason, the leaving out of changes that give nothing, and the order of the list are the core function `list_changes` (`ListedChange`), so the route only loads and converts (the lesson of finding R1 of story 3b).
- The OpenAPI operation names are `changes` and `markSeen` (the handler names `get` and `seen` would mix with other routes in the generated types, as in story 3b).
- `GET /api/schedule` works out `unseen_changes` with the same function as `GET /api/changes`, so the number on the start page always matches the list.

Small choices in the build (step 4, 2026-10-13):
- The quick filter is now one component, `FilterToggle.svelte`, used by the schedule and the list of changes; `filterGames` works on anything with a `reason`.
- The line "N changes not seen" is a link under the team name in the team header, in the team colour. At 390 × 800 the first coming game is still in the first screen (browser test).
- A game no longer in the game list shows "no longer in the game list" in the gone colour, not crossed out: the list shows a change, not the game.
- The browser tests of the page are in the new `web/e2e/changes.e2e.ts`, in the Playwright project `logged-in`; they replace the answers of `GET /api/changes` and `POST /api/changes/seen` (`page.route`). The two start-page tests are in `choose.e2e.ts`, where M16 1 is chosen.

## Review findings

### Accept review (2026-10-13)
Reviewed by `/review accept` (fresh agent). Input: mode `accept` and this change folder, nothing
else. This folder has no `functional.md` or `idea.md`; the functional spec is story 4 (and the
parts of stories 1–3 it touches) of
[the epic's functional spec](../2026-10-10-games-from-game-list/functional.md), with that
folder's `idea.md`. Also read: `change.md`, this file (Deviations read after the view below was
formed), and the diff against `main` (committed and not). Runs: `cargo test -p team_core` (91
passed), `cargo test -p team --test changes` (19 passed), and a probe program outside the
repository that calls `team_core::list_changes` (A1). The browser tests were not run: port 4102
was already in use by another process, which I left alone.

**A1 – concerning deviation: a game of the team whose team is replaced loses the team look and
leaves the "Games of the team" filter.** A home game of the team where the game list puts another
club team in its place (for example M16 1 → M16 2; "a team of the game changed" in story 4) is in
the schedule before and after, so it is `team_changed`. But `list_changes` takes the reason from
the game *now* (`DutyGame`), so the change shows without the team look and the badge "Team", and
the filter "Games of the team" (and "Mark these N as seen" under it) does not include it. This is
exactly a change that breaks a drive the coordinator arranged, and story 4 asks that "a change to
a game of the team is clearly visible". The other branches are right: `out_of_schedule` and `gone`
take the reason as last seen, so they keep the team look. Checked: trace of `list_changes`
(`crates/core/src/comparison.rs`, the `reason` is `now` first, last seen only when `now` is not in
the schedule) and the probe: last seen home 100 (the team) – away 900, now home 101 (club team) –
away 900 gives `change=Changed([Team]) reason=DutyGame`. Fix within the spec: the reason is
`team` when the game was or is a game of the team. The reverse (a duty game that becomes a team
game) already shows as `team`.
  - 2026-10-13: fixed (with R2). `list_changes` gives the reason `team` when the team plays the game now or did when last seen. Tests: core `a_team_game_that_another_club_team_plays_now_is_still_a_change_of_the_team`, router `a_team_game_that_another_club_team_plays_now_is_a_change_of_the_team`.

**A2 – neutral drift: the result line of a reading and the list can differ.** Story 2 says the
counts "match the list of changes". The result line counts this reading (stored before → read
now); the list shows last seen → now. A game moved and moved back before "seen" gives "1 moved" in
the result line and nothing in the list; a game moved twice gives "1 moved" both times while the
list has one change. Both follow from the spec's own rules ("from what the coordinator last saw to
what is true now"), and the result line still never counts what the list would not hold for that
reading. Checked: trace of `compare` and `update_open_changes`.
  - 2026-10-13: no change needed: both follow the spec's rules (the result line counts one reading; the list shows last seen to now). For `docs/product/read-the-game-list.md` at `/close`.

**A3 – neutral drift: a gone past game can stay crossed out with nothing to mark.** The schedule
keeps a game no longer in the game list while it has an open change (`changes::open_ids`), but
`GET /api/changes` and `unseen_changes` leave out changes whose game is now past on both dates. A
gone game whose date passes before the coordinator looks stays crossed out under past games, with
no change to mark as seen, until the next reading removes the row. Before story 5 that is the next
tap on "read". Small, and only in past games. Checked: trace of `schedule` (`open` from
`open_ids`) against `list_changes` and the `Remove` branch of `update_open_changes`.
  - 2026-10-13: fixed (with R3). The schedule now keeps a gone game only while its change is in the list, so a gone game whose date passed is hidden at once. Test: router `a_gone_game_whose_date_passed_is_hidden_with_its_change`. One line in `docs/product/schedule.md` at `/close`.

**A4 – faithful adaptation: the filter on the list is its own choice.** Story 3 says "the same
filter applies to the list of changes". The list has the same three choices and the same rule (by
`reason`), but starts at "All" each time and does not follow the filter chosen on the schedule.
Fine for "what must I act on": the list opens complete. Checked: `web/src/routes/changes/+page.svelte`.
  - 2026-10-13: no change needed (the filter starts at "All" on each page, as the schedule does).

**Per criterion of story 4** (by reading the code and the router tests; the screen only by reading,
the browser tests were not run):
- Every kind (new, moved with old and new value, start time known, home and away with the old and new home team, team changed with the old teams, gone with "was …", came back): matches. `came_back` shows only the new values; that is the rule from story 3a (the coordinator saw it gone) and fits.
- Into or out of the schedule says why, also for A3 of story 3a: matches ("now a home game of another club team", "no longer a game of the team", …).
- Stays until seen; one or all: matches, with the version check so a change a reading touched after the page loaded stays.
- Summary line: matches the intent (deviation agreed); new games of the team never hide in it.
- A change to a game of the team clearly visible: matches, except A1.
- The first reading after a choice lists nothing; a change of club or team removes the old changes (story 1): matches (router tests ran).
- Edge cases (deleted and made again with a new number; two changes before seen; left and came back): match.
- Only coming games (deviation): matches the coordinator's answer, also in the result line.

**The deviations**, read after the above: they match what I found and none of them hides A1–A3.

**Verdict: needs correction** – only A1. It is a small fix inside the spec, so the user may choose
to send it back to `/implement` rather than `/discover review:`. A2–A4 need no change; A3 may get
a line in `docs/product/schedule.md` at `/close`.

### Build review (2026-10-13)
Reviewed by `/review build` (fresh agent). Input: mode `build` and this change folder, no other
text. Read: the diff against `main` (committed, and the uncommitted step 4 files), `change.md`,
this file, `docs/architecture.md`, the valid decision records (0001, 0002, 0004, 0005, 0007,
0010–0015), `CLAUDE.md`, `AGENTS.md`, `docs/language.md`. Runs: `cargo test --workspace` and
`just check` in the Nix shell with the local database (exit 0: all cargo tests, svelte-check,
lint, build, 28 browser tests, `api-types-check`, `check-sqlx`). The findings below were formed
before reading the accept review above; R2 and R3 are the same as its A1 and A3.

**R1 – bug (medium): a version number is used again, so a stale "Seen" can delete a change the
coordinator never saw.** `Insert` always starts at `version = 1` (column default), and a `Remove`
or a "seen" deletes the row. So the pair (id, version) is not unique over time. Trace: the page
shows game G moved 14:30 → 16:00 with version 1; a reading moves it back (`Remove`); a later
reading moves it to 17:30 (`Insert`, version 1 again, last seen 14:30); a tap on "Seen" or "Mark
all as seen" on the page that is still open sends `{G, 1}` and `mark_seen` deletes the new change
(14:30 → 17:30), which was never on the screen. The same happens with a second device after
"seen" on the first one and a new change of that game. This breaks requirement 5 and the decision
row "a change that a reading updated after the screen loaded is not marked as seen". With the
nightly reading (story 5) a page left open on the phone for a day is enough. Checked: trace of
`changes::apply` (`Insert` without a version), `migrations/0004_changes.sql` (`DEFAULT 1`) and
`changes::mark_seen`; not run. Fix: take every version from one sequence (insert and `Touch` both
set `nextval`), so a version is never given twice. `0004_changes` is not released yet, so it can
still be changed in place.
  - 2026-10-13: fixed. Every version comes from one sequence (`change_versions`, in `0004_changes`, which is not released yet, changed in place); `Insert` and `Touch` both take the next value. The version is a 64-bit number in the API. Test: router `a_change_made_again_never_gets_an_old_version`.

**R2 – bug against the spec's intent (medium): a game of the team that becomes a home game of
another club team loses the team look.** For a `Changed([Team])` where the team's home game now
has another club team (M16 1 → M16 2), `list_changes` takes the reason from the game now
(`DutyGame`): no team look, no "Team" badge, not under the filter "Games of the team", and not
in "Mark these N as seen" there. Story 4: "a change to a game of the team is clearly visible";
this is the change that breaks a drive. Checked: trace of `list_changes` and `in_schedule`
(`crates/core/src/schedule.rs`: both sides visible, so `Changed`, reason now). Fix in the core:
the reason is `team` when the game was a game of the team as last seen or is one now. Add a
`list_changes` unit test and a router test.
  - 2026-10-13: fixed; see A1.

**R3 – bug (low): the schedule and the list use two different sets of open changes.**
`api/schedule.rs` gives `team_core::schedule` every id from `changes::open_ids`, while
`unseen_changes` and `GET /api/changes` use `list_changes`, which leaves out changes whose game is
past on both dates. A gone game whose date passes stays crossed out in the past games with nothing
to mark as seen, until the next reading removes the row. The same holds for last season's view
(no reading of that season removes the rows until the choice is saved). Also, `view` now loads the
stored games twice and the open changes twice (`open_ids`, then `changes::open` inside
`api::changes::list`). Checked: trace. Fix: pass the ids of the listed changes to `schedule`
(for last season: no ids, so gone games are hidden, as the deviation "hidden at once" says), load
once, and drop `open_ids`.
  - 2026-10-13: fixed. `schedule` gets the ids of the listed changes (none for last season's view); the view loads the stored games once and passes them to `api::changes::list`; `changes::open_ids` is removed. Test: see A3.

**R4 – possible deadlock (low): "seen" and a reading lock the same rows in different orders.**
The reading locks its open changes with `FOR UPDATE … ORDER BY source_id`; `mark_seen` deletes
them in the order the screen sends (date order) and takes no advisory lock. A "Mark all as seen"
at the moment of a reading can make PostgreSQL abort one of the two with a deadlock error: either
"seen" answers 500, or the reading fails with 500 and is not recorded as a reading (decision 0015
records only source failures). Rare with one coordinator, more likely with story 5. Checked:
trace of the lock order; not run. Fix: `seen` takes `db::lock_readings` in a transaction (as the
save of the choice now does), or sorts the ids and locks with `FOR UPDATE` first.
  - 2026-10-13: fixed. `POST /api/changes/seen` runs in a transaction that takes `db::lock_readings` first, as the save of the choice does. Not run as a race (no test can time it); the lock makes the two run one after the other.

**R5 – logic in the server (low).** The rule "the choice is the same: same club and same team
number, so keep the open changes" (requirement 10) lives in `api/selection.rs::save_choice`, a
route handler. The architecture's pattern is "new rules go in `crates/core` first; `crates/server`
only moves data in and out" (decision 0001), and finding R1 of story 3b was the same kind. Small,
but it is the rule the duties change will build on. Checked: reading `save_choice`. Fix: a core
function (for example `same_choice(before, now)`) with unit tests; the handler only calls it.
  - 2026-10-13: fixed. The rule is `team_core::same_choice(before, now)` with two unit tests; `save_choice` only calls it.

**R6 – tests that cannot fail, or are missing (low).**
- `changes::remove_for` on a first reading cannot be reached in practice: every change of club or team already calls `remove_all` under the lock, so a first reading never finds open rows, and no test has rows at that point. `the_first_reading_after_a_choice_keeps_no_changes` passes without `remove_for` (it does test the `!first_reading` guard). Either test it with rows put in by hand, or remove it and say in a comment why the save covers it. Checked: search `remove_for` (one call) and trace of `save_choice`.
- `a_failure_while_storing_keeps_no_changes` checks that no change row is left, but not the planned "and no games changed". Checked: reading the test.
- Step 4 planned "with the filter 'Games of the team' the button says 'Mark these N as seen' and sends only those N". The browser test uses the filter "Possible duty games" with one change ("Mark this one as seen"); the branch `Mark these ${count} as seen` of `seenAllLabel` runs in no test. Checked: search `Mark these` in `web/` (only `changes.ts`).
- The claim "All checks pass" is true: `just check` exit 0 in this review.
  - 2026-10-13: fixed. `changes::remove_for` is removed, with a comment in `reading.rs` that the save of another choice removes the open changes under the same lock. `a_failure_while_storing_keeps_no_changes` now also checks that the moved game kept its start time. Browser test "with the filter on the team, the button marks only the changes of the team" covers "Mark these 2 as seen".

**R7 – a version goes up without a visible change (low, note).** `Touch` compares the whole
`StoredGame`, also the hall and town names. A new hall name in the source (the place number stays)
raises the version of every open change at that place, so a "Seen" sent from a page loaded before
that reading leaves the change open with nothing new to see. It does what the design says ("Touch
only when … `StoredGame` equality") and costs one more tap; mention it in decision 0016 or compare
only what the list shows. Checked: trace of `update_open_changes` and `games::stored` (newest hall
name from `places`).
  - 2026-10-13: not fixed: it does what the design says and costs one more tap; for decision 0016 at `/close`.

**R8 – docs and comments the diff made false or left behind.**
- `crates/core/src/comparison.rs`, `no_longer_in_list`: "`compare` counts only the ones in the schedule as `Gone`": now only the coming ones in the schedule.
- Two doc comments were not wrapped again and are longer than the rest (113 characters in the module doc of `crates/server/src/reading.rs`; the doc of `schedule` in `crates/core/src/schedule.rs`).
- For `/close`: `docs/architecture.md` (core functions `change_between`, `update_open_changes`, `list_changes`, `Change::kind`; `changes.rs`; migration `0004_changes`; the reading steps; `db::lock_readings` shared with the save of the choice; the API routes; `followed_teams` now runs in the save's transaction), the module list in `CLAUDE.md` (no `changes.rs`), `docs/product/schedule.md` line 18, and decision 0016.
- The step 4 files (`web/src/routes/changes/`, `web/src/lib/changes.ts`, `FilterToggle.svelte`, `web/e2e/changes.e2e.ts` and the other web changes) are not committed yet.

  - 2026-10-13: fixed: the comments in `comparison.rs` (`no_longer_in_list`), `reading.rs` (module doc) and `schedule.rs` (`schedule`). Not fixed: the documents are for `/close`; the step 4 files are committed with the fixes.

**Verdict:** fix R1 and R2 before Close (R2 is A1 of the accept review); R3 and R4 are small and
cheap to fix in the same pass; R5–R8 are clean-up and `/close` items.
