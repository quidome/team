# Technical spec: The nightly reading

**Status**: done
**Size**: story
**Change**: docs/changes/2026-10-14-the-nightly-reading
**Date**: 2026-10-14

## Objective
Read the game list every night without the coordinator, with the same code as a tap, and warn
on the start page when the games may be out of date.

## Requirements summary
Story 5 of the epic [Games from the game list](../2026-10-10-games-from-game-list/technical.md),
from functional story 5 ("Nightly reading"):

1. The tool reads the game list once a night, without the coordinator. Run moment: 03:00 Dutch time (coordinator, 2026-10-14).
2. The nightly reading records changes in the same way as a reading on demand (story 2 of the spec): same comparison, same guard (a reading with no or far fewer games is held back), same open changes.
3. When the nightly reading fails, the start page shows this the next time the coordinator opens it, with the time of the last good reading. (Already true: the "Game list" card shows the newest failed reading and "Last read".)
4. Before a club and a team are chosen, there is no nightly reading. In a new season the tool knows the club but no team yet: no nightly reading then either.
5. Several nights of failure in a row: the screen clearly shows that the games may be out of date, and since when. Fixed as: no good reading for 3 run moments (coordinator, 2026-10-14). A failed or held-back reading is not a good reading.
6. A failed nightly reading is not tried again the same night (coordinator, 2026-10-14).
7. Only the coordinator sees the result (decision 0007); the timer itself has no route.

The coordinator chose a timer inside the app instead of the Kubernetes CronJob of decision 0005
(2026-10-14): the app always runs as one pod (`replicaCount: 1` in the team chart in
`gitops-lab`), the reading lock already makes two readings at once safe, and readings are stored
with their result, so a timer in the app is not hidden. No change in `gitops-lab`.

Decisions that apply: 0001 (rules in a core without I/O), 0002 (generated API types), 0004
(compile-time checked queries), 0005 (one reading at a time by an advisory lock; its CronJob part
is replaced by this story), 0007 (every `/api` route needs a session), 0014 (Dutch local dates
and times), 0015 (readings are recorded results).

## Current state
- `crates/server/src/reading.rs`: `read(pool, source, clock, mode) -> Result<Reading, ReadingError>` does a whole reading: `target` (club, season and team of the current choice; in a new season the club of the newest earlier choice with `team: None`; `NoChoice` when nothing was ever chosen), the source outside the database, then one transaction under `db::lock_readings` (`pg_advisory_xact_lock`), the guard (`Mode::Guarded`), the comparison and the open changes. A failed or held-back reading is stored and returned, not an error. A good reading that started later and is already stored wins (`readings::good_started_after`).
- `crates/server/src/readings.rs`: `insert`, `latest` (newest by id), `last_good`, `newest_not_failed`, `good_started_after`, `by_id`. Table `readings` has `started_at` and `finished_at` (`timestamptz`).
- `crates/server/src/clock.rs`: trait `Clock` (`now() -> DateTime<Utc>`), `SystemClock`, `ShiftedClock` (`DEV_NOW`, debug builds only), `today_in_netherlands`, `current_season` (time zone `chrono_tz::Europe::Amsterdam`).
- `crates/server/src/main.rs`: `serve` builds `AppState { pool, config, oidc, clock, web, source }`, runs `axum::serve` with graceful shutdown, then `pool.close()`.
- `crates/server/src/api/readings.rs`: `GET /api/readings/latest` gives `LatestReadings { latest, last_good, held_back }`; all `null` with no choice.
- `web/src/lib/components/ReadGameList.svelte` (the "Game list" card): the read button, the result line after a tap, "Last read: <time>" (`dutchTime` in `web/src/lib/reading.ts`), "The last reading failed: …" for a failed newest reading, and the held-back warning (`border-destructive/40 bg-destructive/10`, `role="alert"`), which a good tap hides.
- `crates/core` uses `chrono` without a clock and without time zones.
- Tokio features in `crates/server/Cargo.toml`: `macros`, `rt-multi-thread`, `signal`, `net`, `sync` (no `time`).
- Tests: `crates/server/tests/readings.rs` (readings on demand, the guard, two readings at once, an older answer); harness `crates/server/tests/common/game_list.rs` (`Reader::chosen`: clock 12 October 2026 12:00 UTC, Blue Drakes M16 1 chosen, a fake game list whose answer the test sets; `utc(...)`); `Harness` has `pool`, `source` (the fake game list server) and `clock` (`set`, `advance`). A session lives 8 hours: a test that moves the clock to another day and then calls the API needs `Reader::log_in_again` first; `tick` needs no session. Browser tests run the real server with `DEV_NOW=2026-10-12T10:00:00Z` and a fresh database.

