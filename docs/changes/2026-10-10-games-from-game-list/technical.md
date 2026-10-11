# Technical spec: Games from the game list

**Status**: approved
**Size**: epic
**Change**: docs/changes/2026-10-10-games-from-game-list
**Date**: 2026-10-10

## Objective
Build the base of the new product (Rust backend, Svelte front end, login, PostgreSQL) and on it
the first use: the tool reads all club games from the game list, shows the schedule, and keeps
every change visible until the coordinator has seen it.

## Requirements summary
From `functional.md` (approved after two spec reviews):

1. **Choose club and team.** Search the club by name in the source's club list; choose the team from the club's own teams. Change it later: the new choice comes in with no changes recorded, unseen changes of the old choice are removed. At a new season, keep the club and ask for the team again; until then show last season's games.
2. **Read on demand.** One action reads the game list of the chosen club for the current season. A result line counts only games in the schedule ("3 new, 1 moved, 1 no longer in the game list"). No changes in the game list means no changes recorded. The tool never changes a game. Unreachable source, errors or unknown data: games stay as they were, the failure and the last good reading are shown. Empty or far fewer games: hold the reading back, warn, offer "these games are really gone"; a later good reading removes the warning.
3. **The schedule.** All games of the team plus home games of other club teams ("possible duty games"); no away games of other club teams. Coming games first, grouped by date, past games of this season on request. Quick filter: all, games of the team, possible duty games (also on the list of changes). Team games look different. Each game: day, date, start time ("time not known yet" for 00:00), home team, away team, hall and town. No results; "0 - 0" is never a result. A game gone from the game list is crossed out until its change is seen, then hidden. Shows the last good reading and a failed reading. Phone first, one hand. Only the coordinator can see and use it.
4. **The list of changes.** Kinds: new game; moved (date, start time, place; old and new); start time became known; home and away swapped; a team changed; no longer in the game list; came back. Into or out of the schedule shows the reason ("now a home game of another club team"). One change per game: from what was last seen to what is true now. Mark one or all as seen. Many new games in one reading: one summary line that opens. The first reading after a choice records nothing. Team changes stand out.
5. **Nightly reading.** Once a night, same rules as story 2. A failure is shown at the next visit with the last good reading; several failures show "may be out of date since …". No reading before a club and team are chosen.

Facts about the source (checked 2026-10-10): `https://db.basketball.nl/db/json/wedstrijd.pl?seizoen=<season>&clb_ID=<club>` gives all games of a club for a season as JSON (same data at `www.basketballstats.nl`); `…/db/json/club.pl` gives all clubs with name and town. Club and team numbers are the source's own. Times are Dutch local time; 00:00 means unknown; unplayed games show "0 - 0". The data has no "what changed" information. Last season, team numbers did not change between the season halves. The source appears to be run by a private person on the federation's data; it has no published agreement.

## Current state
- The repository holds the old SvelteKit/TypeScript app (`src/`, `drizzle/`, Node tools). The coordinator does not use it. It is replaced from story 1 on; `features/` and `docs/` stay.
- `docs/architecture.md` is an empty template; `docs/decisions/` was empty before this design.
- Deployment lives in `gitops-lab/applications/productivity/team/` (Argo CD, Helmfile, Vault). The contract the new app keeps:
  - images `ghcr.io/quidome/team:latest` (app) and `:latest-migration` (runs as an init container before the app, on every pod start);
  - port 3000, `GET /api/health/liveness` and `/api/health/readiness`;
  - settings `DATABASE_URL`, `OIDC_ISSUER_URL`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, `SESSION_SECRET`, `ORIGIN`, `SESSION_TTL_SECONDS`, `DEV_AUTH_BYPASS`;
  - Pocket ID client `team-coordinator`, issuer `https://id.quido.me`, callback `https://team.quido.me/auth/callback`;
  - one replica, 256 MB memory, read-only root file system, non-root user 1000;
  - an own PostgreSQL server `team-postgresql` with database `team`; a NetworkPolicy lets only pods labelled `app.kubernetes.io/name: team` and `app.kubernetes.io/instance: team` connect.
- No rule blocks outgoing traffic from the team pods (checked by reading `gitops-lab`; story 3 confirms on the cluster).

## Architecture (shared by all stories)

| Part | Job | Depends on |
|---|---|---|
| `crates/core` | Pure rules: game, season, schedule selection and filter, comparing readings into changes, the "far fewer games" guard, placeholders. No network or database libraries. | nothing |
| `crates/server` – reader | Reads club list and club games from the source; strict parsing into core types; format errors name the field. Source address is a setting. Timeout, own User-Agent. | core |
| `crates/server` – storage | PostgreSQL through sqlx; migrations in `migrations/`, compiled in. | core |
| `crates/server` – web | axum: API under `/api` (login required except health), `/auth/*`, serves the built front end. Publishes OpenAPI (utoipa). | core, storage, reader |
| `crates/server` – commands | `team serve`, `team migrate`, `team read-game-list` (one binary). | all |
| `web/` | SvelteKit in static mode, Tailwind, shadcn-svelte; API types generated from OpenAPI (openapi-typescript). Built into files that the server serves. | the API |

