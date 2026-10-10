# Technical spec: The base

**Status**: in progress
**Size**: story
**Change**: docs/changes/2026-10-10-the-base
**Date**: 2026-10-10

## Objective
Replace the old SvelteKit app with the base of the new product: a Rust server with PostgreSQL,
migrations, login through Pocket ID and a Svelte front end with its look, deployed on the existing
cluster contract, so that every later story of the epic
[Games from the game list](../2026-10-10-games-from-game-list/technical.md) builds on it.

## Requirements summary
Story 1 of the epic. Type `technical`: no behaviour from a functional spec. Expected result:

1. The old app is removed. Kept: `docs/`, `features/`, `.claude/`, `AGENTS.md`, `CLAUDE.md` (rewritten for the new app), `.mcp.json`, `compose.yaml`, the app icons and web manifest (moved into `web/`).
2. A Rust workspace: `crates/core` (`team_core`, no I/O) and `crates/server` (binary `team`, commands `serve`, `migrate`, `openapi`).
3. `team migrate` applies the SQL migrations under sqlx's lock; a second run applies nothing. The first migration creates `sessions`.
4. `team serve` on port 3000: health without login; login through Pocket ID; only users with the Pocket ID role `team_coordinator`; `GET /api/me`; logout; serves the built front end.
5. A Svelte front end in `web/` in English with the look (colours, fonts, light and dark mode, phone first): a login state, a page after login with the name and logout, and a sample page `/sample` with fixed example games that the coordinator uses to approve the look.
6. Front-end API types generated from the server's OpenAPI description; a check catches a difference.
7. The image keeps the cluster contract (below). `main` publishes `latest` and `latest-migration`; a version tag also publishes a fixed image (decision 0008).
8. `just check` runs all checks, locally and on GitHub, in the Nix shell.
9. A checklist for the coordinator: back up the old database, create the new one, point `DATABASE_URL` to it, check the Pocket ID client, delete the old database later.
10. Deployed; the coordinator logs in on the phone and approves the look.

Decisions that apply: 0001 (workspace, pure core), 0002 (Svelte static front end), 0004
(PostgreSQL, sqlx, forward-only migrations), 0006 (new database), and from this design 0007
(access by Pocket ID role, own sessions; replaces 0003) and 0008 (image publishing).

## Current state
- The repository holds the old SvelteKit/TypeScript app: `src/` (237 files), `drizzle/`, `scripts/`, `static/`, Node configuration files, a Node `Dockerfile`, a `Justfile` and a Node `flake.nix`. The coordinator does not use it.
- `.github/workflows/container.yml` builds the targets `runtime` and `migration` on every push and publishes `ghcr.io/quidome/team:latest` and `:latest-migration` from `main` (and from `v*` tags).
- Old login: OpenID Connect against Pocket ID with scope `openid`, PKCE, state and nonce; any user that Pocket ID accepts gets in. Callback `/auth/callback`.
- Cluster contract (`gitops-lab/applications/productivity/team/`), which must stay:
  - images `ghcr.io/quidome/team:latest` (app) and `:latest-migration` (init container before the app, on every pod start), `pullPolicy: Always`;
  - port 3000; probes `GET /api/health/liveness` and `GET /api/health/readiness`;
  - settings `DATABASE_URL`, `OIDC_ISSUER_URL`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, `SESSION_SECRET` (from Vault `kv/productivity/team`), `ORIGIN=https://team.quido.me`, `SESSION_TTL_SECONDS=28800`, `DEV_AUTH_BYPASS=false`;
  - Pocket ID: issuer `https://id.quido.me`, client `team-coordinator`, redirect `https://team.quido.me/auth/callback`;
  - one replica, 256 MB memory, `readOnlyRootFilesystem: true`, user and group 1000, no privilege escalation, all capabilities dropped;
  - PostgreSQL `team-postgresql.productivity.svc.cluster.local:5432`, role `team`, old database `team`. A NetworkPolicy allows only pods labelled `app.kubernetes.io/name: team`, `app.kubernetes.io/instance: team`.
- The coordinator has made the Pocket ID role (group) `team_coordinator`.

## Design

