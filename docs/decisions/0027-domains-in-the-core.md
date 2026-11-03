# 0027 – Domains in the core, with a checked direction

**Status**: valid
**Date**: 2026-11-03
**Change**: [2026-11-03-domains-in-the-core](../changes/done/2026-11-03-domains-in-the-core/)

## Context
The vision now states the purpose of the tool: it shows what each player gives to the team, the
effort at training and games and a fair share of the duties (the word "contribution",
`docs/language.md`). The product rules in `crates/core` (decision 0001) are 17 modules side by
side. The split by subject exists, but only someone who knows the product can see it, and
nothing stops a module from using any other module. Two pairs of modules already use each other:
trainings and attendance (the list of a season counts the absent players), and the schedule and
the comparison of readings (the type of a stored game is in the comparison).

The coordinator wants the domains kept clean: a domain must not get the knowledge of another
domain woven in.

## Decision
`crates/core` has six domains, each a folder module. A domain may use only the domains below it
in this list:

| Domain | Modules now | May use |
|--------|-------------|---------|
| output | `messages`, `start_page`, `season_view`; the report later | all domains |
| contribution (the core domain) | `attendance`, `reasons`, `duties`; the totals of the report later | team, events, shared |
| team | `players`, `team_choice` | events, shared |
| game list | `comparison`, `guard`, `nightly` | events, shared |
| events | `club`, `game`, `schedule`, `training` | shared |
| shared | `season`, `access` | nothing |

- **Contribution is the core domain**: what a player gives. Team and events are what it counts; the game list only brings the games in. Output only shows; no domain uses output.
- **Team and game list do not use each other.** Both build on events.
- **A test in `crates/core` checks the direction**: it reads the `use crate::...` lines of each domain and fails when a domain uses a domain it may not use. `cargo test`, and so `just check` and CI, run it.
- **The flat names stay.** `lib.rs` keeps re-exporting the types and functions, so the server and its imports (`team_core::Training`) do not change.
- **The two loops are fixed by moving knowledge to the side it belongs to**: the count of absent players at a training goes to attendance (a training exists without attendance); the stored game goes to the schedule (events).
- `crates/server` is not split by domain now.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| Folders and a test of the direction (chosen) | The rule is enforced at each change; one crate, no new build files | The test reads source text, a simple check and not the compiler |
| Folders, checked only by review | Nothing to build | A wrong connection is easy to miss, for people and agents |
| One crate per domain | The compiler enforces the direction | Six crates with their own build files, more work for each new type; too much for this size |
| Only this record, move the code later | No work now | The rule stays on paper; new code keeps the flat structure |

## Consequences
- A new module goes in one domain. A need to use a domain that is not allowed is a design question, not a quick import: move the knowledge, or change this record.
- The report of what each player did (vision priority 7) gets its totals in contribution and its table in output.
- The architecture document shows the domains instead of the flat list of modules.
- A change to the meaning of an absence or a duty no longer reaches the trainings or the games.
