# 0006 – A new database for the new app

**Status**: valid
**Date**: 2026-10-10
**Change**: [2026-10-10-games-from-game-list](../changes/2026-10-10-games-from-game-list/)

## Context
The PostgreSQL server `team-postgresql` holds the database `team` of the old app. The
coordinator does not use the old app, and the new app shares nothing with its tables.

## Decision
1. Back up the database `team`.
2. Create a new, empty database on the same server for the new app, and point `DATABASE_URL` in Vault to it.
3. Delete the old database `team` only after the new app works.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| New database, delete the old one later | Every step can be undone until the last; no old tables in the new app | One more manual step |
| Drop the old tables in the first migration | No manual step | Cannot be undone; the new app must know the old tables |

## Consequences
- The coordinator runs these steps on the cluster; story 1 of the change writes them down.
