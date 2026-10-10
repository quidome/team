# 0001 – Rust backend in a workspace with a pure core

**Status**: valid
**Date**: 2026-10-10
**Change**: [2026-10-10-games-from-game-list](../changes/2026-10-10-games-from-game-list/)

## Context
The product starts over. The old SvelteKit/TypeScript app is not good enough and is replaced. The
vision left the backend language open (Rust preferred, or Go), and `.claude/team.md` says the
first design must decide the code structure.

## Decision
The backend is Rust, in a Cargo workspace:
- `crates/core`: the product rules (games, seasons, schedule selection, comparing readings into changes, guards). It has no network, database or web dependencies.
- `crates/server`: everything with I/O (source reader, PostgreSQL storage, web server, login) and the commands `serve`, `migrate` and `read-game-list` in one binary.

Dependencies point inward: `server` uses `core`, never the other way.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| Rust, workspace with a separate core crate | The compiler enforces "no I/O in the core"; rules tested without mocks; the coordinator prefers Rust | More files than one crate |
| Rust, one crate with modules | Simplest start | The boundary is only a convention |
| Go | Simple, fast builds | Not the coordinator's preference |

## Consequences
- Most tests are fast unit tests on `core`.
- New rules go in `core` first; `server` only moves data in and out.
- Checks: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`.