Shared rules every story keeps:
- **Comparing is pure**: one core function takes stored games, the new reading and the selection, and returns the changes.
- **One open change per game**: a second change before "seen" updates the open one; old value = last seen, new value = now.
- **One reading at a time**: a PostgreSQL advisory lock; a second reading (on demand or nightly) waits, then compares against the new state. A tap waits for the result.
- **A reading is all or nothing**: games, changes and the reading row in one transaction.
- **Strict source parsing**: unknown or missing data is a failed reading; nothing changes.
- **Time**: source times are read as Europe/Amsterdam and stored as `timestamptz`; 00:00 becomes "no start time"; "0 - 0" is ignored.
- **Errors**: `thiserror` enums in core and server parts; `anyhow` context only in `main`/commands; no `unwrap`/`expect` outside tests.
- **Guard thresholds**: a reading is held back when it has no games, or fewer than half of the games of the last good reading for the same season.
- **Season**: a season starts on 1 August (`2026-2027` until 31 July 2027).
- **Login**: OpenID Connect against Pocket ID, done by the server (authorization code flow with PKCE); server-side sessions in PostgreSQL (tower-sessions); cookie `HttpOnly`, `Secure`, `SameSite=Lax`; the browser never holds a token. An allowed users list (Pocket ID subject) fills from a setting. `DEV_AUTH_BYPASS` works only in debug builds.
- **Migrations**: plain SQL files, forward-only, applied by `team migrate` under sqlx's lock, with checksums; the app does not migrate, and readiness fails if a known migration is missing. Breaking schema changes go in two releases (add, later remove). Queries checked at compile time with the `.sqlx/` offline data in the repository.

Stored data (outline; each story adds its part):

| Table | Holds |
|---|---|
| `selection` | chosen club and team (source numbers and names), season |
| `games` | per source game number: start date, start time (nullable), home and away team and club (number and name), hall, town, competition, still in the game list |
| `readings` | per reading: start, end, result (good, failed, held back), error text, counts |
| `changes` | per game, the open change: kind, old values, new values, seen time |
| `allowed_users`, sessions | who may log in, open sessions |

API outline (details per story): `GET /api/me`, `GET /api/clubs?search=`, `PUT /api/selection`, `GET /api/teams`, `POST /api/readings`, `GET /api/readings/latest`, `POST /api/readings/{id}/confirm-gone`, `GET /api/schedule?filter=all|team|duty&period=coming|past`, `GET /api/changes?filter=…`, `POST /api/changes/{id}/seen`, `POST /api/changes/seen`; `/auth/login`, `/auth/callback`, `/auth/logout`.

## Stories
Each story gets its own change folder (stage `design`) and its own `/design`.

| # | Story | Type | Scope | Depends on | Expected closure | Status |
|---|---|---|---|---|---|---|
| 1 | [The base](../2026-10-10-the-base/) | technical | Remove the old app (keep `features/`, `docs/`); Rust workspace (`core`, `server`); Nix shell with Rust, sqlx-cli, Node for the front end, PostgreSQL; `team serve` and `team migrate`; health checks; login with Pocket ID, sessions, allowed users; first migration; Svelte app with style kit (colours, fonts, light and dark) and one page after login; OpenAPI types; Dockerfile with `runtime` and `migration` targets to the existing contract; `just check` and GitHub checks; written steps for the coordinator: back up `team`, create the new database, point `DATABASE_URL` to it, delete `team` after story 1 works. Ends with a real deployment and a login on the phone. The coordinator approves the look. | – | review build | open |
| 2 | Choose club and team | feature | Spec story 1: club search on the club list, team choice from own teams, change later, new season. Reader for the club list; `selection` table; season rule in core. | 1 | checks | open |
| 3 | Read on demand, and the schedule | feature | Spec stories 2 and 3: reader for club games, strict parsing, storage of games and readings, comparing (counts only), guard and "really gone", lock, transaction; schedule screen with filter, past games, crossed-out games, last reading. Confirms outgoing traffic on the cluster. May split in its own design. | 2 | review build + accept | open |
| 4 | The list of changes | feature | Spec story 4: keep changes (all seven kinds, into/out of the schedule), one open change per game, seen one and all, summary line, filter. Sets the summary threshold. | 3 | review build + accept | open |
| 5 | The nightly reading | feature | Spec story 5: `team read-game-list`; failures and "may be out of date"; a ready change for `gitops-lab`: CronJob template in the team chart, same image and secrets, pods with both team labels, checked with `helm lint` and `helm template`, promoted through Argo CD. | 4 | checks | open |

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| Rust workspace with a core that has no I/O | Rules testable without mocks; the compiler enforces the boundary | 0001 |
| Svelte static app with shadcn-svelte, served by the Rust server, OpenAPI types | Coordinator wants a separate front end that looks good on phone and computer; simplest framework; one image | 0002 |
| Login through Pocket ID in the app, server-side sessions, allowed users | Login on the API itself; no tokens in the browser; more users later is a list | 0003 |
| PostgreSQL with sqlx, forward-only SQL migrations, compile-time checked queries | Existing PostgreSQL on the cluster; SQL is visible; mistakes caught at build | 0004 |
| Nightly reading as a Kubernetes CronJob, one reading at a time by a database lock | Cluster-native, visible, retried; lock covers overlap with readings on demand | 0005 |
| New database for the new app; old one deleted after it works | Every step can be undone until the last; no old tables in the new app | 0006 |
| Guard thresholds: no games, or fewer than half of the last good reading | Spec asks for a guard; coordinator accepted the default | no (design detail) |
| Season starts on 1 August | Spec needs a current season; coordinator accepted the default | no (design detail) |
| A tap during a running reading waits for the result | Simpler for the coordinator than a "try again" error | no (design detail) |
| Keep the deployment contract of the old app | The cluster change for story 1 is only the image | no |

