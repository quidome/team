# Technical spec: Domains in the core

**Status**: done
**Size**: story
**Change**: [2026-11-03-domains-in-the-core](./)
**Date**: 2026-11-03

## Objective
Group the modules of `crates/core` into six domains and check the direction of their use in a
test, so no domain gets the knowledge of another domain woven in (decision 0027).

## Requirements summary
A technical change, no change in behaviour. Expected result:
- `crates/core/src/` has the folders `shared`, `events`, `game_list`, `team`, `contribution`, `output`, each with the modules of decision 0027.
- No pair of domains uses each other; each domain uses only the domains decision 0027 allows.
- A test fails when a domain uses a domain it may not use.
- The server and the front end do not change, apart from the imports the moves need; the API answers stay the same.

## Current state
- 17 modules flat in `crates/core/src/`; `lib.rs` re-exports their types and functions, and the server uses only these flat names (`team_core::X`), never a module path.
- Uses between modules today (from the `use crate::...` lines, test code included):

| Module | Uses |
|--------|------|
| `attendance` | `players`, `reasons`, `season`, `training` |
| `comparison` | `club`, `game`, `schedule` |
| `duties` | `club`, `game`, `players`, `schedule`, `season`, `team_choice` |
| `game` | `club`, `season` |
| `guard` | `game` |
| `messages` | `club`, `duties`, `game`, `players`, `schedule`, `season` |
| `players` | `season` |
| `schedule` | `club`, `comparison`, `game`, `season` |
| `season_view` | `schedule`, `training` |
| `start_page` | `duties`, `schedule`, `season`, `season_view`, `training` |
| `team_choice` | `club`, `game`, `season` |
| `training` | `attendance`, `players`, `reasons` (tests), `season` |
| `access`, `club`, `nightly`, `reasons`, `season` | nothing |

- The two loops:
  - `training::season_list` takes the absences and the roles and gives each opened training the state `DayStatus::Absent(n)` or `AllThere`, through `training::absent_count`. `attendance::season_overview` calls `season_list`. The server calls `absent_count` in `api/trainings.rs` (lines 206, 475) and reads `DayStatus` (line 145) and `season_list` (`api/season.rs`, line 706).
  - `comparison::StoredGame` (a stored game and whether it is still in the game list) is used by `schedule::schedule`; `comparison` uses `schedule::in_schedule`.

## Steps

### Step 1: The count of absent players goes to attendance
- **Goal**: trainings no longer use attendance or players, and no state of a training row that needs the absences is defined in events.
- **Changes**: `season_list` no longer takes absences and roles. Its state of a row has only "not opened", "opened" and "cancelled" (`DayStatus`). `absent_count` moves to `attendance`, with its own state of a training row there (`TrainingState`: not opened, all there, N absent, cancelled) and a function that gives it for a row (`training_state`). A training event of the page "Season" and the start page carries this state (`EventKind::Training(row, state)`; `season_events` takes the absences and the roles), so the server cannot show a training row without it. `attendance::season_overview`, `start_page` and the server call the new form. The test helpers of `training` that make absences move with the tests they serve.
- **Tests**: the existing tests of the list of a season and of "N absent" keep their cases, now on the attendance side; the API tests of trainings and of the page "Season" pass unchanged.
- *Changed 2026-11-03 after review finding R2*: the first form kept "all there" and "N absent" in `DayStatus` (events) with an in-between state that attendance replaced; a caller that forgot the count showed "all there" without an error. The coordinator chose to move the states.

### Step 2: The stored game goes to the schedule
- **Goal**: the schedule no longer uses the comparison.
- **Changes**: `StoredGame` moves from `comparison` to `schedule`; `comparison` imports it from there. The flat name `team_core::StoredGame` stays.
- **Tests**: existing tests; no new behaviour.

