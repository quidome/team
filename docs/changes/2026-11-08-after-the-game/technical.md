# Technical spec: After the game

**Status**: done
**Size**: story
**Change**: [2026-11-08-after-the-game](./)
**Date**: 2026-11-08

## Objective
From the start time of a game the coordinator records who was there: the words become "There" and
"Absent", one tap on "Checked" makes each "no answer" absent, "Undo check" brings back only those,
a past game that is not checked is an open item, and the duty places offer "Someone else" and
"Nobody" and count as done.

## Requirements summary
From story C in `../2026-10-31-attendance-at-games/functional.md` (approved; changed with the
coordinator on 2026-11-08), with the answers of this design. "Started" means: the start time has
passed; with no start time, the game day has begun (`team_core::started`, Dutch time).

- **Words.** From the start of a game of the team, the buttons say "There" and "Absent", the
  groups "No answer", "There" and "Absent", the count "8 there · 1 absent". Before, the words stay
  "coming" and "not coming". Nothing stored changes: "coming" is "there".
- **"Checked"** on each game of the team from its start; not before (`checked_too_early`). Also on
  a not played game, and on a game that left the game list. One tap makes each "no answer" on the
  list "absent" without a reason, in one step, and the page says "2 players with no answer are
  absent." The page then shows "Checked" with "Undo check"; a checked game shows only the count,
  and a tap opens the list. A second "Checked" on a checked game changes nothing.
- **"Undo check"** brings back to "no answer" only the players that "Checked" settled and that were
  not tapped after that. A tap on a player, also a reason added through "Reason", counts as a tap.
  Players marked before the check keep their state.
- **A checked game can still be changed**: a tap moves a player between "there" and "absent"; a tap
  on the chosen state does nothing. A player who gets a role later shows as "no answer" there and
  does not count until tapped; the players page says so at the change of role.
- **A mark holds for a date** (decision 0029): "Checked" stores the date of the game. When the game
  moves, the check has no effect and the players it settled count as "no answer" again; when it
  moves back, both hold again. Once the game is checked on its new date, moving it back does not
  bring back the old check: the check holds the new date. Nothing stored is deleted.
- **The open item** "Game Sat 31 Oct not checked", for more "2 games not checked"; a tap opens the
  oldest. It counts each game of the team before today (Dutch date) that is not checked, not "not
  played", still in the game list, and on or after the day of the release; in the seasons of the
  trainings (the current season, and in July and August the season before). Last in the order of
  the open items. Older games are never an open item, but can be marked and checked.
- **A game from before the release** shows its list closed behind a tap.
- **Rows**: on "Season" and in "Last" a checked game shows "· Checked".
- **Duties** (released rule, changed): from the start of a game, its duty places offer "Someone
  else" and "Nobody", and a filled place counts as done instead of planned; before, both came from
  the day after the date.
- **Errors**: a save that fails: "Not saved. Try again." and the stored state. Two screens: the
  last check or undo wins. A game with no player on its list can be checked.

Rules of the coordinator that this design follows (2026-11-08): everything entered can always be
changed; "in the past" changes only how events are shown; "not played" changes only what counts;
"Checked" means the admin is complete and locks nothing. Each domain owns its tables (decision
0030). Trainings stay as they are.

## Current state
- `game_attendance` (`extra_players`, `answers_for`) and `game_answers` (player, `answer`, optional
  `reason_id`; no row is "no answer"), store `crates/server/src/game_attendance.rs` (`answer`,
  `no_answer`, `set_extra`, `answers_checked`, `of_game`, `of_season`); answers need no lock
  (decision 0028), each tap is one write.
- Core `contribution/game_attendance.rs`: `StoredAnswer`, `GameAttendance`, `ListRow`, `game_list`
  (the players with "plays games" on the date), `game_count` (the only count), `row_count`,
  `game_warnings` over a `ScheduledGame` (nothing after the start time), `move_note`.
- `team_core::started(game, now)` (events) from story E.
- Duties: `who_did(assignment, past)` and `check_outcome(past, …)` (refusal `not_past`) use "the
  date is before today"; `GameDuties.past`; `GamePage.past` drives "Someone else", "Nobody" and the
  note "· absent" on the page; `GamePage.started` exists.
- Trainings: `trainings::lock_for_check`, `check`, `uncheck`; the open item `NotChecked` from
  `training::not_checked`; migration `0010` checked every older training once.
- The note at a role change: `attendance::now_there` and `api/players.rs` `now_there_of`, on the
  players page and the team page (`NowThere { player, trainings }`).
- Migrations end at `0014`.

