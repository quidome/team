# Technical spec: Attendance at games

**Status**: approved
**Size**: epic
**Change**: [2026-10-31-attendance-at-games](./)
**Date**: 2026-11-01

## Objective
Each game of the team gets a list of who comes, who does not and who has not answered, with
warnings in time, a record after the game, and a mark for a game that was not played.

## Requirements summary
From `functional.md` (approved, three spec reviews, S1–S39 answered):

- **A – Who comes**: on the page of a game of the team, the players with the role "plays games" on the date, in three states ("coming", "not coming" with an optional reason, "no answer"), two buttons on each row, a tap saves at once, a second tap goes back to "no answer". Groups: "No answer", "Coming", "Not coming"; rows do not move while the page stays open. A link "Reason" opens the reasons. The count stays in view. "Extra players" 0 to 10. The count on rows of "Next", and of "Season" with at least one answer.
- **B – Warnings**: fewer than 7 (coming plus extra) is an open item from 3 days before the date until the start time; a driver who is not coming is an open item at once; a moved game with answers is an open item from 3 days before the new date until "Answers checked". Order: game list, changes not seen, too few, driver not coming, moved, schedule not entered, open places, training days not opened, trainings not checked, games not checked.
- **E – Not played**: a button on each game of the team and each other game with places of the team, coming or past, with "Undo". Duties and attendance stop counting; nothing stored changes. It holds only for the date on which it was tapped.
- **C – After the game**: "Checked" from the start time (without a start time: from the start of the game day), which makes each "no answer" absent; "Undo check" brings back only those. Words "There" and "Absent" from the start time. Open item "not checked" from the release day, in the seasons of the trainings. It also holds only for its date.
- **D – Totals**: games on "Attendance": only checked games that were played; no reason counts as not valid; extra players never count.

Stricter readings of the spec:

1. A new tap after a move does not confirm the old answers: only "Answers checked" does.
2. A game of an earlier season can be marked, but gives no warnings.
3. "Extra players" sends the new number, not a step.

## Current state
- Decision 0020: attendance at training stores only absences, with a key on the training. It stays valid for trainings; it cannot hold "coming".
- Duties hang on the game key (club, season, source id) (`game_duties`, `duty_teams`, `duty_assignments`). `game_places` sets `counts = past || (in_list && required)`, and the totals, open places and messages read it. `clashes` does not read it.
- `kept_games` keeps a gone game only for its duties.
- `team_core::open_items` is built from the rule of each source (decision 0022).
- Migrations end at `0012`. `crates/server/src/trainings.rs` is the pattern for marks; `crates/server/src/duties.rs` for the row lock of a game.

## Architecture (shared by all stories)
**Storage.** Same game key as duties.
- `game_answers`: one row for each player who answered; no row is "no answer". `answer` (`coming`, `not_coming`), optional reason, player link `ON DELETE RESTRICT`. Story C adds `settled`.
- `game_attendance`: at most one row for each game, made at the first save: `extra_players`, `answers_for`. Story E adds `not_played_for`, story C `checked_for`.

**A mark holds for a date.** Each mark stores the date of the game; the core compares it with the date of the game now. A mark with another date has no effect. A reading of the game list changes nothing stored.

**Core** `crates/core/src/game_attendance.rs` (no I/O, decision 0001):

```rust
pub enum Answer { Coming, NotComing }
pub struct StoredAnswer { pub player: PlayerId, pub answer: Answer, pub reason: Option<ReasonId> }
pub struct GameAttendance { pub extra_players: u8, pub answers_for: Option<NaiveDate> }
pub const ENOUGH_PLAYERS: usize = 7;
pub const WARNING_DAYS: u64 = 3;
```

Rules: `game_list`, `game_count` (the only count), `too_few`, `in_warning_period` (Dutch date and time), `drivers_not_coming`, `answers_moved`. Story E: `not_played` goes into `game_places`:

```rust
let counts = |required: bool| !not_played && (past || (in_list && required));
```

**Server**: a store module `crates/server/src/game_attendance.rs`, routes in `api/games.rs`, errors in a `thiserror` enum with the codes `not_a_team_game`, `not_on_list`, `reason_hidden`, `checked_too_early`.

