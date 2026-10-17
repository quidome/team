# Team

A small tool for one youth team coordinator: games from the federation's game list, attendance,
duties, and messages to copy into the team chat. Product direction, plans and decisions are in
[`docs/`](./docs/) (start with [`docs/vision.md`](./docs/vision.md)).

The backend is Rust (`crates/`), the front end is Svelte (`web/`), the data is in PostgreSQL. See
[`docs/decisions/`](./docs/decisions/) for why.

## Development

Enter the pinned Nix development shell, start the local database, and apply the migrations:

```sh
nix develop
cp .env.example .env
just db-up
just migrate
```

Then:

```sh
just serve      # run the server on http://localhost:3000
just check      # every check (as CI): workflow and script lint, API types, Rust fmt/clippy/tests,
                # front-end check/lint/build, browser tests, sqlx query data
```

The database tests (`#[sqlx::test]`) create their own databases on the server in `DATABASE_URL`.
After changing an SQL query, run `just prepare` and commit `.sqlx/`.

## Database migrations

Migrations are plain SQL files in [`migrations/`](./migrations/), compiled into the binary
(decision 0004). `team migrate` applies the missing ones under a lock. They only go forward:

- never edit a migration that is applied anywhere; a fix is a new file;
- a breaking change goes in two releases (add the new, later remove the old);
- the app does not migrate; its readiness check fails while a migration is missing.

## Images and deployment

The pipeline (decision 0010) has four stages. `sha-<commit>` exists only for commits that passed everything:

| Stage | When | What |
|---|---|---|
| Check | every pull request and push to `main` | the same recipes as `just check` |
| Build | push to `main`, and pull requests that change `Dockerfile`, `.dockerignore`, `Cargo.lock`, `rust-toolchain.toml`, `web/package-lock.json`, `scripts/smoke-test.sh` or `.github/workflows/ci.yml` | builds both images, runs `just smoke`; on `main` pushes `build-<commit>` and `build-<commit>-migration` |
| Publish | push to `main`, after Check and Build passed | names those images `sha-<commit>` and `latest` (migration first) |
| Release | tag `v*` | names the `sha-<commit>` images of the tagged commit `<tag>` and `<tag>-migration`; nothing is built |

The cluster runs `latest` with `pullPolicy: Always`, so a pod restart deploys the newest image:

```sh
kubectl rollout restart deployment/team -n productivity
```

### Making a release

1. Merge into `main` and wait until the CI run on `main` is green.
2. Restart the pod and check the app on the cluster.
3. Tag that commit and push the tag:
   ```sh
   git tag -a v0.1.2 -m "v0.1.2: <what changed>"
   git push origin v0.1.2
   ```
   The release run takes seconds. It fails with "No image for commit …" when `main` has not
   passed for that commit.

Tag only commits on `main`. GitHub runs the workflow files of the tagged commit, so a tag on an
old commit can run an old workflow.

The deployment lives in `gitops-lab/applications/productivity/team/` (Argo CD, Helmfile, Vault).
Never apply it by hand. The image keeps this contract:

| Item | Value |
|---|---|
| Ports and probes | 3000; `GET /api/health/liveness`, `GET /api/health/readiness` |
| Commands | `runtime`: `team serve`; `migration` (init container): `team migrate` |
| Security | user and group 1000, read-only root file system |
| Settings from Vault | `DATABASE_URL`, `OIDC_ISSUER_URL`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, `SESSION_SECRET` |
| Other settings | `ORIGIN`, `SESSION_TTL_SECONDS`, `DEV_AUTH_BYPASS` and `DEV_NOW` (a start moment for the clock, for example `2026-10-12T10:00:00Z`; both ignored by release builds), optional `TEAM_ACCESS_ROLE` (default `team_coordinator`), `GAME_LIST_URL` (default `https://db.basketball.nl`; the cluster uses the default), `HOST`, `PORT`, `WEB_DIR` |

### Switching from the old app (done)

The new app uses its own, empty database (decision 0006). On the cluster this was done on
2026-10-17, when the first migration of the new app clashed with a table of the old app
(`players`): the database `team` of the old app was dropped and made again, empty, with the same
name, so `DATABASE_URL` in Vault did not change. The new app's migrations then ran from the start.

```sh
kubectl exec -n productivity team-postgresql-0 -- \
  psql -U team -d postgres -c 'DROP DATABASE team WITH (FORCE)' -c 'CREATE DATABASE team OWNER team'
```

Pocket ID, client `team-coordinator`: the `groups` scope is allowed, and the coordinator is in the
group `team_coordinator` (decision 0007).
