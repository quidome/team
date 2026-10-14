# 0005 – Nightly reading as a Kubernetes CronJob

**Status**: replaced by 0017
**Date**: 2026-10-10
**Change**: [2026-10-10-games-from-game-list](../changes/2026-10-10-games-from-game-list/)

## Context
The tool reads the game list once a night without the coordinator, and also when the
coordinator asks. Two readings must never record the same change twice or lose a change.

## Decision
- A Kubernetes CronJob in the team chart (`gitops-lab`) runs `team read-game-list` with the same image and secrets. Its pods carry the labels `app.kubernetes.io/name: team` and `app.kubernetes.io/instance: team`, so the database network rule lets them in.
- Every reading takes a PostgreSQL advisory lock. A second reading waits, then compares against the new state.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| Kubernetes CronJob | Visible in the cluster; retries and history; no timer in the app | A chart change in another repository |
| A timer inside the app | No chart change | Hidden; tied to the app pod; runs twice with more replicas |

## Consequences
- The chart change is promoted through Argo CD, never applied by hand.
- The lock also protects readings on demand.
