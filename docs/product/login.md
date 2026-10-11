# Login

What it does **now**, for the coordinator. Updated at each /close that changes it.

## Purpose
Only the coordinator may see and change the team's data. The tool uses the coordinator's own
login server (Pocket ID), so logging in on the phone is quick (for example with a passkey).

## Behaviour
- Without login, the start page shows "Log in". The button goes to Pocket ID; after the login there, the coordinator comes back to the start page, which greets them by name.
- Only users in the Pocket ID group `team_coordinator` get in. Anyone else sees "No access: your account may not use this tool. Ask the coordinator." To give someone access, add them to that group in Pocket ID.
- A login lasts 8 hours (the cluster setting `SESSION_TTL_SECONDS`). After that the start page shows "Log in" again.
- "Log out" in the header ends the login at once.
- Every screen and every API answer needs a login, except the two health checks that Kubernetes uses.

## Errors and limits
- A login that does not finish within 10 minutes, or that was changed on the way, shows "Login failed. Please try again." with a "Log in" link.
- When Pocket ID cannot be reached, the tool shows "Login is not available now. Please try again later." The tool itself keeps running.
- When Pocket ID changes its signing key, the next login fetches the new key by itself; nothing needs to be restarted.
- One coordinator: there is no user management in the tool.

## Interface
- Screens: the start page (`/`) with "Log in", or the name and "Log out" in the header.
- Server addresses: `/auth/login`, `/auth/callback`, `/auth/logout` (form post), `GET /api/me` (the name of the logged-in user).
- Cluster settings: `OIDC_ISSUER_URL`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, `SESSION_SECRET`, `SESSION_TTL_SECONDS`, `TEAM_ACCESS_ROLE` (default `team_coordinator`). See `README.md`.
