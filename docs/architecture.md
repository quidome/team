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
| `crates/core` (`team_core`) | Product rules only, no I/O (decision 0001). Now: the access rule `may_access`. | nothing |
| `crates/server` (`team`) | `config.rs` settings; `db.rs` pool, migrations, readiness; `auth/` login, sessions, request rules; `api/` JSON API and OpenAPI description; `web.rs` front-end files and security policy; `app.rs` the router (`app(state)`); `main.rs` the commands `serve`, `migrate`, `openapi` | core |
| `migrations/` | Forward-only SQL files, compiled into the binary (decision 0004) | – |
| `web/` | SvelteKit static app (decision 0002): theme in `src/routes/layout.css`, shadcn-svelte parts, typed API client generated from `web/openapi.json` | the API |

### Login (decision 0007)
1. `/auth/login` makes `state`, `nonce` and a PKCE verifier, keeps them in the signed cookie `team_login` (10 minutes, `Path=/auth`), and redirects to Pocket ID with the scopes `openid profile groups`.
2. `/auth/callback` checks the cookie and `state`, exchanges the code, and verifies the ID token (issuer, audience, nonce, signature, expiry). Pocket ID's description and keys are kept in memory; when a token fails only on its signature, they are fetched again once (key rotation).
3. Only an identity with the group `team_coordinator` (`TEAM_ACCESS_ROLE`) gets a session: a random token in the cookie `team_session`, its SHA-256 hash in the table `sessions`, a fixed lifetime of `SESSION_TTL_SECONDS`.
4. `/auth/logout` (post) deletes the session row and the cookie.
5. `DEV_AUTH_BYPASS` (user "Developer") is compiled only in debug builds.

### Request rules
- Everything under `/api` needs a session, except `/api/health/liveness` and `/api/health/readiness`. Unknown `/api` paths give 401 without a session and 404 with one; they never get the front end.
- One fallback decides every other path: unknown `/auth` paths give 404; built files (under `/_app/`, or a name with an extension) give the file or 404; any other path gives the file or `index.html`.
- Every changing request (`POST`, `PUT`, `PATCH`, `DELETE`) to `/api` or `/auth` needs `Origin` equal to `ORIGIN`.
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
- Errors: `thiserror` enums in library code; `AppError` turns them into 401, 403, 404 or 500 with no details for the client; `anyhow` only in `main.rs`. No `unwrap`/`expect` outside tests.
- Tests: database tests use `#[sqlx::test]` (a new database per test); router tests call `app(state)` with the fake Pocket ID (wiremock, test keys in `crates/server/tests/data/`) and the settable test clock; browser tests run Playwright on a phone-sized screen against the real server.
- `just check` locally is the Check stage in GitHub; add a check as a `just` recipe and call it from both.
- Database changes only go forward (decision 0004): a breaking change takes two releases, so old app code can run on new migrations.
- A release is a rename: merge, wait for the green `main` run, check the cluster, then tag. Tag only commits on `main` (GitHub runs the workflow files of the tagged commit).
- `cargo sqlx prepare --check` runs last in `just check`: it forces the next compile of the `team` package.

## Known technical debt
- The sample page `/sample` with example games is temporary; story 3 of the epic replaces it with the real schedule.
- Each login with a wrong ID token signature makes one extra request to Pocket ID (to fetch its keys again). Limit it if it is ever abused.
- `npm audit` reports 5 low findings in the front-end libraries (the bits-ui chain, a cookie library); they do not apply to static files. Check again when the libraries update.
- The old database `team` of the replaced app is still on the PostgreSQL server; delete it (backlog, small).
- Old `build-…` and `sha-…` images are never removed from the registry. Fix: a cleanup job (idea in `docs/ideas.md`).
- GitHub actions from other parties are fixed to a major version, not to an exact commit. Fix together with automatic dependency updates (idea in `docs/ideas.md`).
- The cluster follows the moving name `latest`; pinning to a fixed version is planned in the `gitops-lab` README before production use.
- The cached image build after a code-only push to `main` is not yet measured on GitHub (run 5 of the pipeline change; locally 27 s). Measure at the next code change.
- The untested images `sha-a9b5906` and `sha-a9b5906-migration` still exist; a tag on commit `a9b5906` would release them. Delete them (backlog, small).