## Steps

### Step 1: Core rules and the Dutch local time
- **Goal**: when a nightly reading is due, and when the games are out of date; no clock, no time zone.
- **Changes**: new module `crates/core/src/nightly.rs`, exported from `lib.rs`. All times are Dutch local time (`NaiveDateTime`).

  ```rust
  /// The moment of the nightly reading, Dutch time.
  pub const NIGHTLY_AT: NaiveTime; // 03:00
  /// The number of run moments without a good reading after which the games may be out of date.
  pub const OUT_OF_DATE_NIGHTS: u32 = 3;

  /// The last run moment at or before `now`: today 03:00, or yesterday 03:00 before 03:00.
  pub fn last_run_moment(now: NaiveDateTime) -> NaiveDateTime;

  /// Due when a team is chosen and no reading of the club and season (of any result) started
  /// at or after the last run moment.
  pub fn nightly_due(team_chosen: bool, now: NaiveDateTime, newest_start: Option<NaiveDateTime>) -> bool;

  /// True when the last good reading started before the run moment `OUT_OF_DATE_NIGHTS - 1`
  /// nights before the last run moment: that many run moments passed without a good reading.
  pub fn out_of_date(now: NaiveDateTime, last_good_start: NaiveDateTime) -> bool;
  ```

  Earlier run moments are worked out on the date (date minus n days, at 03:00), not by
  subtracting 24 hours, so the clock-change nights do not shift them.

  `crates/server/src/clock.rs`: `pub fn in_netherlands(at: DateTime<Utc>) -> NaiveDateTime`
  (`at.with_timezone(&Amsterdam).naive_local()`), next to `today_in_netherlands`. 03:00 exists
  exactly once on both clock-change nights (the hour that repeats or is skipped is 02:00–03:00),
  so the run moment is never ambiguous.
- **Tests** (unit tests):
  - `last_run_moment`: 12 Oct 2026 02:59 → 11 Oct 03:00; 03:00 → 12 Oct 03:00; 23:00 → 12 Oct 03:00.
  - `nightly_due`: no team → false; no reading at all → true; newest start 02:59 today → true (at 03:01); 03:00 today → false; 08:00 today → false (at 09:00).
  - `out_of_date`: last good Monday 12 Oct 03:00. Wednesday 14 Oct 10:00 → false; Thursday 15 Oct 02:59 → false; Thursday 03:00 → true. Last good Wednesday 08:00 (a tap) → false on Thursday 03:00.
  - Clock-change nights: on 25 October 2026 and 28 March 2027, `last_run_moment` at 04:00 gives 03:00 of the same date, and at 02:30 gives 03:00 of the day before.
  - `clock::in_netherlands`: 2026-10-25 01:00 UTC → 02:00 (winter time), 00:30 UTC → 02:30 (summer time); 2027-03-28 01:00 UTC → 03:00.

