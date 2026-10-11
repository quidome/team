# Technical spec: Fast release pipeline

**Status**: approved
**Size**: story
**Change**: docs/changes/2026-10-11-fast-release-pipeline
**Date**: 2026-10-11

## Objective
Make the GitHub pipeline fast from the start, with separate stages for checking, building,
publishing and releasing, where a release only renames the image that `main` already tested.

## Requirements summary
Type `technical`: no change for the user. The coordinator decided (2026-10-11): no Nix in
GitHub; images on pull requests only when the Dockerfile or dependencies change; a tag only
renames the tested `main` image; do this before the build review of the base.

Expected result:
1. Triggers: pull requests, pushes to `main`, tags `v*`. No runs for pushes to other branches.
2. **Check** (pull requests and `main`), one job without Nix: fmt, clippy, Rust tests, `cargo sqlx prepare --check`, API types up to date, svelte-check, lint, front-end build, browser tests, actionlint, shellcheck. Migrations with sqlx-cli. No separate release build. Target about 3 minutes with warm caches.
3. **Build** (`main`; pull requests that change `Dockerfile`, `.dockerignore`, `Cargo.lock` or `web/package-lock.json`), at the same time as Check: one image build with cargo-chef, smoke test on the real images (including: `DEV_AUTH_BYPASS=true` gives 401 on `/api/me`). On `main` it pushes only `sha-<7>` and `sha-<7>-migration`. Target 1–2 minutes with warm caches.
4. **Publish** (`main`, after Check and Build passed): `latest` and `latest-migration` are renamed to the `sha-<7>` images. Target under 30 seconds.
5. **Release** (tag `v*`): the `sha-<7>` images of the tagged commit are also named `<tag>` and `<tag>-migration`. Without those images it fails with a clear message. Target under 30 seconds.
6. Locally the Nix shell and `just check` stay; GitHub runs the same `just` recipes.
7. Decision 0009 replaces 0008; the README describes the release steps.

## Current state
- `.github/workflows/ci.yml` runs for every push to every branch, every tag and every pull request. Jobs: `check` (Nix install, Nix store cache, Rust cache, `team migrate` compiled with `SQLX_OFFLINE=true`, `npm ci`, `just check`, release build test) then `images` (build `runtime` and `migration`, smoke test inline, publish with docker/metadata-action on `main` and tags).
- Measured (2026-10-10/11): push to `main` 7–20 minutes; tag `v0.1.0` 12 minutes, of which 8 minutes saving the Nix store cache; image build 2:44 without the Rust library cache (cache mounts are not kept between GitHub runs).
- `just check` compiles the `team` package three times: `SQLX_OFFLINE` changes between `api-types`, `cargo test` and the Playwright web servers, and sqlx macros recompile when it changes.
- `crates/server/tests/release_build.rs` asserts 401 in release builds and "Developer" in debug builds.
- `Dockerfile`: stages `web` (Node), `build` (Rust with cache mounts), `runtime`, `migration`.
- Decision 0008: `main` publishes `latest`; tags publish fixed versions (by building again).
- The cluster contract (README) does not change.

## Design

### Workflows
```
.github/workflows/ci.yml        pull_request, push to main
  ├─ changes   pull requests only: image=true when Dockerfile, .dockerignore, Cargo.lock or
  │            web/package-lock.json changed (gh pr diff --name-only)
  ├─ check     ┐ at the same time
  ├─ build     ┘ on pull requests only when changes.image == 'true'
  └─ publish   needs check + build; main only: latest(-migration) → sha-<7>(-migration)

.github/workflows/release.yml   tags v*
  └─ release   sha-<7> exists? → also name it <tag> and <tag>-migration; else fail
```
- `concurrency: { group: ci-${{ github.ref }}, cancel-in-progress: true }` on `ci.yml`: a newer push to the same pull request or to `main` stops the older run, so an older run never moves `latest` after a newer one.
- `permissions: contents: read` at the top; `packages: write` only on the jobs that push (build on `main`, publish, release).
- Actions fixed to a major version: `actions/checkout@v4`, `actions-rust-lang/setup-rust-toolchain@v1` (reads `rust-toolchain.toml`; includes rust-cache), `taiki-e/install-action@v2` (just, shellcheck), `actions/setup-node@v4` (npm cache), `actions/cache@v4`, `docker/setup-buildx-action@v3`, `docker/login-action@v3`, `docker/build-push-action@v6`.

