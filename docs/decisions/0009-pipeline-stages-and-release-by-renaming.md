# 0009 – Pipeline stages, and release by renaming the tested image

**Status**: replaced by 0010
**Date**: 2026-10-11
**Change**: [2026-10-11-fast-release-pipeline](../changes/done/2026-10-11-fast-release-pipeline/)

Replaces 0008. It keeps from 0008: `main` publishes `latest` and `latest-migration`, the cluster
pulls `latest` at every pod start during active development, and `main` must always be safe to
deploy.

## Context
The first pipeline ran the same work for every push and tag: Nix in GitHub, Rust compiled three
or four times, the image built after the checks, and a tag built the image again. A push to
`main` took 7–20 minutes and a tag 12 minutes, 8 of them saving the Nix store cache.

## Decision
- **Check** (pull requests and `main`): all checks in one job, without Nix. Tools from `rust-toolchain.toml`, `package-lock.json` and pinned versions, with caches saved on `main` only.
- **Build** (`main`, and pull requests that change `Dockerfile`, `.dockerignore`, `Cargo.lock` or `web/package-lock.json`), at the same time as Check: one image build with cargo-chef and a smoke test on the real images. On `main` it pushes only `sha-<commit>` and `sha-<commit>-migration`.
- **Publish** (`main`, after Check and Build passed): `latest` and `latest-migration` are renamed to the `sha-<commit>` images. Nothing is built.
- **Release** (tag `v*`): the `sha-<commit>` images of the tagged commit also get the tag name. A tag on a commit without those images fails. Nothing is built.
- A newer run on the same branch stops the older one.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| Release by renaming the tested image | Seconds; the release is exactly what ran on `main` | A tag must wait until `main` passed |
| Build again for each tag (0008) | Independent of `main` | Slow; the release is a new build, not the tested one |
| Nix in GitHub | Same tools as locally | Large cache uploads (up to 8 minutes) per branch and tag |
| Tools from setup actions | Fast, cached | Versions kept aligned by files, not by one lock |

## Consequences
- Release steps: merge, wait for the green `main` run, check the cluster, then tag.
- The Nix shell stays the way to work locally; `just check` runs the same recipes as GitHub.
