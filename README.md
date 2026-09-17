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

For local persistence work, copy `.env.example` to `.env`, set `DATABASE_URL` to a running PostgreSQL instance, then apply migrations:

```sh
cp .env.example .env
just db-migrate
```

Start the development server:

```sh
just dev
```
