# 0030 – Each domain owns its tables

**Status**: valid
**Date**: 2026-11-08
**Change**: [2026-11-08-not-played](../changes/done/2026-11-08-not-played/) (story E of [attendance at games](../changes/done/2026-10-31-attendance-at-games/))

## Context
Decision 0027 gives the core six domains with a fixed direction of use, but says nothing about the
database. The design of story E first put the mark "Not played" in `game_attendance`, a table of
attendance, although the mark also stops the duties: a table of one domain would then hold a rule
that another domain obeys. The coordinator asked for clear boundaries: a domain owns its data, and
no other domain uses its tables directly.

## Decision
- **Treat each domain as if it were a service**, but keep one program, one database and one deployment. The domains are not split into services.
- **Only the store module of a domain reads or writes its tables** (`games.rs` for events, `players.rs` for team, `duties.rs`, `game_attendance.rs`, `trainings.rs` and `reasons.rs` for contribution, `weekly_trainings.rs` for events). Other parts use them through its functions or through the rules of the core. A join stays inside one domain.
- **Allowed:** a foreign key to the key of another domain's row (so the database keeps links right; the other columns are never read), and one transaction in an API handler that calls the store functions of two domains. The API handlers combine the domains, as output does in the core.
- **"Could this domain move out?"** is a question in each design: a domain that reads another's tables is a fault; replacing the foreign keys and the shared transactions is the price paid only when a domain really moves.
- The map of the tables is in `docs/architecture.md`, "Each domain owns its tables".

## Options considered
| Option | For | Against |
|--------|-----|---------|
| Each domain owns its tables, in one program (chosen) | Clear boundaries; the code already almost followed it (one store module for each group of tables, no joins between groups) | A fact used by two domains needs a home and a function, not a quick column |
| Domains ready to move out at any time (no foreign keys, no shared transactions) | A domain can move out with no work | Checks over the network in nearly every request; broken links possible; one coordinator and one small cluster need none of it |
| No rule for tables (as before) | Nothing to do | A table of one domain can hold a rule of another, as the first design of story E did |

## Consequences
- The mark "Not played" is in its own table of events (`games_not_played`), and the switch "The drivers drove" is in `game_duties` (decision 0031). The key of a game, `GameKey`, moved from `duties.rs` to `games.rs`.
- Known debt, fixed when the code is touched: `trainings.rs` reads `player_roles` (team) to find the seasons with data, and the table `trainings` holds two domains (that a training exists, events; `checked_at`, contribution).
- Nothing checks the rule yet; a test like `crates/core/tests/domains.rs` for the SQL of the server is a line in `docs/backlog.md`.