### Step 3: One folder for each domain
- **Goal**: the domains of decision 0027 are visible in the code.
- **Changes**: `git mv` each module into its domain folder (`src/events/training.rs`, ...), with a `mod.rs` for each domain that lists its modules; `use crate::x` becomes `use crate::<domain>::x`. `lib.rs` declares the six domains and keeps all its re-exports. No change of code inside the modules apart from the import paths.
- **Tests**: `cargo test -p team_core` and the server tests pass unchanged.

### Step 4: The test of the direction
- **Goal**: a domain that uses a domain it may not use fails the checks.
- **Changes**: `crates/core/tests/domains.rs` holds the table of decision 0027 (domain → allowed domains). It reads every `.rs` file under `src/<domain>/` and finds the domains named in `crate::<domain>` (in `use` lines and in paths in the code), and fails with the file, the line and the rule. A file directly under `src/` other than `lib.rs`, or a folder not in the table, also fails, so a new module must be placed in a domain. Test code counts like other code.
- **Tests**: the test passes on the moved code; a unit test of the reading function in the same file shows that a forbidden `use crate::output::...` in a `contribution` file, a nested `use crate::{events::game, output::x}`, and a full path in code are found.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| Six domains with a fixed direction | The purpose in the vision; keep the domains clean | 0027 |
| A test reads the source text | Enforced at each change without one crate per domain | 0027 |
| `team_choice` in team, not in game list | The word list: the team is the team chosen from the game list for a season; team and game list do not use each other | 0027 |
| `club` in events | `game` names clubs; the search of clubs only works on a list given to it | 0027 |
| The flat re-exports in `lib.rs` stay | The server does not change; the domains are a rule inside the core | 0027 |

## Not in scope
- A split of `crates/server` by domain.
- The report of what each player did, and a contribution module for it.
- Splitting `duties` (the places of a game and who did them) into smaller modules.
- Any change of behaviour or of the API.

## Test plan
- **Existing tests that must still pass**: all of `cargo test --workspace`, the browser tests of the page "Season", trainings and the start page; `just check` once before the commit.
- **New tests**: the test of the direction and the unit test of its reading function (step 4).

## Compatibility and performance
No change for the coordinator or the API. No migration. `.sqlx/` does not change (no query
changes).

## Progress
- [x] Step 1 (2026-11-03): `season_list` gives an opened training the state `Opened`; `absent_count` moved to attendance. The test with the counts moved to attendance, with a coming training added. Redone after R2 (see the step and R2).
- [x] Step 2 (2026-11-03): `StoredGame` moved to `schedule`; `comparison` imports it from there; the flat name stays. The schedule now uses only `club`, `game` and `season`.
- [x] Step 3 (2026-11-03): `git mv` of the 18 modules into `shared`, `events`, `game_list`, `team`, `contribution` (with `game_attendance`), `output`, each with a `mod.rs`; `crate::x` became `crate::<domain>::x`; `lib.rs` declares the six domains and keeps all re-exports.
- [x] Step 4 (2026-11-03): `crates/core/tests/domains.rs` with the table of decision 0027. It reads each `.rs` file below a domain folder and finds the first name after each `crate::`, also in each part of a group `crate::{...}`. It fails with the file, the line and the rule for a forbidden domain, for a flat name (`crate::Season`, which goes around the rule through `lib.rs`), for a file directly under `src/` other than `lib.rs`, and for a folder not in the table. Unit tests: a forbidden `use` line, a group over several lines, a full path in code, a flat name, `other_crate::` and `$crate::`, and shared. Checked by hand: a forbidden import in `events/training.rs` and a file `src/stray.rs` make the test fail; both removed.
- [x] All checks pass (`just check`, 2026-11-03)

## Deviations
- 2026-11-03: `game_attendance` (who comes to a game, story A of attendance at games) was added after decision 0027, so the decision has 18 modules, not 17, and does not name it. It goes in contribution: it uses `duties`, `players`, `reasons` and `season`, and attendance at games is part of what a player gives. Accepted by the coordinator.
- 2026-11-03: the domain module `team` has the same name as the function `team_core::team` (the page "Team"). Rust keeps them apart, but `use team_core::team` in `crates/server/src/api/players.rs` then also brings in the module, which conflicts with the server's own module `super::team`. That file now calls `team_core::team(...)` by its full name; an import change only.

