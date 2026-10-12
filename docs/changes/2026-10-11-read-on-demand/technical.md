# Technical spec: Read the game list on demand

**Status**: done
**Size**: story
**Change**: docs/changes/2026-10-11-read-on-demand
**Date**: 2026-10-11

## Objective
Let the coordinator read the club's game list with one tap: store the games of the club and
season, compare them with the stored games, and show a short result line, so that story 3b can
show the schedule and story 4 can keep the changes, from the same stored games and the same
comparison.

## Requirements summary
Story 3a of the epic [Games from the game list](../2026-10-10-games-from-game-list/technical.md),
from functional story 2 ("Read the game list when the coordinator asks") and the parts of
functional stories 3 and 4 that reading needs. Story 3b (the schedule screen) has its own design.

1. One action reads the game list of the chosen club for the current season. A tap during a running reading waits for the result.
2. A short result line counts only games in the schedule, for example "3 new, 1 moved, 1 no longer in the game list". With nothing changed: "No changes". The counts come from the same comparison that story 4 uses for the list of changes, so they will match it.
3. The schedule holds all games of the team (home and away) and the home games of other club teams ("possible duty games"); no away games of other club teams.
4. The tool never changes a game; the game list is the truth.
5. Source not reachable, an error answer, or data not understood: the games stay as they were; the screen shows the failure and the time of the last good reading.
6. An empty reading, or far fewer games than the last good reading of the same season: the games are kept, nothing is counted, and a warning shows with the action "These games are really gone". After that action the missing games count as "no longer in the game list". A later good reading removes the warning.
7. Two readings at the same time (a tap and, in story 5, the nightly reading) never count a change twice or lose one.
8. The first reading after a choice of club or team counts nothing: it shows the number of games read.
9. Games are kept for each club and season; a reading never changes another season (coordinator, 2026-10-11).
10. Places (the source's place number, hall, town) are stored in their own table (coordinator, 2026-10-11).
11. A game is identified by the source's `id`; the game number `nr` is stored too (coordinator, 2026-10-11).
12. A season is stored as its start year (2026 for "2026-2027") and always shown as "2026-2027" (coordinator, 2026-10-11).

Decisions that apply: 0001 (pure core), 0004 (forward-only migrations; a breaking change takes
two releases), 0005 (one reading at a time by a PostgreSQL advisory lock), 0007 (every `/api`
route needs a session), 0012 (the choice for each season), 0013 (strict reading of the source).

## Current state
- Core (`crates/core`): `season` (a season starts on 1 July), `club`, `game` (`Game` with `source_id`, `game_number`, `season`, `date`, `start_time: Option<NaiveTime>`, `home`/`away` `Side { team, team_name, club }`, `place: Place { number, hall, town, court }`, `competition`), `team_choice` (`own_teams`, `choose_team`, `choice_state`, `FollowedTeam`).
- Server (`crates/server`): `source::Source::club_games(club, season)` reads and parses a club's games strictly; `followed_teams::{for_season, newest_before, save}` on the table `followed_teams` (one row for each season, `season` as text "2026-2027", `chosen_at`); `clock::current_season`; `AppError` with `BadRequest(&'static str)`, `NotFound`, `SourceUnavailable`, `SourceFormat`, `Internal`; routes on the protected `/api` router in `app.rs`.
- Front end: the start page (`web/src/routes/+page.svelte`) shows the chosen club and team, "Change", and a link to the sample schedule.
- Tests: `crates/server/tests/common/mod.rs` has the harness with a fake source (`harness.source`, wiremock) and `TestClock::set`; real source answers in `crates/server/tests/data/game-list/`. The browser tests have the project `choose` (runs first) with a fake source in Node (`web/e2e/fake-source.mjs`) and a fresh database `team_e2e`.
- Real data (2026-2027, Blue Drakes, club 37, team M16 1 = 184): 71 games; 10 games of the team; 27 home games of other club teams; 34 away games of other club teams; 27 places; a game on 25 October 2026 (end of summer time) at 09:30; no game numbers twice.

## Steps

### Step 1: Core rules
- **Goal**: the schedule rule, the comparison and the guard, testable without I/O.
- **Changes**: new modules in `crates/core`, exported from `lib.rs`:

  ```rust
  // schedule.rs
  pub enum ScheduleReason { TeamGame, DutyGame }
  /// TeamGame: the chosen team plays (home or away). DutyGame: the home club is the chosen club
  /// and the chosen team does not play. None: an away game of another club team, or a game of
  /// other clubs. Without a team, every home game of the club is a DutyGame.
  pub fn in_schedule(game: &Game, club: ClubNumber, team: Option<TeamNumber>) -> Option<ScheduleReason>;

  // comparison.rs
  pub struct StoredGame { pub game: Game, pub in_list: bool }
  pub enum Change { New, Gone, CameBack, IntoSchedule(ScheduleReason), OutOfSchedule, Changed(Vec<Difference>) }
  pub enum Difference { Date, StartTime, StartTimeKnown, Place, HomeAndAway, Team }
  pub struct GameChange { pub source_id: i64, pub change: Change, pub before: Option<Game>, pub after: Option<Game> }
  pub fn compare(stored: &[StoredGame], read: &[Game], club: ClubNumber, team: Option<TeamNumber>) -> Vec<GameChange>;

  #[derive(Default)]
  pub struct Counts { pub new: u32, pub moved: u32, pub start_time_known: u32, pub home_and_away: u32,
                      pub team_changed: u32, pub gone: u32, pub came_back: u32,
                      pub into_schedule: u32, pub out_of_schedule: u32 }
  pub fn count(changes: &[GameChange]) -> Counts;

  /// Pairs (gone source id, new source id) with the same game number: a sign that the source's
  /// id changed for one game. The server logs them.
  pub fn same_number_pairs(changes: &[GameChange]) -> Vec<(i64, i64)>;

  // guard.rs
  pub enum Guard { Accept, HoldBack(HoldReason) }
  pub enum HoldReason { Empty { before: u32 }, FarFewer { before: u32, now: u32 } }
  pub fn guard(last_good: Option<u32>, read: u32) -> Guard;
  ```

  Rules:
  - A game gives a change only when it is in the schedule before or after the reading (by `in_schedule` on the stored and the read version).
  - One change for each game:
    - In the schedule before and after, and different: `Changed` with every difference. Date, start time (both known) or place: "moved". Start time from not known to known: `StartTimeKnown`. Known to not known: `StartTime` ("moved"). Home and away teams change places: `HomeAndAway`. A team number on one side changes, and it is not a swap: `Team`.
    - Read but not stored, in the schedule: `New`. Stored, in the list, in the schedule, not read: `Gone`. Stored with `in_list = false`, read again, in the schedule: `CameBack` (also when it changed meanwhile).
    - Stored and read, in the schedule only after: `IntoSchedule(reason)`; only before: `OutOfSchedule`.
  - `count` counts each change once; a `Changed` counts by its first difference in this order: Date, StartTime, Place ("moved"), StartTimeKnown, HomeAndAway, Team.
  - `guard`: `read == 0` with `last_good > 0`: `Empty`. `read * 2 < last_good`: `FarFewer`. Else `Accept` (also no earlier good reading, or an earlier one with 0 games).
- **Tests** (unit tests in core, and one real-data test in `crates/server/tests/source.rs`):
  - `in_schedule`: team at home and away give TeamGame; a home game of another club team gives DutyGame; an away game of another club team and a game of two other clubs give None; with no team, every club home game is DutyGame.
  - `compare`: the same games twice give no changes; a new game in the schedule gives New, a new away game of another club team nothing; a new date, start time or place gives Changed (moved); not known to 10:00 gives StartTimeKnown; 10:00 to not known gives StartTime; a swap gives HomeAndAway; another team gives Team; missing gives Gone, back gives CameBack; an away game of another club team that becomes a home game gives IntoSchedule(DutyGame), the reverse OutOfSchedule; changes outside the schedule give nothing; a new date and a new place give one change that counts once as moved; `count` adds up each kind; `same_number_pairs` finds a Gone and a New with the same game number.
  - `guard`: (None, 0) Accept; (Some(0), 0) Accept; (Some(71), 0) Empty; (Some(71), 35) FarFewer; (Some(71), 36) Accept; (Some(10), 5) Accept.
  - Real data: the 2026-2027 file with club 37 and team 184 gives 10 TeamGame, 27 DutyGame and none of the 34 away games of other club teams.

### Step 2: Storage
- **Goal**: tables for places, games and readings; the season as its start year.
- **Changes**:
  - Migration `migrations/0003_games.sql`:

    ```sql
    -- A season is stored as its start year: 2026 means "2026-2027" (decision 0014).
    CREATE TABLE places (
        number  bigint PRIMARY KEY,                -- the source's loc_id
        hall    text NOT NULL,
        town    text NOT NULL
    );
    CREATE TABLE games (
        club_number       bigint NOT NULL,         -- the club whose game list it came from
        season_year       integer NOT NULL,
        source_id         bigint NOT NULL,         -- the key
        game_number       text NOT NULL,
        date              date NOT NULL,           -- Dutch local date
        start_time        time,                    -- Dutch local time; NULL = not known
        home_team_number  bigint NOT NULL,
        home_team_name    text NOT NULL,
        home_club_number  bigint NOT NULL,
        away_team_number  bigint NOT NULL,
        away_team_name    text NOT NULL,
        away_club_number  bigint NOT NULL,
        place_number      bigint NOT NULL REFERENCES places,
        court             text,
        competition       text NOT NULL,
        in_list           boolean NOT NULL,        -- false: no longer in the game list
        first_read_at     timestamptz NOT NULL,
        PRIMARY KEY (club_number, season_year, source_id)
    );
    CREATE TABLE readings (
        id             bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
        club_number    bigint NOT NULL,
        season_year    integer NOT NULL,
        started_at     timestamptz NOT NULL,
        finished_at    timestamptz NOT NULL,
        result         text NOT NULL CHECK (result IN ('good', 'failed', 'held_back')),
        error          text CHECK (error IN ('source_unavailable', 'source_format')),
        games_read     integer,
        first_reading  boolean NOT NULL DEFAULT false,
        counts         jsonb,                      -- Counts of a good reading that is not the first
        hold_reason    text CHECK (hold_reason IN ('empty', 'far_fewer')),
        hold_before    integer
    );
    CREATE INDEX readings_club_season ON readings (club_number, season_year, id DESC);

    -- followed_teams keeps writing the text season until a later release (the old app may run
    -- on this schema during a rollout); new code reads only season_year.
    ALTER TABLE followed_teams
        ADD COLUMN season_year integer GENERATED ALWAYS AS (left(season, 4)::integer) STORED;
    CREATE UNIQUE INDEX followed_teams_season_year ON followed_teams (season_year);
    ```
  - `followed_teams.rs`: `for_season` and `newest_before` filter and sort on `season_year`; `save` still writes `season` (text) and conflicts on `season`.
  - New `crates/server/src/games.rs`: `stored(tx, club, season) -> Vec<StoredGame>`, `write(tx, club, season, read: &[Game], gone: &[i64], now)` (upsert places; upsert every read game with `in_list = true`, keep `first_read_at`; set `in_list = false` for `gone`).
  - New `crates/server/src/readings.rs`: `insert(...)`, `latest(pool, club, season)`, `last_good(pool, club, season)`, `good_since(tx, club, season, at) -> bool`, `by_id(pool, id)`.
  - Season conversion in one place: `Season::start_year()` and `Season::from_start_year(i32)` (new, in core).
  - `just prepare` (`.sqlx/`).
- **Tests** (`#[sqlx::test]`, `crates/server/tests/storage.rs`):
  - `followed_teams.season_year` is 2026 for a row with season "2026-2027"; the existing tests of story 2 still pass.
  - Writing 2026-2027 games of club 37 does not change a 2025-2026 game of club 37 put in by hand.
  - A written game reads back equal (date, start time `NULL` for not known, the game on 25 October 2026 at 09:30).
  - `Season::from_start_year(2026)` shows "2026-2027" (core unit test).