### Layout
```
Cargo.toml              workspace
rust-toolchain.toml     fixed Rust version
crates/core/            team_core: pure rules (now: access rule)
crates/server/          team: settings, database, login, API, serving the front end, commands
  src/main.rs           commands: serve | migrate | openapi
  src/config.rs         settings
  src/db.rs             pool, migrate, readiness
  src/auth/             Pocket ID, login cookie, sessions, CurrentUser, origin rule
  src/api/              health, me, OpenAPI
  src/web.rs            static files and index.html fallback
  src/app.rs            app(state) -> Router  (one place that builds the router; tests call it)
  tests/                integration tests (fake Pocket ID, test database)
migrations/             0001_sessions.sql, …
.sqlx/                  sqlx offline query data (committed)
web/                    SvelteKit (adapter-static), Tailwind, shadcn-svelte, Playwright
Dockerfile              targets runtime, migration
Justfile, flake.nix     commands, Nix shell
compose.yaml            local PostgreSQL
.github/workflows/      ci.yml: checks, then images
```

### Settings (`config.rs`)
Read once at start; a missing or wrong value stops the start with an error that names it.

| Setting | Required | Default | Rule |
|---|---|---|---|
| `DATABASE_URL` | yes | – | |
| `OIDC_ISSUER_URL`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET` | yes | – | issuer is a URL |
| `SESSION_SECRET` | yes | – | at least 32 characters |
| `ORIGIN` | yes | – | URL; callback = `ORIGIN` + `/auth/callback` |
| `SESSION_TTL_SECONDS` | no | 28800 | positive integer |
| `TEAM_ACCESS_ROLE` | no | `team_coordinator` | |
| `PORT`, `HOST` | no | 3000, `0.0.0.0` | |
| `WEB_DIR` | no | `/app/web` | built front end |
| `DEV_AUTH_BYPASS` | no | false | only in debug builds (`cfg(debug_assertions)`); a release build ignores it and logs a warning when it is set |

`team migrate` needs only `DATABASE_URL`. `team openapi` needs no settings.

### Core
```rust
pub struct Identity { pub subject: String, pub name: String, pub groups: Vec<String> }
pub fn may_access(identity: &Identity, role: &str) -> bool   // exact, case-sensitive match
```

### Database
`migrations/0001_sessions.sql`:
```sql
CREATE TABLE sessions (
    token_hash  bytea PRIMARY KEY,      -- SHA-256 of the cookie value
    subject     text NOT NULL,          -- Pocket ID user id
    name        text NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT now(),
    expires_at  timestamptz NOT NULL
);
CREATE INDEX sessions_expires_at ON sessions (expires_at);
```
- `team migrate`: `sqlx::migrate!("../../migrations")` (compiled in), run under sqlx's lock, each file in a transaction, checksums checked. Exit 0 when nothing to do.
- The app never migrates. Readiness: `SELECT 1` works and every migration in the binary is in `_sqlx_migrations` with `success = true`; else 503 and the reason in the log.
- Queries use `sqlx::query!` macros; `.sqlx/` is committed; `cargo sqlx prepare --check` is a check.

### Login (decision 0007)
1. `GET /auth/login`: make `state`, `nonce` and a PKCE verifier; put them in the cookie `team_login` (signed with HMAC-SHA256 using `SESSION_SECRET`; `HttpOnly`, `Secure`, `SameSite=Lax`, `Path=/auth`, Max-Age 600, with the creation time inside); redirect (303) to the issuer's authorization endpoint with `scope=openid profile groups`, `code_challenge_method=S256`.
2. `GET /auth/callback`: check the `team_login` signature and age (≤ 10 minutes) and `state`; exchange the code (with the PKCE verifier and the client secret); verify the ID token (issuer, audience, nonce, signature, expiry); read `sub`, `name` (fall back to `preferred_username`, then `sub`) and `groups`; delete `team_login`.
   - `may_access` false → 403 page "No access", no session.
   - true → delete expired sessions; insert a session (`expires_at = now + SESSION_TTL_SECONDS`); set `team_session` (32 random bytes, base64url; `HttpOnly`, `Secure`, `SameSite=Lax`, `Path=/`, Max-Age = TTL); redirect (303) to `/`.
   - Any check fails → 400 page "Login failed, try again" with a link to `/auth/login`.
   - Pocket ID cannot be reached → 502 page "Login is not available now".
3. `POST /auth/logout`: delete the session row, clear `team_session`, redirect (303) to `/`.
- Discovery (the issuer's description and keys) is fetched at the first login and kept. Start-up and health never depend on Pocket ID.
- Pocket ID, the time and randomness come through the app state (`Clock`, settings), so tests can set them.

### Requests
- `CurrentUser` extractor: reads `team_session`, hashes it, loads a row with `expires_at > now`; else 401 `{"error":"unauthenticated"}`.
- The `/api` router requires `CurrentUser` as a layer; only `/api/health/liveness` and `/api/health/readiness` are outside it. Any other `/api` path without login is 401, also when it does not exist.
- Origin rule: every `POST`, `PUT`, `PATCH` or `DELETE` under `/api` and `/auth` needs `Origin` equal to `ORIGIN`; else 403.
- Security headers on every response: `Content-Security-Policy: default-src 'self'; frame-ancestors 'none'`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: same-origin`.
- `GET /api/me` → `{"name": "…"}`.
- Front end: files from `WEB_DIR` with the right content type; any other non-`/api`, non-`/auth` path gets `index.html`.
- Errors: `AppError` (thiserror) → 401, 403, 404, 500 `{"error":"internal"}`; details only in the log. Logs are JSON on stdout (`tracing`) and never contain cookie values, codes, tokens or secrets.
- SIGTERM: stop taking requests, finish open ones, close the pool.

