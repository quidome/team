# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project knowledge

@AGENTS.md

## Commands

The pinned Nix dev shell (Rust, sqlx-cli, Node for the front end, PostgreSQL client, just, and on Linux the mold linker) is already loaded: run the tools directly. Never use `nix develop` or `nix build` with `path:.` (see AGENTS.md, "Dev environment").

- Local database: `just db-up` / `just db-down` (PostgreSQL from `compose.yaml`); `just migrate` creates the database in `DATABASE_URL` and applies migrations
- Run the server: `just serve` (settings from `.env`, see `.env.example`)
- All checks: `just check` = `lint-ci` (actionlint, shellcheck), `api-types-check`, `check-rust` (fmt, clippy `-D warnings`, tests), `check-web` (svelte-check, lint, build), `e2e` (Playwright), `check-sqlx` (last: it forces the next compile). CI runs the same recipes.
- Images: `just images`, then `just smoke` (needs `just db-up`)
- Front end: `just web-install` once; `just dev` for the Rust server (logged in as Developer) plus the Vite dev server on port 5173
- One test: `cargo test -p team --test health readiness`
- After changing an SQL query: `just prepare`, commit `.sqlx/`
- A new migration that new queries need: `sqlx migrate run` first (queries are checked against the database at compile time), then `just prepare`

Database tests use `#[sqlx::test]` and need `DATABASE_URL` pointing at a running PostgreSQL (`just db-up`); each test gets its own database.

## Architecture

See `docs/architecture.md` and `docs/decisions/`. In short:

- `crates/core` (`team_core`): product rules only (access, season, club search, game, team choice, schedule, comparison of readings and the open changes, guard, nightly reading, players and their roles, weekly trainings, trainings and attendance, reasons). No network, database or web crates (decision 0001).
- `crates/server` (`team`): settings (`config.rs`), clock and current season (`clock.rs`), database and migrations (`db.rs`), router (`app.rs`, built by `app(state)` so tests call it directly), API (`api/`), login and sessions (`auth/`), the strict reader of the game list source (`source/`, decision 0013), the open changes (`changes.rs`, decision 0016), the timer of the nightly reading (`nightly.rs`, decision 0017), the choice for each season (`followed_teams.rs`, decision 0012), the encryption of the player names (`cipher.rs`, key `PLAYER_NAME_KEY`, decision 0021), players and roles (`players.rs`, with the row lock of a player and the encrypted names), reasons (`reasons.rs`) and weekly trainings (`weekly_trainings.rs`, decision 0018), trainings and absences (`trainings.rs`, decision 0020), the front-end files and security policy (`web.rs`), and the binary (`main.rs`: `serve` with the timer, `migrate`, `openapi`).
- `migrations/`: forward-only SQL files, compiled in (decision 0004).
- Everything under `/api` needs a session by default; only the two health routes are outside (decision 0007).
- Images: `Dockerfile` (cargo-chef) targets `runtime` and `migration`. CI stages Check, Build, Publish, Release (decision 0010): Build pushes `build-<commit>`; Publish, after Check and Build passed, names it `sha-<commit>` and `latest`; a `v*` tag names the `sha-<commit>` image again. Release steps and the cluster contract are in `README.md`.

## Rules

- No `unwrap`/`expect` outside tests (clippy denies them).
- Errors: `thiserror` enums in library code, `anyhow` only in `main.rs`.
- Logs never contain player names, secrets, cookies, codes or tokens.