### Step 3: The reading and the API
- **Goal**: one reading, safe with two at the same time, and its routes.
- **Changes**:
  - New `crates/server/src/reading.rs`:

    ```rust
    pub enum FailureKind { SourceUnavailable, SourceFormat }
    pub enum Outcome {
        Good { id: i64, games_read: u32, first_reading: bool, counts: Option<Counts> },
        HeldBack { id: i64, reason: HoldReason },
        Failed { id: i64, error: FailureKind },
    }
    #[derive(Debug, thiserror::Error)]
    pub enum ReadingError { #[error("no club chosen")] NoChoice, #[error(transparent)] Store(#[from] StoreError), ... }
    pub async fn read(pool: &PgPool, source: &Source, clock: &dyn Clock, guard_on: bool) -> Result<Outcome, ReadingError>;
    ```

    Steps of one reading:
    1. The club and team: the choice of the current season; or, in a new season with no row yet, the club of the newest earlier choice (team `None`). No choice at all: `NoChoice`.
    2. Read the source (outside the database). An error writes a `failed` row (with `error`) and gives `Failed`.
    3. One transaction: `SELECT pg_advisory_xact_lock(<fixed key for readings>)`.
    4. Load the stored games of the club and season, and `games_read` of the last good reading.
    5. If `guard_on`: apply `guard`; a hold writes a `held_back` row (with `hold_reason`, `hold_before`, `games_read`), commits, and gives `HeldBack`.
    6. First reading: no good reading of this club and season with `finished_at >= chosen_at` of the current choice.
    7. `compare` and `count` (no counts for the first reading); log `same_number_pairs` as a warning (game numbers and ids only).
    8. Write places and games (`games::write`), write the `good` row, commit.
  - Routes on the protected `/api` router (new `api/readings.rs`, in `ApiDoc`):

    | Route | Answer | Errors |
    |---|---|---|
    | `POST /api/readings` | `Reading` (also for a failed or held-back reading: a recorded result, not an HTTP error) | 400 `no_choice` |
    | `GET /api/readings/latest` | `{ latest: Reading \| null, last_good: { finished_at, games_read } \| null }` for the current club and season (`null`s with no choice) | – |
    | `POST /api/readings/{id}/confirm-gone` | `Reading` of a new reading with the guard off | 400 `not_held_back` (not the newest reading of the current club and season, or not held back); 404 (no such reading) |

    ```text
    Reading = { id, result: "good" | "failed" | "held_back", finished_at, games_read: number | null,
                first_reading: boolean, counts: Counts | null,
                hold: { reason: "empty" | "far_fewer", before: number, now: number } | null,
                error: "source_unavailable" | "source_format" | null }
    Counts  = { new, moved, start_time_known, home_and_away, team_changed, gone, came_back,
                into_schedule, out_of_schedule }
    ```
  - `just api-types`.