## Steps

### Step 1: Duties from the start time
- **Goal**: "Someone else", "Nobody" and "done" come from the start of the game.
- **Changes**:
  - `who_did(assignment, started)`; `check_outcome(started, …)`; `offers(…, started)`;
    `GameDuties.started` beside `past` (`past` stays for "a past game counts what is stored" in
    `place_counts`). `DutyError::NotPast` becomes `NotStarted`, code `not_started`.
  - Server: `SeasonPlaces` fills `started` from `started(game, now)`; `api/games.rs` uses
    `found.started(local)` for `check_outcome` and `offers`. `GamePage.past` is removed.
  - Front end: the page uses `page.started` for "Someone else", "Nobody" and `driverNote`.
  - The messages pass the date as now (a game of today stays all day): no change of text.
- **Tests**:
  - Core: `who_did` and `check_outcome` on the game day before and after the start time, and with
    no start time.
  - `crates/server/tests/games.rs`: the refusal test changes to `not_started` (before the start
    time on the game day); after the start time "Someone else" is saved and the totals of
    "Duties" show "done".
  - `web/e2e/duties.e2e.ts`: the test of "Someone else" on a past game still passes.
  - Find out how the browser test database can set `game_check_start` (step 4).

### Step 2: "Checked" is stored
- **Goal**: "Checked" and "Undo check" work through the API, with the settled answers.
- **Changes**:
  - Migration `0015_game_checks.sql`:
    ```sql
    -- "Checked" (story C of attendance at games): the date of the game at the tap (decision 0029).
    ALTER TABLE game_attendance ADD COLUMN checked_for date;
    -- An answer that "Checked" made ("no answer" became absent); a tap clears it.
    ALTER TABLE game_answers ADD COLUMN settled boolean NOT NULL DEFAULT false;
    -- From this date on (the day of the release), a past game that is not checked is an open item.
    CREATE TABLE game_check_start (
        only_row  boolean PRIMARY KEY DEFAULT true CHECK (only_row),
        from_date date    NOT NULL
    );
    INSERT INTO game_check_start (from_date)
    VALUES ((now() AT TIME ZONE 'Europe/Amsterdam')::date);
    ```
  - Core (`contribution/game_attendance.rs`):
    ```rust
    pub struct StoredAnswer   { pub player, pub answer, pub reason, pub settled: bool }
    pub struct GameAttendance { pub extra_players, pub answers_for, pub checked_for: Option<NaiveDate> }
    pub fn checked(attendance: &GameAttendance, date: NaiveDate) -> bool;      // checked_for == Some(date)
    pub fn answers_in_effect(answers: &[StoredAnswer], attendance: &GameAttendance,
                             date: NaiveDate) -> Vec<StoredAnswer>;            // settled only while checked
    pub fn to_settle(list: &[ListRow]) -> Vec<PlayerId>;                       // the rows with no answer
    pub enum CheckError { TooEarly }                                          // `checked_too_early`
    pub fn check_checking(game: &Game, now: NaiveDateTime) -> Result<(), CheckError>; // `started`
    ```
    Every list and count goes through `answers_in_effect` first: the page (`stored_list`), the rows
    (`SeasonDuties::list_of`), the warnings and `move_note` (`has_answers` counts the answers in
    effect).
  - Store (`game_attendance.rs`): `check(tx, key, date, players) -> usize` locks the row of the
    game (insert when missing, then `SELECT … FOR UPDATE`); when `checked_for` is already `date` it
    changes nothing and gives 0; else it stores `checked_for = date` and inserts a settled
    "not_coming" answer without a reason for each player, `ON CONFLICT DO NOTHING`, and gives the
    number of `players` (the players with "no answer" in effect just before the tap, from
    `to_settle`), not the rows inserted: after a move, the settled rows of the old date already
    exist and become "absent" again at this tap (spec review S41). `uncheck(tx, key)`: the same lock; deletes the answers with `settled`, sets
    `checked_for` to `NULL`. `answer` and the save of a reason write `settled = false`.
    `check_start(db) -> NaiveDate`. `of_game` and `of_season` read `checked_for` and `settled`.
  - Routes (`api/games.rs`, registered in `app.rs` and `api/mod.rs`): `PUT
    /api/games/{season}/{id}/checked` answers `CheckedAnswer { settled, page: GamePage }`;
    refusals `not_a_team_game`, `checked_too_early`. `DELETE …/checked` answers `GamePage`. One
    transaction each.
  - Views: `GameAttendanceView.checked` and `before_release` (the game is before `from_date`).
  - `just prepare`, `just api-types`.