### Login bypass (development)
In debug builds only, `DEV_AUTH_BYPASS=true` makes `CurrentUser` a fixed user "Developer" with the role. The code does not exist in release builds.

### Front end (`web/`)
- SvelteKit with `adapter-static` (fallback `index.html`), Svelte 5, strict TypeScript, Tailwind, shadcn-svelte. English text.
- Theme in one file: colours, fonts, light and dark mode (follows the phone setting). Phone first; works on a computer.
- Pages:
  - `/`: not logged in → "Log in" button to `/auth/login`. Logged in → the name and "Log out" (a form `POST /auth/logout`).
  - `/sample` (after login): about eight fixed example games in the front-end code, no API: dates as group headings; the quick filter (All, Games of the team, Possible duty games); games of the team marked; one "time not known yet"; one crossed out with "no longer in the game list"; a line "2 changes not seen". Story 3 replaces it.
- API calls with `openapi-fetch`; types in `web/src/lib/api/schema.d.ts`, generated by `just api-types` (`team openapi` → `web/openapi.json` → openapi-typescript).
- Icons and `manifest.webmanifest` from the old `static/` move to `web/static/`.

### Image (`Dockerfile`)
1. Node stage: build `web/`.
2. Rust stage: `cargo build --release --locked` (with dependency caching), `SQLX_OFFLINE=true`.
3. `runtime`: `debian:bookworm-slim` (CA certificates only), binary at `/app/team`, front end at `/app/web`, `USER 1000:1000`, `EXPOSE 3000`, `CMD ["/app/team", "serve"]`. Runs with a read-only file system.
4. `migration`: same as `runtime`, `CMD ["/app/team", "migrate"]`.

### Publishing (decision 0008)
`.github/workflows/ci.yml`:
- On every push and pull request: install Nix, run `nix develop -c just check`, run the release build test, build both targets, run the smoke test.
- On `main`: also publish `latest` and `latest-migration`.
- On a tag `v*`: also publish `<version>` and `<version>-migration`.

### Development
- Nix shell: Rust (cargo, rustc, clippy, rustfmt, rust-analyzer), sqlx-cli, Node 22 (front end only), PostgreSQL client, just, jq, Chromium (Linux, for Playwright).
- Justfile: `db-up`, `db-down`, `migrate`, `serve`, `dev` (server with bypass + front-end dev server), `api-types`, `check`, `test`, `fmt`.
- `.env.example` with all settings for local use.

## Steps

### Step 0: Coordinator checklist (before step 1 is merged)
- **Goal**: the first new image never touches the old database; Pocket ID sends the role.
- **Changes** (by the coordinator, on the cluster and in Pocket ID; Claude writes the checklist in `README.md`):
  1. Back up the database `team` (`pg_dump` from the `team-postgresql` pod to a file outside the cluster).
  2. Create a new database `team_v2` owned by role `team` on the same server.
  3. Set `DATABASE_URL` in Vault `kv/productivity/team` to the new database; sync through Argo CD.
  4. In Pocket ID: client `team-coordinator` may use the `groups` scope; optionally limit the client to the group `team_coordinator`; the coordinator is in the group.
- **Tests**: by hand — `psql` connects to `team_v2`; the backup file restores into a scratch database.

