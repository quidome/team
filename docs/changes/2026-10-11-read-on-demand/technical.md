# Technical spec: Read the game list on demand

**Status**: approved
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
- [ ] Step 1: core rules
- [ ] Step 2: storage
- [ ] Step 3: the reading and the API
- [ ] Step 4: front end
- [ ] All checks pass

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

## Review findings
