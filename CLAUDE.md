# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project knowledge

This is the implementation repository for the `team` project (a team coordinator tool). Product, architecture, and planning knowledge is maintained in a separate Obsidian vault, not in this repo — see `AGENTS.md` for the canonical entry points (`projects/team/project.md`, MVP PRD/scope, technical direction, implementation plan). Read `project.md` first when product or architecture context is needed, and don't invent requirements the vault leaves open. Treat the vault as project knowledge and this repo as the implementation; don't copy content between them unnecessarily.

## Commands

Enter the pinned Nix dev shell first: `nix develop path:.`

- Install: `npm install`
- Dev server: `just dev` (or `npm run dev`)
- All quality checks (format, lint, typecheck, test): `just check`
- Format check only: `just format-check` / apply: `just format`
- Lint: `just lint` (`npm run lint`)
- Typecheck: `just typecheck` (`svelte-kit sync && svelte-check`)
- Run full test suite: `just test` (`vitest run`)
- Watch tests: `npm run test:watch`
- Single test file: `npx vitest run path/to/file.test.ts`
- Single test by name: `npx vitest run -t "test name"`
- Build: `just build`
- Production server (build + start): `just serve`

### Database (PostgreSQL via Docker Compose)

- Start/stop local Postgres: `just db-up` / `just db-down` (data persists in the `team-postgres-data` volume)
- Generate migrations from the Drizzle schema: `just db-generate`
- Apply migrations: `just db-migrate`
- Seed dev data (idempotent — 10 players, 7 games): `just db-seed`
- Backup / verify / restore-verify: `just db-backup <path>`, `just db-backup-verify <path>`, `just db-backup-restore-verify <path>`

Integration tests under `src/lib/server/persistence/*.integration.test.ts` connect to `DATABASE_URL` and automatically `describe.skip` themselves when it's unset, so `vitest run` is safe without a database. Set `DATABASE_URL` (see `.env.example`) and have `just db-up` running to exercise them.

### Auth for local UI work

Set `DEV_AUTH_BYPASS=true` in `.env` to skip OIDC in the Vite dev server only (ignored in production builds). Without `DATABASE_URL`, pages fall back to empty in-memory responses.

### Dependency notes

- `xlsx` is pinned to a tarball from the official SheetJS CDN (0.20.3) because the public npm registry still serves a vulnerable 0.18.5. Never replace it with a normal npm version range.
- `just audit-high` fails CI on new high/critical advisories; `just audit-production` checks production-only deps at moderate+.

## Architecture

SvelteKit (Svelte 5) app using a hexagonal/ports-and-adapters layering under `src/lib`. Understanding a feature requires reading across these layers:

1. **`src/lib/domain/`** — pure business logic and value objects (e.g. `game.ts`, `training-series.ts`, `normal-age-group.ts`), no framework or I/O dependencies.
2. **`src/lib/application/<feature>/`** — one directory per feature (players, teams, seasons, duties, participation, imports, messages, tasks, history, program, memberships, training, audit, team). Each typically has:
   - a `*-repository.ts` file defining the port (interface) the use case depends on,
   - one or more use-case functions (e.g. `configure-player.ts`, `record-attendance.ts`) that take the repository as a parameter and contain the orchestration logic,
   - colocated `*.test.ts` unit tests that exercise the use case against the in-memory adapter.
3. **`src/lib/adapters/`** — in-memory implementations of the repository ports, used in unit tests (and as the no-`DATABASE_URL` fallback).
4. **`src/lib/server/persistence/`** — PostgreSQL implementations of the same repository ports, built on Drizzle ORM (`schema.ts` is the Drizzle schema; migrations live in `/drizzle`). Each has a matching `*.integration.test.ts` that runs only when `DATABASE_URL` is set.
5. **`src/lib/server/composition-root.ts`** — the single place that wires ports to Postgres adapters, lazily creating and caching one repository instance per process, plus a `withCurrentImportTransaction` helper for cross-repository transactions and graceful DB shutdown on SIGINT/SIGTERM.
6. **`src/lib/server/load-*.ts`** — page-data assembly functions (`load-team.ts`, `load-program.ts`, `load-admin.ts`, `load-duties.ts`, `load-history.ts`, `load-messages.ts`, `load-home.ts`) that call use cases via the composition root and shape data for routes. These are what `+page.server.ts` files call into.
7. **`src/routes/`** — SvelteKit routes. `+page.server.ts` loaders delegate to `src/lib/server/load-*.ts`; `src/routes/api/**/+server.ts` are the JSON API endpoints, generally backed by the same application-layer use cases.

New functionality generally means: add/extend domain logic → define or extend a repository port in `application/` → implement the use case against the port → add both an in-memory adapter (for tests) and a Postgres adapter (for production) → wire the new repository into `composition-root.ts` → expose it through a `load-*.ts` function and/or an API route.

### Behavior scenarios (`features/`)

User-visible behavior is documented as Gherkin-style `.feature` files under `features/<area>/`. Each scenario should have automated test(s) with matching names runnable via `npm test`. Write the scenario before implementing it. Keep scenarios independent of SvelteKit/database/identity-provider specifics — adapter-level tests cover those integrations separately.

### Auth and sessions

`src/hooks.server.ts` is the single request gate: it reads the coordinator session cookie, verifies it via `src/lib/server/authentication/session.ts`, applies `DEV_AUTH_BYPASS` in dev, and rejects unauthenticated `/api/*` requests (except `/api/health/liveness` and `/api/health/readiness`) with a 401. OIDC (Pocket ID) config and the login/callback/logout flow live under `src/lib/server/authentication/` and `src/routes/auth/`.

### Testing layout

- Unit tests: colocated `*.test.ts` next to the code they test, run against in-memory adapters — no external services needed.
- Integration tests: `*.integration.test.ts` in `src/lib/server/persistence/`, self-skipping without `DATABASE_URL`.
- Route/API tests: `src/tests/routes/**`.
- `vitest.config.ts` includes `src/**/*.test.ts`.