### Step 1: Clean slate, skeleton, database, image
- **Goal**: the new Rust app builds, migrates, reports health and is published as an image; the old app is gone.
- **Changes**: remove the old app (see Requirements 1); add the workspace, `team_core::may_access`, `config.rs`, `db.rs`, `app.rs`, health routes, the closed `/api` router (with a temporary `CurrentUser` that always gives 401 until step 2), security headers, `team serve`, `team migrate`, `0001_sessions.sql`, `.sqlx/`; the Dockerfile; `ci.yml` replacing `container.yml`; Nix shell, Justfile, `.env.example`; rewrite `README.md` and `CLAUDE.md` for the new app.
- **Tests**: core access tests; settings tests; liveness 200; readiness 200 when migrated and 503 without migrations; `team migrate` twice; `GET /api/does-not-exist` without login → 401; security headers present; smoke test of both images (`--read-only`, user 1000).

### Step 2: Login and sessions
- **Goal**: only users with the role log in; the API knows who calls it. Proven on the cluster.
- **Changes**: `auth/` (login cookie, Pocket ID, callback, sessions, logout, `CurrentUser`, origin rule), `GET /api/me`, login bypass, `team openapi`, a plain server page at `/` ("Logged in as …" with a logout form, or "Log in"). After merge, the coordinator restarts the pod and logs in on the phone.
- **Tests**: all login, session, origin, log and bypass tests (Test plan); `cargo test --release --test release_build`.

### Step 3: Front end and look
- **Goal**: the coordinator sees and approves the look on a realistic screen.
- **Changes**: `web/` as designed; the server serves it instead of the plain page; `just api-types` and its check; Playwright, axe, screenshots; feedback rounds with the coordinator using `just dev` until the look is approved.
- **Tests**: front-end tests (Test plan).

### Step 4: Final check on the cluster
- **Goal**: the base runs in its final look; a first fixed version exists.
- **Changes**: after merge, restart the pod; the coordinator checks login and the look in light and dark mode on the phone; tag `v0.1.0`. When the coordinator decides: delete the old database `team`.
- **Tests**: by hand — login, "No access" for a user without the role (if one exists), readiness green, look approved.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| Access by Pocket ID role `team_coordinator` (groups claim) instead of an allowed users list | The coordinator manages users in Pocket ID; no extra setting; adding users later is adding to the group | 0007 (replaces 0003) |
| Own `sessions` table, cookie holds a random token, database holds its hash | Fewer parts than tower-sessions, one migration path, a database copy gives no usable session | 0007 |
| Login values in a signed short-lived cookie, not a table | No table for login attempts; `SESSION_SECRET` already exists | 0007 |
| `main` publishes `latest`; version tags publish fixed images | Quick rollouts during active development; fixed versions available | 0008 |
| `Origin` check on all changing requests | Forged requests from other sites are blocked for every later story | no (part of 0007) |
| Database steps before the first merge | The new migration must never run against the old database | no (0006 covers it) |
| Sample page with fixed example games, removed in story 3 | The coordinator judges the look on a realistic screen | no |
| Screens in English | As the old app; messages choose their own language later | no |

## Not in scope
- Changes in `gitops-lab` (its team README still says the migrator has no lock; that is now outdated but harmless — follow-up).
- Pinning the cluster to a fixed image digest (planned by the cluster README for later).
- User management in the app.
- Any game, club or team data (stories 2–5).

## Test plan
- **Existing tests that must still pass**: none; the old Vitest tests are removed.
- **Rust unit tests**:
  - `may_access`: true with `team_coordinator`; false with no groups, other groups, `Team_Coordinator`.
  - settings: missing `DATABASE_URL` names it; 31-character `SESSION_SECRET` refused; `SESSION_TTL_SECONDS=abc` refused; defaults 3000, `0.0.0.0`, 28800, `team_coordinator`.
