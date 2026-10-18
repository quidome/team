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
| `crates/core` (`team_core`) | Product rules only, no I/O (decision 0001). Now: the access rule `may_access`; `season` (a season starts on 1 July, as the NBB counts it; "2026-2027"; `previous`, `first_day`, `last_day`); `club` (club search); `game` (a game in Dutch local time, 00:00 = no start time); `team_choice` (the club's own teams with short names, the team rule `choose_team`, the state of the choice `choice_state`, whose schedule shows `schedule_target`, whether a new choice keeps the open changes `same_choice`); `schedule` (which games are in the schedule `in_schedule`, and the schedule split into coming and past games `schedule`, with gone games only while their change shows); `comparison` (the change from the game as last seen to the game now `change_between`, only for games of today or later; stored games against a reading `compare`, `count`, `Change::kind`, `no_longer_in_list`, `same_number_pairs`; the open changes after a reading `update_open_changes`, and the list to show `list_changes`, decision 0016); `guard` (hold back a reading with no or far fewer games, `guard_after_confirm`, and the first-reading rule `is_first_reading`); `nightly` (the run moment 03:00 Dutch time `NIGHTLY_AT`, `last_run_moment`, whether a nightly reading is due `nightly_due`, and `out_of_date` after `OUT_OF_DATE_NIGHTS` run moments without a good reading; decision 0017); `players` (players and their roles, decision 0018: the sort order of players `Player::sort_key`, the same-name check `same_name` (decision 0021), `PlayerRole::last_day` (its end, or 30 June while it continues), `role_on`, `check_role` and `check_change` (inside the season, no overlap), `change_role`, `remove_from_team`, `undo_removal`, the team page `team` (in the team, left, last season's team, the choices of "Add player", "last day today"), `proposed_start` (1 September), `clean_name`); `training` (weekly trainings: `check_weekly_training`, `proposed_window`, `training_days`, `proposals_from` for a new season; trainings: `Training`, the list of a season `season_list` with `absent_count`, `check_training`, `day_times`, `proposed_times`; decision 0020); `attendance` (`Absence`, the players at a training `players_at`, `now_there`, `check_mark`, and the totals of a season `season_overview`: for each player the trainings that count, there, absent valid and not valid, the missed trainings, the groups of `place`, and the training days not opened from `season_list`); `reasons` (`clean_reason_name`, `ReasonId`). Uses `chrono` without a clock and without time zones. | nothing |
| `crates/server` (`team`) | `config.rs` settings; `clock.rs` the clock, the current season and Dutch local time `in_netherlands` (`chrono-tz`), and `ShiftedClock` for `DEV_NOW` (debug builds only); `db.rs` pool, migrations, readiness, `StoreError`, the reading lock `lock_readings`; `auth/` login, sessions, request rules; `source/` the reader of the game list source (decision 0013); `followed_teams.rs` the choice of club and team for each season (decision 0012); `games.rs` the stored games and places; `readings.rs` the record of each reading; `reading.rs` one reading of the game list (decisions 0015, 0017); `nightly.rs` the timer of the nightly reading (decision 0017); `changes.rs` the open changes (decision 0016); `cipher.rs` the encryption of the player names (AES-256-GCM, `NameKey` from `PLAYER_NAME_KEY`, `seal`, `open`; decision 0021); `players.rs` players and roles, with the row lock of a player `lock`, the names encrypted and decrypted with the key, and the key check `check_key`; `reasons.rs` the reasons; `weekly_trainings.rs` the weekly trainings, with their advisory lock (decision 0018); `trainings.rs` trainings and absences, and the seasons with data `seasons` (decision 0020); `api/` JSON API (`me`, `selection`, `clubs`, `readings`, `schedule`, `changes`, `players`, `team`, `reasons`, `weekly_trainings`, `trainings`, `attendance`, health; dates, times and the `?season=` of a query as text in `api/format.rs`) and OpenAPI description; `error.rs` `AppError`; `web.rs` front-end files and security policy; `app.rs` the router (`app(state)`); `main.rs` the commands `serve` (the web server and the timer), `migrate`, `openapi` | core |
| `migrations/` | Forward-only SQL files, compiled into the binary (decision 0004): `0001_sessions`, `0002_followed_teams`, `0003_games` (places, games, readings; `followed_teams.season_year`), `0004_changes` (the open changes and the sequence `change_versions`), `0005_players_and_settings` (`players`, `player_roles`, `reasons` with 6 presets, `weekly_trainings`; decision 0018), `0006_trainings` (`trainings`, `absences`; decision 0020), `0007_encrypted_player_names` (the encrypted name columns), `0008_drop_plain_player_names` (the plain name columns removed), `0009_vacuum_players` (`VACUUM FULL players`, the one migration outside a transaction; decision 0021) | – |
| `web/` | SvelteKit static app (decision 0002): theme in `src/routes/layout.css`, shadcn-svelte parts, typed API client generated from `web/openapi.json` with small helpers in `src/lib/api/client.ts`; the start page (`/`) with the choose step (`src/lib/components/ChooseTeam.svelte`), the "Game list" card (`src/lib/components/ReadGameList.svelte`, words in `src/lib/reading.ts`) and the schedule (`src/lib/components/Schedule.svelte`, filter and date words in `src/lib/schedule.ts`, the filter `src/lib/components/FilterToggle.svelte`) with the line of changes not seen, `/changes` (the list of changes, words in `src/lib/changes.ts`), `/choose`, `/team` (the team of the current season), `/players` and `/players/[id]` (all players, the player with the history), `/settings` (weekly trainings and reasons; words for players and days in `src/lib/players.ts`), `/attendance` (the chosen season from `?season=`: the training days not opened, the totals and the list) and `/training/[date]` (marking; words in `src/lib/trainings.ts`); the start page shows the card of the next training and links to `/attendance`, `/team`, `/players` and `/settings` | the API |

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

