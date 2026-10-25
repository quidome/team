# Backlog

Open work, most important first. Changes link to their folder; small fixes are one line.

1. [Start page and season](changes/2026-10-25-start-page-and-season/) – feature, epic, stage implement: three stories, technical spec approved. Story A [The page "Season"](changes/2026-10-25-season-page/) is next: built and reviewed, next: `/close docs/changes/2026-10-25-season-page`. Then C (checked trainings), B (the start page), and the epic review accept.
2. [Duties](changes/2026-10-18-duties/) – feature, epic, back to discover after item 1: the screens change (start page, "Season", open items). Story A [Duty places](changes/2026-10-18-duty-places/) stopped in design after Phase 1 (2026-10-25); its structure (table `game_duties`, core `duties`, routes) stays usable. A row with all 0 places now means "handled, no places".

Small:
- From 08:09 to 11:29 UTC on 2026-10-18 the Vault value `DATABASE_URL` of the team app named the database `team_v2` (empty, made by the app at 08:09) instead of `team`, so the app showed an empty tool. Set back by the coordinator. The empty database `team_v2` was deleted on 2026-10-18. `README.md` (the cluster contract) and `.env.example` now name both databases. To do: find the cause in the Vault version history (the value was right on 2026-10-17). A check that the app's database is not empty, after each release, would have found this at 09:16.
- Delete the untested images `sha-a9b5906` and `sha-a9b5906-migration` from GHCR (needs `gh auth refresh -s read:packages,delete:packages`, or the GitHub web page). Until then, do not tag commit `a9b5906`. Decision 0010 says they were deleted; that is true only after this.
- GitHub moves the `ubuntu-latest` runners to Ubuntu 26 from 19 October 2026 (notice in the `main` run of 2026-10-17; actions/runner-images#14748). All jobs in `.github/workflows/ci.yml` and `release.yml` use `ubuntu-latest`. Check the first run after the move (Check, Build with the smoke test, Publish); if something breaks, pin the runners to `ubuntu-24.04` until it is fixed.