**Concurrency.** Answers need no lock (decision 0020): one write for each tap, the last tap wins; taps of one player go one after the other in the front end. "Extra players" sends the new number. "Checked" and "Undo check" use one transaction under the row lock of the game; a tap on a settled player clears `settled`.

**Front end**: a component for the list, so the page of a game does not grow much; the tap logic of the training page.

## Stories
| # | Story | Type | Scope | Depends on | Closure | Release | Status |
|---|-------|------|-------|------------|---------|---------|--------|
| A | [Who comes to a game](../done/2026-11-01-who-comes-to-a-game/) | feature | Migration `game_answers` and `game_attendance` (`extra_players`, `answers_for`). Core `game_list`, `game_count`. Routes for an answer, a reason and extra players. The list on the page of a game, the count on rows of "Next" and "Season". `kept_games` keeps games with answers. "Delete player" refused. | – | review build + accept | 1 | done |
| B | [Warnings in time](../done/2026-11-04-warnings-in-time/) | feature | Core `too_few`, `in_warning_period`, `drivers_not_coming`, `answers_moved`. Three open items and their order. The line "Too few players", "Anna · not coming", the note with "Answers checked", the note in the choice of a driver. | A | review build + accept | 1 | done |
| E | [Not played](../done/2026-11-08-not-played/) | feature | Migration `not_played_for`. `not_played` in `game_places`, the clash rule, the messages and overviews, the warnings. The button with "Undo". "· Not played" on rows. | A | review build + accept | 2 | done |
| C | [After the game](../done/2026-11-08-after-the-game/) | feature | Migration `checked_for`, `settled`. "Checked" and "Undo check" under the row lock, the words from the start time, the closed list, the open item "not checked", "· Checked" on rows. | A, E | review build + accept | 2 | done |
| D | [Totals on "Attendance"](../done/2026-11-08-totals-on-attendance/) | feature | Games in the totals of "Attendance" (a not played game does not count: `ScheduledGame.not_played`, decision 0031), "Left the team", the texts. Every checked game that was played counts; the line about the release reads `game_check_start` (decision 0032). | C | review build + accept | 2 | done |

Each story has its own change folder (type `feature`, stage `design`) and its own `/design`, which
writes its steps, routes, screens and exact tests from this spec and `functional.md`.

**Limit of release 1**: until story E, a cancelled game still warns; the coordinator accepted this.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| No row means "no answer" | Needs no list of all players, follows role changes, as decision 0020 | Yes, at the close of story A |
| Attendance has its own tables, apart from duties | The two features stay apart; only games of the team get rows | Same record |
| A mark holds for a date, by comparing stored dates | Nothing changes by itself; survives "Seen" (S30) | Yes, at the close of story B |
| `not_played` goes through `counts`, and through the clash rule apart | One rule for what counts (decision 0022) | Yes, at the close of story E |
| E comes before C | The open item "not checked" needs a way out | No |
| Release after B | The first goal needs only A and B | No |

## Not in scope
- The driver proposal (a line in `docs/backlog.md`).
- Names of extra players.
- A setting for the number 7.
- A quick way to enter older games.
- The report of what each player did.

## Test plan
- **Existing tests that must still pass**: the core tests of `duties`, `messages`, `start_page`, `attendance`; `crates/server/tests/`; `web/e2e/`.
- **New tests** (exact tests in each story):
  - A: groups and sort, hidden and "only trains" players, the count, each tap, extra players at 0, 10 and 11, 401, last tap wins, no names in the log; two taps from the start page, rows that do not move, a phone-width and accessibility check.
  - B: the edges of `in_warning_period`, `too_few` at 6 and 7, each text form and the order of the open items, each way an item ends, `answers_moved` not cleared by "Seen", no warning for a gone game.
  - E: each case of `counts`; one test for each place that reads it (totals, open places, both messages, both overviews) and the clash rule; "Undo" gives the same totals; a mark with another date has no effect.
  - C: not before the start time; one tap settles; "Undo check" brings back only those; a race test that fails without the row lock; the open item.
  - D: only checked and played games, no extra players, no reason is not valid, renaming "No reason" changes nothing, a player with only games, "Left the team".
- **Test risk**: the browser tests need a game at a known date, and the test week has none (decision 0025). Solve this first in story A.
- While `just check` runs, nothing else builds `web/build`.

