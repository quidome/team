# 0004 – PostgreSQL with sqlx and forward-only migrations

**Status**: valid
**Date**: 2026-10-10
**Change**: [2026-10-10-games-from-game-list](../changes/2026-10-10-games-from-game-list/)

## Context
The vision left open where the data is stored. The cluster already runs a PostgreSQL server
only for this app, with backups owned by the homelab storage process. The deployment runs a
migration image before the app on every pod start.

## Decision
- Data is stored in PostgreSQL. Access through sqlx.
- Database changes are plain SQL files in `migrations/`, numbered, compiled into the binary.
- `team migrate` applies missing migrations under sqlx's lock, each in a transaction, with checksums. The `:latest-migration` image runs it. An applied file is never edited; a fix is a new file.
- Forward-only: no undo migrations. A breaking change goes in two releases (add, later remove). Recovery from a failed irreversible change is a restore from backup.
- The app does not migrate. Its readiness check fails when a known migration is missing.
- Queries are checked at compile time; the `.sqlx/` offline data is committed and checked.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| sqlx | Migrations and checked SQL in one library; SQL stays visible | Offline data must be kept up to date |
| Diesel | Strong typing | Own query language, separate migration tool |
| SeaORM | Less SQL to write | Hides the SQL |
| refinery | Simple migrations | Migrations only |
| SQLite | Simplest to run | The cluster already has PostgreSQL with backups |

## Consequences
- Database tests use `#[sqlx::test]`: a new database per test with all migrations, so migrations are tested on every run.
- `cargo sqlx prepare --check` is part of the checks.
