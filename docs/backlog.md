# Backlog

Open work, most important first. Changes link to their folder; small fixes are one line.

1. Store the player names encrypted in the database (AES-256-GCM in the app, the key from Vault and also kept outside the platform), so a copy of the database or a backup does not show who the players are; sorting and the same-name warning move from SQL to the core; existing players are encrypted once. The nightly backup runs (since 2026-10-18). Start: `/design technical: …` in this repository (coordinator, 2026-10-18).

Small:
- A new game that the coordinator has not seen, confirmed as gone and then back in the game list, shows as "came back" instead of "new", so it is not in the summary line of new games (epic review accept A2; `update_open_changes` and `change_between` in `crates/core/src/comparison.rs`).
- Delete the untested images `sha-a9b5906` and `sha-a9b5906-migration` from GHCR (needs `gh auth refresh -s read:packages,delete:packages`, or the GitHub web page). Until then, do not tag commit `a9b5906`. Decision 0010 says they were deleted; that is true only after this.
- GitHub moves the `ubuntu-latest` runners to Ubuntu 26 from 19 October 2026 (notice in the `main` run of 2026-10-17; actions/runner-images#14748). All jobs in `.github/workflows/ci.yml` and `release.yml` use `ubuntu-latest`. Check the first run after the move (Check, Build with the smoke test, Publish); if something breaks, pin the runners to `ubuntu-24.04` until it is fixed.