## Compatibility and performance
Forward-only migrations with new tables; no data to move. Story E changes released totals only for games marked "not played". The number of requests is no limit (coordinator).

## Progress
- [x] Story A – who comes to a game
- [x] Story B – warnings in time
- [x] Release 1
- [x] Story E – not played
- [x] Story C – after the game
- [x] Story D – totals on "Attendance"
- [ ] Release 2
- [ ] Epic review accept

## Deviations
From `functional.md`, each a choice of the coordinator; the stricter readings are in the
requirements summary.

- **Story C** (2026-11-08, also written in `functional.md`): the moment is the start time for "Checked", the words "There" and "Absent", and the duties; a not played game can be checked. See the deviations of [after the game](../done/2026-11-08-after-the-game/).
- **Story D** (2026-11-08, not written in `functional.md`): every checked game that was played counts, also one from before the release day; so "Attendance" does not say that "the games start at the release", but "Games count once they are checked. The start page asks for games from Wed 4 Nov." (in the season of the release day). See [totals on "Attendance"](../done/2026-11-08-totals-on-attendance/).
- **The release day** of the open item "not checked" is 4 November 2026, the day of release 1 (migration 0016), not the day release 2 runs: the coordinator entered answers from release 1 on (acceptance review A3).

## Review findings
### Acceptance review of the epic
Fresh review, 2026-11-08. Mode `accept`. The arguments had no other text. I read `change.md`,
`idea.md`, `functional.md` (stories A to E and the three spec reviews), this file, `docs/vision.md`,
`docs/language.md`, `docs/product/game-attendance.md` and `principles.md`, and decision 0032 (only
the part about the release day). I read the code of the result (`git diff fc2d042 HEAD`: the core
rules in `contribution/game_attendance.rs` and `output/start_page.rs`, `api/games.rs`,
`api/start.rs`, `api/season.rs`, `api/attendance.rs`, migrations 0013 to 0015,
`GameAttendance.svelte`, the page of a game, "Attendance", `web/src/lib/start.ts`,
`attendance.ts` and `trainings.ts`). I did not read the story folders in `changes/done/`. I formed
my view before reading **Deviations**, which says "None".

Runs: `cargo test -p team_core` (353 passed); `cargo test -p team --test` with `game_attendance`,
`checked`, `not_played`, `attendance_games` and `start` (65 passed); `npm run build`, then
`npx playwright test game-attendance checked not-played start` (160 passed, at phone width). I
did not open the running program by hand.

**Verdict: accepted with notes.** Each story does what the coordinator meant. The list, the two
taps, the count that stays in view, the warnings from one rule, "Not played" that changes only
what counts, and "Checked" that settles "no answer" and undoes only what it settled all match the
spec and its review answers. A1 is a small gap in story B. Fix it before release 2 as a small fix;
it needs no new discovery. A2 to A6 are notes.

| Criterion or open point | Built as meant? | Kind |
|---|---|---|
| A: list, states, taps, groups, rows that do not move, "Reason", the count in view, "Extra players", the count in "Next" and on "Season" | yes | – |
| A: a move keeps the answers, with the note and "Answers checked" | yes; the open item ends at the start time, which the spec left open | faithful adaptation |
| B: too few, driver not coming, moved; their order; one rule | yes, except a checked game without a start time | A1 |
| B: "Anna · not coming" / "· absent" at the place | yes; only while the place is "as planned" | faithful adaptation |
| C: words, "Checked" from the start time, settle, "Undo check", a check for one date, the open item, Someone else and Nobody from the start | yes | – |
| C: "from the day of the release" | the release day of "Checked" (release 2), not of the list (v0.8.0) | A3 |
| D: games in "Totals", no reason is not valid, missed games, the texts | yes; every checked game counts, also from before the release | A2 |
| E: "Not played" on every game page, counts nothing, clashes, "The drivers drove", one date, "Undo" | yes | – |

