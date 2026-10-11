# Backlog

Open work, most important first. Changes link to their folder; small fixes are one line.

1. [Games from the game list](changes/2026-10-10-games-from-game-list/) – implement (epic, 5 stories; stories 1 and 2 done)
   - Story 1, the base: done (v0.1.0).
   - Story 2, choose club and team: done (branch `feat/choose-club-and-team`, not merged or released yet).
   - Story 3a, [read on demand](changes/2026-10-11-read-on-demand/): implement (4 steps, closure review build + accept). Next: `/implement docs/changes/2026-10-11-read-on-demand`.
   - Then story 3b, the schedule (`/design` from the epic).

Small:
- Delete the untested images `sha-a9b5906` and `sha-a9b5906-migration` from GHCR (needs `gh auth refresh -s read:packages,delete:packages`, or the GitHub web page). Until then, do not tag commit `a9b5906`. Decision 0010 says they were deleted; that is true only after this.
- Delete the old database `team` of the replaced app (README, "Switching from the old app", point 5).