## Not in scope
- User management screens (the allowed users list comes from a setting).
- An automatic test against the live source (a test marked ignored, run by hand).
- Changes to the Helm chart before story 5.
- Everything listed under "Not in scope" in `functional.md`.

## Test plan
- **Existing tests that must still pass**: none. The old Vitest tests are removed with the old app in story 1.
- **Test data**: the real game list of this season (71 games) and last season (118 games), plus small edited copies per case, in `crates/server/tests/data/`. Public data only (club, team, hall names and times).
- **Checks** (`just check`, also on GitHub): `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo sqlx prepare --check`; `svelte-check`, formatter, linter, browser tests (Playwright on a phone-sized screen); generated API types equal to the committed types.
- **New tests per story**:
  - Story 1: no login → 401 on `/api/*`; health open; full login with a fake OpenID Connect server → session → `/api/me`; not on the allowed list → 403; wrong `state` refused; cookie flags; release build ignores `DEV_AUTH_BYPASS`; `team migrate` on an empty database, again is a no-op, readiness fails when a migration is missing; phone-sized page after login.
  - Story 2: "drakes" finds "Blue Drakes, Woerden"; no match is an empty result; only own teams (M16 1, M16 2, M18 1, MSE 1, X12 1, X14 1); 31 July 2027 → 2026-2027, 1 August 2027 → 2027-2028; new season asks for the team; change of team removes unseen changes and records none.
  - Story 3: real data parses to 71 games; 00:00 → no start time; "0 - 0" → no result; games on 25 October 2026 and in January keep their time; unknown or missing field → format error naming the field, reading failed, nothing changed; filter on real data (team: the 10 M16 1 games; duty: home games of other club teams; no away games of other club teams); same data twice → no change; guard (empty, below half, "really gone", later good reading clears the warning); failure halfway rolls back; two readings at once record each change once.
  - Story 4: one test per kind of change; into/out of the schedule shows the reason; two changes before seen → last seen to now; first reading records nothing; summary line opens to all new games; seen one, seen all; filter on changes.
  - Story 5: no selection → nothing and exit 0; with a selection the same result as on demand; failure shows last good reading; three failures show "may be out of date since …"; CronJob change passes `helm lint`/`helm template` and has both team labels.
- **By hand**: `cargo test -- --ignored` reads the live source and checks its format.

## Compatibility and performance
- The old app and its data are not migrated. The coordinator does not use it.
- The cluster contract stays the same until story 5 adds a CronJob.
- Load is tiny: one user, a few hundred games per season, one reading a night plus taps. The source is read politely.

## Progress
- [ ] Story 1: the base
- [ ] Story 2: choose club and team
- [ ] Story 3: read on demand, and the schedule
- [ ] Story 4: the list of changes
- [ ] Story 5: the nightly reading
- [ ] Epic review accept

## Deviations
- 2026-10-11: images are published as decision 0010 says (it replaces 0008 and 0009).
From the design of story 1 ([The base](../2026-10-10-the-base/technical.md), 2026-10-10), all technical:
- Access by the Pocket ID role `team_coordinator` instead of an `allowed_users` table; own `sessions` table instead of tower-sessions (decision 0007 replaces 0003). The stored data above has no `allowed_users` table.
- `main` publishes `latest` for quick rollouts; version tags publish fixed images (decision 0008).
- The coordinator's database steps happen before the first merge of story 1.

The design reads these points more strictly than the spec, within its words:
- "Far fewer games" is fixed as: no games, or fewer than half of the games of the last good reading for the same season.
- The current season is fixed to start on 1 August.
- A reading on demand during a running reading waits and then shows its result.

## Review findings
