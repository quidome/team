---
title: Totals on "Attendance"
type: feature
stage: close
closure: review build + accept   # released totals and the form of a released API answer change; many small text rules from the spec reviews
created: 2026-11-08
---

# Totals on "Attendance"

Story D of the epic [Attendance at games](../2026-10-31-attendance-at-games/). Each row of
"Totals" on "Attendance" also shows the games the player was at: only checked games that were
played count, for a player on the list of that date with "there" or "absent". A tap on a player
also lists the missed games.

Spec: story D in [functional.md](../2026-10-31-attendance-at-games/functional.md) of the epic.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-11-08 | design | created from `/design story D of attendance at games`; every checked game that was played counts, also one from before the release day; a checked game that left the game list counts, a kept game of another team does not; missed games read "Game Sat 10 Oct · Ill" and "Game Sat 10 Oct · without a reason" (coordinator) |
| 2026-11-08 | implement | technical.md approved: 3 steps (the core rules; the route; the page); no deviations |
| 2026-11-08 | close | 3 steps built; reviews build (R1–R5) and accept (A1–A5, accepted with notes); fixed A1 (key of the missed list), R1 (the totals read the list of the page of a game), R2–R4; the rest for Close or no change needed; `just check` passes |
