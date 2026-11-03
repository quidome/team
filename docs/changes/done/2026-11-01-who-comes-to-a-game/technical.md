# Technical spec: Who comes to a game

**Status**: done
**Size**: story
**Change**: [2026-11-01-who-comes-to-a-game](./)
**Date**: 2026-11-01

## Objective
The list of who comes to a game of the team, with the count and the extra players, so the
coordinator sees at once how many players there are and whom to ask.

## Requirements summary
From story A of `../2026-10-31-attendance-at-games/functional.md`:

- The page of a game of the team has at the top the players with the role "plays games" on the date, also hidden players, not players who only train.
- Three states: "coming", "not coming" (with or without a reason), "no answer" (the start).
- Each row has "Coming" and "Not coming". One tap saves at once. A tap on the chosen state goes back to "no answer".
- From the start page: a tap on a game in "Next", a tap on a button: two taps.
- A row with "not coming" has the link "Reason", which opens the reasons that are not hidden; they do not open by themselves. One tap adds or changes the reason, a tap on the chosen reason removes it. They close at a tap on another row. The reason stays when the state goes to "Coming" and back.
- Groups "No answer", "Coming", "Not coming", in that order, sorted by first name; an empty group is not shown. Rows do not move while the page stays open; the groups sort again when the page is opened.
- Above the list the count, which stays in view: "6 coming · 1 not coming · 2 no answer". A part with 0 is left out. "Coming" is the players coming plus the extra players.
- Below the list "Extra players" with [−] and [+], 0 to 10, saved at each tap.
- The row of a coming game of the team in "Next" shows "6 coming · 2 no answer"; a game with no answers shows "9 no answer". On "Season" only with at least one answer or extra player.
- A tap that fails: the stored state and "Not saved. Try again."; last tap wins; taps of one player go one after the other.
- "No players on this date." when the list is empty; "Extra players" still works.
- A gone game with answers stays reachable. "Delete player" is refused for a player with an answer; the player can be hidden.

Not in this story: the line "Too few players", the open items, the note about a moved game (story B); "Checked", and the words "There" and "Absent" (story C). Until story C a past game keeps the words "coming" and "not coming" (coordinator).

## Current state
- `api/games.rs`: `GamePage`, `get`, the routes under `/games/{season}/{id}`. `app.rs` lists the routes, `api/mod.rs` the paths.
- `crates/server/src/trainings.rs` `mark` and `unmark`, and `web/src/routes/training/[date]/+page.svelte` (queue and "saving…" for each player) are the pattern for a tap.
- `api/players.rs`: `has_absences`, `has_duties`, the delete route with `player_has_duties`.
- `plays_games_on`, `players_playing_on` in `duties.rs`. `ReasonId` in `reasons.rs`. `kept_games` in `duties.rs`.
- `EventRow.svelte` shows the rows of "Next" and "Season".
- `web/e2e/duties.e2e.ts` finds a coming game of the team through `/api/season`, so the list needs no game in the test week. Only the two taps from "Next" fix the answer of `/api/start`.

## Steps