#### Findings
- **A1. A checked game without a start time still warns all of its game day.** Story B: an item is for a game "that is not checked", and it "goes away by itself when ... the game is checked". For a game without a start time, "Checked" opens at the start of the game day (`started` in `events/game.rs`, used by `check_game_checking`). The warnings, however, last until the end of that date (`start_of` in `contribution/game_attendance.rs`). `game_warnings` does not take the check into account, and `api/start.rs` computes warnings for every coming game of the team, and a game of today is coming (test `a_game_today_is_coming_and_a_game_yesterday_is_past`). So on that day, after "Checked", the start page can still show "Game Sat 31 Oct: 5 of 7 players" or "driver Anna is not coming". The line "Too few players" also comes back after "Show the list". This only happens for a game without a start time. The overlap came from the coordinator's choice of 2026-11-08 ("Checked" from the start of the game day), which story B did not follow. The fix is likely one condition in `game_warnings` (no warnings when `checked(attendance, date)`), with a test. Kind: **concerning deviation**, small. Checked: a trace of `game_warnings`, `before_start`, `start_of`, `started` and `api/start.rs`; a search for `start_time` in `crates/server/tests/checked.rs` found no test for this case; not run.
- **A2. Story D counts every checked game, also from before the release, and "Attendance" says something else than the spec.** Functional.md, story D: "'Attendance' says that the games start at the release"; story A: "the totals start at the release". The build counts every checked game that was played (`counted_games`, with no date limit). "Attendance" says "Games count once they are checked. The start page asks for games from …" (`gamesFromText`). This follows the coordinator's choice in the log of `change.md` (2026-11-08), and it serves journey 2 better: an older game the coordinator checks by hand is not lost. But functional.md never got this change, unlike the dated choices in stories C and E, and **Deviations** above says "None". Kind: **faithful adaptation**. The record should say so. Checked: a read of `functional.md`, the `change.md` log, `counted_games` and `web/src/lib/trainings.ts`.
- **A3. Games between release 1 and release 2 are "before the release".** `game_check_start` gets the date on which migration 0015 runs, which is the day of release 2. Since v0.8.0 (2026-11-04) the coordinator has entered answers in the tool, for example for the games of 7 and 8 November. Those games never become an open item "not checked", their list opens closed, and they count on "Attendance" only when the coordinator checks them by hand. The spec says "from the day of the release on" without saying which release. It was written when A, B and C were planned for one release. Kind: **neutral drift**. After release 2 the coordinator may want to check those few games by hand. Checked: migration `0015_game_checks.sql`, `games_not_checked`, `docs/releases.md`; the number of such games is not counted.
- **A4. "Checked" is the last button on the page.** On a past game that is not checked, the open item "Game … not checked" opens a page where the list is open. Below it come the table and referee places, the players on the places and the drivers form, and only then "Checked". Story C asks for "one tap", and the spec does not say where the button goes. So this is one tap after a scroll, on a phone. Kind: **neutral drift**; a point to watch in real use (Learn). Checked: a read of `web/src/routes/game/[season]/[id]/+page.svelte`; not tried on a phone.
- **A5. This file's requirements summary is out of date for story C.** It says "Checked" "without a start time: the day after" and the words "after the date". Functional.md (coordinator, 2026-11-08) and the build use the start of the game day and the start time. This change document becomes history at Close, so it does no harm. Kind: **neutral drift**. Checked: a read of both documents and of `started`.
- **A6. The vision still speaks of attendance at games as future work.** Journey 8 says the report "comes before attendance at games: until then it shows no games". "Where we are going" lists attendance at games among the next steps, and "Where we are" ends with "(priorities 7 and 8)" after the report. This is for the Vision stage, not for Close. Kind: **neutral drift**. Checked: a read of `docs/vision.md`.

- A1, 2026-11-08: fixed: `game_warnings` gives no warning for a game checked on its date; core test `a_checked_game_without_a_start_time_gives_no_warning_on_its_day`.
- A2, 2026-11-08: fixed: written under Deviations.
- A3, 2026-11-08: fixed: migration `0016_game_check_start_from_release_1` sets the release day to 4 November 2026; `tests/checked.rs` checks the date.
- A4, 2026-11-08: not fixed: to watch in real use.
- A5, 2026-11-08: fixed: the requirements summary and the row of story C say "from the start time".
- A6, 2026-11-08: not fixed here: for `/vision`, with the word "checked" in `language.md`.

Faithful adaptations that need no action: the reasons also close at a tap on "Coming" or "Not
coming"; a hidden reason stays offered on a row that already has it; a game with only extra
players is not kept after it left the game list; a not played game shows no note of a move or of
a driver who is not coming. Each follows the intent of the criterion it fills in.
