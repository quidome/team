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

# Run Svelte and TypeScript checks.
typecheck:
    npm run check

# Run the test suite.
test:
    npm test
