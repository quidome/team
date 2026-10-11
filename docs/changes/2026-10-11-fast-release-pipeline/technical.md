# Technical spec: Fast release pipeline

**Status**: in progress
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
- [x] Step 1: local parts (2026-10-11; first image build with cargo-chef 198 s with smoke test; rebuild after a code-only change 27 s)
- [x] Step 2: workflows and pull request #1 (2026-10-11): run 2 cancelled the older run; run 3 built and smoke-tested the images without pushing (cold caches: Check 5:03, Build 2:34)
- [x] Step 3: merge and release (2026-10-11):
  - run 4 (first try): Publish skipped, `latest` unchanged; fixed by pull request #2
  - run 1 (pull request #2, workflow only): no image built; whole run 2:00 with warm caches (Check 1:57)
  - run 4: whole run 2:45 (Check 2:13, Build 0:49 with the image cache, Publish 0:25); `latest` points to exactly `sha-30c9313`
  - run 5: not run; there was no real code change to push. Measured at the next code change (story 2 of the epic); locally a code-only rebuild took 27 s
  - run 6: release `v0.1.1` in 16 s; points to exactly `sha-30c9313`
  - run 7: a tag on a commit not on `main` failed with "No image for commit ..."; nothing published
- [x] Step 4: documents (README release steps and pipeline table, CLAUDE.md recipes)
- [ ] All checks pass

## Deviations
- 2026-10-11, step 1: `cargo sqlx prepare --check` is its own recipe `check-sqlx` and runs last in `just check`. sqlx-cli forces the next compile of the `team` package; in its old place the browser tests compiled `team` again in the same run. Now `team` compiles once per `just check` run (in `cargo test`).
- 2026-10-11, step 1: in the first cache test the cargo-chef library step ran again although its input (`recipe.json`) was the same. Two clean repeats were cached (a code-only rebuild took 27 s). The cause was not found; runs 4–5 on GitHub check the cache again.
- 2026-10-11, step 2: test run 1 (a pull request without image changes) moves to step 4: this pull request changes the Dockerfile, so its runs build the image (run 3). The documents of step 4 go through their own pull request.
- 2026-10-11, step 2: the Build job pushes the images it loaded and smoke-tested (`docker tag` and `docker push`) instead of a second `build-push` with `push: true`. What is pushed is exactly what was tested.
- 2026-10-11, step 2: the Rust cache uses `Swatinem/rust-cache@v2` directly (with `save-if` on `main`); the cache inside `setup-rust-toolchain` is switched off, because it cannot be limited to `main`.
- 2026-10-11, step 3: Publish did not run on the first `main` run: on a push the `changes` job is skipped, and GitHub then skips every job that needs it, also indirectly. Publish now has an explicit condition (`always() && !cancelled()`, push, Check and Build passed), which is the behaviour the spec asked for. Fixed through pull request #2.
- 2026-10-11, step 3: `latest` and `v...` have their own fingerprint (digest): `docker buildx imagetools create` puts a list around the image. The list holds exactly one entry, the digest of the tested `sha-<7>` image. The test plan expected equal digests; the check is now on the entry in the list.
- 2026-10-11, step 3: GitHub runs the workflow files of the tagged commit. The first try of run 7 tagged an old commit, which started the old workflow ("Container"); it was cancelled before it published anything. Run 7 was then done on an empty commit made from `main` on a short-lived branch. Consequence for releases: tag only commits on `main` (the README says so in step 4).
- 2026-10-11, step 3: run 5 (code-only push to `main`) is moved to the next real code change; no change was made only for the test.
- 2026-10-11, review: `scripts/smoke-test.sh` always uses its own database `team_smoke` on `localhost:5432` (role team/team), made and dropped by the script; it does not read `DATABASE_URL` as the design said. A fixed, separate database cannot clash with the developer's own data.
- 2026-10-11, review: decision 0010 replaces 0009 after finding R1: Build pushes `build-<7>`, Publish makes `sha-<7>` and `latest`.

## Review findings
Build review, 2026-10-11, fresh agent. Read: the diff `5b03610..d29d1b2` (the working tree was
clean), this change folder, decisions 0008 and 0009, `CLAUDE.md`, `README.md`. Ran: `just lint-ci`
(passes), `gh run list` / `gh run view`, `docker buildx imagetools inspect` on GHCR. Did not run
`just check` or the image build.

- **R1 – A release can name an image that did not pass Check (medium).** Build and Check run at the
  same time, and Build pushes `sha-<7>` as soon as its own smoke test passes, before Check is done
  and also when Check fails or the run is cancelled. Release only asks "does `sha-<7>` exist?", so
  "exists" is taken for "main passed". Decision 0009 ("the release is exactly what ran on `main`")
  and the error text ("Wait until the run on main has passed") promise more than the check gives.
  Seen on GitHub: run 38127911590 (`a9b5906`) pushed `sha-a9b5906` at 09:22:51, Check finished at
  09:24:02 and Publish was skipped, yet `sha-a9b5906` exists in GHCR, so a tag on it would release
  (checked with `gh run view` and `imagetools inspect`). The README step "wait for green" is the
  only guard. Ways out (the coordinator chooses; a change to 0009's mechanism): let Release check
  that the CI run for that commit passed (`gh run list --commit`/check runs), or let Publish add a
  marker that Release looks for, or push `sha-<7>` only after Check (costs time). Checked by a
  trace of `ci.yml`/`release.yml` and the run above.
  - 2026-10-11: fixed (coordinator chose separate names). Build pushes `build-<7>`; Publish, only after Check and Build passed, names it `sha-<7>` and `latest`; Release checks `sha-<7>`. Decision 0010 replaces 0009. The untested `sha-a9b5906` entries are **not deleted yet**: the `gh` login has no `delete:packages` permission, and the coordinator postponed it (backlog line). Until then, a tag on commit `a9b5906` would release that untested image.
- **R2 – Runtime and migration can be published or released out of step (medium, sibling of R1).**
  Publish moves `latest` first and `latest-migration` second; if the second command fails, the
  cluster pulls a new app with the old migration image at the next pod start. Release checks only
  `$IMAGE:$sha`, not `$sha-migration`; Build pushes the runtime image first, so a failed migration
  push leaves a `sha-<7>` that passes the Release check, and Release then makes `<tag>` and fails
  on `<tag>-migration`. Suggest: check both images before naming anything, and name the migration
  image first. Checked by a trace; not reproduced.
  - 2026-10-11: fixed. Publish and Release check both images before any name moves, and rename the migration image first (decision 0010).
- **R3 – The `changes` job fails open (medium).** When `gh pr diff` fails (API error, or a diff
  GitHub refuses as too large), `grep` sees nothing and the job writes `image=false` and succeeds;
  Build is skipped without a sign. The `needs.changes.result != 'failure'` guard in Build can then
  never help. It works today because the repository is public (`gh repo view`: PUBLIC) and the
  diffs are small. Suggest: run `gh pr diff` on its own and fail the job when it fails (or compare
  with `git diff` against the base). Checked by a trace of the step.
  - 2026-10-11: fixed. The `changes` job reads the file list in its own command, so a `gh pr diff` failure fails the job.
- **R4 – The list of image inputs misses some (low; touches decision 0009).** A pull request that
  changes only `rust-toolchain.toml` (rustup in the build stage reads it after `COPY . .`),
  `scripts/smoke-test.sh` or the Build job in `.github/workflows/ci.yml` does not build or smoke
  the image; the first test is on `main`, where a failure blocks Publish but not the merge. Search:
  the `grep -qxE` pattern in `ci.yml` against the inputs in `Dockerfile` and the Build job.
  - 2026-10-11: fixed. Added `rust-toolchain.toml`, `scripts/smoke-test.sh` and `.github/workflows/ci.yml` to the image inputs (decision 0010).
- **R5 – Open change documents still describe the old pipeline (low).** The base is still open
  (`stage: implement`, `closure: review build`) and its `technical.md` lists
  `cargo test --release --test release_build` as a test (lines 178, 225) and decision 0008 for
  publishing (lines 23, 148, 196, 256); the epic's `technical.md` line 131 cites 0008. The base's
  build review will look for a test this change removed. Suggest a log line or deviation in the
  base that points to `dev_bypass.rs`, the smoke test and 0009. Search: `git grep -E
  "0008|release_build|cargo test --release"` outside `done/`.
  - 2026-10-11: fixed. The base and the epic have a note that points to `dev_bypass.rs`, the smoke test and decision 0010.
- **R6 – Undocumented differences from this spec (low).** (a) The sqlx-cli cache uses
  `actions/cache@v4`, which also saves on pull requests; the spec says caches are saved on `main`
  only (the Rust and Playwright caches do follow it). (b) `scripts/smoke-test.sh` always uses
  `team_smoke` on `localhost:5432` and ignores `DATABASE_URL`; the spec says it reads
  `DATABASE_URL` with that default. (c) The container names `team-smoke`/`team-smoke-bypass` are
  fixed, so a run killed without its trap blocks the next local run until the containers are
  removed by hand. None is a bug for CI; (a) and (b) belong under Deviations. Checked by reading.
  - 2026-10-11: fixed. (a) sqlx-cli cache saved on `main` only (restore and save steps). (b) recorded under Deviations. (c) the smoke script removes leftover `team-smoke` containers at its start (tested with a leftover container).
- **R7 – The Rust build image no longer names its Debian release (low).** The old build stage was
  `rust:1.86-bookworm`, matching `debian:bookworm-slim` for the runtime; the new
  `lukemathwalker/cargo-chef:latest-rust-1.86` is a moving tag that does not say the distribution.
  Today it is Debian 12 (checked: `/etc/debian_version` = 12.10), so the glibc matches. If that tag
  is ever rebuilt on a newer Debian, the binary may not start on the runtime image; the smoke test
  would catch it on `main`. A tag with `bookworm` in it, if cargo-chef publishes one, keeps the
  match explicit (not checked).
  - 2026-10-11: fixed. Build stage uses `lukemathwalker/cargo-chef:latest-rust-1.86-bookworm`; the image builds and runs on Debian 12.15.


Checked and true: Release `v0.1.1` holds exactly `sha-30c9313` (digest `29f27889…`) and
`v0.1.1-migration` its migration image; `latest` and `latest-migration` hold exactly the
`sha-d29d1b2` images; run 7 (`v0.0.0-not-on-main`) failed and the old "Container" workflow run was
cancelled; Publish ran on the last `main` run after Check and Build. The smoke test's 401 check can
fail (it waits for 401 and fails on a timeout if the bypass answers 200). `README.md`, `CLAUDE.md`
and decision 0009 match the workflows, apart from R1's promise.
