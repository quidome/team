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
    cargo run --quiet --bin team -- migrate

# Run the server (debug build) with the settings from .env.
serve:
    cargo run --bin team -- serve

# Save the sqlx query data in .sqlx/ (run after changing a query; needs a migrated database).
prepare:
    cargo sqlx prepare --workspace

# Format all code.
fmt:
    cargo fmt

# Run the tests (database tests need DATABASE_URL and `just db-up`).
test:
    cargo test --workspace

# Run every check. CI runs this too.
check:
    cargo fmt --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    cargo sqlx prepare --workspace --check