- **Tests** (router tests, `crates/server/tests/readings.rs`, fake source and test clock; the two concurrency tests first):
  - Two `POST /api/readings` at the same time on changed data: together they count each change once; the stored games are right.
  - A test trigger that makes the insert of the `good` reading row fail: the answer is 500; games and places are unchanged.
  - Without login all three routes give 401; with no choice `POST` gives 400 `no_choice`.
  - First reading: good, first reading, 71 games read, no counts; 71 games and 27 places stored.
  - Same data again: good, all counts 0.
  - Changed copy (one team game on another date, the 00:00 game of 16 January 2027 at 10:00, one duty game removed, one new home game of another club team): new 1, moved 1, start time known 1, gone 1; the removed game has `in_list = false`, the moved game its new date.
  - The removed game in the next reading again: came back 1.
  - A change to an away game of another club team: no count, but stored.
  - Guard: an empty reading is held back (`empty`, before 71), games unchanged; 30 games: held back (`far_fewer`, before 71, now 30); a later good reading is the newest, so `latest` has no hold.
  - Really gone: confirming the held-back reading gives a good reading in which the missing schedule games count as gone; confirming a good reading or an older reading gives 400 `not_held_back`; an unknown id gives 404.
  - Source down: a failed reading with `source_unavailable`, games unchanged, `latest` shows the failure and `last_good` the earlier good reading; a format error gives `source_format`.
  - A 2025-2026 game of club 37 is not changed by a 2026-2027 reading.
  - After a change of team, the next reading is a first reading again.
  - In a new season with a team needed (no row for the season), a reading of the kept club works.
  - A gone and a new game with the same game number write a warning to the log (with the `logs` test helper).

