# Architecture: team

How the product is built **now**. Updated at each /close. The reasons for choices are in
`docs/decisions/`.

## Overview
A Rust server (`crates/`) with a Svelte front end (`web/`) and PostgreSQL, shipped as one
container image to the coordinator's Kubernetes cluster. The cluster deployment lives in the
`gitops-lab` repository (`applications/productivity/team/`) and pulls `latest` at every pod
start during active development.

## Layers and modules
| Part | Responsibility | Depends on |
|------|----------------|------------|
| `crates/core` (`team_core`) | Product rules only, no I/O (decision 0001). Now: the access rule `may_access`; `season` (a season starts on 1 August, "2026-2027"); `club` (club search); `game` (a game in Dutch local time, 00:00 = no start time); `team_choice` (the club's own teams with short names, the team rule `choose_team`, the state of the choice `choice_state`). Uses `chrono` without a clock. | nothing |
| `crates/server` (`team`) | `config.rs` settings; `clock.rs` the clock and the current season (Dutch date, `chrono-tz`); `db.rs` pool, migrations, readiness; `auth/` login, sessions, request rules; `source/` the reader of the game list source (decision 0013); `followed_teams.rs` the choice of club and team for each season (decision 0012); `api/` JSON API (`me`, `selection`, `clubs`, health) and OpenAPI description; `error.rs` `AppError`; `web.rs` front-end files and security policy; `app.rs` the router (`app(state)`); `main.rs` the commands `serve`, `migrate`, `openapi` | core |
| `migrations/` | Forward-only SQL files, compiled into the binary (decision 0004): `0001_sessions`, `0002_followed_teams` | – |
| `web/` | SvelteKit static app (decision 0002): theme in `src/routes/layout.css`, shadcn-svelte parts, typed API client generated from `web/openapi.json` with small helpers in `src/lib/api/client.ts`; the start page (`/`) with the choose step (`src/lib/components/ChooseTeam.svelte`) and `/choose` | the API |

### Login (decision 0007)
1. `/auth/login` makes `state`, `nonce` and a PKCE verifier, keeps them in the signed cookie `team_login` (10 minutes, `Path=/auth`), and redirects to Pocket ID with the scopes `openid profile groups`.
2. `/auth/callback` checks the cookie and `state`, exchanges the code, and verifies the ID token (issuer, audience, nonce, signature, expiry). Pocket ID's description and keys are kept in memory; when a token fails only on its signature, they are fetched again once (key rotation).
3. Only an identity with the group `team_coordinator` (`TEAM_ACCESS_ROLE`) gets a session: a random token in the cookie `team_session`, its SHA-256 hash in the table `sessions`, a fixed lifetime of `SESSION_TTL_SECONDS`.
4. `/auth/logout` (post) deletes the session row and the cookie.
5. `DEV_AUTH_BYPASS` (user "Developer") is compiled only in debug builds.

### The game list source (decision 0013)
- `source::Source` reads `{GAME_LIST_URL}/db/json/club.pl` (all clubs) and `{GAME_LIST_URL}/db/json/wedstrijd.pl?seizoen=<season>&clb_ID=<club>` (all games of a club in a season). Timeout 10 s, User-Agent `team-coordinator/<version> (+<ORIGIN>)`.
- Parsing is strict and field by field (`source/parse.rs`): anything missing or not understood fails the reading with `SourceError::Format { place, reason }`, which names the club or game and the field. Unused fields are not read. Each game must be of the season asked for.
- Club names come from the club list only; a club number is checked in the club list before its games are read (an unknown club gives an empty game list).
- Routes read the source before they touch the database. The reader logs club, season and error kind.

### The choice of club and team (decision 0012)
- Table `followed_teams`, one row for each season. `followed_teams::for_season` and `newest_before` give the two rows that `team_core::choice_state` needs; a row of a later season is never read.
- `PUT /api/selection` reads the club list and the club's games again, takes all names from the source, applies `choose_team`, and saves the row of the current season (an upsert). Rows of other seasons are never changed.

### Request rules
- Everything under `/api` needs a session, except `/api/health/liveness` and `/api/health/readiness`. Unknown `/api` paths give 401 without a session and 404 with one; they never get the front end.
- One fallback decides every other path: unknown `/auth` paths give 404; built files (under `/_app/`, or a name with an extension) give the file or 404; any other path gives the file or `index.html`.
- Every changing request (`POST`, `PUT`, `PATCH`, `DELETE`) to `/api` or `/auth` needs `Origin` equal to `ORIGIN`.
- Cache rule on every front-end file: files under `/_app/immutable/` (hash in the name) get `Cache-Control: public, max-age=31536000, immutable`; every other file, `index.html` first, gets `no-cache`, so a new release shows at the next page load.
- Security headers on every answer (decision 0011): a Content-Security-Policy that allows only our own files plus the hash of the front end's start script, `X-Content-Type-Options: nosniff`, `Referrer-Policy: same-origin`.
- The request log has method and path only; logs never contain cookies, codes, tokens or secrets.

## Build and release
The GitHub pipeline has four stages (decision 0010):

| Stage | Runs on | Does |
|---|---|---|
| Check | pull requests, pushes to `main` | the same `just` recipes as `just check`: workflow and script lint, API types up to date, Rust fmt/clippy/tests, front-end check/lint/build, browser tests, sqlx query data |
| Build | pushes to `main`; pull requests that change an image input | one image build (cargo-chef keeps the Rust libraries in a cached layer), `scripts/smoke-test.sh` on the real images; on `main` pushes `build-<commit>` |
| Publish | pushes to `main`, after Check and Build passed | names `build-<commit>` as `sha-<commit>` and `latest` (migration image first) |
| Release | tags `v*` | names the `sha-<commit>` images of the tagged commit as the tag; builds nothing |

Image names in `ghcr.io/quidome/team`:
- `build-<commit>`: built and smoke-tested, not necessarily checked. Never deployed or released.
- `sha-<commit>`: passed Check and Build on `main`.
- `latest`: the newest `sha-<commit>`; the cluster runs it.
- `v…`: a release; the same image as its `sha-<commit>`.

Each name has a `-migration` image next to it; the cluster runs it as an init container
(`team migrate`) before the app.

Image inputs (a pull request that changes one of them also builds the image): `Dockerfile`,
`.dockerignore`, `Cargo.lock`, `rust-toolchain.toml`, `web/package-lock.json`,
`scripts/smoke-test.sh`, `.github/workflows/ci.yml`.

Caches (Rust build, sqlx-cli, Playwright browser, image layers) are saved on `main` only;
pull requests read them. GitHub does not use Nix; locally the Nix shell gives the same tools.

## Patterns to follow
- New rules go in `crates/core` first; `crates/server` only moves data in and out.
- A new API route goes on the protected `/api` router in `app.rs`, so it needs a session by default; describe it with `utoipa` and run `just api-types`. The front-end API types are generated, never written by hand.
- Errors: `thiserror` enums in library code; `AppError` turns them into an answer `{"error": "<code>"}` with no details for the client: 400 with a short code (for example `team_required`), 401, 404, 502 `source_unavailable` or `source_format`, 500 `internal` (403 comes from the origin rule); `anyhow` only in `main.rs`. No `unwrap`/`expect` outside tests.
- Tests: database tests use `#[sqlx::test]` (a new database per test); router tests call `app(state)` with the fake Pocket ID and a fake game list source (both wiremock; test keys and the real source answers of 2026-10-11 in `crates/server/tests/data/`) and the settable test clock (`set`, `advance`); browser tests run Playwright on a phone-sized screen against the real server, with a fresh database `team_e2e` for each run (`web/e2e/global-setup.ts`) and a fake source in Node (`web/e2e/fake-source.mjs`). Tests that save the choice run in the Playwright project `choose`; the project `logged-in` runs after it.
- The live source is tested only by hand: `cargo test -p team --test source -- --ignored`.
- `just check` locally is the Check stage in GitHub; add a check as a `just` recipe and call it from both.
- Database changes only go forward (decision 0004): a breaking change takes two releases, so old app code can run on new migrations.
- A release is a rename: merge, wait for the green `main` run, check the cluster, then tag. Tag only commits on `main` (GitHub runs the workflow files of the tagged commit).
- `cargo sqlx prepare --check` runs last in `just check`: it forces the next compile of the `team` package.
- A new migration that new queries need: apply it locally with `sqlx migrate run` first (the queries are checked against the database at compile time, so `team migrate` does not compile yet), then `just prepare`.

## Known technical debt
- While a team is still needed, each visit to the start page reads the game list twice (club list and the club's season). A few requests a week now; look at it with the nightly reading (story 5).
- The sample page `/sample` with example games is temporary; story 3 of the epic replaces it with the real schedule.
- Each login with a wrong ID token signature makes one extra request to Pocket ID (to fetch its keys again). Limit it if it is ever abused.
- `npm audit` reports 5 low findings in the front-end libraries (the bits-ui chain, a cookie library); they do not apply to static files. Check again when the libraries update.
- The old database `team` of the replaced app is still on the PostgreSQL server; delete it (backlog, small).
- Old `build-…` and `sha-…` images are never removed from the registry. Fix: a cleanup job (idea in `docs/ideas.md`).
- GitHub actions from other parties are fixed to a major version, not to an exact commit. Fix together with automatic dependency updates (idea in `docs/ideas.md`).
- The cluster follows the moving name `latest`; pinning to a fixed version is planned in the `gitops-lab` README before production use.
- The cached image build after a code-only push to `main` is not yet measured on GitHub (run 5 of the pipeline change; locally 27 s). Measure at the next code change.
- The untested images `sha-a9b5906` and `sha-a9b5906-migration` still exist; a tag on commit `a9b5906` would release them. Delete them (backlog, small).