### Step 1: The rules and the storage
- **Goal**: the rules are tested and the storage exists; the user sees no change.
- **Changes**: `migrations/0013_game_attendance.sql`:

  ```sql
  CREATE TABLE game_attendance (
      club_number   bigint   NOT NULL,
      season_year   integer  NOT NULL,
      source_id     bigint   NOT NULL,
      extra_players smallint NOT NULL DEFAULT 0 CHECK (extra_players BETWEEN 0 AND 10),
      -- The date of the game when its answers were given or confirmed (story B).
      answers_for   date,
      PRIMARY KEY (club_number, season_year, source_id),
      FOREIGN KEY (club_number, season_year, source_id) REFERENCES games
  );

  -- One row for each player who answered; no row is "no answer".
  CREATE TABLE game_answers (
      club_number bigint NOT NULL,
      season_year integer NOT NULL,
      source_id   bigint NOT NULL,
      player_id   bigint NOT NULL REFERENCES players ON DELETE RESTRICT,
      answer      text   NOT NULL CHECK (answer IN ('coming', 'not_coming')),
      reason_id   bigint REFERENCES reasons ON DELETE RESTRICT,
      PRIMARY KEY (club_number, season_year, source_id, player_id),
      FOREIGN KEY (club_number, season_year, source_id) REFERENCES game_attendance
  );
  ```

  `reason_id` may stay on a `coming` row: the reason stays when the state goes back. Core `crates/core/src/game_attendance.rs`:

  ```rust
  pub enum Answer { Coming, NotComing }
  pub struct StoredAnswer { pub player: PlayerId, pub answer: Answer, pub reason: Option<ReasonId> }
  pub struct ListRow { pub player: PlayerId, pub answer: Option<Answer>, pub reason: Option<ReasonId> }
  pub struct GameCount { pub coming: usize, pub not_coming: usize, pub no_answer: usize }
  /// The players with "plays games" on `date`, sorted by first name.
  pub fn game_list(players: &[Player], roles: &[PlayerRole], answers: &[StoredAnswer], date: NaiveDate) -> Vec<ListRow>;
  /// `coming` includes `extra_players`.
  pub fn game_count(list: &[ListRow], extra_players: u8) -> GameCount;
  ```

  `kept_games` gets the games with answers as an input. Store `crates/server/src/game_attendance.rs`: `answer`, `no_answer`, `set_extra`, `of_game`, `of_season`, `player_has_answers`. A tap runs in one transaction: it makes the `game_attendance` row when needed, and when the game has no answers yet, it sets `answers_for` to the date of the game; otherwise it leaves it. Then `sqlx migrate run` and `just prepare`.
- **Tests**: core: `game_list` (players on the date sorted, hidden player on it, a player who only trains not on it, a role that ended or starts later not on it, an answer of a player off the list left out); `game_count` (each state, extra players add to coming, no answers); `kept_games` with answers, and without answers or duties.

### Step 2: The routes
- **Goal**: everything works through the API.
- **Changes**: `PUT /api/games/{season}/{id}/answers/{player}` with `{ answer, reason }`; `DELETE` of the same address; `PUT /api/games/{season}/{id}/extra-players` with `{ extra_players }`. Each returns `AttendanceView`: the row and the count. A tap checks that the game is a game of the team (`not_a_team_game`), the player is on the list of the date (`not_on_list`), and the reason is not hidden (`reason_hidden`); extra players outside 0 to 10 give 400. `GamePage` gets `attendance`, `null` for a game that is not a game of the team, with the rows in the order no answer, coming, not coming, the reasons, and the count. `has_answers` in `api/players.rs`, and the delete route refuses with `player_has_answers`. `just api-types`.
- **Tests** in `crates/server/tests/game_attendance.rs`: 401 and a wrong `Origin` on each route; `attendance` on the page, `null` for another game; each state and back to "no answer"; a reason saved, changed, removed, and kept when going to "Coming" and back; refused: a player not on the list, a hidden reason, another game, 11 extra players; 0 and 10 extra players; last tap wins; `answers_for` at the first answer, not changed by a later answer, and after all answers are removed and the game moves, a new first answer sets the new date; "Delete player" refused, hidden allowed; a gone game with answers can be opened. Add the routes to `crates/server/tests/player_logs.rs` and to the list in `crates/server/tests/players.rs`.

### Step 3: The list on the page of a game
- **Goal**: the coordinator can use the list.
- **Changes**: `web/src/lib/components/GameAttendance.svelte`, at the top of the page of a game: the groups, two buttons on each row, a queue for each player and "saving…" as at a training, the link "Reason", the count in view, "Extra players". The order of the first load stays while the page is open. The count comes from the answer of the server, shown when no tap is waiting. "No players on this date." Client functions in `web/src/lib/api/client.ts`. The player page hides "Delete player" for a player with answers.
- **Tests** in `web/e2e/game-attendance.e2e.ts`, in the project `duties`, on a game no other test reads: a tap saves and survives a reload in the right group; rows do not move after a tap and sort again after a reload; a second tap goes back; "Reason" works and does not open by itself; a tap on the chosen reason removes it; "Extra players" changes the count and stops at 0 and 10; a failed tap shows "Not saved. Try again."; the count stays in view; the page fits a phone and is accessible.