### Step 2: One check of the timer
- **Goal**: `tick` decides and reads; callable from tests without waiting.
- **Changes**:
  - `crates/server/src/readings.rs`: `pub async fn newest_start(db, club, season) -> Result<Option<DateTime<Utc>>, StoreError>` (`SELECT max(started_at) FROM readings WHERE club_number = $1 AND season_year = $2`). The latest start, not the newest row: a reading that started earlier can be stored later. Then `just prepare` and commit `.sqlx/`.
  - New module `crates/server/src/nightly.rs` (declared in `lib.rs`):

    ```rust
    /// One check: reads the game list when a nightly reading is due. `None` when not due.
    pub async fn tick(pool: &PgPool, source: &Source, clock: &dyn Clock)
        -> Result<Option<Reading>, ReadingError>
    ```

    `reading::target` → `None` or `team: None` gives `Ok(None)`. Else `readings::newest_start`,
    both times through `clock::in_netherlands`, `team_core::nightly_due`; when due,
    `reading::read(pool, source, clock, Mode::Guarded)` and `Ok(Some(reading))`.
    The check is outside the lock: two pods can both read during a rollout at 03:00; the lock and
    `good_started_after` keep the data correct, and the cost is one extra request to the source.
- **Tests** (new file `crates/server/tests/nightly.rs`, `#[sqlx::test]`, `Reader::chosen` and a `Source` built on `harness.source.uri()`):
  - No choice at all (plain `Harness`) → `None`; the fake game list got no request.
  - New season without a team: clock 2027-07-02 01:30 UTC (03:30 Dutch) → `None`, no request.
  - `Reader::chosen` and a first reading by `read_good` on 12 Oct 12:00 UTC (the harness reads nothing by itself); one game moved in the fake answer; clock 13 Oct 01:01 UTC (03:01 Dutch) → a good reading; `GET /api/changes` shows the moved game, as after a tap.
  - Directly after it, `tick` again → `None`.
  - Pod down at 03:00: clock 13 Oct 07:00 UTC → `tick` reads.
  - A tap at 13 Oct 06:00 UTC, clock 07:00 UTC → `None`.
  - Fake game list answers 500 at 13 Oct 01:01 UTC → a failed reading is stored; at 01:06 UTC `None`; at 14 Oct 01:01 UTC it reads again.
  - Empty game list at night → a held-back reading; `GET /api/readings/latest` gives it in `held_back`.

### Step 3: The timer in the server
- **Goal**: the nightly reading runs in the deployed app.
- **Changes**:
  - `nightly.rs`: `pub async fn run(pool: PgPool, source: Arc<Source>, clock: Arc<dyn Clock>)`: `tokio::time::interval(5 minutes)` with `MissedTickBehavior::Delay`; the first tick is at once (a pod that starts after 03:00 reads then). Each tick calls `tick`. `Ok(Some(reading))` logs one info line "nightly reading" with the result kind and the number of games; `Ok(None)` logs nothing; `Err(error)` logs an error line and the loop goes on. No retry: a failed reading is stored, so the next check waits for the next night.
  - `main.rs`, `serve`: `let nightly = tokio::spawn(nightly::run(pool.clone(), source, clock))` (clones of the `Arc`s also used in `AppState`) before `axum::serve`; after it stops: `nightly.abort()`, then `pool.close()`. Without the abort, `pool.close()` can wait for a reading that holds a connection; a reading stopped halfway rolls back (one transaction).
  - `crates/server/Cargo.toml`: tokio feature `time`.
- **Tests**: no automatic test of the loop (no logic of its own; a test with real waiting is slow and unstable). Check by hand and write the result under Progress: `just dev` with `DEV_NOW` at 03:00 Dutch time and a chosen team whose last reading is before it → one "nightly reading" log line; stop with Ctrl-C → the server stops at once.