### Reading the game list (decisions 0014, 0015, 0017)
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
- Routes: `POST /api/readings`, `GET /api/readings/latest` (newest, last good, held back to confirm, `out_of_date_since`), `POST /api/readings/{id}/confirm-gone`. A failed or held-back reading is a 200 answer.

### The nightly reading (decision 0017)
- `serve` starts `nightly::spawn`: a task that runs `nightly::run`, and a second task that logs an error when the first one ends without the stop at shutdown. At shutdown `main.rs` stops the timer before the pool closes; a reading stopped halfway rolls back.
- `run` calls `nightly::tick` at start and then every 5 minutes (`tokio::time::interval`, missed ticks delayed). `tick` reads nothing without a chosen team for the current season. Else it compares the last run moment (03:00 Dutch time, `last_run_moment`) with the latest start of any reading of the club and season (`readings::newest_start`), and reads with `reading::read` (`Mode::Guarded`) when none started since. `Tried` keeps in memory the run moment this timer tried, so a reading that ends in an error and stores nothing is not tried again that night.
- The check is outside the reading lock: during a rollout at 03:00 both pods can read; the lock and "a later good reading wins" keep the data correct.
- `GET /api/readings/latest` gives `out_of_date_since` (the `finished_at` of the last good reading) when a team is chosen and `team_core::out_of_date` holds: the last good reading started before the run moment 2 nights before the last run moment. The "Game list" card shows it as "The games may be out of date since …" until a good reading on the page.
- Times are compared in Dutch local time (`clock::in_netherlands`); earlier run moments are counted on the date, so the nights the clock changes do not shift them.

### The schedule
- `GET /api/schedule`: `team_core::schedule_target` decides whose schedule shows from the state of the choice: the chosen team of the current season; while a team is needed, the team of the newest earlier choice (`last_season: true`); else `null`. The games come from `games::stored`; `team_core::schedule` keeps the games in the schedule and splits them on the Dutch date of the server's clock: coming in time order (unknown start times last in their date), past newest date first.
- It also gives `unseen_changes`, worked out with the same function as `GET /api/changes`, and passes the ids of those changes to `team_core::schedule`, so a gone game shows only while its change shows. Last season's view shows no changes.
- The front end filters (all, team, duty) on each game's `reason`; one tap needs no request. After a good reading, the start page loads only the schedule again, so the "Game list" card keeps its result line.

