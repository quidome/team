# Team

Source repository for the team coordinator tool. Product and architecture knowledge is maintained in the development Obsidian vault; see [`AGENTS.md`](./AGENTS.md) for the project links.

## Development

Enter the pinned Nix development shell:

```sh
nix develop path:.
```

Install JavaScript dependencies:

```sh
npm install
```

Run the quality checks:

```sh
just check
```

For local persistence work, start the bundled PostgreSQL container, copy `.env.example` to `.env`, then apply migrations:

```sh
just db-up
cp .env.example .env
just db-migrate
just db-seed
```

`just db-seed` adds ten development players and seven sample games, including future home and away fixtures. It is idempotent.

Stop the container when finished with `just db-down`. The database data is kept in the `team-postgres-data` Docker volume.

Create and verify a PostgreSQL backup with explicit destination paths:

```sh
just db-backup backups/team-$(date +%Y%m%d-%H%M%S).dump
just db-backup-verify backups/team-20260101-120000.dump
```

`db-backup-verify` checks that the custom-format archive can be read. Periodically perform a restore into an isolated database as part of deployment operations; the Docker volume is not a substitute for tested backups.

Spreadsheet imports use SheetJS `xlsx` 0.20.3 from the official SheetJS CDN. The public npm registry still serves the vulnerable 0.18.5 release, so do not replace the pinned tarball with a normal `xlsx` version range.

Run `just audit-high` to fail on newly introduced high or critical dependency advisories. Moderate development-tooling advisories are tracked separately because their suggested fixes are breaking upgrades.

For UI-only local development without OIDC, set this in `.env`:

```env
DEV_AUTH_BYPASS=true
```

This bypass is active only in the Vite development server. Without `DATABASE_URL`, pages use empty in-memory responses; configure PostgreSQL and run migrations for persisted data.

Start the development server:

```sh
just dev
```
