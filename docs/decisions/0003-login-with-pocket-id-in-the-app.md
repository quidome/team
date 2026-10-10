# 0003 – Login through Pocket ID, done by the app

**Status**: valid
**Date**: 2026-10-10
**Change**: [2026-10-10-games-from-game-list](../changes/2026-10-10-games-from-game-list/)

## Context
Only the coordinator may see and change data. The coordinator wants login on the API itself,
and it should be easy to add users later (not in scope now). Pocket ID (the coordinator's own
login server) already has a client for this app.

## Decision
- The server does OpenID Connect (authorization code flow with PKCE) against Pocket ID.
- Sessions are kept on the server (PostgreSQL). The browser gets only a cookie with `HttpOnly`, `Secure` and `SameSite=Lax`. The front end never holds a token.
- Every `/api` request needs a valid session, except `/api/health/liveness` and `/api/health/readiness`.
- A user may log in only when the Pocket ID subject is on the allowed users list, which a setting fills.
- `DEV_AUTH_BYPASS` works only in debug builds.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| OpenID Connect in the app | Login on the API itself; the app knows who logs in; more users is a list | Login code in the app |
| A gate in front (oauth2-proxy) | Less code in the app | The app trusts headers; security depends on the cluster setup |

## Consequences
- Login is tested with a fake OpenID Connect server.
- Adding users later means adding to the allowed list and, when needed, data per user.
- The cluster settings stay: `OIDC_ISSUER_URL`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, `SESSION_SECRET`, `SESSION_TTL_SECONDS`.