### The list of changes (decision 0016)
- An open change keeps the game as the coordinator last saw it; `change_between` works out the kind from that copy and the stored game. `list_changes` leaves out what gives nothing (for example past games), gives the reason (`team` when the team plays the game now or did when last seen) and the order (date and start time of the game now, or as last seen when it is gone).
- `GET /api/changes`: the list for the chosen team of the current season, each change with its kind, differences, reason, `before`, `after` and version.
- `POST /api/changes/seen` `{ changes: [{ id, version }] }`: under the reading lock, deletes each change whose version is the same; a change that a reading changed or made again has another version and stays. Answers the number left.
- The screen `/changes` filters on `reason` as the schedule does; 5 or more new possible duty games show as one line that opens (`summaryFrom` in `changes.ts`).

### Players, the team and the settings (decision 0018)
- Tables: `players` (`first_name_enc` required and `last_name_enc`: the names encrypted by the app, decision 0021; `hidden`), `player_roles` (player, `season_year`, role, `from_date`, `until_date` `NULL` while it continues), `reasons` (name unique without case and outer spaces, valid, hidden, position), `weekly_trainings` (`season_year`, weekday 1 = Monday, start and end time, window). Training days are worked out from the weekly trainings, never stored.
- Every write of a player's roles is one transaction: `players::lock` (`SELECT … FOR UPDATE` on the player's row; no row gives 404), read the player's roles, a core rule, write. So two saves of one player run one after the other, and the checks use the stored roles. "Add all" (`POST /api/players/carry`) locks the players in order of id, so two of them cannot block each other.
- `GET /api/team` loads all players, the roles of this season and the season before, and the team choice (the heading), and maps `team_core::team`. Every action of the team page answers this page.
- "Remove from the team" (`POST /api/players/{id}/team-leave` `{ from, removes }`): `team_core::remove_from_team` gives the role that ends and the roles to delete. A role to delete that is not in `removes` answers 409 `confirm_removal` with those roles (and whether a role `ends`); nothing is written. The answer carries the season's roles from before (`undo`).
- "Undo" (`POST /api/players/{id}/team-undo`) sends `undo` back; `team_core::undo_removal` checks that the stored roles are what the removal left, else 409 `changed_meanwhile`. Then the season's roles are deleted and the roles from before inserted. The page keeps `undo` only while it is open; nothing is stored.
- "Change role" (`POST /api/players/{id}/role-change`): `team_core::change_role` picks the role on the date; its end and the new role are written in one transaction.
- Weekly trainings: each save takes the advisory lock "team" "wktr" (`weekly_trainings::lock`) before it reads the others, then `check_weekly_training`. Only the current season is changed.
- Logs carry player ids only, never names (test `tests/player_logs.rs`).
- Encrypted names (decision 0021): `players::all` and `by_id` decrypt with `config.player_name_key`; `all` sorts in Rust with `Player::sort_key`. "Add player" and a rename write the encrypted names, then read all players in the same transaction for `team_core::same_name` (the warning). A name that cannot be decrypted is `StoreError::Name` (500; the log names the player id only). `team migrate` checks the key after the migrations and stops on any failure; `team serve` checks it before it listens and stops on a wrong key, but starts with a warning when the database cannot be read (readiness reports it).

