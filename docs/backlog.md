# Backlog

Open work, most important first. Changes link to their folder; small fixes are one line.

Small:
- The browser test `web/e2e/choose.e2e.ts:200` ("games that cannot be loaded after a reading do not look like a failed reading") failed once in `just check` on 2026-10-31: "No changes" did not appear within 5 seconds. It then passed 3 times alone and in the next full run. Cause not found.
- Tap "Copy" on the page "Message of the week" (built first as "Message for the next game") on the phone after the release of story C, and note the result (accept review A7 of the message for the next game; the clipboard is checked only in the browser tests).
- Look at the start page of v0.4.0 on the phone: its speed, and the open items after a nightly reading (epic review A8 of start page and season; released 2026-10-25, migration 0010 checked the 8 trainings before that day).
- From 08:09 to 11:29 UTC on 2026-10-18 the Vault value `DATABASE_URL` of the team app named the database `team_v2` (empty, made by the app at 08:09) instead of `team`, so the app showed an empty tool. Set back by the coordinator. The empty database `team_v2` was deleted on 2026-10-18. `README.md` (the cluster contract) and `.env.example` now name both databases. To do: find the cause in the Vault version history (the value was right on 2026-10-17). A check that the app's database is not empty, after each release, would have found this at 09:16.
- Delete the untested images `sha-a9b5906` and `sha-a9b5906-migration` from GHCR (needs `gh auth refresh -s read:packages,delete:packages`, or the GitHub web page). Until then, do not tag commit `a9b5906`. Decision 0010 says they were deleted; that is true only after this.
- GitHub moves the `ubuntu-latest` runners to Ubuntu 26 from 19 October 2026 (notice in the `main` run of 2026-10-17; actions/runner-images#14748). All jobs in `.github/workflows/ci.yml` and `release.yml` use `ubuntu-latest`. Check the first run after the move (Check, Build with the smoke test, Publish); if something breaks, pin the runners to `ubuntu-24.04` until it is fixed.