- **Tests**:
  - Core: `checked` (the date, another date, none); `answers_in_effect` (settled while checked,
    gone after a move, back after the move back, not settled always); `to_settle`;
    `check_checking` (a minute before, at the start time, with no start time from 00:00).
  - `crates/server/tests/checked.rs`: "Checked" settles each "no answer" without a reason and
    answers `settled: 2`, the count says absent; a second "Checked" gives 0; after a move, a new
    "Checked" on the new date answers the number of players settled on the old date (S41); "Undo check" brings
    back only the settled players, and keeps a player tapped after the check, a player who got a
    reason after it, and a player marked before it; `checked_too_early`; `not_a_team_game`; 401;
    a moved game shows the settled players as "no answer" and is not checked, and both hold again
    after the move back; a not played game can be checked; a race test: an open transaction holds
    the row of the game in `game_attendance`, "Checked" waits 300 ms and then answers, and fails
    without `FOR UPDATE` (checked by removing it).
  - Logs: "Checked" logs no player name (own file, pattern of `warning_logs.rs`).

### Step 3: The page
- **Goal**: the coordinator checks a game on the page.
- **Changes** (`web/src/lib/components/GameAttendance.svelte`, the page of a game,
  `web/src/lib/api/client.ts` `setChecked`):
  - From `page.started`: the buttons "There" and "Absent", the groups and the count with the same
    words.
  - "Checked" from `page.started` on a game of the team, not checked; after the tap the message
    "2 players with no answer are absent." (from `settled`; "1 player with no answer is absent.";
    none for 0), then "Checked" with "Undo check"; a checked game shows only the count, and a tap
    opens the list.
  - `before_release`: the list is closed behind a tap.
  - Each tap saves at once; a failure shows "Not saved. Try again." and the stored state.
- **Tests** (`web/e2e/checked.e2e.ts`, a project `checked` after `not-played`, on a past game):
  the words "There" and "Absent"; "Checked" with the message, "Undo check" and the closed list;
  "Undo check" brings the list back; a failed save; a game before the release with the closed
  list; 320 pixels and the accessibility check.

### Step 4: The open item and the rows
- **Goal**: a past game that is not checked shows on the start page; a checked game shows on its
  rows.
- **Changes**:
  - Core (`output/start_page.rs`): `games_not_checked(games, checks, from_date, today)` and
    `OpenItem::GamesNotChecked { first, count }`, last in `open_items`; `api/start.rs` reads
    `check_start` and the checks of the seasons of the open items.
  - Front end `web/src/lib/start.ts`: "Game Sat 31 Oct not checked", "2 games not checked"; a tap
    opens the oldest.
  - `ScheduleGame.checked`; `EventRow.svelte` shows "· Checked".
- **Tests**:
  - Core: `games_not_checked` for each case that leaves a game out (not past, checked, not played,
    gone, before `from_date`, not a game of the team); the oldest first; last in the order.
  - `crates/server/tests/checked.rs`: the open item shows for a past game after `from_date` (the
    test sets `game_check_start`), and goes away at "Checked" and at "Not played"; not for a game
    before `from_date` nor a gone game; `checked` on the rows of "Season" and "Last".
  - Browser: "· Checked" on "Season".

### Step 5: The note on the players page
- **Goal**: after a role change the page says at which checked games the player has "no answer".
- **Changes**: core `now_no_answer(player, before, after, games)` (the checked games covered by a
  role "plays games" after and not before, where the player has no answer in effect);
  `NowThere.games` (the dates) in `api/players.rs` `now_there_of`, for the players page and the
  team page; the text in `web/src/lib/players.ts`: "Anna has no answer at 2 checked games: Sat 10
  Oct, Sat 17 Oct."
- **Tests**: core `now_no_answer` (a covered checked game without an answer; not a game that is
  not checked, has an answer, was covered before, or a role "only trains"); a server test after a
  role change from 1 September; the existing tests of `now_there` in
  `crates/server/tests/trainings.rs` and `web/e2e/trainings.e2e.ts` keep passing.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| "Checked" uses its own lock on the row of the game in `game_attendance` | Attendance must not lock the tables of duties (decision 0030) | Yes, 0032 at close |
| The settled answers count only while the check holds | A moved game must not show answers nobody gave (decision 0029, S34) | Same record |
| The day of the release is stored in `game_check_start`, filled by the migration | Exact; older games must not show "Checked" as migration `0010` did for trainings | Same record |
| One moment, "started", for "Checked", the words and the duties | From the start the real attendance and drivers can be filled in (coordinator, 2026-11-08) | Same record; it changes when "done" begins in decision 0023 |
| A not played game can be checked | "Checked" means the admin is complete (coordinator) | Same record |
| A second "Checked" changes nothing | One clear result; a player with a later role stays "no answer" until tapped (spec) | No |