### Step 4: "May be out of date" on the start page
- **Goal**: the warning after 3 nights with no good reading.
- **Changes**:
  - `api/readings.rs`: `LatestReadings` gets `out_of_date_since: Option<DateTime<Utc>>` (`#[schema(value_type = Option<String>, format = DateTime)]`, doc comment: the time of the last good reading, when 3 run moments passed without a good reading; `null` without a chosen team). Set when `target.team` is `Some`, a last good reading exists, and `team_core::out_of_date(in_netherlands(now), in_netherlands(last_good.started_at))`; the value is `last_good.finished_at` (the time "Last read" shows). Then `just api-types`.
  - `ReadGameList.svelte`: when `out_of_date_since` is set, the first line of the card: "The games may be out of date since Mon 12 October 2026 03:00." (`dutchTime`), `role="alert"`, `data-testid="out-of-date"`, in the warning style of the held-back warning. A good reading from this page hides it (as for the held-back warning).
- **Tests**:
  - `tests/readings.rs`: good reading by `read_good` on Monday 12 Oct 12:00 UTC; failed readings (fake answer 500) on 13, 14 and 15 Oct 01:01 UTC; on 14 Oct 08:00 UTC `out_of_date_since` is `null` (the third-last run moment is Monday 03:00, before the good reading); on 15 Oct 01:02 UTC it is the `finished_at` of Monday's reading; after a good tap it is `null`.
  - `tests/readings.rs`: no choice → `out_of_date_since` is `null`.
  - Browser test (Playwright, project `logged-in`): `page.route` gives `/api/readings/latest` with `out_of_date_since` set → the line shows as an alert; after "Read the game list" (a good reading from the fake source) it is gone.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| Nightly reading by a timer in the app, not a Kubernetes CronJob | One pod; the lock makes two readings safe; readings are stored with their result; no chart change in another repository (coordinator, 2026-10-14) | yes, 0017 at Close; 0005 becomes "replaced by 0017" (its lock part moves into 0017) |
| The timer checks every 5 minutes whether a reading is due, instead of sleeping until 03:00 | Covers a pod that starts after 03:00 and a clock that jumps (`DEV_NOW`) with one rule | no (design detail, in 0017) |
| Due = no reading of any result started since the last run moment | A tap after 03:00 makes the nightly reading unnecessary; a failed night is not retried (coordinator) | no |
| Run moment 03:00 Dutch time, fixed in the core | The spec's example; no wish to change it (coordinator, 2026-10-14) | no |
| Out of date = the last good reading started before the third-last run moment; shows the time of that reading | No database change; a good tap clears it (coordinator, 2026-10-14) | no |
| No warning without a chosen team, or without any good reading in the season | No nightly reading is expected then; the card already shows a failed reading | no |

## Not in scope
- The `team read-game-list` command and the CronJob in `gitops-lab` (replaced by the timer).
- A retry in the same night, a setting for the run time, a setting to turn off the timer.
- The known technical debt "the start page reads the club list twice while a team is not chosen" (stays in `docs/architecture.md`; its words "look at it with the nightly reading (story 5)" go at Close).

## Test plan
- **Existing tests that must still pass**: all of `just check`, especially `tests/readings.rs` (the timer uses the same `reading::read`) and the browser tests (the real server now runs the timer; see Compatibility).
- **New tests**: per step, above.
- **By hand**: the loop and the stop at shutdown (step 3).
- **Checks**: `just check`; `just prepare` after step 2; `just api-types` after step 4.

## Compatibility and performance
- No migration. The API gets one new field; the front-end types are generated again.
- The cluster contract (`README.md`, `gitops-lab`) does not change.
- Load: two small queries every 5 minutes, and one request to the game list a night.
- Browser tests: the server's clock starts on 12 October 2026 12:00 Dutch time and the database is empty, so the first check finds no choice. After the choose test a reading exists after 03:00, so the timer reads nothing. Only a check in the short time between the save of the choice and its first reading could read. Very unlikely; if the tests become unstable, add a setting to turn off the timer.
- During a rollout at 03:00 both pods can read once. Accepted (see step 2).

