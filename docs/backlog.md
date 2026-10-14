# Backlog

Open work, most important first. Changes link to their folder; small fixes are one line.

1. [Games from the game list](changes/2026-10-10-games-from-game-list/) – implement (epic, 5 stories; all done)
   - Story 1, the base: done (v0.1.0).
   - Story 2, choose club and team: done (on `main`, not released yet).
   - Story 3a, read on demand: done (on `main`, not released yet).
   - Story 3b, the schedule: done (on `main`, not released yet).
   - Story 4, the list of changes: done (on `main`, not released yet).
   - Story 5, the nightly reading: done (on `main`, not released yet).
   - Review of the whole epic: done (accepted with notes). Next: `/close docs/changes/2026-10-10-games-from-game-list`.

Small:
- A new game that the coordinator has not seen, confirmed as gone and then back in the game list, shows as "came back" instead of "new", so it is not in the summary line of new games (epic review accept A2; `update_open_changes` and `change_between` in `crates/core/src/comparison.rs`).
- Delete the untested images `sha-a9b5906` and `sha-a9b5906-migration` from GHCR (needs `gh auth refresh -s read:packages,delete:packages`, or the GitHub web page). Until then, do not tag commit `a9b5906`. Decision 0010 says they were deleted; that is true only after this.
- Delete the old database `team` of the replaced app (README, "Switching from the old app", point 5).