## Not in scope
- The totals of "Attendance" with games: story D.
- Any change to trainings (coordinator, 2026-11-08: trainings stay as they are).
- "Someone else" before the start time (`docs/ideas.md`, 2026-11-08).
- "All there" for older games (S11) and a look at the "no answer" players before the tap (S21).

## Test plan
- **Existing tests that must still pass**: all core tests; `crates/server/tests/` (the duties
  refusal test changes to `not_started`); `web/e2e/`.
- **New tests**: per step, see above.
- **Test risk**: the browser test clock (12 October 2026, 10:00 UTC) has no game of the team that
  has started and is not past, so the browser tests use a past game and the game day is tested in
  the server. The browser test database runs migration `0015` on the real date, so `from_date` is
  after the test clock; step 1 finds out how the browser tests set it.
- While working: the tests of the changed part; `just check` once before the commit.

## Compatibility and performance
- Forward-only migration with two columns with defaults and a table with one row; no data to
  move. Games before the release are never an open item.
- Released rules change: "Someone else", "Nobody" and "done" of a duty come from the start time
  instead of the next day; the refusal code `not_past` becomes `not_started`.
- One more read (`check_start`) on the start page.

## Progress
- [x] Step 1: duties from the start time
- [x] Step 2: "Checked" is stored
- [x] Step 3: the page
- [x] Step 4: the open item and the rows
- [x] Step 5: the note on the players page
- [x] All checks pass

## Deviations
- **D1** (from the epic `technical.md`): "Checked" and "Undo check" use their own lock on the row of
  the game in `game_attendance`, not the row lock of the game in duties. Reason: each domain owns
  its tables (decision 0030).
- **D2** (from `functional.md` as first approved; changed in `functional.md` with the coordinator on
  2026-11-08): the moment is the start time for "Checked" (also with no start time: from the start
  of the game day, not the day after), for the words "There" and "Absent" (not the day after the
  date), and for "Someone else", "Nobody" and "done" of the duties (a released rule); a not played
  game can be checked (story E said it cannot); a moved game: the check and its settled answers
  have no effect until it moves back.
- **D3** (from the epic `technical.md`): the day of the release is stored in a table with one date,
  `game_check_start`, filled by the migration.
- Stricter readings: a second "Checked" on a checked game changes nothing; a reason added through
  "Reason" after the check counts as a tap (S34a).
- Implement, step 1: the choice of a player (`offers`) and the save of a planned player
  (`check_assignment`) also use "started": from the start a clash does not block a place, because
  it then records what happened, as "Someone else" does (one rule with the correction). The browser
  tests set the release day in `web/e2e/global-setup.ts` after the migrations, to 12 October 2026
  (the test clock), so the existing open items of the start page stay; the open item is tested in
  the server (added in step 2, when the table exists).
- Implement, step 2: the rule is named `check_game_checking`, because `check_checking` is the rule
  of trainings. The race test lets the other screen take the row lock first and store its check
  only after the request started; a first version, where the other screen had already changed the
  row, also passed without `FOR UPDATE` (the insert at the start of the lock waits for such a
  change by itself), so it proved nothing. The server tests are in `tests/checked.rs` and
  `tests/checked_logs.rs`.
- Implement, step 3: the closed list (checked, before the release, or not played) shows the count
  with "Show who comes". The browser test of a driver who is not coming
  (`web/e2e/game-attendance.e2e.ts`) mocked the removed field `past`; it now sets `started`. Step 1
  did not catch it, because `npx playwright test duties` runs only the files named "duties". The
  browser tests are in `web/e2e/checked.e2e.ts`, project `checked`, after `not-played`.
- Implement, step 4: `games_not_checked` also takes the season, so the open item can open the
  page of its oldest game (`NotCheckedGame`: season, id, date). The test of a new season without a
  team (`tests/start.rs`, clock 1 July 2027) now sets the release day to its clock: in July the
  open item also covers the season before, whose games are not checked.
- Implement, step 5: `now_no_answer` takes the games of the schedule with what is stored for them
  (`checks`, `answers`), and leaves out games of other club teams. The note of a player now also
  comes when only games are named; the texts follow the note of the trainings: "This player has no
  answer at 1 checked game: Sat 10 Oct.", with the name when more players are named. The server
  tests of the training note (`tests/trainings.rs`) now also check `"games": []`.