### Check job
1. Checkout; Rust toolchain with rust-cache (`save-if: main`); just and shellcheck; sqlx-cli from `actions/cache` keyed by its version, else `cargo install sqlx-cli --version <pinned> --no-default-features --features rustls,postgres --locked` (then cached; saved on `main` only); Node 22 with npm cache; `npm ci` in `web/`; Playwright Chromium (`npx playwright install --with-deps chromium`, browser folder cached by Playwright version, saved on `main` only).
2. PostgreSQL 16 service, `DATABASE_URL=postgresql://team:team@localhost:5432/team_v2`.
3. `sqlx migrate run --source migrations`.
4. `just lint-ci api-types-check check-rust check-web e2e`.
5. Upload `web/test-results/` only when a step failed.

### Build job
1. Checkout; buildx; log in to GHCR on `main` only.
2. Build `runtime` and `migration` with `load: true`, layer cache `type=gha,scope=image` (read always, write on `main` only, `mode=max`), labels `org.opencontainers.image.revision=${{ github.sha }}`, `org.opencontainers.image.source=https://github.com/${{ github.repository }}`.
3. `just smoke team:runtime team:migration` against a PostgreSQL 16 service.
4. On `main`: push `ghcr.io/<repo>:sha-<7>` and `:sha-<7>-migration` (build-push with `push: true`, same cache, so no new build).

### Publish job
`docker buildx imagetools create -t <image>:latest <image>:sha-<7>` and the same for `-migration`.

### Release job (`release.yml`)
```sh
image=ghcr.io/${GITHUB_REPOSITORY}
sha=sha-${GITHUB_SHA::7}
if ! docker buildx imagetools inspect "$image:$sha" >/dev/null 2>&1; then
  echo "::error::No image for commit ${GITHUB_SHA::7}. Wait until the run on main has passed, then tag again."
  exit 1
fi
docker buildx imagetools create -t "$image:$GITHUB_REF_NAME" "$image:$sha"
docker buildx imagetools create -t "$image:$GITHUB_REF_NAME-migration" "$image:$sha-migration"
```

### Dockerfile (cargo-chef)
```dockerfile
FROM lukemathwalker/cargo-chef:latest-rust-1.86 AS chef
WORKDIR /src
FROM chef AS plan
COPY . .
RUN cargo chef prepare --recipe-path recipe.json
FROM chef AS build
COPY --from=plan /src/recipe.json recipe.json
RUN cargo chef cook --release --locked --recipe-path recipe.json
COPY . .
RUN SQLX_OFFLINE=true cargo build --release --locked --bin team
```
The `web`, `runtime` and `migration` stages stay; `runtime` copies `/src/target/release/team`.

### Justfile
```make
check: lint-ci api-types-check check-rust check-web e2e
lint-ci:          actionlint; shellcheck scripts/*.sh
api-types-check:  api-types, then git diff --exit-code -- web/openapi.json web/src/lib/api/schema.d.ts
check-rust:       cargo fmt --check; clippy -D warnings; cargo test; cargo sqlx prepare --check
check-web:        cd web && npm run check && npm run lint && npm run build
e2e:              cd web && npx playwright test
smoke runtime="team:runtime" migration="team:migration":  scripts/smoke-test.sh {{runtime}} {{migration}}
```
`api-types` and the Playwright web servers no longer set `SQLX_OFFLINE`; they use the migrated database in `DATABASE_URL`. Only `just migrate` keeps `SQLX_OFFLINE=true` (a new, empty database).

### Smoke test (`scripts/smoke-test.sh RUNTIME MIGRATION`)
Needs `DATABASE_URL` (default `postgresql://team:team@localhost:5432/team_smoke`, created and dropped by the script when it is the default). Steps:
1. Migration image twice, `--read-only --user 1000:1000 --network host`.
2. Runtime image, read-only, user 1000, port 3000: liveness 200, readiness 200, `/` 200 with `text/html`.
3. Runtime image with `DEV_AUTH_BYPASS=true` on port 3001: `/api/me` 401.
4. Containers removed on exit (trap). Any failure: print the container log and exit 1.

### Tests in Rust
`crates/server/tests/release_build.rs` → `dev_bypass.rs`: only the debug case (bypass gives "Developer"). The release case is step 3 of the smoke test.

## Steps

