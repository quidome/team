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
| `crates/core` (`team_core`) | Product rules only, no I/O (decision 0001). Now: the access rule `may_access`; `season` (a season starts on 1 July, as the NBB counts it; "2026-2027"); `club` (club search); `game` (a game in Dutch local time, 00:00 = no start time); `team_choice` (the club's own teams with short names, the team rule `choose_team`, the state of the choice `choice_state`, whose schedule shows `schedule_target`, whether a new choice keeps the open changes `same_choice`); `schedule` (which games are in the schedule `in_schedule`, and the schedule split into coming and past games `schedule`, with gone games only while their change shows); `comparison` (the change from the game as last seen to the game now `change_between`, only for games of today or later; stored games against a reading `compare`, `count`, `Change::kind`, `no_longer_in_list`, `same_number_pairs`; the open changes after a reading `update_open_changes`, and the list to show `list_changes`, decision 0016); `guard` (hold back a reading with no or far fewer games, `guard_after_confirm`, and the first-reading rule `is_first_reading`). Uses `chrono` without a clock. | nothing |
| `crates/server` (`team`) | `config.rs` settings; `clock.rs` the clock and the current season (Dutch date, `chrono-tz`), and `ShiftedClock` for `DEV_NOW` (debug builds only); `db.rs` pool, migrations, readiness, `StoreError`, the reading lock `lock_readings`; `auth/` login, sessions, request rules; `source/` the reader of the game list source (decision 0013); `followed_teams.rs` the choice of club and team for each season (decision 0012); `games.rs` the stored games and places; `readings.rs` the record of each reading; `reading.rs` one reading of the game list (decisions 0005, 0015); `changes.rs` the open changes (decision 0016); `api/` JSON API (`me`, `selection`, `clubs`, `readings`, `schedule`, `changes`, health) and OpenAPI description; `error.rs` `AppError`; `web.rs` front-end files and security policy; `app.rs` the router (`app(state)`); `main.rs` the commands `serve`, `migrate`, `openapi` | core |
| `migrations/` | Forward-only SQL files, compiled into the binary (decision 0004): `0001_sessions`, `0002_followed_teams`, `0003_games` (places, games, readings; `followed_teams.season_year`), `0004_changes` (the open changes and the sequence `change_versions`) | – |
| `web/` | SvelteKit static app (decision 0002): theme in `src/routes/layout.css`, shadcn-svelte parts, typed API client generated from `web/openapi.json` with small helpers in `src/lib/api/client.ts`; the start page (`/`) with the choose step (`src/lib/components/ChooseTeam.svelte`), the "Game list" card (`src/lib/components/ReadGameList.svelte`, words in `src/lib/reading.ts`) and the schedule (`src/lib/components/Schedule.svelte`, filter and date words in `src/lib/schedule.ts`, the filter `src/lib/components/FilterToggle.svelte`) with the line of changes not seen, `/changes` (the list of changes, words in `src/lib/changes.ts`), and `/choose` | the API |

### Login (decision 0007)
1. `/auth/login` makes `state`, `nonce` and a PKCE verifier, keeps them in the signed cookie `team_login` (10 minutes, `Path=/auth`), and redirects to Pocket ID with the scopes `openid profile groups`.
2. `/auth/callback` checks the cookie and `state`, exchanges the code, and verifies the ID token (issuer, audience, nonce, signature, expiry). Pocket ID's description and keys are kept in memory; when a token fails only on its signature, they are fetched again once (key rotation).
3. Only an identity with the group `team_coordinator` (`TEAM_ACCESS_ROLE`) gets a session: a random token in the cookie `team_session`, its SHA-256 hash in the table `sessions`, a fixed lifetime of `SESSION_TTL_SECONDS`.
4. `/auth/logout` (post) deletes the session row and the cookie.
5. `DEV_AUTH_BYPASS` (user "Developer") is compiled only in debug builds. So is `DEV_NOW`: the clock starts at that moment and runs on (`ShiftedClock`), so tests and local work do not depend on the real date. A release build ignores both and logs a warning; `scripts/smoke-test.sh` checks this on the real image.

### The game list source (decision 0013)
- `source::Source` reads `{GAME_LIST_URL}/db/json/club.pl` (all clubs) and `{GAME_LIST_URL}/db/json/wedstrijd.pl?seizoen=<season>&clb_ID=<club>` (all games of a club in a season). Timeout 10 s, User-Agent `team-coordinator/<version> (+<ORIGIN>)`.
- Parsing is strict and field by field (`source/parse.rs`): anything missing or not understood fails the reading with `SourceError::Format { place, reason }`, which names the club or game and the field. Unused fields are not read. Each game must be of the season asked for.
- Club names come from the club list only. When the choice is saved, the club number is checked in the club list before its games are read (an unknown club gives an empty game list); a reading uses the saved choice and does not check again (decision 0015).
- The choice routes read the source before they touch the database; a reading reads the saved choice first. The reader logs club, season and error kind.

### The choice of club and team (decision 0012)
- Table `followed_teams`, one row for each season. `followed_teams::for_season` and `newest_before` give the two rows that `team_core::choice_state` needs; a row of a later season is never read.
- `PUT /api/selection` reads the club list and the club's games again, takes all names from the source, applies `choose_team`, and saves the row of the current season (an upsert). Rows of other seasons are never changed. The save is one transaction under the reading lock (`db::lock_readings`); when `team_core::same_choice` says the club or team is another one (or it is the first choice of the season), it removes all open changes (decision 0016).
- After a save with a team, the front end starts a reading (`POST /api/readings`), so the schedule shows at once. `/choose` then goes to the start page with `invalidateAll`: SvelteKit may have preloaded the start page's data before the save.

### Reading the game list (decisions 0005, 0014, 0015)
- Seasons are stored as start years (`season_year`, 2026 = "2026-2027"); dates and start times of games as Dutch local `date` and `time`, `NULL` = start time not known (decision 0014). `followed_teams` still writes its text `season`; `season_year` is computed from it.
- Tables: `places` (the source's place number, hall, town; one row for all seasons, so earlier seasons show the newest hall name), `games` (key: club, season, the source's game id; the game number `nr` is stored too; `in_list` false = no longer in the game list), `readings` (one row for each reading: club, season, team, times, result `good` / `failed` / `held_back`, games read, first reading, counts as JSON, hold reason), `changes` (one row for each game with a change not seen yet: the game as last seen as JSON, `NULL` for a new game, and a version from the sequence `change_versions`; decision 0016).
- `reading::read(pool, source, clock, mode)`:
  1. `reading::target`: the club and team of the current season's choice; in a new season without a choice, the club of the newest earlier choice and no team. No choice at all: `NoChoice`.
  2. Read the source, outside the database. A failure stores a `failed` reading and changes no games.
  3. One transaction, `db::lock_readings` (`pg_advisory_xact_lock` on a fixed key): one reading at a time; the save of the choice and "seen" take the same lock.
  4. A good reading that started later is already stored: write nothing, answer with it.
  5. The guard (`Mode::Guarded`): no games, or fewer than half the games of the last good reading, stores a `held_back` reading. `Mode::Confirm { held_id }` ("really gone"): the held reading must still be the newest that did not fail, and fewer games than it showed are held back again.
  6. The stored games, and the open changes `FOR UPDATE`. First reading (`is_first_reading`): no good reading of this club and season with the same team. It counts nothing and keeps no changes.
  7. `compare` and `count` (only games of today or later, Dutch date); a gone and a new game with the same game number are logged as a warning (ids and game numbers only). With a team and not a first reading: `update_open_changes`.
  8. Write places and games (games no longer in the list get `in_list = false`), then the open changes (insert, new version, or delete), store the `good` reading, commit.
- Routes: `POST /api/readings`, `GET /api/readings/latest` (newest, last good, held back to confirm), `POST /api/readings/{id}/confirm-gone`. A failed or held-back reading is a 200 answer.

### The schedule
- `GET /api/schedule`: `team_core::schedule_target` decides whose schedule shows from the state of the choice: the chosen team of the current season; while a team is needed, the team of the newest earlier choice (`last_season: true`); else `null`. The games come from `games::stored`; `team_core::schedule` keeps the games in the schedule and splits them on the Dutch date of the server's clock: coming in time order (unknown start times last in their date), past newest date first.
- It also gives `unseen_changes`, worked out with the same function as `GET /api/changes`, and passes the ids of those changes to `team_core::schedule`, so a gone game shows only while its change shows. Last season's view shows no changes.
- The front end filters (all, team, duty) on each game's `reason`; one tap needs no request. After a good reading, the start page loads only the schedule again, so the "Game list" card keeps its result line.

### The list of changes (decision 0016)
- An open change keeps the game as the coordinator last saw it; `change_between` works out the kind from that copy and the stored game. `list_changes` leaves out what gives nothing (for example past games), gives the reason (`team` when the team plays the game now or did when last seen) and the order (date and start time of the game now, or as last seen when it is gone).
- `GET /api/changes`: the list for the chosen team of the current season, each change with its kind, differences, reason, `before`, `after` and version.
- `POST /api/changes/seen` `{ changes: [{ id, version }] }`: under the reading lock, deletes each change whose version is the same; a change that a reading changed or made again has another version and stays. Answers the number left.
- The screen `/changes` filters on `reason` as the schedule does; 5 or more new possible duty games show as one line that opens (`summaryFrom` in `changes.ts`).

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
- Errors: `thiserror` enums in library code; `AppError` turns them into an answer `{"error": "<code>"}` with no details for the client: 400 with a short code (for example `team_required`, `no_choice`, `not_held_back`), 401, 404, 502 `source_unavailable` or `source_format` (the choice routes; readings record the failure and answer 200, decision 0015), 500 `internal` (403 comes from the origin rule); `anyhow` only in `main.rs`. No `unwrap`/`expect` outside tests.
- Tests: database tests use `#[sqlx::test]` (a new database per test); router tests call `app(state)` with the fake Pocket ID and a fake game list source (both wiremock; test keys and the real source answers of 2026-10-11 in `crates/server/tests/data/`) and the settable test clock (`set`, `advance`); browser tests run Playwright on a phone-sized screen against the real server, with a fresh database `team_e2e` for each run (`web/e2e/global-setup.ts`), a fake source in Node (`web/e2e/fake-source.mjs`; the real 2026-2027 games, one with start time 00:00) and the server's clock at 12 October 2026 (`DEV_NOW`). A state the fake source cannot give is tested by replacing an API answer in the browser (`page.route`). Tests that save the choice or read the game list run in the Playwright project `choose`; the project `logged-in` (`logged-in.e2e.ts`, `changes.e2e.ts`) runs after it. Router tests of readings and changes (`tests/changes.rs`) use `tests/common/game_list.rs` (a harness with Blue Drakes M16 1 chosen and a fake game list whose answer the test changes); a test that reads the log uses `LogBuffer` from `tests/common/` and has its own test file.
- The live source is tested only by hand: `cargo test -p team --test source -- --ignored`.
- `just check` locally is the Check stage in GitHub; add a check as a `just` recipe and call it from both.
- Database changes only go forward (decision 0004): a breaking change takes two releases, so old app code can run on new migrations.
- A release is a rename: merge, wait for the green `main` run, check the cluster, then tag. Tag only commits on `main` (GitHub runs the workflow files of the tagged commit).
- `cargo sqlx prepare --check` runs last in `just check`: it forces the next compile of the `team` package.
- A new migration that new queries need: apply it locally with `sqlx migrate run` first (the queries are checked against the database at compile time, so `team migrate` does not compile yet), then `just prepare`.

## Known technical debt
- While a team is still needed, each visit to the start page reads the game list twice (club list and the club's season). A few requests a week now; look at it with the nightly reading (story 5).
- `followed_teams.season` (text) is still written for the released story 2 code; drop it in a later release (decision 0014).
- `Counts` exists three times with the same fields (core, the stored JSON, the API view); a new kind of change must be added in each.
- The browser tests cover only the normal path of reading; the held-back warning, "really gone" and the failure line are tested at the API only. `page.route` can now give these states on the screen (idea in `docs/ideas.md`).
- A first reading of a season accepts an empty game list, also for a club number the source does not know; accepted because the number comes only from a saved, checked choice (decision 0015).
- Each login with a wrong ID token signature makes one extra request to Pocket ID (to fetch its keys again). Limit it if it is ever abused.
- `npm audit` reports 5 low findings in the front-end libraries (the bits-ui chain, a cookie library); they do not apply to static files. Check again when the libraries update.
- The old database `team` of the replaced app is still on the PostgreSQL server; delete it (backlog, small).
- Old `build-…` and `sha-…` images are never removed from the registry. Fix: a cleanup job (idea in `docs/ideas.md`).
- GitHub actions from other parties are fixed to a major version, not to an exact commit. Fix together with automatic dependency updates (idea in `docs/ideas.md`).
- The cluster follows the moving name `latest`; pinning to a fixed version is planned in the `gitops-lab` README before production use.
- The cached image build after a code-only push to `main` is not yet measured on GitHub (run 5 of the pipeline change; locally 27 s). Measure at the next code change.
- The untested images `sha-a9b5906` and `sha-a9b5906-migration` still exist; a tag on commit `a9b5906` would release them. Delete them (backlog, small).