### Step 4: The count on rows
- **Goal**: the count shows without opening the game.
- **Changes**: `attendance` (the count) on coming games of the team in `GET /api/start` and `GET /api/season`, from `game_count`. `EventRow.svelte` shows it in "Next", and on "Season" with at least one answer or extra player. A part with 0 is left out.
- **Tests**: server: the start page and "Season" give the same count as the page of the game; "Season" has no count for a game with no answer and no extra players. Browser: two taps from "Next" to a saved answer, with a fixed answer of `/api/start`; the count on a row.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| No row means "no answer"; attendance has its own tables | As in the epic | Yes, at Close |
| The list loads with `GamePage` | One request, and the page opens complete | No |
| The count comes from the server after each tap | One rule (decision 0022), correct with two screens | No |
| The front end keeps the order of the first load | The spec says rows do not move | No |
| A tap sets `answers_for` only when the game has no answers | The stricter reading of the epic: a tap does not confirm old answers | At the close of story B |
| The queue is copied from the training page | Two uses are too few for a shared helper; small technical debt | No |

## Not in scope
- The line "Too few players", open items, the note about a moved game.
- "Checked", the words "There" and "Absent".
- A shared tap queue.

## Test plan
- **Existing tests that must still pass**: the core tests of `duties`; `crates/server/tests/`; `web/e2e/duties.e2e.ts`, `season.e2e.ts`, `start.e2e.ts`.
- **New tests**: per step, see above.
- While working: `cargo test -p team_core game_attendance`, `cargo test -p team --test game_attendance`, `cd web && npx playwright test game-attendance`. Once before a commit: `just check`, with nothing else building `web/build`.

## Compatibility and performance
A forward-only migration with two new tables; no data to move. `kept_games` keeps more gone games than before, only those with answers.

## Progress
- [x] Step 1 – the rules and the storage
- [x] Step 2 – the routes
- [x] Step 3 – the list on the page of a game
- [x] Step 4 – the count on rows
- [x] All checks pass

## Deviations
None from `functional.md`. Details that step 2 did not name: a hidden reason stays allowed when the answer already has it, or "Coming" and back would fail (the rule of `check_mark`); extra players outside 0 to 10 give the code `out_of_range`; `DELETE` of an answer also refuses a player off the list.

## Review findings
Build review, 2026-11-01, of the uncommitted diff against `main`. Reproductions ran in a scratch copy; no code in the repository changed.

**To fix**

- **R1. `game_list` rebuilds the sort of players, and it is wrong.** `Player::sort_key` (`crates/core/src/players.rs`) is "the order of players in lists": first name, then last name, both without case, then id. `game_list` sorts on the raw `first_name`, then id. So lower case sorts after upper case, and two players with the same first name are not ordered by last name. The training list uses `sort_key`. Checked by a run: players "bram", "Cas", "Anna" come back as `Anna, Cas, bram`. No test covers it. Fix: use `sort_key` inside the group, with a core test.
  - 2026-11-01: Fixed: `game_list` uses `sort_key` inside the group; core test added.
- **R2. A coming game that left the game list still shows the count on its row.** Spec, story A: "A coming game that left gives no warning and no count." `coming_view` (`api/season.rs`) checks only `reason == TeamGame`, and a gone game of the team keeps that reason (`in_schedule` does not read `in_list`). Checked by a run: after the game left and its change was seen, `/api/season` and `/api/start` both give `in_list: false`, `reason: "team"`, with the count. New with this change: answers now keep such a game, so it also stays in "Next" until its date (`next_events` does not read `in_list`), and the count there says "1 coming" for a game that is not in the list. Twin: `a_gone_game_with_answers_stays_reachable` asserts a count on the page of a gone game. The spec line can be read as rows only or as rows and page; the coordinator should say which. Story B must use the same condition.
  - 2026-11-01: Fixed for the rows: `row_count` gives no count for a game that left the game list. The page of the game keeps its count and buttons, because answers keep that page reachable (coordinator, 2026-11-01; see A2). A gone game stays in "Next" as a game with duties does; no change.