### Step 4: Front end
- **Goal**: the read action and its result on the start page.
- **Changes** (start page, state "chosen"; `client.ts` helpers `readNow`, `loadLatestReading`, `confirmGone`):
  - Button "Read the game list"; while it runs it shows "Reading…" and is disabled. The request waits, also behind another reading.
  - The result line from the counts, only the kinds that are not zero, in this order and with these words: "N new", "N moved", "N start time known", "N home and away swapped", "N team changed", "N no longer in the game list", "N came back", "N now in the schedule", "N no longer in the schedule". All zero: "No changes". First reading: "N games read".
  - "Last read: <day> <date> <time>" from `last_good` (Dutch local time), and "The last reading failed: <message>" when the newest reading failed (the messages of `client.ts` for `source_unavailable` and `source_format`).
  - A held-back reading: "The game list has no games now." or "The game list has far fewer games than before (N instead of M).", then "The games are kept." and the button "These games are really gone".
- **Tests** (Playwright, project `choose`, after the team is chosen, on the fresh database and the fake source):
  - "Read the game list" gives "71 games read" and "Last read: …".
  - A second tap gives "No changes".
  - The page passes the accessibility check and has no horizontal scrolling at 320 px.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| Split story 3 into 3a (read on demand) and 3b (the schedule) | About 8–9 steps together; each part can be released alone (coordinator) | no (epic) |
