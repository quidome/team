# 0017 – Nightly reading by a timer in the app

**Status**: valid
**Date**: 2026-10-14
**Change**: [2026-10-14-the-nightly-reading](../changes/done/2026-10-14-the-nightly-reading/)

## Context
The tool reads the game list once a night without the coordinator (functional story 5), and also
when the coordinator taps. Decision 0005 chose a Kubernetes CronJob in the team chart in
`gitops-lab` that runs a command `team read-game-list`, and a PostgreSQL advisory lock so that two
readings never record the same change twice or lose one. When story 5 was designed, the app ran
as one pod (`replicaCount: 1`), the lock existed, and every reading was stored with its result
(decision 0015), so the screen already showed a failed or held-back reading. The main reasons
against a timer in the app ("hidden", "runs twice with more replicas") no longer held.

## Decision
- `team serve` runs a timer next to the web server (`nightly.rs`). It checks at start and then every 5 minutes whether a nightly reading is due, and then reads with the same function as a tap (`reading::read`, guard on).
- The run moment is 03:00 Dutch time (`team_core::NIGHTLY_AT`). A reading is due when a team is chosen for the current season and no reading of the club and season, of any result, started at or after the last run moment. So a pod that was down at 03:00 reads when it starts, and a tap after 03:00 makes the nightly reading unnecessary.
- Each run moment is tried once: the timer keeps in memory the run moment it tried, also when the reading ends in an error that stores nothing. No retry in the same night.
- After 3 run moments without a good reading, `GET /api/readings/latest` gives `out_of_date_since` (the time of the last good reading) and the start page says that the games may be out of date.
- The lock of 0005 stays: every reading, the save of the choice and "seen" take `pg_advisory_xact_lock`; a second reading waits, then compares against the new state.
- There is no command `team read-game-list` (named in 0001 and 0005) and no CronJob.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| Timer in the app, check every 5 minutes (chosen) | No change in another repository; one rule covers a missed night, a tap in the morning and a clock that jumps (`DEV_NOW`); the result is stored and shown as for a tap | Two pods (a rollout at 03:00, or more replicas) can both read; no run history in the cluster |
| Kubernetes CronJob (0005) | Visible in the cluster; retries and history from Kubernetes | A chart change in `gitops-lab` with the same image, secrets and labels; a second command to keep |
| Timer that sleeps until 03:00 | No checks during the day | Misses the night when the pod starts after 03:00 or the clock jumps |
| A "nightly" mark on each reading | Could count failed nights exactly | A migration; the start times of readings are enough |

## Consequences
- The cluster contract does not change; story 5 needed no chart change.
- During a rollout at 03:00 both pods can read once: one extra request to the game list; the lock and "a later good reading wins" (0015) keep the data correct.
- With more than one replica each pod reads each night. Check this decision again before raising `replicaCount`.
- An end of the timer that was not asked for (a panic) is logged; the server then has no nightly reading until it restarts.