## Review findings

Build review, 2026-11-03, fresh agent. Input: mode `build`, this change folder; no other
arguments. Read: the diff against `main` (committed and not, including `crates/core/tests/`),
this folder, `docs/architecture.md`, decision 0027 and the decisions it builds on, `CLAUDE.md`.
`docs/architecture.md` has no domain map yet, so the yardstick for uses between domains is the
table of decision 0027.

Checked and found right:
- The uses between domains follow decision 0027. Checked by a search (`crate::(\{[^}]*|[a-z_]+)` in each domain folder): shared uses nothing; events uses shared; game list uses events; team uses events and shared; contribution uses team, events and shared; output uses contribution, team, events and shared. Nobody uses output; team and game list do not use each other.
- The moved modules change only their imports, apart from steps 1 and 2. Checked by a diff of each module at `HEAD` against its new place, with the `crate::` lines and import lists left out: nothing else changed.
- Step 1 keeps the behaviour: all three server users of the list of a season (`api/season.rs` `SeasonTrainings::list`, which `api/start.rs` lines 194 and 235 also use) go through `count_absent`; `season_overview` uses only `not_opened` and does not need the counts. Checked by a search for `season_list`, `count_absent`, `absent_count` and `.list(` in `crates/`, and by a run: `cargo test -p team --test trainings --test season --test start --test attendance` passes (46 tests; `trainings.rs` lines 151–154 and 455–456 assert "absent" with a count and "all_there").
- The claims under Progress: `cargo test -p team_core` passes (312 unit tests and the 5 of `tests/domains.rs`); `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check` are clean (runs). The hand check is true: in a copy of the crate in the scratch folder, `use crate::contribution::attendance::Absence` in `events/training.rs` and an empty `src/stray.rs` make the test fail with the file, the line and the rule (run). The unit tests of the reading function assert exact results and can fail.

**R1 – The test of the direction does not see a `super::` path (medium).** It reads only the
names after `crate::`. In the same scratch copy, `use super::super::output::messages as _m;` and
`super::super::contribution::attendance::absent_count` in `events/training.rs` compile, and the
test passes (run). So a forbidden use is one rewrite of the path away, the case decision 0027
names ("a wrong connection is easy to miss, for people and agents"). The twin: the test counts
`$crate::` as not a break (unit test `a_flat_name_and_other_crates_are_told_apart`), but in a
`macro_rules!` of this crate `$crate::output::y` is this crate's `output`, so it is a use like
`crate::output::y`. Neither form is in the code today (search for `super::super`, `$crate`,
`macro_rules`, `#[path`, `crate as` in `crates/core/src`: no hits). A possible fix: fail on any
`super::super` (or read a path of `super::` that climbs to `src/` as `crate::`), and read
`$crate::` like `crate::`.