| A season is stored as its start year (`season_year integer`); `followed_teams` gets a computed column now, the text column goes in a later release | Sorts as a number, cannot hold a wrong text; decision 0004 needs two releases for `followed_teams` (coordinator's idea) | 0014 |
| Dates and start times as Dutch local `date` and `time`, not `timestamptz` | The source gives local time; a shown time can never shift; 00:00 stays `NULL` | no (in 0014's context, and in `architecture.md` at Close) |
| The source's `id` is the key; `nr` is stored; a warning in the log when a gone and a new game share a game number | Unknown which number is stable; both are kept so a change of key is small (coordinator) | no |
| The comparison detects every kind of change in 3a | The result line must match story 4's list of changes | no |
| "These games are really gone" reads the source again with the guard off | Simpler than storing every held-back reading's games; uses the newest data | no |
| Readings, also failed ones, are recorded and answered with 200 | The failure is a result the screen shows with the last good reading | no |
| Closure `review build + accept` | Lock, transaction and comparison (build); behaviour the coordinator sees (accept) | no |

## Not in scope
- The schedule screen, the filter, past games, the crossed-out look, last season's games, and removing the sample page (story 3b).
- Storing changes, "seen", the list of changes and its summary line (story 4).
- The nightly reading (story 5).
- Removing the text column `season` from `followed_teams` (a later release; technical debt).
- Reading an earlier season on request (`docs/ideas.md`).

## Test plan
- **Existing tests that must still pass**: all of story 1 and story 2 (choice, reader, web, login), the browser tests of the projects `choose`, `logged-in`, `logged-out`.
- **New tests**: per step, above. Order of value: the two concurrency tests and the rollback test, the comparison, the guard, the router cases, the browser flow.
- **By hand**: `cargo test -p team --test source -- --ignored` reads the live source.

## Compatibility and performance
- Migration `0003` adds three tables and a computed column with a unique index on `followed_teams`; the released story 2 code keeps working on it.
- No new setting; no cluster change.
- One reading: one request to the source (about 75–125 KB), then one short transaction. A held lock waits; the source is never read while the lock is held.

## Progress
- [x] Step 1: core rules (2026-10-12)
- [x] Step 2: storage (2026-10-12)
- [x] Step 3: the reading and the API (2026-10-12)
- [x] Step 4: front end (2026-10-12)
- [x] All checks pass (2026-10-12; `api-types-check` passes once the new types are committed)

## Deviations
From the epic's technical spec (none from the functional spec):
- Story 3 is split into 3a (this story) and 3b (the schedule) (coordinator, 2026-10-11).
- Dates and start times are stored as Dutch local `date` and `time`, not as `timestamptz`.
- A season is stored as its start year (`season_year`), not as text (coordinator, 2026-10-11).
- "These games are really gone" reads the source again with the guard off.

The design reads the functional spec more strictly than written, within its words:
- "Far fewer games": fewer than half of the last good reading of the same season (from the epic).
- An empty reading is held back only when the last good reading had games; a first empty season in summer is accepted.
- A game that changes in more than one way counts once in the result line, by its first difference (date, start time, place, start time known, home and away, team).
- The first reading after a choice shows "N games read" and counts nothing.
- A "place" difference is another place number or another court; another hall or town name of the
  same place number is not a move of the game (step 1, 2026-10-12).
- A game that was no longer in the game list and comes back outside the schedule gives no change
  (step 1, 2026-10-12).

Small choices in the build (step 2, 2026-10-12):
- `StoreError` moved from `followed_teams.rs` to `db.rs`, because three storage modules share it.
  Its `Season` case went (the season text is no longer read); a new case `Invalid` is for stored
  data the code does not understand (for example an unknown `result` text).
- `FailureKind` is in `readings.rs` (the stored record needs it); `reading.rs` uses it from there.
- sqlx has the `json` feature now, for the `counts` column.
- Extra storage tests: a second write marks gone games and keeps `first_read_at`; every kind of
  reading reads back equal, with `latest`, `last_good` and `good_since`.

Small choices in the build (step 3, 2026-10-12):
- `reading::read` gives the stored `readings::Reading` (id, times, result) instead of a separate
  `Outcome` type: the API needs `finished_at`, and the two types would hold the same data.
- `reading::target` finds the club, team, season and `chosen_at` of a reading; `GET
  /api/readings/latest` and "really gone" use it too. `followed_teams::chosen_at` is new.
- Every stored game that is in the game list but not read is marked "no longer in the game list",
  also outside the schedule (the game list is the truth); only games in the schedule are counted.
- Test data (coordinator, 2026-10-12): the spec's "changed copy" expected "start time known 1" from
  the 00:00 game of 16 January 2027, but that is an away game of Blue Drakes MSE 1, outside the
  schedule of M16 1. The test uses an M16 1 game (31 October 2026, not known → 10:00) instead; the
  16 January game gets 10:00 too and must give no count.
- The log capture helper (`LogBuffer`) moved from `tests/logs.rs` to `tests/common/`, without
  colour codes; the log test of story 3a has its own file (`tests/reading_logs.rs`). Shared reading
  test helpers are in `tests/common/game_list.rs`.

Small choices in the build (step 4, 2026-10-12):
- The result line shows only after a tap on this page; after a page load it would show an old
  result as new. "Last read", the failure and the held-back warning come from the newest reading,
  so they also show after a page load.
- "Last read" shows as for example "Mon 12 October 2026 14:00" (Dutch local time).
- The browser tests run in the project `choose` right after M16 1 is chosen: that project ends with
  a club without a team, where the read action does not show.
- The words of the result line and the time format are in `web/src/lib/reading.ts`; the screen part
  is `web/src/lib/components/ReadGameList.svelte`.

After the reviews (2026-10-12, see **Review findings**):
- `reading::read` takes a `Mode` (`Guarded` or `Confirm { held_id }`) instead of `guard_on`.
- "First reading" comes from the team stored with the last good reading (`readings.team_number`),
  not from `chosen_at`; migration 0003 got that column before its first release.
- `GET /api/readings/latest` has a third field `held_back`.
- A reading that finds a newer good reading under the lock answers with that reading.

## Review findings

### Accept review (2026-10-12)
By `/review accept` (fresh agent). Input: mode `accept` and this change folder; no other
arguments. Read: the epic's `idea.md` and `functional.md` (stories 1, 2 and 4, as far as 3a
covers them), `change.md`, this file, `docs/language.md`, and the result in commit `ad9a9a9`
(`reading.rs`, `api/readings.rs`, `comparison.rs`, `followed_teams.rs`, `api/selection.rs`,
`ReadGameList.svelte`, `reading.ts`, `+page.svelte`, the router tests). Process note: the
**Deviations** above are in the same file as the objective, so they were read before the view was
complete; A1–A3 are not in them.

Run: `cargo test -p team_core` (63 passed), `cargo test -p team --test readings --test storage
--test reading_logs` (14, 5 and 1 passed), against the local PostgreSQL. The browser tests were not
run.

**A1 – concerning deviation: saving the same choice again hides the next changes.** Story 1 says
the first reading after a *change* of club or team counts nothing. The build decides "first
reading" by `chosen_at` (`reading.rs`: `good_since(..., target.chosen_at)`), and `POST
/api/selection` sets `chosen_at = now` on every save, also when the club and team are the same
(`api/selection.rs:108`, `followed_teams::save` `ON CONFLICT ... chosen_at = EXCLUDED.chosen_at`).
So: the coordinator opens "Change", picks the same team again; a game of the team has moved since
the last reading; the next reading shows "71 games read", stores the moved game and counts
nothing. In story 4 that change would never appear, which is the one thing the coordinator must
not miss. Checked by trace, not run. Fix in the build (the spec is right): keep `chosen_at` when
club and team do not change, or decide "first reading" from the club and team of the last good
reading. Add a router test for "same choice saved again".
- 2026-10-12: fixed. Each reading stores its team (`readings.team_number`, migration 0003 changed before its first release). `team_core::is_first_reading` decides from the team of the last good reading of the club and season; `chosen_at` is no longer used. Test: `saving_the_same_choice_again_still_counts_the_changes`.

**A2 – neutral drift: a failed reading hides the held-back warning.** Story 2: "If the next
reading gives the games again, the warning goes away." The screen shows the warning only when the
newest reading is held back (`ReadGameList.svelte`: `latest?.result === 'held_back'`; `GET
/api/readings/latest` gives only the newest). After a held-back reading, a failed tap (source down)
removes the warning and the button "These games are really gone", although the list did not give
the games again. The games stay safe, and the next reading that gets the short list is held back
again, so the warning comes back. Checked by trace. Suggest: show the hold of the newest reading
that is not failed.
- 2026-10-12: fixed. `GET /api/readings/latest` has `held_back`: the newest reading that did not fail, when it is held back. The screen and the confirmation both use it. Test: `a_failed_reading_does_not_hide_the_held_back_reading`.

**A3 – neutral drift (for story 4): "came back" for a game the coordinator never saw go.** A game
outside the schedule (an away game of another club team) that leaves the list is marked "no longer
in the game list" silently (by design, Deviations step 3). If it comes back as a home game of
another club team, `compare` gives `CameBack` (the `!stored.in_list` branch looks only at
`schedule(after)`), so the result line says "1 came back" for a game that was never shown as gone.
Story 4 asks for "now a home game of another club team" here. Rare; checked by trace. Story 4's
design should decide it (for example `IntoSchedule` when the stored version was outside the
schedule).
- 2026-10-12: not fixed: for the design of story 4, as the review says. The rule for which games go out of the list is now one rule in core (`no_longer_in_list`, see R1), so story 4 can decide it in one place.

**A4 – neutral drift: no read action before a team is chosen, and no reading right after the
choice.** The button shows only in the state "chosen" (`+page.svelte`). In a new season before the
team is chosen, the server can read the kept club, but the screen offers no action (story 1:
"shows the games of the last season" until the new choice, which is story 3b). After choosing the
team, story 1's behaviour says "the tool reads the game list and shows the games"; now the
coordinator must tap "Read the game list" first. Fine for 3a, since there is no schedule yet; story
3b should decide whether a choice starts the first reading. Checked by reading.
- 2026-10-12: not fixed: for the design of story 3b.

**A5 – faithful adaptations.** Checked by reading the code and the router tests, and by the test
run above:
- One action reads the club's list for the current season; the button shows "Reading…" and waits;
  two readings at the same time count each change once (advisory lock; concurrency test passes).
- The result line counts only games in the schedule, from the same `compare` that story 4 will
  use; the words and order match the spec's example ("3 new, 1 moved, 1 no longer in the game
  list"); "No changes" when nothing changed; "71 games read" on a first reading (story 4: not every
  game "new"). "N now in the schedule" / "N no longer in the schedule" is a fair short form in a
  count line; the reason belongs in story 4's list.
- The tool never changes a game: the stored games are only what the list gave, and a change
  outside the schedule is stored but not counted.
- Source down or data not understood: the games stay, the screen says "The last reading failed: …"
  and "Last read: …" from the last good reading.
- Empty or fewer than half the games: kept, nothing counted, a warning with "These games are
  really gone"; after it, the missing schedule games count as "no longer in the game list"; an
  empty first summer season is accepted (spec warning "season change").
- A reading never changes another season; a change of team makes the next reading a first reading.

**A6 – note: the browser tests cover only the happy path.** The held-back warning, "These games are
really gone" and the failure line are tested at the API, not on the screen. Checked by search
(`Read the game list|games read|No changes|Last read` in `web/e2e/choose.e2e.ts`).
- 2026-10-12: not fixed: the held-back warning, the confirmation and the failure line stay tested at the API (router tests); the fake source of the browser tests has no way to change its answer yet.

**Verdict: accepted with notes.** What was built is what was meant, with one exception: A1 lets a
real change pass uncounted, and should be fixed in Implement before Close (a build fix; the spec
does not need to change). A2 is a small screen fix worth doing at the same time. A3 and A4 are for
the designs of stories 4 and 3b.

### Build review (2026-10-12)
By `/review build` (fresh agent). Input: mode `build` and this change folder; no other arguments.
Read: commit `ad9a9a9` (the build; `main` has no other changes since the design commit), this
folder, `docs/architecture.md`, decisions 0001, 0004, 0005, 0007, 0012, 0013, `CLAUDE.md`. The
accept review above was already in this section when this review wrote; it was read after the
findings below were formed, and A1 is referred to where a finding is its sibling.

Runs: `cargo test --workspace` (all pass: core 63, readings 14, storage 5, reading_logs 1, the
rest of stories 1 and 2 unchanged); `cargo fmt --check`, `cargo clippy --workspace --all-targets
-D warnings`, `cargo sqlx prepare --workspace --check`, `just api-types-check`, `just check-web`:
all pass. The browser tests (`just e2e`) were not run. Mutation run in a scratch copy (not in the
repository): with `pg_advisory_xact_lock` removed from `reading.rs`, the test
`two_readings_at_the_same_time_count_each_change_once` fails with every count doubled, so the
concurrency test can fail.

**R1 – logic in the wrong place: which games become "no longer in the game list" is decided in
the server, next to and differently from the comparison.** `reading.rs` builds `gone` itself (every
stored game with `in_list` that was not read), while `team_core::compare` decides `Gone` with an
extra schedule filter. Two rules over the same data now decide what is stored and what is counted;
decision 0001 and `architecture.md` ("new rules go in `crates/core` first; `crates/server` only
moves data in and out") put this in core. The split is also the root of A3: the store marks a game
gone that `compare` never reported, so its return looks like "came back". The "first reading" rule
(`good_since(chosen_at)`) is the same kind: a product rule (story 1) as a query in the server, which
is where A1 hides. Checked by reading `reading.rs` and `comparison.rs`. Suggest: one core function
that gives both the changes and the ids to mark gone (one loop, one rule); decide "first reading" in
core from the club and team of the last good reading (stored with the reading) and the current
target, which also fixes A1.
- 2026-10-12: fixed. `team_core::no_longer_in_list` is the one rule; `compare` uses it for `Gone` and the server uses it for what it stores. "First reading" is `team_core::is_first_reading` (see A1). Tests: core `every_missing_game_in_the_list_is_no_longer_in_the_list`, `the_first_reading_is_the_one_without_a_good_reading_of_the_same_team`.

**R2 – decision 0013 is contradicted in three places, with no record.** 0013 (valid) says: the API
answers 502 `source_unavailable` / `source_format`; "nothing is saved after a failed reading"; and
"a club number is checked in the club list before its games are read". The build answers 200 with
a recorded failure (designed, decision table: "Decision record? no"), saves a `failed` row (games
untouched, so the intent holds), and `reading::read` calls `source.club_games` without the club
list check (search: `clubs(` in `reading.rs`, no hit). The last one matters most: an unknown club
gives an empty game list, the same as an empty season (0013's context). Inside a season the guard
holds that back, but a first reading of a new season, or of a club with no good reading yet, has no
`last_good` and accepts the empty list as good. Checked by reading and search. Needed at Close at
the latest: a record that replaces the parts of 0013 that readings change (200 with a recorded
result; failed readings are recorded), and either the club check in the reading or a written reason
why the guard is enough.
- 2026-10-12: not fixed in the build: for Close. A decision record replaces the parts of 0013 that readings change (200 with a recorded result; failed readings are recorded). The club check: the club comes from the stored choice, which was checked against the club list when it was saved; Close writes that reason, or adds the check.

**R3 – "These games are really gone" accepts whatever the next read gives.** The coordinator
confirms what the warning showed (for example "30 instead of 71"), but `confirm_gone` reads the
source again with the guard fully off. If the source now gives 0 or 5 games, that is stored as a
good reading and all missing schedule games count as gone, without a warning the coordinator saw.
This runs against the spec's intent ("never lose games on a bad answer") and against the guard's
reason. Also, the check "the held-back reading is the newest" runs outside the advisory lock, so a
reading that lands between the check and the read is not seen (low impact today; the nightly
reading of story 5 makes it real). Checked by trace (`api/readings.rs` `confirm_gone`,
`reading::read(.., false)`), not run. Suggest: pass the held reading into the read and, under the
lock, accept only when it is still the newest reading and the new count is at least the held
count; otherwise hold back again.
- 2026-10-12: fixed. The confirmation is a reading mode (`reading::Mode::Confirm`). Under the lock it checks that the reading is still the newest that did not fail and is held back (else 400 `not_held_back`), and `team_core::guard_after_confirm` accepts only at least as many games as the warning showed; else it is held back again. Tests: `really_gone_never_loses_more_games_than_confirmed`, core `after_a_confirmation_*`.

**R4 – an older source answer can be stored over a newer one.** The source is read before the
lock (by design, so a slow source never holds it). With two readings, A reads the source first and
B second, but B can take the lock first. Then A compares its older answer with B's newer stored
games, counts each change of that gap back again (for example "1 moved" back to the old date) and
stores the older data; the next reading counts it forward once more. For story 4 that is a change
recorded more than once, which requirement 7 rules out. Rare (needs a change at the source between
two reads seconds apart, and the lock order reversed), and the concurrency test cannot see it (both
readings get the same answer). Checked by trace. Suggest: under the lock, if a good reading of this
club and season with a later `started_at` is already stored, do not write; answer with that
reading, or with zero counts.
- 2026-10-12: fixed. Under the lock, if a good reading of the club and season that started later is stored (`readings::good_started_after`), the reading writes nothing and answers with that newer reading. Test: `an_older_answer_is_not_stored_over_a_newer_one`.

**R5 – note: places are shared between seasons, so a reading can change what an earlier season
shows.** `places` is keyed by the source's place number only, and `games::write` updates hall and
town. In the test data 15 places appear in both 2025-2026 and 2026-2027 (run: a short Node script
over `tests/data/game-list/games-37-*.json`). A new hall name in 2026-2027 changes the hall of the
2025-2026 games. That follows the design (requirement 10), but requirement 9 reads wider, and the
test `a_reading_never_changes_another_season` cannot see it: its place 1 is not in the read data,
and it does not check places. Decide in 3b whether earlier seasons may show the current hall; if
yes, say so in `architecture.md` at Close.
- 2026-10-12: not fixed: for story 3b and Close (`architecture.md`).

**R6 – claims in this file.** "All checks pass" is true for everything this review ran (above);
e2e not checked. The concurrency test can fail (mutation run). The rollback test can fail: its
reading adds a new place (99999) and a moved game, so a partial write would change the snapshot or
the places count. The log test checks the warning's fields and keeps ids and game numbers only, as
`CLAUDE.md` requires. Checked by reading the tests and the runs above.
- 2026-10-12: no change needed.

**R7 – siblings.**
- A1 (same choice saved again resets "first reading") has twins: "Save club" for an empty season
  and any save from "Change" without a real change also renew `chosen_at` (`followed_teams::save`
  always sets it). The fix in R1 (compare with the club and team of the last good reading) covers
  all of them; the fix "keep `chosen_at`" covers them only if done in `save` itself. Checked by
  reading `followed_teams.rs` and `api/selection.rs`.
- A2 has a server twin: after held-back then failed, `confirm-gone` on the held reading answers
  400 `not_held_back`, so the coordinator cannot confirm until a new reading is held back again.
  Consistent with A2's suggested fix only if both use "the newest reading that is not failed".
  Checked by reading `confirm_gone`.
- `target` reads `followed_teams` twice (`for_season` or `newest_before`, then `chosen_at`) in
  separate statements; a save between them pairs one row's club with another save's `chosen_at`.
  Harmless today (one coordinator); disappears with R1's fix. Checked by reading.
- `Counts` exists three times with the same nine fields (`team_core::Counts`,
  `readings::StoredCounts`, `api::readings::CountsView`) plus two hand-written conversions each; a
  tenth kind in story 4 must be added in five places, and a missed one compiles. Low; checked by
  reading.
- 2026-10-12: the A1 twins are fixed with A1 (no `chosen_at`); the A2 server twin is fixed with A2; the double read of the choice is gone (`target` reads it once, `followed_teams::chosen_at` removed). Not fixed: `Counts` three times (low; the core has no serde, decision 0001).

**R8 – documents and comments the diff made false or ahead of time.**
- `crates/server/src/followed_teams.rs` (comment on `save`) and `migrations/0003_games.sql` cite
  decision 0014, which does not exist yet (planned at Close; the migration file can no longer be
  edited after release, so 0014 must get that number). Search: `0014` in `crates`, `migrations`.
- `docs/architecture.md`: "Routes read the source before they touch the database" is no longer true
  (`reading::read` reads `followed_teams` first, and records failures); the module list, the
  migration list (`0001`, `0002` only), the API list and the error list (readings answer 200 with a
  recorded failure, new 400 codes `no_choice`, `not_held_back`) need the 3a state at Close.
- `docs/product/`: no document for reading the game list yet; `choose-club-and-team.md` line 24
  ("The game list is read each time the coordinator searches, opens the teams, or saves") is now
  incomplete. For Close.
- Decision 0013: see R2.
- 2026-10-12: not fixed: for Close (decision 0014, `architecture.md`, `docs/product/`, decision 0013).

**Summary.** No finding blocks the checks, and the lock, the transaction and the comparison work as
designed (tests can fail, all pass). To fix before Close: R3 (the confirmation can lose more games
than the coordinator confirmed) and, together with A1, R1's "first reading" from the last good
reading's club and team. To decide or record at Close: R2 (decision 0013), R4 (fix or accept as
known debt), R5, R8. R7's `Counts` duplication is optional.