## Progress
- [x] Step 1: core rules and the Dutch local time
- [x] Step 2: one check of the timer
- [x] Step 3: the timer in the server (and the check by hand)
  - 2026-10-14, check by hand (a scratch database, the fake source of the browser tests, `DEV_NOW`): with the clock at Monday 12 October 12:00 Dutch time and an empty database, no nightly reading at start; M16 1 chosen and read. A new start at Tuesday 13 October 03:00:30 Dutch time logged at once `"nightly reading", result good, games_read 71`, and stored reading 2. Ctrl-C stopped the server in 8 ms ("stopping", "stopped").
- [x] Step 4: "may be out of date" on the start page
- [x] All checks pass (`just check`, 2026-10-14; 30 browser tests)
- [x] Review build (2026-10-14; R1 and R2 fixed, `just check` passes again)

## Deviations
None from `functional.md`. The design reads the spec more strictly within its words:
- "Once a night" is fixed as 03:00 Dutch time, and "the exact time is a Design choice".
- "Several nights of failure in a row" is fixed as 3 run moments without a good reading (a held-back reading is not a good reading either). The "since" time is the time of the last good reading.
- "Before a club and team are chosen, there is no nightly reading" also covers a new season in which the club is known but no team is chosen yet.

Differences from the epic's technical spec (agreed by the coordinator, 2026-10-14): a timer in
the app instead of `team read-game-list` and a CronJob in `gitops-lab`; the epic's story 5 tests
"no selection → nothing and exit 0" and "the CronJob passes `helm lint`/`helm template`" become
"no choice → `tick` reads nothing" and the check by hand of step 3; closure `review build`
instead of `checks`.

Implement:
- 2026-10-14, step 4: the browser test is in `web/e2e/choose.e2e.ts` (Playwright project `choose`), not in the project `logged-in`. The card shows only with a team chosen, and only `choose.e2e.ts` has M16 1 chosen; `logged-in` runs after it with a team still needed.

## Review findings

Build review, 2026-10-14 (fresh agent; input: `build docs/changes/2026-10-14-the-nightly-reading`).
Diff `main..feat/nightly-reading` (commit 915a08d, working tree clean), read against
`docs/architecture.md`, decisions 0001–0016 and `CLAUDE.md`. Run: `cargo test -p team_core nightly`
(9 passed), `cargo test -p team --test nightly` (7 passed), `--test readings` (19 passed),
`cargo test -p team --lib clock` (passed), `cargo clippy --workspace --all-targets -D warnings`
(clean), `just e2e` (30 passed). Not run: the whole `just check`, the check by hand of step 3.

Summary: the build follows the design and the project's rules (rules in the core, no time zone in
the core, checked query with `.sqlx/` committed, generated API types, no new route, no secrets in
the logs, abort before `pool.close()`). One finding to fix or decide (R1); the rest are notes.

- **R1 – medium: a reading that ends in an error is tried again every 5 minutes, unseen.** A
  failure of the source is stored as a failed reading, so `nightly_due` is false until the next
  night. But `reading::read` can also end in `Err` without storing a row: any `StoreError`
  (database unreachable, the write of games, changes or the reading fails; the existing test
  `a_failure_while_storing_changes_nothing` shows such a failure stores nothing). Then
  `readings::newest_start` is unchanged, the next `tick` 5 minutes later finds the reading due
  again, reads the source again and logs another error – until the next good or failed row. A
  failure that keeps coming back (a write that the data of the game list breaks) means one request
  to the federation's game list every 5 minutes, all day, every day. This goes against
  requirement 6 ("not tried again the same night") and the claim under Compatibility "one request
  to the game list a night", and the coordinator sees nothing of it: no "The last reading failed"
  line (requirement 3), only the out-of-date warning after 3 nights. Checked by a trace
  (`nightly::run` → `tick` → `newest_start` → `nightly_due`; `reading::read` returns `Err` before
  `readings::insert` on every `?` inside the transaction); not reproduced by a run. Either keep
  the last run moment tried in `run` (memory is enough: one pod) and skip until the next one, or
  decide that retrying after an internal error is wanted, bound it, and write that down as a
  deviation. Sibling checked: `ReadingError::Client` comes only from building the HTTP client
  (`Source::with_timeout`), not from a read, so it cannot loop (search `SourceError::Client` in
  `crates/server/src/source/mod.rs`).
  - 2026-10-14: fixed. `nightly::tick` takes a `Tried` (the run moment this timer already tried, in memory) and tries each run moment once, also when the reading ends in an error that stores nothing; `run` keeps one `Tried`. New test `a_reading_that_ends_in_an_error_is_not_tried_again_the_same_night` (a trigger makes the store fail: no second request at 03:06, a good reading the next night). After a pod restart the moment is tried once more (accepted: one pod, rare).