### Spec review of the deviations
Fresh review, 2026-11-08 (`/review spec … deviations`). Read: this file (only the deviations are
judged), story C and E of `../2026-10-31-attendance-at-games/functional.md` with its spec review,
the epic `technical.md`, `docs/vision.md`, `docs/language.md`, `docs/product/principles.md`,
`game-attendance.md`, `duties.md`, `start-page.md`, `season.md`, `trainings-and-attendance.md`,
decisions 0023, 0029, 0030. The arguments held no extra text.

| Deviation | What the user is doing here | Verdict | Note |
|---|---|---|---|
| D1: own lock in `game_attendance` | Tapping "Checked" or "Undo check", maybe on two screens | serves | Not visible to the coordinator; "the last check or undo wins" still holds. |
| D2: duties from the start time | At or just after an away game, putting the real drivers in | serves | Fills in what happened while it is known; core value 2. `principles.md`, `duties.md` and `game-attendance.md` say "after the date" and need the new moment at close. |
| D2: words and "Checked" from the start time | Closing the record on the evening of the game | serves – note | Words and warnings now change at the same moment, which fixes S12 and S31, but not for a game without a start time (S40). |
| D2: a not played game can be checked | Finishing the admin of a game that did not happen | serves | Story D counts only played games, so the settled absences count only after "Undo" of "Not played", which story E asks for. |
| D2: a moved game | A postponed game, weeks later on its new date | serves – note | Answers S34b. After a move, "Checked" on the new date gives the wrong message (S41). |
| D3: `game_check_start` filled by the migration | Not seeing older games as open items | serves | The date is the day the migration runs in production. Story D ("the games start at the release") should read the same date, so the tool has one day of the release. |
| Stricter readings | Correcting a checked game | serves | A reason after the check surviving "Undo check" is what S34a asked. |

**Overall: serves – note.** No deviation conflicts with the vision. Each follows the rules of the
coordinator of 2026-11-08 (changes are always possible, "Checked" locks nothing, a mark holds for
a date).

- **S40. A game without a start time has two moments on its day.** `team_core::started` is true
  from 00:00 of the game day (`crates/core/src/events/game.rs`), but `before_start`, and with it
  the warnings "too few players" and "driver not coming", runs until the end of the date
  (`crates/core/src/contribution/game_attendance.rs`, `in_warning_period`). So on that day the
  page shows "Too few players: 6 of 7" above "6 there · 1 absent", and "Checked" in the morning
  makes each "no answer" absent and ends the warnings: the case of S31, for these games only. The
  Decisions table says "one moment, started" for all of this, but the warnings keep the other one.
  Options: accept it (the game list writes 00:00 only while the start time is not known, and how
  often that is still so on the game day is not checked); or, without a start time, start the
  words and "Checked" at the end of the date, as the warnings end. The coordinator's choice.
  Checked: code read of `started`, `before_start` and `in_warning_period`.
  - 2026-11-08: Kept: accepted as rare; "started" stays the start of the game day for a game without a start time (coordinator).

- **S41. "Checked" after a move says it settled nobody.** Anna and Bram are settled at the old
  date; the game moves, they show "no answer"; on the new date the coordinator taps "Checked".
  `to_settle` gives Anna and Bram, but their settled rows are still stored, so the insert `ON
  CONFLICT DO NOTHING` gives 0 and the page shows no message, while both are absent again (their
  old rows count once `checked_for` is the new date). Story C: "says so: 2 players with no answer
  are absent." The number must be the players that became absent at this tap, not the rows
  inserted. Linked: after a check on the new date, a move back to the old date leaves the game
  not checked (`checked_for` holds the new date), so "when it moves back, both hold again" holds
  only when the game was not checked on its new date; that reading is fine, but say it. Checked:
  trace of step 2 against the primary key of `game_answers` (`migrations/0013_game_attendance.sql`:
  game and player).
  - 2026-11-08: Fixed: `check` gives the number of players with "no answer" in effect just before the tap (`to_settle`), not the rows inserted; a test in step 2; the spec says that a check on the new date replaces the old one (coordinator).

## Review findings

### Acceptance review
Fresh review, 2026-11-08 (`/review accept`). Read: `change.md`, story C (and the parts of A, B
and E it touches) of `../2026-10-31-attendance-at-games/functional.md`, the epic `idea.md`, the
objective and requirements of this file, the diff against `main` (committed and not), and only
then the **Deviations**. The arguments held no extra text. Run: `cargo test -p team_core` (342
passed), `cargo test -p team --test checked --test checked_logs --test games --test start --test
trainings` (all passed). Not run: `web/e2e/checked.e2e.ts` (the e2e ports were in use by another
run); the browser parts below are checked by a code trace only.