### Step 1: Local parts
- **Goal**: everything that can be checked on the computer is ready before GitHub sees it.
- **Changes**: Justfile recipes; no `SQLX_OFFLINE` in `api-types` and `web/playwright.config.ts`; `scripts/smoke-test.sh`; `dev_bypass.rs`; cargo-chef Dockerfile; actionlint and shellcheck in `flake.nix`. Branch `feat/fast-pipeline`.
- **Tests**: `just check` passes and compiles `team` once (count "Compiling team" in its output); both images build; `just smoke` passes; a second build after a code-only change shows `cargo chef cook` as cached; `just smoke team:does-not-exist team:does-not-exist` fails.

### Step 2: Workflows and pull request
- **Goal**: the new pipeline runs on GitHub without publishing.
- **Changes**: new `ci.yml`, new `release.yml`; open a pull request from `feat/fast-pipeline`.
- **Tests**: runs 1–3 (Test plan).

### Step 3: Merge and release
- **Goal**: `main` publishes by renaming; a tag releases by renaming.
- **Changes**: merge the pull request; one more code-only push to `main`; tag `v0.1.1`; test tag `v0.0.0-not-on-main` on a branch commit, deleted afterwards.
- **Tests**: runs 4–7 (Test plan); a pod restart on the cluster with the new `latest`.

### Step 4: Documents
- **Goal**: the documents describe the new pipeline (decision 0009 is written with this design).
- **Changes**: README (release steps: merge, wait for green, check the cluster, tag); `CLAUDE.md` (recipes); the measured times in this file.
- **Tests**: by reading.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| Stages Check, Build, Publish, Release; release by renaming the tested image | A release is exactly what ran on `main`; tags take seconds | 0009 (replaces 0008) |
| No Nix in GitHub | The Nix store cache cost up to 8 minutes per run; versions stay aligned by `rust-toolchain.toml` and `package-lock.json` | 0009 |
| Images on pull requests only when the Dockerfile or dependencies change | Code-only pull requests do not need an image | 0009 |
| Caches saved on `main` only | GitHub keeps caches per branch; saving per pull request repeats uploads | no |
| Bypass check on the real image instead of a release build in Rust | Tests the artifact that reaches the cluster; removes one compile | no |
| Checks without `SQLX_OFFLINE` | Changing it recompiles the `team` package | no |
| Third-party actions fixed to a major version | Same as the current workflow; pinning to commits can come later | no |

## Not in scope
- Pinning actions to exact commits.
- Pinning the cluster to a digest (planned in the cluster README).
- Changes in `gitops-lab`.
- Speeding up the local `just check` beyond removing the recompiles.

## Test plan
- **Existing tests that must still pass**: all Rust tests (with `release_build.rs` renamed to `dev_bypass.rs`, debug case only) and the 10 browser tests.
- **Static**: actionlint on `.github/workflows/*.yml`; shellcheck on `scripts/*.sh`.
- **Local** (step 1): `just check` passes and compiles `team` once; both images build; `just smoke` passes; code-only rebuild has `cargo chef cook` cached; `just smoke` with a missing image fails.
- **GitHub runs** (times recorded under Progress):
  1. Pull request, code and workflow only: `changes` says no image; Build skipped; Check passes; Publish skipped; no cache saved.
  2. Second push to the pull request while run 1 runs: run 1 is cancelled.
  3. Push to the pull request that changes the Dockerfile: Build runs with the smoke test; nothing pushed.
  4. Merge into `main`: Check and Build at the same time; Publish passes; digest of `latest` = digest of `sha-<7>` (`docker buildx imagetools inspect`). Target about 4 minutes warm.
  5. Code-only push to `main`: Build under 2 minutes; `cargo chef cook` cached.
  6. Tag `v0.1.1`: Release under 30 seconds; digest of `v0.1.1` = digest of `sha-<7>`.
  7. Tag `v0.0.0-not-on-main` on a branch commit: Release fails with "No image for commit …"; nothing published; tag deleted.

## Compatibility and performance
- Image names, image contents and the cluster contract do not change.
- Old image tags made by the metadata action for branches (`main`) are no longer made; the cluster does not use them.
- Expected: push to `main` about 4 minutes instead of 7–20; tag under 30 seconds instead of 12 minutes.

## Progress
- [ ] Step 1: local parts
- [ ] Step 2: workflows and pull request (runs 1–3)
- [ ] Step 3: merge and release (runs 4–7)
- [ ] Step 4: documents
- [ ] All checks pass

## Deviations

## Review findings
