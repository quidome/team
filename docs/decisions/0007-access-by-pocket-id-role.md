# 0007 – Access by Pocket ID role, own sessions

**Status**: valid
**Date**: 2026-10-10
**Change**: [2026-10-10-the-base](../changes/2026-10-10-the-base/)

Replaces 0003. It keeps what 0003 decided about login in the app, and changes how access is
decided and how sessions are stored.

## Context
Only the coordinator may see and change data, and login must protect the API itself. It must
be easy to add users later (not in scope now). 0003 planned an allowed users list filled by a
setting, and sessions through tower-sessions. The coordinator has since made the Pocket ID role
(group) `team_coordinator`, and wants users managed in Pocket ID.

## Decision
- The server does OpenID Connect (authorization code flow with PKCE) against Pocket ID, with the scopes `openid profile groups`.
- Only a user whose `groups` claim contains `team_coordinator` (setting `TEAM_ACCESS_ROLE`, default `team_coordinator`) gets a session. Others get "No access" (403). Pocket ID may also limit the client to that group as a second lock.
- Sessions are rows in an own table `sessions`. The cookie `team_session` holds 32 random bytes; the table holds only their SHA-256 hash. Cookie flags: `HttpOnly`, `Secure`, `SameSite=Lax`, `Path=/`. Fixed lifetime `SESSION_TTL_SECONDS`.
- `state`, `nonce` and the PKCE verifier live in a cookie `team_login`, signed with `SESSION_SECRET`, valid 10 minutes.
- Every `/api` route needs a session, except the two health routes. Every changing request under `/api` and `/auth` needs `Origin` equal to `ORIGIN`.
- `DEV_AUTH_BYPASS` exists only in debug builds.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| Role from Pocket ID groups | Users managed in one place; no extra setting; adding users is adding to the group | Depends on Pocket ID sending `groups` (fails closed) |
| Allowed users list from a setting (0003) | Independent of groups | A second place to keep up to date; a cluster change per user |
| Own sessions table | One migration path; about 80 lines; a database copy gives no usable session | Own code to test |
| tower-sessions | Ready-made | Own table schema and migrations; a data bag we do not need |

## Consequences
- No `allowed_users` table.
- Login is tested against a fake OpenID Connect server; the release build is tested without the bypass.
- Adding a user later: add the user to the group in Pocket ID.
