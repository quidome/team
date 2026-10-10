# Team

A small tool for one youth team coordinator: games from the federation's game list, attendance,
duties, and messages to copy into the team chat. Product direction, plans and decisions are in
[`docs/`](./docs/) (start with [`docs/vision.md`](./docs/vision.md)).

The backend is Rust (`crates/`), the front end is Svelte (`web/`), the data is in PostgreSQL. See
[`docs/decisions/`](./docs/decisions/) for why.

## Development

Enter the pinned Nix development shell, start the local database, and apply the migrations:

```sh
nix develop path:.
cp .env.example .env
just db-up
just migrate
```

Then:

```sh
just serve      # run the server on http://localhost:3000
just check      # every check: format, clippy, tests, sqlx query data
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

`main` publishes `ghcr.io/quidome/team:latest` and `:latest-migration`; a tag `v*` also publishes
`<version>` and `<version>-migration` (decision 0008). The cluster runs `latest` with
`pullPolicy: Always`, so a pod restart deploys the newest image:

```sh
kubectl rollout restart deployment/team -n productivity
```

The deployment lives in `gitops-lab/applications/productivity/team/` (Argo CD, Helmfile, Vault).
Never apply it by hand. The image keeps this contract:

| Item | Value |
|---|---|
| Ports and probes | 3000; `GET /api/health/liveness`, `GET /api/health/readiness` |
| Commands | `runtime`: `team serve`; `migration` (init container): `team migrate` |
| Security | user and group 1000, read-only root file system |
| Settings from Vault | `DATABASE_URL`, `OIDC_ISSUER_URL`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET`, `SESSION_SECRET` |
| Other settings | `ORIGIN`, `SESSION_TTL_SECONDS`, `DEV_AUTH_BYPASS` (ignored by release builds), optional `TEAM_ACCESS_ROLE` (default `team_coordinator`), `HOST`, `PORT`, `WEB_DIR` |

### Switching from the old app (once)

Do this **before** the first image of the new app reaches `main` (decision 0006), so the new
migrations never run against the old database:

1. Back up the old database `team` to a file outside the cluster:
   ```sh
   kubectl exec -n productivity team-postgresql-0 -- \
     pg_dump --format=custom --no-owner -U team team > team-old-app.dump
   ```
   Check it: `pg_restore --list team-old-app.dump | head`.
2. Create the new database:
   ```sh
   kubectl exec -n productivity team-postgresql-0 -- \
     psql -U team -d postgres -c 'CREATE DATABASE team_v2 OWNER team'
   ```
3. In Vault `kv/productivity/team`, set `DATABASE_URL` to
   `postgresql://team:<password>@team-postgresql.productivity.svc.cluster.local:5432/team_v2`,
   then sync through Argo CD.
4. In Pocket ID, client `team-coordinator`: allow the `groups` scope; make sure you are in the
   group `team_coordinator`; optionally limit the client to that group.
5. After the new app works: delete the old database
   (`psql -U team -d postgres -c 'DROP DATABASE team'`).
