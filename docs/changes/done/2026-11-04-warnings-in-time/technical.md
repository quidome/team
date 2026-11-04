# Technical spec: Warnings in time

**Status**: done
**Size**: story
**Change**: [2026-11-04-warnings-in-time](./)
**Date**: 2026-11-04

## Objective
Warn the coordinator in time, on the start page and on the page of the game, when a coming game
of the team has too few players, when a driver does not come, and when its answers are from
before a move, so the coordinator can act before the game (the first goal of the epic).

## Requirements summary
From `../2026-10-31-attendance-at-games/functional.md` (approved; story B, and the move in the
edge cases of story A), with the answers of the coordinator of 2026-11-04:

- **Too few players.** From 00:00 three days before the date of a game of the team until its
  start time (no known start time: until the end of that date), Dutch time, a game that is in the
  game list with fewer than 7 coming (players coming plus extra players) is an open item:
  "Game Sat 31 Oct: 5 of 7 players, 3 no answer"; without "no answer": "Game Sat 31 Oct: 5 of 7
  players". More games: "2 games: too few players", and a tap opens the list of those games below
  the item. The page of the game shows "Too few players: 6 of 7" above the count by the same rule
  (the same 7, the same end), from 3 days before the game, and earlier once the game has an
  answer; only the open item waits for the 3 days (story A, criterion "A coming game with fewer
  than 7 …"; Q1, answered again on 2026-11-04).
- **Driver not coming.** A coming game of the team with a planned player on a driver place whose
  answer is "not coming" (and who is on the list of that date) is an open item at once, also more
  than 3 days before, until the start time: "Away game Sat 31 Oct: driver Anna is not coming";
  two drivers: "Away game Sat 31 Oct: 2 drivers are not coming"; more games: "2 games: a driver is
  not coming", with a list as above. The page of the game shows "Anna · not coming" at the driver
  place; on a past game "Anna · absent". In the choice of a driver, such a player stays a choice,
  with the note "Not coming". The tool never removes a driver.
- **Moved: check the answers.** When a game has answers and they were given for another date
  than its date now, the page says "Answers from before the move to Wed 4 Nov." with the button
  "Answers checked"; only that tap ends the note ("Seen" in the list of changes does not). From
  3 days before the new date until its start time (Q2, see Deviations) it is an open item:
  "Game Wed 4 Nov moved: check the answers".
- **A tap on an item for one game** opens the page of that game; "Back" returns to the start page.
- **An item goes away by itself** when its problem is gone: the 7 is reached, the driver is
  "coming" or "no answer" again, the driver place is cleared or has another player, the period
  ended, or "Answers checked".
- **Order of the open items**: the game list; changes not seen; too few players; driver not
  coming; moved: check the answers; home games with the schedule not entered; games with open
  places; training days not opened; trainings not checked. ("Games not checked" follows in story C.)
- **No warning** for a game that left the game list, a game that is no longer a game of the team,
  a game of an earlier season, a driver with "no answer", a driver with a stored "not coming" who
  is not on the list of that date, and table and referee places.
- **One rule**: the page of the game and the open item give the same answer (decision 0022).
- Until story E ("Not played"), a cancelled game that stays in the game list still warns
  (accepted by the coordinator for release 1, confirmed on 2026-11-04, Q3).

## Current state
- Story A (`../done/2026-11-01-who-comes-to-a-game/`, decision 0028): core
  `crates/core/src/contribution/game_attendance.rs` with `Answer`, `StoredAnswer`,
  `GameAttendance { extra_players, answers_for }`, `ListRow`, `GameCount`, `game_list`,
  `game_count` (the only count), `row_count`, `check_answer`. Store
  `crates/server/src/game_attendance.rs`: the first answer of a game sets `answers_for` to the date
  of the game; a later answer leaves it (`answer`). `of_game`, `of_season` (`SeasonAttendance`
  with `answers` and `games` by source id).
- The page of a game: `crates/server/src/api/games.rs`, `page`, with `GameAttendanceView` (the
  list) and `AttendanceView` (the answer after a tap: the row, the count, the extra players). The
  choice of a player: `ChoiceView` from `choice`. The places: `PlaceView` with
  `PlayerOnPlaceView` (`api/season.rs`).
- `SeasonDuties` (`api/season.rs`) holds the duties, assignments (`Assignment { kind, number,
  planned, outcome }`), roles, players, names and the attendance of a season; `row_count` builds
  the list of a game from it.
- The start page: core `crates/core/src/output/start_page.rs` (`OpenItem`, `open_items`), server
  `crates/server/src/api/start.rs` (`OpenItemView`, `GameItemView`), front end `web/src/lib/start.ts`
  (words and links of the open items).
- Dutch time: `clock::in_netherlands(clock.now())` gives a `NaiveDateTime`; the core has no clock.
- The domain map (`docs/architecture.md`, decision 0027): contribution may use team, events and
  shared; output may use all domains. No new use between domains is needed.
- Browser tests: the clock is Monday 12 October 2026 (`DEV_NOW`), and the test data has no game of
  the team from 12 to 15 October, so a real warning period cannot be reached there.

## Steps

### Step 1: The rules of the warnings
- **Goal**: one rule for the warnings of a game, in the core.
- **Changes**: in `contribution/game_attendance.rs`:

```rust
/// Fewer coming than this is too few.
pub const ENOUGH_PLAYERS: usize = 7;
/// The warnings of "too few" and "moved" start this many days before the date, at 00:00.
pub const WARNING_DAYS: u64 = 3;

/// What a game of the team warns about at `now`.
pub struct GameWarnings {
    /// The open item: in the warning period and fewer than `ENOUGH_PLAYERS` coming: the count.
    pub too_few: Option<GameCount>,
    /// The line on the page: fewer than `ENOUGH_PLAYERS` coming, before the start, and in the
    /// warning period or once the game has an answer (`has_answers`).
    pub too_few_line: bool,
    /// Before the start: the planned players on driver places with the answer "not coming".
    pub drivers_not_coming: Vec<PlayerId>,
    /// In the warning period: answers from before a move (`answers_moved`).
    pub moved: bool,
}

/// From 00:00 `WARNING_DAYS` days before the date until the start time (not included); without a
/// start time until the end of the date. `now`: Dutch local time.
pub fn in_warning_period(game: &Game, now: NaiveDateTime) -> bool;
/// Before the start time (without a start time: before the end of the date).
pub fn before_start(game: &Game, now: NaiveDateTime) -> bool;
/// The planned players on the driver places whose answer on `list` is "not coming", in the
/// order of the places.
pub fn drivers_not_coming(list: &[ListRow], assignments: &[Assignment]) -> Vec<PlayerId>;
/// The game has answers (`has_answers`: at least one stored answer, also of a player off the
/// list), and they were given for another date than `date`.
pub fn answers_moved(attendance: &GameAttendance, has_answers: bool, date: NaiveDate) -> bool;
/// All warnings of a game of the team; none for a game that is not in the game list.
pub fn game_warnings(
    game: &Game,
    in_list: bool,
    list: &[ListRow],
    attendance: &GameAttendance,
    has_answers: bool,
    assignments: &[Assignment],
    now: NaiveDateTime,
) -> GameWarnings;
```

  `game_warnings` uses `game_count` and one comparison with `ENOUGH_PLAYERS` for both
  `too_few` and `too_few_line`, so they differ only in their start. Only `Assignment.kind == DutyKind::Drive` and
  its `planned` player count; the outcome does not (a correction exists only after the game).
- **Tests**:
  - `in_warning_period`, game Sat 31 Oct 2026 14:30: false Tue 27 Oct 23:59; true Wed 28 Oct
    00:00; true Sat 31 Oct 14:29; false at 14:30. Without a start time: true Sat 31 Oct 23:59;
    false Sun 1 Nov 00:00.
  - `before_start`: true 10 days before; false at the start time.
  - `too_few` through `game_warnings`: 6 coming gives the count; 7 gives `None`; 5 players and
    2 extra players is 7; outside the period `None`.
  - `too_few_line`: 5 days before with an answer: true, and `too_few` is `None`; 5 days before
    without an answer: false; in the period without an answer: true; at the start time: false;
    with 7 coming: false.
  - `drivers_not_coming`: a driver "not coming" is found; "coming", "no answer", a table or
    referee place, and a stored "not coming" of a player off the list give nothing; two drivers
    give both, in the order of the places.
  - `answers_moved`: false without answers; false with the same date; true after a move; true
    when all answering players are off the list; false again when the stored date is set to the
    date of the game.
  - `game_warnings`: a game not in the game list gives none; 10 days before only the driver
    warning; each warning ends at the start time.

### Step 2: The three open items
- **Goal**: the start page shows the three warnings, in the order of the spec.
- **Changes**:
  - `output/start_page.rs`: `OpenItem` gets
    `TooFewPlayers { games: Vec<(i64, GameCount)> }`,
    `DriverNotComing { games: Vec<(i64, Vec<PlayerId>)> }`, `AnswersMoved { games: Vec<i64> }`,
    each with its games in time order. `open_items` gets one more input, the warnings of the
    coming games of the team as `&[(i64, GameWarnings)]` in time order, and places the three
    items after "changes not seen"; an item without games is left out.
  - Server `api/season.rs`: `SeasonDuties::warnings(&self, scheduled: &ScheduledGame, now)`
    builds the list as `row_count` does and calls `game_warnings` (with `has_answers` from
    `attendance.answers`).
  - Server `api/start.rs`: the warnings of the coming events of the current season that are
    games of the team (`ScheduleReason::TeamGame`), with `now = in_netherlands(clock.now())`.
    `OpenItemView` gets `too_few_players { games: [{ ...GameItemView, coming, no_answer, enough }] }`
    (`enough` = `ENOUGH_PLAYERS`), `driver_not_coming { games: [{ ...GameItemView, drivers: [name] }] }`
    (names from `SeasonDuties::names`) and `answers_moved { games: [GameItemView] }`. `just api-types`.
  - Front end `web/src/lib/start.ts` and the open items of the start page: the texts of the
    requirements, the list of games below an item for more games (as the item of open places), and
    a link to the page of the game for one game (with `?from=start`, as the other game items, so
    "Back" returns to the start page).
  - No log line holds a name: the names are only in the JSON.
- **Tests**:
  - Core `start_page`: the order of all items; each new item keeps the time order of its input;
    an item without games is left out.
  - Server `crates/server/tests/start.rs` (the clock is set in each test; the games are after
    25 October, so the change to winter time is crossed once):
    - 3 days before a game of the team, 5 "coming" and 2 "no answer": `too_few_players` with 5,
      2 and 7. One day earlier: none. At the start time: none. With 2 extra players: none.
    - A driver "not coming", 10 days before: `driver_not_coming` with the name; none after
      "coming" again; none after the driver place is cleared.
    - The game moved in the fake game list, then a reading: `answers_moved` only from 3 days
      before the new date; "Seen" does not end it.
    - A game that left the game list gives no item.
  - Server, a new test file with `LogBuffer` (`tests/common/`): `GET /api/start` with a driver
    "not coming" writes no player name to the log.
  - Browser `web/e2e/start.e2e.ts`, with `/api/start` replaced (`page.route`, as
    `messages.e2e.ts`): each text (one game with and without "no answer", 2 games with the list,
    one and two drivers, the move); a tap on an item for one game opens its page; "Back"
    returns to the start page.

### Step 3: The page of a game: too few players, and drivers who do not come
- **Goal**: the page of a game shows the same facts as the start page.
- **Changes**:
  - `GameAttendanceView` and `AttendanceView` get `too_few: bool` (`GameWarnings::too_few_line`,
    the same rule as the item) and `enough_players: usize`. `now` is passed to the page and to each tap.
  - Front end (the component of the list): the line "Too few players: 6 of 7" above the count
    when `too_few`. At a driver place: "Anna · not coming" when the player's row in
    `attendance.rows` has the answer "not coming", "Anna · absent" when the game is `past`. In the
    choice of a driver: the note "Not coming" for such a player; the player stays a choice. Both
    read the answers in `attendance.rows`, which each tap keeps up to date; there is no new rule.
- **Tests**:
  - Server `crates/server/tests/game_attendance.rs`: 5 days before the game, after the first
    answer, the answer of the tap and the page give `too_few` while the start page has no item;
    in the period both have it; at the start time neither; `enough_players` is 7.
  - Browser `web/e2e/game-attendance.e2e.ts`: with real data, a player on a driver place who
    taps "Not coming" shows "· not coming" at the place at once, and the choice of a driver shows
    "Not coming"; with the page answer replaced, "Too few players: 6 of 7"; phone width and
    accessibility of the page with the new lines.

### Step 4: A game that moved
- **Goal**: the coordinator sees answers from before a move and confirms them.
- **Changes**:
  - `GameAttendanceView` and `AttendanceView` get `answers_moved: bool` (`answers_moved`, with no
    period: the note shows until "Answers checked").
  - Store `crates/server/src/game_attendance.rs`: `answers_checked(db, key, date)`: the row of the
    game (insert with `ON CONFLICT DO NOTHING`), then `answers_for = date`. One write; no lock:
    a second tap writes the same date.
  - Route `PUT /api/games/{season}/{id}/answers-checked` (`api/games.rs`), answer
    `AttendanceView` with `row: null`; `400 not_a_team_game` for another game, as the other
    routes of answers; a log line with only the season and the game id.
  - Front end: the note "Answers from before the move to Wed 4 Nov." with the button "Answers
    checked", above the list; a tap that cannot be saved shows "Not saved. Try again.", as a tap
    on an answer.
- **Tests**:
  - Server `crates/server/tests/game_attendance.rs`: after a move, the page has `answers_moved`
    also outside the period; `answers-checked` sets `answers_for` to the date of the game and
    ends `answers_moved` on the page and the item; a second call gives the same answer; another
    game answers 400 `not_a_team_game`; without a login 401.
  - Browser `web/e2e/game-attendance.e2e.ts`: with the page answer replaced, the note shows; a
    tap on "Answers checked" calls the real route and the note goes away.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| One rule `game_warnings` for the page of a game and the start page | The spec asks for one rule three times (decision 0022); the period cannot differ (review finding S22 of the spec) | No (follows 0022) |
| A mark holds for a date: a move is a stored `answers_for` that is not the date of the game now | Nothing changes by itself; survives "Seen" (S30) | Yes, at the close of this story (planned in the epic) |
| The core gets the Dutch local time from the server | The core has no clock (decision 0001); the change to winter time stays in the server | No |
| The notes "not coming" at a driver place and in the choice read the answers already on the page | The answer of a player is data, not a rule; each tap keeps it up to date | No |
| "Answers checked" needs no lock | One write, the same result for a second tap (as decision 0020 for taps) | No |

## Not in scope
- "Games not checked", and the end of a warning at "Checked" (story C).
- "Not played", which would end the warnings of a cancelled game (story E).
- The driver proposal (a line in `docs/backlog.md`).
- A setting for the number 7 or the 3 days.
- Warnings for table and referee places.

## Test plan
- **Existing tests that must still pass**: the core tests of `game_attendance`, `duties` and
  `start_page`; `crates/server/tests/game_attendance.rs`, `start.rs`, `games.rs`, `duties.rs`;
  `web/e2e/game-attendance.e2e.ts`, `start.e2e.ts`, `duties.e2e.ts`.
- **New tests**: per step, see above.
- **Gap**: the browser tests cannot reach a real warning period (fixed clock, no game in the
  3 days after it); the period is tested in the core and server tests, the texts in the browser
  with replaced answers.
- While working: `cargo test -p team_core game_attendance`, `cargo test -p team_core start_page`,
  `cargo test -p team --test start`, `cargo test -p team --test game_attendance`,
  `cd web && npx playwright test start game-attendance`. Once before a commit: `just check`.

## Compatibility and performance
No migration: `answers_for` exists since story A. The start page loads the duties and the
attendance of the current season already; the warnings add one list for each coming game of the
team (about 20), in memory. The API gets new kinds of open items; the front end is released with
it. The open items of the start page now hold player names (only in the JSON, never in a log).

## Progress
- [x] Step 1 (2026-11-04): `ENOUGH_PLAYERS`, `WARNING_DAYS`, `GameWarnings` (with `too_few_line`), `before_start`, `in_warning_period`, `drivers_not_coming`, `answers_moved`, `game_warnings` in `contribution/game_attendance.rs`; 9 new core tests. The case "all answering players off the list" of `answers_moved` is a comment, not a separate case: the function reads `has_answers`, not the list.
- [x] Step 2 (2026-11-04): `OpenItem::TooFewPlayers`, `DriverNotComing`, `AnswersMoved` and their place in `open_items` (a new last input, the warnings in time order); `SeasonDuties::list_of` (shared by `row_count`) and `warnings`; `GET /api/start` with `too_few_players`, `driver_not_coming` (names) and `answers_moved`, from the coming games of the team of the current season; the texts, the list below an item and the links in `web/src/lib/start.ts` and `+page.svelte`. Tests: 2 core tests of the order; 6 server tests in `tests/start.rs` (the period of "too few" with its edges at 23:59/00:00 and the start time, extra players, the driver at once and its ends, the driver at the start time, the move with "Seen", a game that left the list); `tests/warning_logs.rs`; 3 browser tests in `start.e2e.ts`.
- [x] Step 3 (2026-11-04): `too_few` and `enough_players` in `GameAttendanceView` and `AttendanceView`, from `game_warnings` on the list stored after the save (`stored_list` in `api/games.rs`); the line "Too few players: 6 of 7" in `GameAttendance.svelte`; "· not coming" / "· absent" at a driver place and "Not coming" in the choice of a driver on the page of a game. Tests: `too_few_on_the_page_starts_earlier_once_the_game_has_an_answer` (server); 2 browser tests in `game-attendance.e2e.ts` (the line with a replaced answer; a driver who taps "Not coming", at the place and in the choice, with the data put back). Not in a browser test: "· absent" on a past game (one condition on `past`, which the page already has).
- [x] Step 4 (2026-11-04): `answers_moved` in `GameAttendanceView` and `AttendanceView` (no period); store `game_attendance::answers_checked` (one upsert of `answers_for`; `.sqlx/` updated); route `PUT /api/games/{season}/{id}/answers-checked` (`setAnswersChecked`), answer `AttendanceView` with `row: null`; the note "Answers from before the move to …" with "Answers checked" in `GameAttendance.svelte` (the queue of the taps, so a failure shows "Not saved. Try again."; the component gets the date of the game). Tests: 3 server tests (the note at once and after a new answer, the item, "Answers checked" with a second call; `not_a_team_game`; 401); 1 browser test (the note with a replaced answer, the tap on the real route).
- [x] All checks pass (`just check`, 2026-11-04; `api-types-check` compares with the git index, so the generated types were staged first)

## Deviations
- **Step 3, 2026-11-04: how the page gets the answers.** The design says the notes at a driver place and in the choice read `attendance.rows`, "which each tap keeps up to date". That is not true: a tap changes only the state inside `GameAttendance.svelte`, not the data of the page. The component now gives the page the answer shown of each player through a callback (`onanswers`), and the notes read that. No change for the coordinator; the notes still need no rule of their own.
- **Detail not named by the spec (step 2, 2026-11-04).** The spec gives the item of a move only for one game. For more games the text is "2 games moved: check the answers", as the other items of more games, with the list below it.
- **D1 (Q2, coordinator 2026-11-04).** The spec (story A, edge cases) says only "Answers
  checked" ends the open item "Game Wed 4 Nov moved: check the answers". Here the open item ends
  at the start time of the game, as the other two warnings; the note on the page of the game stays
  until "Answers checked". Reason: a moved game that is over would otherwise stay an open item
  until a tap that no longer helps the game.
- ~~Stricter reading (Q1)~~: withdrawn on 2026-11-04 after the spec review below. Q1 was asked
  as if the spec left the period of the line open, but story A's criterion gives it. The
  coordinator chose the spec: the line shows from 3 days before, and earlier once the game has an
  answer. No deviation is left for the line.

### Spec review of the deviations
Fresh review, 2026-11-04, of the two deviations above only. Read: `functional.md` of the epic
(stories A to E and its three reviews), the epic's `technical.md` (stories and releases),
`docs/vision.md`, `docs/language.md`, `docs/product/game-attendance.md` and `start-page.md`, and
this file. Not the steps of this design, and no code.

| Deviation | What the user is doing here | Verdict |
|---|---|---|
| D1: the item "moved: check the answers" ends at the start time | A look at the start page in the days before a moved game: are the answers I count still the answers for this date? | serves |
| Q1: "Too few players" on the page only in the warning period | On the page of a game early in the week, tapping in answers from the chat | serves – note |

- **D1 serves.** The item exists for the first goal: act before the game. After the start time
  a check of the answers can no longer get a player to the game, so an item that stays would
  be the kind of noise S2 and S9 warned about, on the start page that should be read in a few
  seconds. The guard for the record is kept: the note with "Answers checked" stays on the page,
  and the answers count in the totals only once the game is checked (story D), which happens on
  that same page (story C), so a stale answer cannot count without the coordinator passing the
  note. The deviation is also smaller than it says: story A's "Only that tap ends it" is written
  about the note, the item sentence gives no end, and story B ends each item "when the warning
  period has ended", so D1 is the reading that fits story B. One gap, harmless: stories C and D
  come in release 2, so in release 1 a moved game whose start time has passed is no item at all
  and nothing on the start page points to its note; no total depends on it until then. Checked:
  read of story A (edge cases), story B (criteria 1 and 4), story C, story D and the release
  column of the epic's `technical.md`.
- **Q1 serves – note.** Early in the week the count above the list ("4 coming · 5 no answer")
  already gives the number, and a "Too few players" line on every game with one answer, days
  before anyone could act, is the noise S6 raised; one period for the page and the item is
  simpler to trust. What is lost is small: the target "of 7" before Wednesday. The note is on
  the text of the deviation: it is not a stricter reading of a mockup. Story A's criterion says
  outright "and earlier once the game has an answer" and "only the open item waits for the 3
  days" (the fix of S6 and S22), so this changes an acceptance criterion the coordinator
  approved. The coordinator confirmed it (Q1), so it may stand, but call it a deviation from
  story A's criterion, and at Close `docs/product/game-attendance.md` and the epic's story A
  must say the line shows only in the warning period. Checked: read of story A, criterion
  "A coming game with fewer than 7 …", against findings S6 and S22.
- Not judged, as they are not listed as deviations: the end of the items at "Checked" (left to
  story C under "Not in scope") and the warnings of a cancelled game until story E (accepted in
  the epic).

## Review findings

Answers, 2026-11-04:
- D1 (serves): kept. The gap in release 1 (a moved game after its start time has no item) is accepted; stories C and D close it.
- Q1 (serves – note): the deviation is withdrawn. The design follows story A's criterion (`too_few_line`); Requirements, step 1 and step 3 changed. Nothing for Close.


### Build review
Fresh review, 2026-11-04, of the diff against `main` (committed and not), against
`docs/architecture.md`, the valid decisions (0001, 0020, 0022, 0027, 0028) and `CLAUDE.md`.
Runs: `cargo test -p team_core` (323 + 7 pass), `cargo test -p team --test start --test
game_attendance --test warning_logs` (16, 17, 1 pass). Not run: the browser tests and `just check`.

Checked with no finding: the domains (`contribution::game_attendance` uses `events::game` and
`contribution::duties`; `output::start_page` uses contribution and team; all allowed by 0027,
read of the `use` lines); one rule (the page and `GET /api/start` both call `game_warnings`, with
`has_answers` from the stored answers in both, trace of `stored_list` and `SeasonDuties::list_of`);
the order of the open items against the spec (read of `open_items` and its core test); the item
games and `by_id` come from the same `games.schedule.coming`, so no item can lose its games
(trace); a game that left the list, a kept game and a game of another team give no warning (trace:
`in_list`, `ScheduleReason::TeamGame`); after all answers are cleared, the next first answer sets
`answers_for` again, so no false "moved" (read of `game_attendance::answer`); the new log line has
only season and game id (read; `tests/warning_logs.rs` covers the start page).

- **R1 – "· absent" reads as the wrong player after a correction.** On a past game the suffix is
  added after `playerText(on)`, but `notComingDriver` tests `on.id`, the planned player. With
  "Someone else" the place shows "Bram (was Anna) · absent" and with "Nobody" "not done (was Anna)
  · absent": it reads as if Bram, or nobody, was absent. The spec gives only "Anna · absent".
  Either show the suffix only with `outcome === 'as_planned'`, or put it next to the planned name.
  There is no test of "· absent" (Progress, step 3, says so). Checked: trace of
  `web/src/routes/game/[season]/[id]/+page.svelte` (the place line) and `playerText` in
  `web/src/lib/duties.ts`; not run.
- **R2 – `answers_checked` has no test of a same-key insert at the same moment.** It is an
  `INSERT … ON CONFLICT … DO UPDATE` on a row that may not exist yet. `docs/architecture.md`
  ("A save guarded by a unique index or `ON CONFLICT` has a test that holds an insert of the same
  key in an open transaction on another connection") asks for that test; the design's "no lock"
  does not remove it. The upsert itself is safe in PostgreSQL, so the risk is low; the finding is
  the broken pattern. Sibling: story A's `set_extra` and `answer` (both `ON CONFLICT` on
  `game_attendance`) have no such test either. Checked: search `pool.begin` in
  `crates/server/tests/` (no hit in `game_attendance.rs`; hits in `trainings.rs`, e.g.
  `open_twice_gives_one_training`).
- **R3 – the order check in `too_few_players_is_an_open_item_from_3_days_until_the_start` cannot
  fail as written.** `at("too_few_players") < at("schedule_not_entered").or(Some(usize::MAX))`
  passes whenever "schedule not entered" is absent, and nothing checks "after the changes not seen"
  that the comment names. The order is tested for real in the core
  (`the_warnings_come_after_the_changes_and_before_the_schedule`), so this is a weak test, not a
  gap. Check the position of both neighbours, or drop the line. Checked: read of the test; not
  checked whether the item "schedule not entered" is there at that moment.
- **R4 – for Close: documents the diff made false.** `docs/architecture.md`: the `start_page`
  part of "Domains of the core" lists the order of `open_items` without the three new items;
  "Attendance at games" does not name `game_warnings`, `answers_moved`, `answers_checked`, the
  route `PUT …/answers-checked`, or the new fields of `GameAttendanceView`/`AttendanceView`; the
  module list of `api/` and `game_attendance.rs` is unchanged. Decision 0028, Consequences, says
  the rule of `answers_for` "gets its record at the close of story B" (the Decisions table here
  plans it too). `docs/product/game-attendance.md` and `start-page.md` (not read) will need the
  warnings. Checked: search of `docs/architecture.md` for `open_items` and "Attendance at games";
  read of decision 0028.

Answers to the build review, 2026-11-04:
- R1: fixed. The note of a driver is one function, `driverNote` in `web/src/lib/duties.ts`: only for the outcome "as planned", so "Bram (was Anna)" and "not done (was Anna)" get no note. The browser test of the driver replaces the page answer as a past game: "Anna · absent" with "as planned", "not done (was Anna)" without a note after "Nobody".
- R2: fixed for this story. New test `answers_checked_waits_for_a_first_save_on_another_screen`: an insert of the same game in an open transaction on another connection; the save waits (300 ms), then answers 200, and both saves are kept. Checked by hand: without `ON CONFLICT` the test fails with 500; restored. Not fixed: the same test for `set_extra` and `answer` of story A (outside this change; a line for the backlog).
- R3: fixed. The order check in `too_few_players_is_an_open_item_from_3_days_until_the_start` now has a change not seen (another game gets a new start time) and checks that "too few players" is after each earlier item and before each later item, with an item on both sides.
- R4: for Close (architecture, the decision record "A mark holds for a date", `docs/product/game-attendance.md` and `start-page.md`).


### Acceptance review
Fresh review, 2026-11-04, of the result against the epic's story B and the move in story A
(`../2026-10-31-attendance-at-games/functional.md`), its `idea.md`, and the requirements and
objective above. Read: the diff against `main` (committed and not). Runs: `cargo test -p team_core`
(323 + 7 pass); `cargo test -p team --test start --test game_attendance --test warning_logs` (17,
17, 1 pass); `npx playwright test start game-attendance` (108 pass). Not run: the app by hand, and
`just check`. The view below was formed from the spec and the code before the Deviations were
weighed.

| Criterion (story B unless named) | Built | Fit |
|---|---|---|
| "Too few" item from 00:00 3 days before until the start (no start time: end of date), Dutch time, with both texts and "2 games: too few players" with a list | `in_warning_period`, `game_warnings`, `tooFewText`; server tests at 23:59/00:00 and the start, across winter time | matches |
| Driver "not coming" at once until the start; one, two drivers; more games with a list | `drivers_not_coming` in place order, `driverText` | matches |
| A tap on an item for one game opens it; "Back" to the start page | `gameLink` with `?from=start` for all three; browser test | matches |
| An item goes away by itself (7 reached, driver back, place changed, period over) | each item is recomputed from the stored state on each load; server tests for each end | matches; "game is checked" waits for story C |
| "Anna · not coming" at the place, "Anna · absent" on a past game | `driverNote` | A2 |
| "Not coming" in the choice; the driver stays a choice; never removed | `choice-not-coming`; no write to the places | matches |
| Order of the open items | `open_items`; core test of all neighbours | matches ("games not checked" comes in story C) |
| One rule of "too few" for the page and the item | both call `game_warnings` with the same list and `has_answers` (trace of `stored_list` and `SeasonDuties::list_of`) | matches |
| No warning: left the list, not a team game, "no answer", off the list, table/referee | `in_list`, `ScheduleReason::TeamGame`, list lookup, `DutyKind::Drive` | matches |
| Story A: "Too few players: 6 of 7" above the count, 3 days before and earlier once answered, until the start | `too_few_line`, sticky header above the count | matches |
| Story A: the move note with "Answers checked"; "Seen" does not end it; the item from 3 days before the new date | `answers_moved`, `PUT …/answers-checked`, server test with "Seen" | A1, A3 |
| Logs hold no names | `tests/warning_logs.rs`; the one new log line has season and game id only | matches |

- **A1 – faithful adaptation: the text for more moved games.** The spec gives the move item only
  for one game; "2 games moved: check the answers" with a list below follows the other two items.
  Checked: read of `openItemText`; browser test of more games.
- **A2 – faithful adaptation: "· absent" only while the planned driver did the duty.** After
  "Someone else" or "Nobody" the note goes, so "Bram (was Anna)" is not read as Bram being absent.
  The spec's purpose ("so the coordinator can use Someone else or Nobody") is served: the note
  shows until the coordinator acts, then the correction speaks. Checked: read of `driverNote`;
  browser test of the past game.
- **A3 – neutral drift, accepted: the move item ends at the start time (D1).** The note on the
  page stays until "Answers checked", which keeps the guard on the record. In release 1 nothing on
  the start page points to the note of a moved game after its start time; stories C and D close
  that. Checked: core test `the_warnings_of_a_game_follow_their_periods`; read of the page fields
  (`answers_moved` has no period).
- **A4 – neutral drift: the move note shows on any game of the team with moved answers**, also a
  past game or one that left the game list, while the item follows the warning period and the game
  list. The spec ties the note only to "the game has answers", so this is a reading, not a change.
  Checked: read of `stored_list` (`answers_moved` without `in_list` or a period).
- **A5 – known gaps of release 1, not deviations of this story:** an item does not end at
  "Checked" (story C), and a cancelled game still warns (story E; Q3). Both are in "Not in
  scope" and were accepted by the coordinator.
- **A6 – for Close:** `docs/product/game-attendance.md` and `start-page.md` must describe the three
  items, the line, the driver notes and "Answers checked" (also R4). Checked: not read; R4 names it.

No concerning deviation found. The browser tests cannot reach a real warning period (fixed clock);
the period itself is covered by the core and server tests, which I ran.

**Verdict: accepted with notes.**

Answers to the acceptance review, 2026-11-04:
- A1: no change needed; recorded under Deviations (detail not named by the spec).
- A2: no change needed; this is the fix of build finding R1.
- A3: no change needed; deviation D1, accepted by the coordinator and the spec review; stories C and D close the gap.
- A4: no change needed. The spec ties the note to "the game has answers"; a past game or a game that left the list with answers from before a move still has old answers, and the note says so until "Answers checked".
- A5: no change needed; stories C and E, accepted for release 1.
- A6: for Close, with R4.