- **R2 – low: a panic in the timer ends it silently.** `main.rs` keeps the `JoinHandle` only to
  abort it; if `nightly::run` ever panics, the task ends, the server goes on serving and no
  nightly reading happens again until the pod restarts (only the out-of-date warning after 3 nights
  shows it). No panic source found in `tick` or `team_core::nightly` (search `unwrap|expect|panic|\[`
  in both: only the `const` 03:00, which is checked at compile time). A note, not a fix: a
  `catch_unwind` around each `tick`, or an error log when the handle ends early, would make it
  visible.
  - 2026-10-14: fixed. `nightly::spawn` starts `run` and a second task that waits for it; any end that is not the abort at shutdown logs the error "the nightly reading stopped; it starts again with the server". `main.rs` keeps only the abort handle. Not tested automatically (no panic can be made in `run`).
- **R3 – note: the timer can read in the window between saving a choice and the front end's
  first reading.** Architecture line 36: after a save with a team the front end starts
  `POST /api/readings`. With a new club (no reading of the club this season), `nightly_due` is true
  at any hour, so a timer check in that window does the first reading and the tap gets "No
  changes". Harmless under the lock and `is_first_reading` (traced, not run); the design already
  names this for the browser tests. It holds in production too, which the Compatibility section
  does not say.
  - 2026-10-14: not changed. Harmless in production too (the lock and the first-reading rule); the choose flow shows no result line for that reading.
- **R4 – claims under Progress.** "All checks pass … 30 browser tests": the parts re-run above
  pass, 30 browser tests. The check by hand of step 3 was not repeated. No test that cannot fail
  was found: `a_pod_that_was_down_at_three_reads_when_it_checks` would pass with an always-due
  rule, but `a_tap_after_three…` and the second `tick` in
  `the_nightly_reading_records_changes_as_a_tap_does` would not; the browser test fails if the
  good tap does not clear the warning (the component keeps its first props, so the replaced
  answer after `onRead` cannot bring it back). Not covered by a test: the `target.team.is_some()`
  filter in `api/readings.rs::latest` (a good reading of the current season with no team chosen);
  seems unreachable today, since the read button needs a team.
  - 2026-10-14: not changed. The `team.is_some()` filter on the warning is not reachable today; no test.
- **R5 – documents the diff made false, for Close.** Planned already: decision 0005 (still
  "valid", says CronJob; 0017 at Close) and the debt line in `docs/architecture.md` (line 110).
  Not named in the spec: `docs/product/read-the-game-list.md` line 27 and
  `docs/product/list-of-changes.md` line 26 ("a nightly reading comes later");
  `docs/architecture.md` line 15 (core modules, no `nightly`), line 16 (server modules, no
  `nightly.rs`; `main.rs` `serve` now also runs the timer), and the reading section (line 38 on)
  says nothing of the timer or `newest_start`; `CLAUDE.md` architecture list (no `nightly.rs`).
  Search: `grep -n -i "nightly\|cronjob\|read-game-list"` in `docs/`, `README.md`, `CLAUDE.md`.
  `README.md` cluster contract stays true (`team serve`, no CronJob named).
  - 2026-10-14: for `/close`, together with decision 0017.
