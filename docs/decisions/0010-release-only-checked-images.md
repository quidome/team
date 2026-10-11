# 0010 – Release only images that passed every check

**Status**: valid
**Date**: 2026-10-11
**Change**: [2026-10-11-fast-release-pipeline](../changes/2026-10-11-fast-release-pipeline/)

Replaces 0009. It keeps the stages of 0009 (Check, Build, Publish, Release; no Nix in GitHub;
images on pull requests only when image inputs change; release by renaming) and changes which
image names mean what.

## Context
In 0009, Build pushed `sha-<commit>` as soon as its own smoke test passed, while Check could still
be running or fail later. Release only asked if `sha-<commit>` existed, so it could release an
image that never passed Check. This happened once (`sha-a9b5906`), found by the build review.
Also, the app image and the migration image were renamed one after the other without checking
that both existed.

## Decision
- **Build** (on `main`) pushes `build-<commit>` and `build-<commit>-migration`: built and smoke-tested, not yet checked.
- **Publish** (on `main`, only after Check and Build passed) gives those images the names `sha-<commit>` and `latest` (and the same for `-migration`). So `sha-<commit>` exists only for commits that passed everything.
- **Release** (tag `v*`) names the `sha-<commit>` images of the tagged commit; it fails when either of them does not exist.
- Publish and Release check that **both** images exist before any name moves, and rename the **migration first**: if the app step fails, old app code runs on forward-only migrations, which 0004 keeps compatible.
- The image inputs on pull requests are: `Dockerfile`, `.dockerignore`, `Cargo.lock`, `rust-toolchain.toml`, `web/package-lock.json`, `scripts/smoke-test.sh`, `.github/workflows/ci.yml`. When the list of changed files cannot be read, the job fails.
- Tag only commits on `main`: GitHub runs the workflow files of the tagged commit.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| Separate names: `build-` from Build, `sha-` from Publish | A name says what passed; Release stays a simple existence check | One more name in the registry |
| Release asks GitHub if the `main` run passed | No extra name | Release depends on the run history and API, not on the registry |
| Build waits for Check before pushing | One name | Build and Check no longer overlap; slower |

## Consequences
- `build-<commit>` images exist for every `main` push, also when Check failed; they are never deployed or released.
- The untested `sha-a9b5906` images from before this decision were deleted.
