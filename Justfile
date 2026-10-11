set dotenv-load := true

default: check

# Start the local PostgreSQL container.
db-up:
    docker compose up -d --wait postgres

# Stop the local PostgreSQL container.
db-down:
    docker compose down

# Create the database in DATABASE_URL if needed and apply the migrations.
migrate:
    sqlx database create
    SQLX_OFFLINE=true cargo run --quiet --bin team -- migrate

# Run the server (debug build) with the settings from .env and the built front end.
serve: web-build
    WEB_DIR=web/build cargo run --bin team -- serve

# Install the front-end packages.
web-install:
    cd web && npm ci

# Build the front end into web/build.
web-build:
    cd web && npm run build

# Develop the front end: the Rust server (logged in as Developer) plus the Vite dev server with
# live reload on http://localhost:5173. Needs `just db-up` and `just migrate`.
dev:
    #!/usr/bin/env bash
    set -euo pipefail
    DEV_AUTH_BYPASS=true ORIGIN=http://localhost:5173 WEB_DIR=web/build cargo run --bin team -- serve &
    trap 'kill %1' EXIT
    cd web && npm run dev -- --host

# Generate the API description and the front-end API types from the Rust code.
api-types:
    cargo run --quiet --bin team -- openapi > web/openapi.json
    cd web && npm run -s api-types

# Save the sqlx query data in .sqlx/ (run after changing a query; needs a migrated database).
prepare:
    cargo sqlx prepare --workspace

# Format all code.
fmt:
    cargo fmt

# Run the tests (database tests need DATABASE_URL and `just db-up`).
test:
    cargo test --workspace

# Run every check. GitHub runs the same recipes (decision 0010). Needs `just db-up`,
# `just migrate` and `just web-install`.
check: lint-ci api-types-check check-rust check-web e2e check-sqlx

# Check the GitHub workflows and the shell scripts.
lint-ci:
    actionlint
    shellcheck scripts/*.sh

# Check that the committed API description and types match the Rust code.
api-types-check: api-types
    git diff --exit-code -- web/openapi.json web/src/lib/api/schema.d.ts

# Rust: format, lints, tests.
check-rust:
    cargo fmt --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

# The saved sqlx query data (.sqlx/) matches the queries. Runs last: sqlx-cli forces the next
# compile of the team package, so anything after it would compile again.
check-sqlx:
    cargo sqlx prepare --workspace --check

# Front end: types, lint and format, build.
check-web:
    cd web && npm run check && npm run lint && npm run build

# Browser tests against the real server (after `check-web` built the front end).
e2e:
    cd web && npx playwright test

# Build both images locally.
images:
    docker build --target runtime -t team:runtime .
    docker build --target migration -t team:migration .

# Smoke test of built images (after `just images`); needs `just db-up`.
smoke runtime="team:runtime" migration="team:migration":
    scripts/smoke-test.sh {{runtime}} {{migration}}