**Verdict: accepted with notes.** Each acceptance criterion of story C is built as meant; the
differences are small and none works against the coordinator's goal (the record after the game
with almost no work).

| Criterion of story C | Built | How checked |
|---|---|---|
| Words "There" / "Absent", groups and count from the start time | Yes, on the page of the game | trace: `GameAttendance.svelte` (`words`), `attendance.ts` `countText(count, started)`, `+page.svelte` |
| "Checked" from the start (no start time: from 00:00); also not played and gone games; "· Checked" on "Season" and "Last"; a checked game shows only the count | Yes | run: `checked.rs` (`checked_is_refused_before…`, `a_not_played_game_can_be_checked`, `a_past_game_from_the_release…`); core `checked_is_possible_from_the_start_time`; trace of `find` for a gone game |
| One tap settles each "no answer", without a reason, and says how many | Yes | run: `checked_settles_each_no_answer…`; trace: `settledText` |
| "Undo check" brings back only the settled players not tapped since (also a reason counts as a tap) | Yes | run: same test (Bram, Cas, Anna, Dora) |
| On a checked game: there ↔ absent, a tap on the chosen state does nothing; a later role shows "no answer" and the players page says so | Yes (see A2) | run: `a_role_moved_earlier_names…`; trace: `tapAnswer` |
| Open item "Game Sat 31 Oct not checked" / "2 games not checked", the oldest opens; from the release, in the seasons of the trainings; not for not played or gone games | Yes | run: `a_past_game_from_the_release…`; core `games_not_checked` tests; trace: `api/start.rs` over `open_item_seasons` |
| No "Checked" before the start; before the release the list is closed | Yes | run: `a_game_before_the_release…`; trace: `+page.svelte` |
| A check holds for its date; a move back restores it; a check on the new date replaces it | Yes | run: `a_moved_game_shows_its_settled_players…` |
| Duty places offer "Someone else" / "Nobody" and count as done from the start | Yes | run: `games.rs` `from_the_start_time_a_place_can_be_corrected…` |

- **A1. The clash no longer blocks a place once the game has started** (Deviations, step 1).
  **Faithful adaptation.** It is the same rule as "Someone else" from the same moment: from the
  start the coordinator records what happened, and a clash cannot make that wrong. Checked: trace
  of `check_assignment` and `offers` with `started`.
  - 2026-11-08: not fixed: a faithful adaptation, written under Deviations (step 1).
- **A2. "A tap on the chosen state does nothing" on a checked game holds only in the page.** The
  server still accepts "no answer" for a player on a checked game (`api/games.rs`, the route that
  calls `game_attendance::no_answer`, has no check of `checked`), so a second screen opened before
  the check can bring a player back to "no answer". **Neutral drift**: the spec says "two screens:
  the last tap wins", the page itself follows the rule, and the coordinator sees the "no answer"
  in the count. Checked: search `no_answer(` in `crates/server/src/api/games.rs` and a read of
  `set_answer`.
  - 2026-11-08: not fixed: "the last tap wins" covers a second screen; the page follows the rule.
- **A3. On the rows of "Next" and "Season" a game of today keeps the words "coming" and "not
  coming" after its start time**, also when it is checked: "· Checked" with "9 coming · 1 not
  coming" in "Next" on the evening of the game (it is in "Next" all day, in "Last" from the next
  day). The spec names the new words only for the page of the game, and story A names the row
  words, so this is **neutral drift**; for a coordinator who checks right after the game the row
  reads oddly until midnight. Checked: trace of `next_events` (`events.coming`, from `today`),
  `coming_view` and `EventRow.svelte` (`countText(game.attendance)` without `started`).
  - 2026-11-08: fixed: a coming game of the team has `started` on its row (`coming_view` gets `now`), and the count says "there" and "absent" from the start time; test `the_row_of_a_game_of_today_says_started_from_its_start_time`.
- **A4. The closed list says "Show who comes"** on a checked game and on a game before the
  release, where the page says "There" and "Absent". **Neutral drift** in wording; the tap does
  what the spec asks. Checked: search `Show who comes` in `web/src`.
  - 2026-11-08: fixed: the button says "Show the list".
- **A5. A game without a start time starts its words and "Checked" at 00:00 while its warnings run
  until the end of the date** (S40). **Faithful adaptation**: the coordinator accepted it as rare.
  Checked: the note under S40.
  - 2026-11-08: not fixed: S40, accepted by the coordinator as rare.
