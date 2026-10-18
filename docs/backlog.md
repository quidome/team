# Backlog

Open work, most important first. Changes link to their folder; small fixes are one line.


Small:
- From 08:09 to 11:29 UTC on 2026-10-18 the Vault value `DATABASE_URL` of the team app named the database `team_v2` (empty, made by the app at 08:09) instead of `team`, so the app showed an empty tool. Set back by the coordinator. The empty database `team_v2` was deleted on 2026-10-18. `README.md` (the cluster contract) and `.env.example` now name both databases. To do: find the cause in the Vault version history (the value was right on 2026-10-17). A check that the app's database is not empty, after each release, would have found this at 09:16.
- A new game that the coordinator has not seen, confirmed as gone and then back in the game list, shows as "came back" instead of "new", so it is not in the summary line of new games (epic review accept A2; `update_open_changes` and `change_between` in `crates/core/src/comparison.rs`).
- Delete the untested images `sha-a9b5906` and `sha-a9b5906-migration` from GHCR (needs `gh auth refresh -s read:packages,delete:packages`, or the GitHub web page). Until then, do not tag commit `a9b5906`. Decision 0010 says they were deleted; that is true only after this.
- GitHub moves the `ubuntu-latest` runners to Ubuntu 26 from 19 October 2026 (notice in the `main` run of 2026-10-17; actions/runner-images#14748). All jobs in `.github/workflows/ci.yml` and `release.yml` use `ubuntu-latest`. Check the first run after the move (Check, Build with the smoke test, Publish); if something breaks, pin the runners to `ubuntu-24.04` until it is fixed.