- **R3. The test "the count stays in view" cannot fail.** Checked by a run: in the scratch copy I removed `sticky` and its offset from `GameAttendance.svelte`, and all tests passed, this one too (103 passed). `toBeInViewport` passes whether or not the count stays, so it does not show the scroll it needs. S33 made this a criterion, so the claim in step 3 is not proven by that test. The rest of the behaviour was not checked on a real phone. Fix: scroll to a row below the fold, with enough players that the page really scrolls, and assert the position of the count.
  - 2026-11-01: Fixed: the test uses a low screen, scrolls, and asserts the place of the count.
- **R4. A rule in the server (decision 0001), and it reads the wrong thing.** "Show the count on 'Season' only with at least one answer or extra player" lives in `SeasonDuties::has_attendance` and the `always` flag of `coming_view`, in `api/season.rs`. The fix of R2 adds a third condition, so all of it should be a core rule next to `game_count`. `has_attendance` also counts stored answers of players no longer on the list, which the spec says do not count: such a game shows "9 no answer" on "Season". Checked by a trace.
  - 2026-11-01: Fixed: the rule is `row_count` in the core, with a test, and reads the list, so answers off the list do not count.

**Worth a decision or a small fix**

- **R5. `load_duties` takes a second connection while it holds one.** `game_attendance::of_season(&state.pool, …)` runs while `db` is held, and the pool has `max_connections(5)`. The other reads in that function all use `&mut *db`. Five requests at once could wait on each other until the pool times out; with one coordinator this is unlikely. `of_season` should take the connection, as `duties::for_season` does. Checked by a trace, not run.
  - 2026-11-01: Fixed: `of_season` takes the connection.
- **R6. A gone game with only extra players disappears.** `SeasonAttendance::answered` reads only answers, while `has_attendance` treats extra players as attendance. Checked by a run: a game with 2 extra players and no answers, gone and seen, is no longer on "Season"; the number stays stored. The spec says "answers or duties", so this is not against it, but the coordinator typed that number. Decide on purpose.
  - 2026-11-01: Not fixed: the spec says "answers or duties". Line in `docs/ideas.md`.
- **R7. The new branch in `find` has no test, and it hides answers.** `a_gone_game_with_answers_stays_reachable` passes through `TeamGame`, because a gone game of the team stays in the schedule; it tests the `kept` of "Season", not the `None` branch of `find`. That branch is reached by a game no longer of the team, for example after another team is chosen. Then the reason is `Kept`, `attendance` is `null`, the answers cannot be seen, and "Season" labels the game as a possible duty game (`whatOf`). Checked by a trace. The spec covers only the warnings of such a game (story B), so this needs a word from the coordinator, or a line in `docs/ideas.md`.
  - 2026-11-01: Test added. Behaviour not changed: line in `docs/ideas.md`.

**Claims in this document**

- **R8. "The count … shown when no tap is waiting" is not what is built.** `enqueue` sets `count` at each answer, also while other taps wait; `refreshCount` corrects it afterwards. Harmless for a moment, but one case stays wrong: a tap made while `refreshCount` runs, and answered before it, is overwritten by the older count until the next tap. Decision 0020 says the answer of a reload sent before a tap does not undo that tap. Checked by a trace, not run. Rare, and only the number.
  - 2026-11-01: Fixed: the count changes only when no tap is waiting, and a reload sent before a tap is not trusted.