- **A6. For Close:** the living documents still give the old moment: `docs/product/duties.md`
  (lines 30–32: "after the game (a game before today)", "a clash never blocks a place of a past
  game", "from today it counts as planned"), `docs/product/game-attendance.md` (line 15 "a past
  game keeps the words coming and not coming", line 31 "after the date") and
  `docs/product/principles.md` line 11; the header comment of
  `web/src/routes/game/[season]/[id]/+page.svelte` (line 11, '"· absent" after the date') too.
  Not a defect of the build; Close must update them. Checked: search `day after|after the date|before today|past game` in `docs/product/`.
  - 2026-11-08: not fixed here: for Close.

### Build review
Fresh review, 2026-11-08 (`/review build`). Read: the diff against `main` (committed and not,
with the new files `migrations/0015_game_checks.sql`, `tests/checked.rs`, `tests/checked_logs.rs`,
`web/e2e/checked.e2e.ts`), this change folder, `docs/architecture.md`, decisions 0022, 0023, 0028,
0029, 0030, 0031, `CLAUDE.md`. The arguments held no extra text. Run: `cargo test -p team_core`
(342 passed), `cargo test -p team --test checked --test checked_logs --test games --test start
--test trainings --test team` (all passed), `cargo clippy --workspace --all-targets -- -D
warnings` and `cargo fmt --check` (clean), `just check-web` (passed), `npx playwright test checked
 game-attendance` (158 passed, with the projects these depend on). The domain map holds:
`now_no_answer` (contribution) uses events and team, `games_not_checked` (output) uses
contribution; only `game_attendance.rs` touches the new columns and table (search
`checked_for|settled|game_check_start` in `crates/server/src`).

- **R1. On a checked game, and on every game before the release, the driver note "· absent" is
  gone.** The note at a driver place and "Not coming" in the choice of a driver come from
  `answers` in `+page.svelte`, which only `GameAttendance` fills (`onanswers`). The list is now
  closed on a checked game and on a game before `game_check_start`, so `GameAttendance` is not
  mounted and `answers` stays `{}` after a load: a planned driver who did not come shows no
  "· absent" exactly when the coordinator corrects the drivers after the game (story B), and on
  all older games until "Show who comes" is tapped. Right after "Checked" the page keeps the
  answers of before the tap, so a driver settled from "no answer" shows nothing either. The
  browser test of the note (`game-attendance.e2e.ts`) mocks a coming game with `started`, whose
  list is open, so it cannot see this. Fix: fill `answers` from `page.attendance.rows` at each
  load, not only from the mounted list. Checked: trace of `notComingDriver`, `answers` and the
  `{#if … !listShown}` block in `web/src/routes/game/[season]/[id]/+page.svelte`; search
  `answers =` there (one hit, `onanswers`).
  - 2026-11-08: fixed: the page takes the answers from the open list while it shows, else from the stored answers of the page (`answers` in `+page.svelte`); the browser test of the driver note also checks a checked game with its list closed. The answers of the open list are kept with `$state.raw`: a `$state` wraps the page, so it no longer equals the page shown (found by `just check`).
- **R2. A check of a game with no answers leaves `answers_for` empty for good, so a later move is
  not noticed** (decision 0029). `check` inserts the settled rows but never sets `answers_for`;
  `answer` sets it only `NOT EXISTS (SELECT 1 FROM game_answers …)`, which the settled rows now
  fail. Case: nobody answered before the game; the coordinator taps "Checked", then taps Anna
  "There"; the game list later moves the game. `answers_moved` needs `answers_for` to be set, so
  Anna's "there" of the old date counts on the new date with no note "answers from before the
  move" and no warning. The same happens when the settled rows of an old date stay after a move
  (they no longer count, but still block the first answer on the new date). Fix: `check` sets
  `answers_for = COALESCE(answers_for, date)`, or the `NOT EXISTS` of `answer` counts only the
  answers in effect; a server test of the case. Checked: trace of `game_attendance::check`,
  `game_attendance::answer` and `team_core::answers_moved`; not run.
  - 2026-11-08: fixed: the answers that "Checked" made no longer stop the first answer from storing its date (`NOT a.settled` in `game_attendance::answer`); tests `a_game_checked_without_answers_still_notices_a_move` (fails without the fix) and `after_a_move_the_settled_answers_do_not_hold_back_the_first_answer`. The proposed `COALESCE` at "Checked" was left out: with this fix it changed no result.
- **R3. "Checked" reads the list before it takes the lock.** `set_checked` reads the answers and
  works out `to_settle` with the pool, then `game_attendance::check` locks the row in its own
  transaction. Decision 0023 and the architecture ("reads the places and the players under the
  lock, checks in the core, and writes") read under the lock. Results: "Checked" on one screen
  while "Undo check" commits on another stores a check whose list was read while the game was
  still checked: the players "Undo check" brought back are left out, and the game shows
  "Checked" with players at "no answer" (the spec: "the last check or undo wins"); and `settled`
  counts a player whose tap at the same moment won the `ON CONFLICT DO NOTHING`. Rare (two
  screens), but the fix is small: read the answers in `tx` after `lock`. The race test holds the
  lock before the request reads, so it passes either way for this case. Checked: trace of
  `set_checked` and `check`; not run.
  - 2026-11-08: fixed: `game_attendance::check` reads what is stored under the lock and gets the rule of who is settled as a function; test `checked_reads_the_list_after_an_undo_check_on_another_screen` (fails when the read is moved before the lock).
- **R4. Two test rules of the architecture are not met for the new saves.** "A save of more than
  one write has a test that makes the second write fail with a trigger … and checks that nothing
  changed": `check` (insert of the row, `checked_for`, the settled answers) and `uncheck` (delete,
  update) have none. "A save guarded by … `ON CONFLICT` has a test that holds an insert of the
  same key in an open transaction": the settled insert (`ON CONFLICT … DO NOTHING`, "a tap at the
  same moment wins") has none. Checked: search `trigger` and `begin()` in
  `crates/server/tests/checked.rs` (only the row-lock test).
  - 2026-11-08: fixed: tests `a_checked_that_fails_halfway_changes_nothing`, `an_undo_check_that_fails_halfway_changes_nothing` (a trigger makes the second write fail) and `a_tap_at_the_same_moment_as_checked_wins` (an insert of the same answer held open).
- **R5. Claims in this file.** True: the race test of the row lock waits and would not wait without
  `FOR UPDATE` (reasoned: `INSERT … ON CONFLICT DO NOTHING` does not wait for a lock on a committed
  row; not checked by removing it); S41 is tested (`a_moved_game_shows…`, `settled: 2` on the new
  date); the log test asserts that the names are absent. "All checks pass" is not ticked: of
  `just check`, this review ran the Rust checks, `check-web` and the e2e; `check-sqlx` and `lint-ci` were not run. The new `.sqlx/` files are untracked and five old ones are deleted: they
  must go into the commit.
  - 2026-11-08: no change needed; `.sqlx/` goes into the commit, `just check` runs before it.
- **R6. Siblings.** The week message and the duty overviews still use `who_did(…, past)`
  (`output/messages.rs`), so on the evening of a game a correction ("Someone else") shows on the
  page and in the totals but not in the message until the next day; the design chose this ("a game
  of today stays all day"), so it is a note, not a defect. Every other reader of the answers goes
  through `answers_in_effect` (search `of_game(|of_season(` in `crates/server/src/api`; the
  exceptions read only the reason of the player for `check_answer`, and `kept_games`, where a
  settled row rightly keeps the game).
  - 2026-11-08: not fixed: the design chose it.
- **R7. Reads done twice.** `api/start.rs` now calls `games_of` for the season before twice in
  July and August (line 320 for the open item, line 368 for "Last"), and `now_there_of` calls
  `games_of` (with the list of changes) for each player of a role save, also in the loop of the
  team page. Correct, and a few requests a week; reuse the first read when the code is touched.
  Checked: search `games_of(` in `crates/server/src/api/`.
  - 2026-11-08: not fixed: correct and rare; reuse the first read when the code is touched.
- **R8. Comments and docs the diff made false.** Besides A6: the doc comment of `Total` in
  `crates/core/src/contribution/duties.rs` ("done before today, planned from today"; now from the
  start time); `docs/product/duties.md` lines 57 and 64 (the page field `past`, the refusal
  `not_past`; now `started`, `not_started`); `docs/architecture.md` (migrations end at `0014`,
  the store functions of `game_attendance.rs`, the Playwright project `checked`, the release day
  set in `global-setup.ts`) at Close. Checked: search `not_past|past\b|after the date|before
  today` in `docs/` and the diff.
  - 2026-11-08: fixed in part: the doc comment of `Total`; the docs at Close.

**Overall:** R1 and R2 are bugs against the spec's intent and should be fixed before Close; R3
and R4 are breaks of the project's patterns, small to fix; the rest is for Close.
