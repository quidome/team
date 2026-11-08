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
- **C – After the game**: "Checked" from the start time (without a start time: the day after), which makes each "no answer" absent; "Undo check" brings back only those. Words "There" and "Absent" after the date. Open item "not checked" from the release day, in the seasons of the trainings. It also holds only for its date.
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
| C | After the game | feature | Migration `checked_for`, `settled`. "Checked" and "Undo check" under the row lock, the words after the date, the closed list, the open item "not checked", "· Checked" on rows. | A, E | review build + accept | 2 | not started |
| D | Totals on "Attendance" | feature | Games in the totals of "Attendance" (a not played game does not count: `ScheduledGame.not_played`, decision 0031), "Left the team", the texts. | C | review build + accept | 2 | not started |

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
- [ ] Story C – after the game
- [ ] Story D – totals on "Attendance"
- [ ] Release 2
- [ ] Epic review accept

## Deviations
None from `functional.md`. The stricter readings are in the requirements summary.

## Review findings