**R2 – The meaning of "N absent" and "all there" stays in events, and an uncounted list shows "all there" without an error (medium).**
Step 1 moved the count, not the states: `DayStatus::AllThere` and `DayStatus::Absent(usize)`
are still defined in `events/training.rs`, with the doc line "The number of absences of players
with a role on the date", and `season_list` now gives a state `Opened` that only
`attendance::count_absent` replaces. Nothing in the types makes a user call `count_absent`; the
server's `row_view` (`api/trainings.rs` line 146) maps `Opened` to `"all_there"` with a comment
that it does not come. A later route that calls `season_list` and forgets `count_absent` shows
"all there" for a training with absences, and no test or check fails. That is the opposite of the
consequence in decision 0027 ("a change to the meaning of an absence no longer reaches the
trainings") and of the rule that new rules go in the core, not in a mapping in the server. It
follows the letter of step 1, so the spec, not only the build, would need to say which. Options:
let `row_view` refuse `Opened` (a 500 with a log line, or a debug assertion) so a forgotten count
shows; or give the attendance its own state for a training row (the count, or "all there" /
"N absent") next to an events state with only not opened, opened and cancelled. Checked by a
trace (search for `DayStatus::` in `crates/core/src` and `crates/server/src`).

**R3 – Documents the diff made false (low; for Close).**
- `docs/architecture.md` line 15 still lists the 18 modules flat and says `training` has "the list of a season `season_list` with `absent_count`"; `absent_count` and `count_absent` are now in `attendance`, `StoredGame` is in `schedule`. Line 83 says `season_list` gives "the status of each row"; it now gives the status without the count. Decision 0027 (Consequences) says the architecture document shows the domains; it has no domain map yet. Checked by a search for `absent_count`, `season_list` and `domain` in `docs/`.
- Decision 0027 says 17 modules and does not name `game_attendance`; the Deviation records it. The record is permanent, so the place of `game_attendance` in contribution should land in the domain map of `docs/architecture.md` at Close.
- `CLAUDE.md` (Architecture) describes `crates/core` as a flat list of subjects, without the six domains or decision 0027. Still true as a list, but the rule a new module must follow (a domain folder, the direction test) is not there. Checked by reading.
- Checked and still true: `docs/product/season.md` line 37 ("which training rows show is `team_core::season_list`"), and the module docs of the six `mod.rs` files against decision 0027.

Answers, 2026-11-03:
- R1: fixed. The test reads `$crate::` like `crate::`, and any `super::super` in a domain file is a break with the file and the line. New unit test `a_macro_path_and_a_path_up_two_levels_are_found`; the test of `other_crate::` no longer counts `$crate::` as allowed. No `super::super` or `$crate` in the code.
- R2: fixed by a change of step 1, chosen by the coordinator (move the states, not a loud error in the server or an accepted deviation). `DayStatus` in events has only `NotOpened`, `Opened`, `Cancelled`. `attendance` has `TrainingState` and `training_state(row, absences, roles)`; `count_absent` is gone. `EventKind::Training` carries the row and its state, filled by `season_events` from the absences and the roles. The server's `row_view` takes the state; `SeasonTrainings` gives `events` and `state`; the last training of the start page takes the state from the trainings of its season, and a missing season is an error, not a silent state. `just check` passes.
- R3: for Close (the domain map in `docs/architecture.md` with `game_attendance` in contribution, lines 15 and 83, the rule for a new module in `CLAUDE.md`).


Build review 2, 2026-11-03, fresh agent, after the answers to R1–R3. Input: mode `build`, this
change folder; no other arguments. Read: the diff against `main` (committed and not, including
`crates/core/tests/`), this folder, `docs/architecture.md`, decision 0027, `CLAUDE.md`. The
yardstick for uses between domains is again the table of decision 0027 (`docs/architecture.md`
has no domain map yet, R3).

Checked and found right:
- The R2 fix puts the states where decision 0027 wants them: `DayStatus` in `events/training.rs` has only `NotOpened`, `Opened`, `Cancelled`, and events has no use of contribution or team (search for `absen`, `role`, `attendance` and `crate::` in `crates/core/src/events/`: only doc text). `EventKind::Training` needs a `TrainingState`, so a caller cannot build a training event without one (trace). `next_events` now skips a cancelled training by `TrainingState::Cancelled`, the same rows as before (trace).
- `RULES` in `tests/domains.rs` matches the table of decision 0027, and each `mod.rs` doc line matches it (reading).
- The moved modules other than `attendance`, `training`, `schedule`, `comparison`, `season_view`, `start_page` change only `crate::` paths and import lists (diff of each module at `HEAD` against its new place with the import lines left out: no other lines).
- The claims under Progress: `cargo test -p team_core` passes (312 unit tests, 6 in `tests/domains.rs`); `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check` are clean; `cargo test -p team --test trainings --test season --test start --test attendance` passes (46 tests). All runs. `just check` as a whole was not run again.

**R4 – The test of the direction still misses a path up to the crate root from a `mod.rs` (medium).**
The R1 fix treats `super::super` as the only way up. In a domain's `mod.rs`, one `super::` is
already the crate root. In a scratch copy of the crate, `use super::contribution::attendance::absent_count as _escape;`
at the end of `src/events/mod.rs` compiles, and `each_domain_uses_only_the_domains_it_may_use`
passes (run). The same copy also had `use crate as root;` with `root::output::messages::full_names`
in `src/shared/season.rs`: it compiles and passes too (run); that form is rare and the name after
`crate` is not followed by `::`, so the reader never sees it. Today no `mod.rs` has more than its
`pub mod` lines, and there is no `crate as` in `crates/core/src` (search for `super::` in
`src/*/mod.rs` and for `crate as`: no hits), so nothing breaks the rule now. Possible fixes: a
`super::` in a `mod.rs` of a domain is a break like `super::super`; a `crate as` (or `self as` in a
group of `crate::{...}`, which the reader already flags as "not a domain") is a break. Add a unit
test for each, as for R1.

**R5 – The state of the last training on the start page has no test that can fail (low).**
The last training of `GET /api/start` gets its state from new server code: it finds the
`SeasonTrainings` of the row by `Season::of(row.date)` and calls `state(row)` (`api/start.rs`
lines 212–218). The only API assertion of that state is `"all_there"` for a training without
absences (`crates/server/tests/start.rs` line 182), and the July test (line 276) checks only the
date. A build that always gave `TrainingState::AllThere` there, or took the trainings of the wrong
season, passes all tests (trace: search for `"absent"` and `["last"]` in `crates/server/tests/`;
the counts are asserted only on the list of `GET /api/season`, `trainings.rs` lines 151 and 455).
A test with an absence at the last training (and one in July, from the season before) would
cover it. Sibling: `SeasonTrainings::events(games, list)` and `state(row)` take a list or row that
need not come from the same `SeasonTrainings`; each caller today makes it from the same one
(trace of the three calls in `api/start.rs` and `api/season.rs`). A core `last_training` that also
returns the state, or the index of its list, would leave the server no season to look up; this is
a choice, not a bug.

**R6 – More documents the diff made false (low; for Close, with R3).**
- `docs/architecture.md` line 61 (`GET /api/start`): "`last_training` (as an `EventView`)" and "`season_events` of the current season" no longer say that the state of each training comes from the absences and roles of its season (`training_state`). Line 95 (`GET /api/season`): "`season_list`; then `season_events`" leaves out that `season_events` now takes the absences and the roles. Checked by a search for `season_events`, `season_list` and `last_training` in `docs/` and by reading.
- Decision 0022 line 20 and `docs/product/season.md` line 37 name `season_events` and `season_list` for the order and the rows: still true (reading).

Answers, 2026-11-03:
- R4: fixed. A domain file is a break when it has `super::super`, `super::` in the `mod.rs` of a domain (its parent is the crate root), or `crate as`. New unit test `a_path_up_from_the_mod_rs_of_a_domain_and_another_crate_name_are_found` (also: `super::` in a module of the domain is allowed). Checked by hand: `use super::contribution::attendance::absent_count` in `src/events/mod.rs` makes the test fail at line 7; removed.
- R5: fixed. `crates/server/tests/start.rs`: new test `last_shows_the_absent_players_of_its_training` (status "absent", 1 absent), and `in_july_the_open_items_and_last_cover_the_season_before` now has an absence at the last training of the season before and asserts "absent" and 1. Checked by hand: with the season lookup in `api/start.rs` changed to the first season, the July test fails; restored. The helpers `events` and `state` of `SeasonTrainings` stay as they are (every caller passes the list of the same season). `just check` passes.
- R6: for Close, with R3.

