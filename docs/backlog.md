# Backlog

Open work, most important first. Changes link to their folder; small fixes are one line.

1. [Attendance at training](changes/2026-10-14-attendance-at-training/) – design (next: `/design docs/changes/2026-10-14-attendance-at-training`)

Small:
- A new game that the coordinator has not seen, confirmed as gone and then back in the game list, shows as "came back" instead of "new", so it is not in the summary line of new games (epic review accept A2; `update_open_changes` and `change_between` in `crates/core/src/comparison.rs`).
- Delete the untested images `sha-a9b5906` and `sha-a9b5906-migration` from GHCR (needs `gh auth refresh -s read:packages,delete:packages`, or the GitHub web page). Until then, do not tag commit `a9b5906`. Decision 0010 says they were deleted; that is true only after this.
- Delete the old database `team` of the replaced app (README, "Switching from the old app", point 5).
