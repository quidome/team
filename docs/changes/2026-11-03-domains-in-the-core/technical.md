# Technical spec: Domains in the core

**Status**: approved
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
- **Goal**: trainings no longer use attendance or players.
- **Changes**: `season_list` no longer takes absences and roles; an opened training gets a state without a count. `absent_count` moves to `attendance`, with a function there that adds the count to the rows of a season list (the state "N absent" or "all there"). `attendance::season_overview`, `start_page` and the server call the new form. The test helpers of `training` that make absences move with the tests they serve.
- **Tests**: the existing tests of the list of a season and of "N absent" keep their cases, now on the attendance side; the API tests of trainings and of the page "Season" pass unchanged.

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
- [ ] Step 1
- [ ] Step 2
- [ ] Step 3
- [ ] Step 4
- [ ] All checks pass

## Deviations

## Review findings
