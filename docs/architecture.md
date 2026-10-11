# Architecture: team

How the product is built **now**. Updated at each /close. The reasons for choices are in
`docs/decisions/`.

## Overview
A Rust server (`crates/`) with a Svelte front end (`web/`) and PostgreSQL, shipped as one
container image to the coordinator's Kubernetes cluster. The cluster deployment lives in the
`gitops-lab` repository (`applications/productivity/team/`) and pulls `latest` at every pod
start during active development.

<!-- The layers and modules (core, server, web) and the login flow are added when the base
     (docs/changes/2026-10-10-the-base) is closed. -->

## Layers and modules
| Part | Responsibility | Depends on |
|------|----------------|------------|

## Build and release
The GitHub pipeline has four stages (decision 0010):

| Stage | Runs on | Does |
|---|---|---|
| Check | pull requests, pushes to `main` | the same `just` recipes as `just check`: workflow and script lint, API types up to date, Rust fmt/clippy/tests, front-end check/lint/build, browser tests, sqlx query data |
| Build | pushes to `main`; pull requests that change an image input | one image build (cargo-chef keeps the Rust libraries in a cached layer), `scripts/smoke-test.sh` on the real images; on `main` pushes `build-<commit>` |
| Publish | pushes to `main`, after Check and Build passed | names `build-<commit>` as `sha-<commit>` and `latest` (migration image first) |
| Release | tags `v*` | names the `sha-<commit>` images of the tagged commit as the tag; builds nothing |

Image names in `ghcr.io/quidome/team`:
- `build-<commit>`: built and smoke-tested, not necessarily checked. Never deployed or released.
- `sha-<commit>`: passed Check and Build on `main`.
- `latest`: the newest `sha-<commit>`; the cluster runs it.
- `v…`: a release; the same image as its `sha-<commit>`.

Each name has a `-migration` image next to it; the cluster runs it as an init container
(`team migrate`) before the app.

Image inputs (a pull request that changes one of them also builds the image): `Dockerfile`,
`.dockerignore`, `Cargo.lock`, `rust-toolchain.toml`, `web/package-lock.json`,
`scripts/smoke-test.sh`, `.github/workflows/ci.yml`.

Caches (Rust build, sqlx-cli, Playwright browser, image layers) are saved on `main` only;
pull requests read them. GitHub does not use Nix; locally the Nix shell gives the same tools.

## Patterns to follow
- `just check` locally is the Check stage in GitHub; add a check as a `just` recipe and call it from both.
- Database changes only go forward (decision 0004): a breaking change takes two releases, so old app code can run on new migrations.
- A release is a rename: merge, wait for the green `main` run, check the cluster, then tag. Tag only commits on `main` (GitHub runs the workflow files of the tagged commit).
- `cargo sqlx prepare --check` runs last in `just check`: it forces the next compile of the `team` package.

## Known technical debt
- Old `build-…` and `sha-…` images are never removed from the registry. Fix: a cleanup job (idea in `docs/ideas.md`).
- GitHub actions from other parties are fixed to a major version, not to an exact commit. Fix together with automatic dependency updates (idea in `docs/ideas.md`).
- The cluster follows the moving name `latest`; pinning to a fixed version is planned in the `gitops-lab` README before production use.
- The cached image build after a code-only push to `main` is not yet measured on GitHub (run 5 of the pipeline change; locally 27 s). Measure at the next code change.
- The untested images `sha-a9b5906` and `sha-a9b5906-migration` still exist; a tag on commit `a9b5906` would release them. Delete them (backlog, small).