- **R9. "None" under Deviations leaves two things out.** A hidden reason is allowed when the answer already has it (`check_answer`), where step 2 says only "the reason is not hidden". The behaviour is right, as "Coming" and back needs it, but it belongs here and in the decision record at Close. The code `out_of_range` is also not named in step 2. `MAX_EXTRA = 10` in `GameAttendance.svelte` repeats `MAX_EXTRA_PLAYERS`; the server could send it, as `can_check` does (decision 0022). Checked by reading the code.
  - 2026-11-01: Fixed: Deviations has them; the server sends `max_extra_players`.
- "All checks pass" is true: `just check` exited 0 on this diff, with 143 browser tests passed. `.sqlx/` is current. The check left the tracked files as they were.

**Docs and comments made false**

- **R10. "Kept for its duties" is no longer the whole truth.** Search for `kept`: `api/games.rs` lines 3 and 442, `schedule.rs` line 19, `api/season.rs` line 281 (this one also reaches `web/openapi.json`), `web/src/lib/season.ts` line 61. The doc comment of `kept_games` now says "for their duties" and then lists answers. `README`, `docs/architecture.md`, `CLAUDE.md` and `docs/product/` wait for Close, as the workflow says; they need the new tables, the routes and `kept`.
  - 2026-11-01: Fixed: the comments and the doc comment of `kept_games`. The living documents wait for Close.

**Checked, nothing found**: reasons have no delete route and nothing deletes a game (search `DELETE FROM`), so the `RESTRICT` keys cannot give an error page; `answers_for` follows the stricter reading and its tests can fail; the routes need a login and the `Origin`; logs hold ids, no names; the core has no I/O.

**Not settled**: the width of the name on a phone. At 320 px a name measured 24.6 px wide, but the probe did not show whether that name was short or cut off, and rows with "saving…" were not measured. Look at it with long names in the accept review.

**Verdict**: fix R1 to R4 before Close. The design holds; the storage, the routes and the tap queue follow the patterns of the project.

### Accept review, 2026-11-01

Of the uncommitted diff against story A of the epic's `functional.md`, the epic's `idea.md`, and the running result. Run: the core and server tests of `game_attendance` (12 and 12 passed), and the browser projects up to and including `duties`, with a probe from a scratch folder on top (126 passed; the projects `messages` and `logged-out` did not run). The probe used the real server at 390 and 320 px, and added two players with long names. No code in the repository changed; the run rebuilt `web/build` and made the test database again.

**Verdict: accepted with notes.** The coordinator can mark who comes in two taps and sees the count at once, which is what was meant. Nothing needs correction before Close of this story. A2 needs one word from the coordinator. A1 means that this story must not be released without story B.

**Acceptance criteria of story A**

| Criterion | Result | How checked |
|-----------|--------|-------------|
| List at the top; "plays games" on the date, also hidden players; not those who only train | Matches | Tests; run: the list is above "Table and referee" |
| Three states; one tap saves; a tap on the chosen state goes back | Matches | Tests, run |
| Two taps from "Next" | Matches, within the window of "Next" | Test with a fixed `/api/start`; the real "Next" had no game on the test date (run). Trace: `next_events` always holds a game of the team up to the Sunday of next week, so a game 3 days away is always there |
| "Reason": does not open by itself, one tap, removes, closes at another row, stays via "Coming" | Matches | Tests, run. See A3 and A4 |
| Groups, order, rows do not move, sort again on opening | Matches | Tests, run. Search: nothing on the page of a game reloads on focus or return, so a return from the chat moves nothing |
| Count in view, parts with 0 left out, extra players added | Matches | Tests, run |
| "Extra players" 0 to 10 | Matches | Tests |
| "Too few players" above the count | Not built here: planned for story B | See A1 |
| Count on rows of "Next" and "Season" | Matches | Tests, run: "Season" shows only games with an answer |
| Messages and duty overviews do not change | Matches | Search: the diff has no change in `api/messages.rs` or in the core messages |
| Failed tap, two screens, no players, all reasons hidden | Matches | Tests, trace. See A5 |
| Moved game: answers stay, list follows the new date | Matches; the note is story B | Tests. See A1 |
| Game left the list: reachable, no count | Matches on rows | Tests. See A2 |
| Earlier season, delete refused, games of other teams, past game | Matches | Tests; run: a past game has the buttons with the words "Coming" and "Not coming", as the coordinator chose. Earlier season: by trace only (`find` and `duties_of` use the season of the address) |

