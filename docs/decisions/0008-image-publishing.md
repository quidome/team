# 0008 – Image publishing: latest from main, fixed versions from tags

**Status**: replaced by 0009
**Date**: 2026-10-10
**Change**: [2026-10-10-the-base](../changes/done/2026-10-10-the-base/)

## Context
The cluster runs `ghcr.io/quidome/team:latest` and `:latest-migration` with `pullPolicy: Always`,
so a pod restart deploys the newest image. During active development the coordinator wants
quick rollouts, and also fixed versions.

## Decision
- Every push to `main` runs the checks, then publishes `latest` and `latest-migration`.
- A tag `v*` also publishes `<version>` and `<version>-migration`.
- Pull requests and other branches run the checks and build the images, but publish nothing.
- `main` must always be safe to deploy: a step is merged only when its checks pass.
- Pinning the cluster to a fixed version (digest) is a later change, as the cluster README plans before production use.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| `latest` from `main` plus version tags | Quick rollouts with a pod restart; fixed versions exist | A merge to `main` can reach the cluster at the next restart |
| Publish only from version tags | Full control over what reaches the cluster | Slower rollouts during development |

## Consequences
- Anything that must not reach the cluster stays off `main` (for example: database steps happen before the first new image).
- Rollback: restart with a fixed version tag in `gitops-lab`.
