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
    SQLX_OFFLINE=true cargo run --quiet --bin team -- openapi > web/openapi.json
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

# Run every check. CI runs this too. Needs `just db-up`, `just migrate` and `just web-install`.
check: api-types
    git diff --exit-code -- web/openapi.json web/src/lib/api/schema.d.ts
    cargo fmt --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    cargo sqlx prepare --workspace --check
    cd web && npm run check && npm run lint && npm run build
    cd web && npx playwright test