**Findings**

- **A1. Faithful adaptation, with a condition: "Too few players" and the note about a moved game are not in this story.** Story A names both, and `technical.md` moves them to story B with the coordinator's approval. But the first goal of the epic is the warning in time, and this story alone does not give it: it shows "6 coming" and leaves the counting to 7 to the coordinator. A moved game also keeps its old answers without saying so. The epic releases A and B together, so this is right as long as that plan holds. Checked by reading the epic's stories and a search for `Too few` in `web/src` (none).
  - 2026-11-01: Not fixed: story B comes next, before release 1, as planned in the epic.
- **A2. Neutral drift, to confirm: the page of a game that left the list keeps its count and its buttons.** The spec says "A coming game that left gives no warning and no count." R2 calls the reading of that line a question for the coordinator, but its note says "Decided" and names no one. The result is sensible: the answers stay visible, and rows have no count. The list can still be changed on a game that is gone, and the count there can suggest that the game still takes place. Checked by reading the test `a_gone_game_with_answers_stays_reachable` and `row_count`.
  - 2026-11-01: Not fixed: confirmed by the coordinator: the page of a gone game keeps its count and buttons.
- **A3. Faithful adaptation: a hidden reason that a row already has is offered too.** The spec says the reasons that are not hidden. Without this the coordinator could not see or remove a stored reason, and "Coming" and back would fail. It is under Deviations. Checked by reading `offered` and `check_answer`.
  - 2026-11-01: Not fixed: needed for "Coming" and back; listed under Deviations.
- **A4. Neutral drift: the reason shows on its own line as "Reason: School", not behind the name as in the sketch ("Eva · Ill").** The name line has no room for it on a phone, and the second line also serves as the link. A tap back to "no answer" drops the reason; the spec keeps it only for "Coming" and back, so this is within the spec. Checked by a run (screenshot at 390 px).
  - 2026-11-01: Not fixed: accepted, the row stays one height until a reason is opened.
- **A5. Neutral drift: "Not saved. Try again." can stay after a newer tap did save.** A failed tap followed by a second tap on the same row: the row and the count are right, and they stay right after a reload, but the message stays until the next tap. It errs on the safe side, and the training page sets and clears it in the same places. Checked by a run: first request aborted, second saved, the message still shown.
  - 2026-11-01: Not fixed: the training page behaves the same way.
- **A6. Neutral drift: long names are cut on a small phone; this settles the open point of the build review.** At 320 px a name has 65 px: "Wilhelmina" (84 px) shows as "Wilhel…", "Alexander-Maximiliaan" as "Alexa…". At 390 px a name has 135 px: "Wilhelmina" fits, and the long name shows "Alexander-Max…". All short names fit, and nothing scrolls sideways. "saving…" takes room for a moment, and "(hidden)" stands behind the name, so a long name can lose that word. "saving…" and "(hidden)" were not measured. The sketch has room for names, but the name is how the coordinator finds the row, so look at it on the real phone with the real names. Checked by a run (`clientWidth` against `scrollWidth`).
  - 2026-11-01: Not fixed: the coordinator checks it with the real names on a phone.
- **A7. Neutral drift, already written down: R6 and R7 leave two rare cases.** A gone game with only extra players goes from "Season", and answers of a game no longer of the team do not show. Neither goes against the spec, both are stored and neither is lost, and both are lines in `docs/ideas.md`. The second is the one to watch: the tool holds answers the coordinator cannot see. Checked by reading `docs/ideas.md` and the build review; not run again.
  - 2026-11-01: Not fixed: lines in `docs/ideas.md`.

**Not checked**: a real phone, one-handed use, and the real flow from "Next" with a game in its window (the browser test fixes that answer).