- **Rust integration tests** (router in memory via `app(state)`, `#[sqlx::test]`, fake Pocket ID with wiremock and an RSA key made in the test):
  - liveness 200 without cookie; readiness 200 migrated, 503 with `migrations = false`; `team migrate` twice → second applies nothing.
  - `/api/does-not-exist`: 401 without login, 404 with login.
  - `/auth/login` → 303 to the fake issuer with `scope=openid profile groups`, `code_challenge_method=S256`, `state`, `nonce`; `team_login` with `HttpOnly`, `Secure`, `SameSite=Lax`, `Path=/auth`, Max-Age 600.
  - good callback with the role → 303 to `/`; `team_session` with `HttpOnly`, `Secure`, `SameSite=Lax`, `Path=/`; `/api/me` gives the name.
  - no role → 403, no session row. Wrong `state` → 400. Changed `team_login` → 400. `team_login` older than 10 minutes → 400. ID token with wrong nonce, wrong audience, wrong issuer, or another signing key → 400.
  - Pocket ID unreachable at login → 502; liveness and readiness still 200.
  - stored `token_hash` is the SHA-256 of the cookie value and differs from it; expired session → 401; logout deletes the row and clears the cookie, old cookie → 401; a new login deletes expired rows.
  - `POST /auth/logout` with another `Origin` → 403, without `Origin` → 403, with `ORIGIN` → works.
  - security headers on `/` and `/api/me`.
  - captured log of a full login contains no cookie value, code, token or client secret.
  - debug build: `DEV_AUTH_BYPASS=true` → `/api/me` gives "Developer".
  - release build (`cargo test --release --test release_build`): `DEV_AUTH_BYPASS=true` → `/api/me` 401.
  - `/` and `/sample` give `index.html`; a built `.js` file gets `text/javascript`; `/api/...` never gets `index.html`.
- **Front end** (Playwright, 390 × 844):
  - without login: "Log in" goes to `/auth/login`.
  - with the bypass user: name and "Log out"; logout ends in the "Log in" state.
  - `/sample`: dates as headings; "Games of the team" hides other club teams' games; "Possible duty games" shows only those; crossed-out game has a line through it and "no longer in the game list"; "time not known yet" shown; "2 changes not seen" shown.
  - at 360 pixels wide, no horizontal scrolling.
  - axe finds no serious problems on `/` and `/sample`.
  - screenshots in light and dark mode saved for the coordinator (not compared).
- **Checks** (`just check`): `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo sqlx prepare --check`, API types generated again with no git difference, `svelte-check`, ESLint, Prettier, Playwright.
- **GitHub**: `just check` in the Nix shell, the release build test, both images built, smoke test (`migration` against a PostgreSQL service exits 0; `runtime` with `--read-only` and user 1000 gives 200 on liveness and readiness).
- **By hand** (steps 0, 2, 4): database steps; Pocket ID client; login on the phone; "No access" for a user without the role; readiness on the cluster; look in light and dark mode.

## Compatibility and performance
- The old app and its data are not migrated. The old database stays until the coordinator deletes it.
- The cluster contract does not change; no change in `gitops-lab`.
- Old sessions are not valid after the switch; the coordinator logs in again.
- Small load: one user. Memory well below 256 MB.

## Progress
- [ ] Step 0: coordinator checklist
- [x] Step 1: clean slate, skeleton, database, image (branch `feat/the-base`; merge after step 0)
- [x] Step 2: login and sessions — code and tests done; login on the cluster still to check after the merge
- [ ] Step 3: front end and look (look approved)
- [ ] Step 4: final check on the cluster, `v0.1.0`
- [ ] All checks pass

## Deviations
Differences from the epic's technical spec (`../2026-10-10-games-from-game-list/technical.md`), all technical, none change behaviour from its functional spec:
- Access by the Pocket ID role `team_coordinator` instead of an `allowed_users` table filled by a setting (decision 0007 replaces 0003).
- Own `sessions` table instead of tower-sessions (decision 0007).
- `main` publishes `latest` for quick rollouts; version tags publish fixed images (decision 0008).
- The coordinator's database steps move before the first merge (the epic had them inside story 1).

Differences from this spec, found during Implement:
- 2026-10-10, step 1: `team migrate` is compiled with `SQLX_OFFLINE=true` in `just migrate` and in CI. Compiling against a new, empty database fails, because the compile-time query check cannot find the migrations table. Found by the first CI run.
- 2026-10-10, step 2: the request log writes the method and path only, not the query. The test "logs contain no secrets" found that the default request log wrote the login code from the callback address. This makes the design rule "no codes in logs" true; it adds no behaviour.
- 2026-10-10, step 2: the fake Pocket ID uses two fixed test-only RSA keys in `crates/server/tests/data/` instead of a key made in each test. Same test, faster, no extra crate.
- 2026-10-10, step 2: `wiremock` is pinned to 0.6.3; 0.6.5 needs Rust 1.88 (the project uses 1.86).
- 2026-10-10, step 2: the workspace has no license field. A license was added by mistake in step 1 and removed; choosing a license is the coordinator's decision. `team openapi` leaves the license out.

## Review findings
