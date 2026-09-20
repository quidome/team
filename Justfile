set dotenv-load := true

default: check

# Start the local development server.
dev:
    npm run dev -- --host 0.0.0.0

# Build the production bundle.
build:
    npm run build

# Run the production Node server after building the application.
serve: build
    npm run start

# Start the local PostgreSQL container.
db-up:
    docker compose up -d postgres

# Stop the local PostgreSQL container.
db-down:
    docker compose down

# Generate PostgreSQL migrations from the Drizzle schema.
db-generate:
    npm run db:generate

# Apply generated PostgreSQL migrations using DATABASE_URL.
db-migrate:
    npm run db:migrate

# Seed the local database with development players and games.
db-seed:
    psql "$DATABASE_URL" -v ON_ERROR_STOP=1 --file scripts/seed-dev.sql

# Write a PostgreSQL custom-format backup. Pass a destination path.
db-backup output="backups/team.sql":
    mkdir -p "$(dirname \"{{output}}\")"
    docker compose exec -T postgres pg_dump --format=custom --no-owner postgresql://team:team@localhost:5432/team > "{{output}}"

# Verify that a custom-format backup can be inspected by pg_restore.
db-backup-verify input:
    pg_restore --list "{{input}}" >/dev/null

# Restore a backup into a temporary local database and remove it afterward.
db-backup-restore-verify input:
    node scripts/verify-postgres-backup.mjs "{{input}}"

# Run all local quality checks.
check: format-check lint typecheck test

# Check formatting without changing files.
format-check:
    npm run format:check

# Format the repository.
format:
    npm run format

# Run ESLint.
lint:
    npm run lint

# Fail when an npm dependency has a high or critical advisory.
audit-high:
    npm run audit:high

# Fail when a production dependency has a moderate or higher advisory.
audit-production:
    npm run audit:production

# Run Svelte and TypeScript checks.
typecheck:
    npm run check

# Run the test suite.
test:
    npm test
