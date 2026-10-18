# Backlog

Open work, most important first. Changes link to their folder; small fixes are one line.

1. Release attendance at training (epic closed 2026-10-18): merge the pull request of `feat/the-overview` (story C) and tag a version, before 1 July 2027. Until then a training of an earlier season is reached on the cluster only by its address.

Small:
- Set up and check a backup of the team database (TrueNAS snapshots or a dump off the cluster, with a test restore). Before story A of attendance at training, all data could be read again from the game list; players and attendance cannot. The coordinator chose to do this later (2026-10-15).
- A new game that the coordinator has not seen, confirmed as gone and then back in the game list, shows as "came back" instead of "new", so it is not in the summary line of new games (epic review accept A2; `update_open_changes` and `change_between` in `crates/core/src/comparison.rs`).
- Delete the untested images `sha-a9b5906` and `sha-a9b5906-migration` from GHCR (needs `gh auth refresh -s read:packages,delete:packages`, or the GitHub web page). Until then, do not tag commit `a9b5906`. Decision 0010 says they were deleted; that is true only after this.
- GitHub moves the `ubuntu-latest` runners to Ubuntu 26 from 19 October 2026 (notice in the `main` run of 2026-10-17; actions/runner-images#14748). All jobs in `.github/workflows/ci.yml` and `release.yml` use `ubuntu-latest`. Check the first run after the move (Check, Build with the smoke test, Publish); if something breaks, pin the runners to `ubuntu-24.04` until it is fixed.