### Trainings and attendance (decision 0020)
- Tables: `trainings` (`date` unique, start and end time, `cancelled_at`), `absences` (training, player, reason; primary key (training, player); `ON DELETE CASCADE` from the training, `RESTRICT` from the player). "There" is not stored; training days are worked out from the weekly trainings.
- `GET /api/trainings?season=` maps `team_core::season_list` (one row per date, "Coming" and "Past", the status of each row); `GET /api/trainings/next` is its first coming row of the current season (the card); `GET /api/trainings/{date}` gives one date with `players_at`, the reasons and `proposed_times`. All routes of a training use its date.
- "Open" (`PUT /api/trainings/{date}/open`): the training of the date, or `day_times` and `INSERT … ON CONFLICT (date) DO NOTHING`, then a read; safe to send twice. "Add training" and "Change" check with `check_training`; a clash at the same moment fails on the unique index (`StoreError::is_unique_violation` → `training_exists`). "Cancel" of a day that is not opened inserts a cancelled training (`ON CONFLICT DO UPDATE` keeps the first moment).
- Marking (`PUT`/`DELETE /api/trainings/{date}/absences/{player}`): `check_mark` (a role on the date, no hidden reason for a new mark), then an upsert on (training, player); the last tap wins; no lock. The answer is small (`Marked`: the reason and the new count).
- The page sends the taps of one player one after the other (a promise chain per player), shows the new state at once with "saving…", and puts the stored state back on a failure or after 10 s without an answer (`TAP_TIMEOUT_MS`, `AbortSignal.timeout`). An answer of a reload or an action sent before a tap does not change the marks.
- "Delete player": the `RESTRICT` foreign key refuses a player with absences (`StoreError::is_foreign_key_violation` → `player_has_absences`); `PlayerDetail.has_absences` hides the button. The role routes answer `now_there` (`team_core::now_there`).
- `GET /api/attendance?season=` maps `team_core::season_overview` (the totals of each player, "Left the team", the training days not opened) with the reason names, and `seasons`: `trainings::seasons` (the seasons of roles and weekly trainings, and of the training dates through `Season::of`), the current and the chosen season, newest first. The reads are separate queries, without a transaction (one coordinator; the next load is right again). Both `GET /api/trainings` and this route read `?season=` with `query_season`.
- `/attendance` loads the list and the overview at the same time; `afterNavigate` closes "Add training" and clears the error and the open missed lists when the season changes (SvelteKit keeps the page when only the query changes).
- `GET /api/weekly-trainings` gives `proposals` (`proposals_from`: last season's weekly trainings that ran until its end, for days without one this season).

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
- Errors: `thiserror` enums in library code; `AppError` turns them into an answer `{"error": "<code>"}` with no details for the client: 400 with a short code (for example `team_required`, `no_choice`, `not_held_back`); `AppError::Refused` adds details to the code (`role_starts_later` with `date`; 409 `confirm_removal` with `roles`, 409 `changed_meanwhile`); 401, 404, 502 `source_unavailable` or `source_format` (the choice routes; readings record the failure and answer 200, decision 0015), 500 `internal` (403 comes from the origin rule); `anyhow` only in `main.rs`. No `unwrap`/`expect` outside tests.
- Tests: database tests use `#[sqlx::test]` (a new database per test); router tests call `app(state)` with the fake Pocket ID and a fake game list source (both wiremock; test keys and the real source answers of 2026-10-11 in `crates/server/tests/data/`) and the settable test clock (`set`, `advance`); browser tests run Playwright on a phone-sized screen against the real server, with a fresh database `team_e2e` for each run (`web/e2e/global-setup.ts`), a fake source in Node (`web/e2e/fake-source.mjs`; the real 2026-2027 games, one with start time 00:00) and the server's clock at 12 October 2026 (`DEV_NOW`). A state the fake source cannot give is tested by replacing an API answer in the browser (`page.route`). Tests that save the choice or read the game list run in the Playwright project `choose`; the project `logged-in` (`logged-in.e2e.ts`, `changes.e2e.ts`) runs after it. Router tests of readings and changes (`tests/changes.rs`) use `tests/common/game_list.rs` (a harness with Blue Drakes M16 1 chosen and a fake game list whose answer the test changes); a test that reads the log uses `LogBuffer` from `tests/common/` and has its own test file.
- A save that takes a lock has a test that holds the lock on another connection and checks that the request waits (300 ms), then answers. A save guarded by a unique index or `ON CONFLICT` has a test that holds an insert of the same key in an open transaction on another connection (two requests with `tokio::join!` on one task do not run at the same time). A save of more than one write has a test that makes the second write fail with a trigger made in the test's own database (`refuse_role_writes` in `tests/team.rs`), and checks that nothing changed. Each such test must fail without its lock or transaction.
- Logs never contain player names, cookies, codes, tokens or secrets.
- Player names are read and written only through `players.rs`, with the key (decision 0021); SQL never sorts, compares or searches them.
- A migration runs outside a transaction (`-- no-transaction` as its first line) only for a command that cannot run in one, such as `VACUUM`, and only when it is safe to run twice (decision 0021).
- The live source is tested only by hand: `cargo test -p team --test source -- --ignored`.
- `just check` locally is the Check stage in GitHub; add a check as a `just` recipe and call it from both.
- Database changes only go forward (decision 0004): a breaking change takes two releases, so old app code can run on new migrations.
- A release is a rename: merge, wait for the green `main` run, check the cluster, then tag. Tag only commits on `main` (GitHub runs the workflow files of the tagged commit).
- `cargo sqlx prepare --check` runs last in `just check`: it forces the next compile of the `team` package.
- A new migration that new queries need: apply it locally with `sqlx migrate run` first (the queries are checked against the database at compile time, so `team migrate` does not compile yet), then `just prepare`.

## Known technical debt
- While a team is still needed, each visit to the start page reads the game list twice (club list and the club's season). A few requests a week now.
- The nightly reading assumes one pod: with more than one replica each pod reads each night, and `Tried` is kept per pod (decision 0017). Check before raising `replicaCount`.
- `followed_teams.season` (text) is still written for the released story 2 code; drop it in a later release (decision 0014).
- `Counts` exists three times with the same fields (core, the stored JSON, the API view); a new kind of change must be added in each.
- The browser tests cover only the normal path of reading; the held-back warning, "really gone" and the failure line are tested at the API only. `page.route` can now give these states on the screen (idea in `docs/ideas.md`).
- A first reading of a season accepts an empty game list, also for a club number the source does not know; accepted because the number comes only from a saved, checked choice (decision 0015).
- Each login with a wrong ID token signature makes one extra request to Pocket ID (to fetch its keys again). Limit it if it is ever abused.
- `npm audit` reports 5 low findings in the front-end libraries (the bits-ui chain, a cookie library); they do not apply to static files. Check again when the libraries update.
- Old `build-…` and `sha-…` images are never removed from the registry. Fix: a cleanup job (idea in `docs/ideas.md`).
- GitHub actions from other parties are fixed to a major version, not to an exact commit. Fix together with automatic dependency updates (idea in `docs/ideas.md`).
- The cluster follows the moving name `latest`; pinning to a fixed version is planned in the `gitops-lab` README before production use.
- The cached image build after a code-only push to `main` is not yet measured on GitHub (run 5 of the pipeline change; locally 27 s). Measure at the next code change.
- Limits of the encrypted player names (decision 0021): a copy of the database shows how long each name is, and an encrypted name copied to another player's row still decrypts; old plain names can stay in WAL segments until new writes fill them (two on 2026-10-18) and in TrueNAS snapshots; the coordinator's own name in `sessions` is plain (coordinator's choice); no command changes the key (idea in `docs/ideas.md`).
- A backup from before v0.3.1 has plain names, which `0008` refuses: restore it with the v0.3.1 migration image once (`README.md`, "Restoring a backup from before v0.3.1"). Such backups leave the share after about a week (from 2026-10-25 none is left).
- The team database has a nightly dump at 01:30, encrypted with age in the cluster (the private key only outside the platform), on a TrueNAS share with retention, with recovery steps and one tested restore. It is built and owned by `gitops-lab` (the team deployment). It is on TrueNAS only, the same system as the database volume, so it is not yet a full backup; the off-site copy is on the `gitops-lab` backlog.
- The overlap of roles and of weekly trainings is checked in the core under a lock, not by the database (no `btree_gist` on the cluster; decision 0018). A write that skips the lock or the core is not stopped.
- "Undo" trusts the roles the page sends back; the core checks only that the removal of them gives the stored roles, and `check_role` (decision 0018).
- That one refusal undoes the whole "Add all" is checked by reading the code (one transaction); the only refusal a test can reach fails on the first player.
- The start page links to "Attendance · Team · Players · Settings", and "Attendance" has no links to the other pages: the coordinator's choice (2026-10-17) until the start page becomes one page of past and coming events (`docs/ideas.md`).
- The browser tests cannot make data in an earlier season (the screens add trainings and weekly trainings only in the current season, and the clock is fixed); an earlier season with data, and "Left the team", are tested in the core and server tests only.
- `season_overview` has the only `#[allow(clippy::too_many_arguments)]` in the crates (8 inputs: the season's data); a second one should be a choice.
- A tap that reaches the server after its 10 seconds is saved, but shown as not saved until the page loads again (decision 0020).
- The browser tests of trainings (`web/e2e/trainings.e2e.ts`, Playwright project `trainings`) run after the player tests and use what earlier tests in the file made (one database, in order).
- The header of migration `0005` names decision 0018 as "0019" (the number planned before it was written); the applied file is not edited, and the number 0019 is not used (decision 0020).
- The untested images `sha-a9b5906` and `sha-a9b5906-migration` still exist; a tag on commit `a9b5906` would release them. Delete them (backlog, small).
