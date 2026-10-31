# Team

A small tool for one youth team coordinator: games from the federation's game list, attendance,
duties, and messages to copy into the team chat. Product direction, plans and decisions are in
[`docs/`](./docs/) (start with [`docs/vision.md`](./docs/vision.md)).

The backend is Rust (`crates/`), the front end is Svelte (`web/`), the data is in PostgreSQL. See
[`docs/decisions/`](./docs/decisions/) for why.

## Development

Enter the pinned Nix development shell. With [direnv](https://direnv.net), run `direnv allow` once:
the shell and the settings in `.env` then load when you enter this directory. Without direnv, run
`nix develop`.

Make your settings file:

```sh
cp .env.example .env
head -c 32 /dev/urandom | base64   # put the output in .env as the value of PLAYER_NAME_KEY
```

Then install the front-end packages, start the local database, and apply the migrations:

```sh
just web-install
just db-up
just migrate
```

Then:

```sh
just dev        # run the server and the front end with live reload on http://localhost:5173
just serve      # run the server on http://localhost:3000, with no login
just check      # every check (as CI): workflow and script lint, API types, Rust fmt/clippy/tests,
                # front-end check/lint/build, browser tests, sqlx query data
```

The local database is `team_v2` (`.env.example`, and CI); the cluster database is `team` (see the
cluster contract below). The database tests (`#[sqlx::test]`) create their own databases on the
server in `DATABASE_URL`.
After changing an SQL query, run `just prepare` and commit `.sqlx/`.

## Benchmarks of development machines

`scripts/bench.sh` times a clean build, small rebuilds, the core tests and the front-end check, to
compare development machines. Run it in the Nix shell, on the same commit on each machine, after
`just web-install`; no database is necessary. It writes `benchmarks/<host>-<date>.txt`: commit
that file.

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
| Commands | `runtime`: `team serve`; `migration` (init container): `team migrate` (needs `DATABASE_URL` and `PLAYER_NAME_KEY`) |
| Security | user and group 1000, read-only root file system |
| Database | `team` on the server `team-postgresql` (namespace `productivity`): `DATABASE_URL` in Vault ends in `/team`. The nightly backup dumps this database. `team_v2` is only the local name; never copy the local value to Vault (from 08:09 to 11:29 UTC on 2026-10-18 the app used an empty `team_v2` that way) |
| Settings from Vault | `DATABASE_URL`, `OIDC_ISSUER_URL`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, `SESSION_SECRET`, `PLAYER_NAME_KEY` (32 random bytes in base64, the key of the encrypted player names; a copy is kept outside the platform, because without it the names in the database and in every backup are lost) |
| Other settings | `ORIGIN`, `SESSION_TTL_SECONDS`, `DEV_AUTH_BYPASS` and `DEV_NOW` (a start moment for the clock, for example `2026-10-12T10:00:00Z`; both ignored by release builds), optional `TEAM_ACCESS_ROLE` (default `team_coordinator`), `GAME_LIST_URL` (default `https://db.basketball.nl`; the cluster uses the default), `HOST`, `PORT`, `WEB_DIR` |

### Restoring a backup from before v0.3.1

From v0.3.2 the player names are only stored encrypted, and `team migrate` no longer encrypts
plain names: migration `0008` refuses a database that still has them, `team migrate` stops, and the
pod waits in Init. Backups made before v0.3.1 went live (2026-10-18 11:29 UTC, for example
`team-20261018T102433Z.dump.age`) have plain names; the nightly job keeps the newest 7 backups, so
they are gone after about a week.

To restore such a backup: follow the recovery steps in `gitops-lab` (the app scaled to 0), then,
before the app starts again, run the migration image of v0.3.1 once with the same settings. It
applies `0007` and encrypts the names with `PLAYER_NAME_KEY`; the next normal start applies `0008`
and `0009`. The labels let the pod through the database's NetworkPolicy. Not tested on the cluster.

```sh
kubectl run team-migrate-v031 -n productivity --rm -i --restart=Never \
  --labels=app.kubernetes.io/name=team,app.kubernetes.io/instance=team \
  --image=ghcr.io/quidome/team:v0.3.1-migration \
  --overrides='{"spec":{"securityContext":{"runAsUser":1000,"runAsGroup":1000,"runAsNonRoot":true},
    "containers":[{"name":"team-migrate-v031","image":"ghcr.io/quidome/team:v0.3.1-migration",
    "envFrom":[{"secretRef":{"name":"team"}}]}]}}'
```

A backup made during the update to v0.3.1 itself (about a minute) can hold a player with both an
encrypted name and a newer plain name (a rename by the old pod); this path keeps the newer one.

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
