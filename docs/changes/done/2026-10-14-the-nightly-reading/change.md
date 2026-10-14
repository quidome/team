---
title: The nightly reading
type: feature
stage: done
closure: review build   # a background task in the server: the loop and its stop at shutdown are checked only by hand
created: 2026-10-14
---

# The nightly reading

Story 5 of the epic [Games from the game list](../2026-10-10-games-from-game-list/). The tool
reads the game list by itself every night at 03:00, so the schedule is up to date without the
coordinator thinking of it. When no good reading happened for 3 nights, the start page says that
the games may be out of date, and since when.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-10-14 | design | created from story 5 of the epic; coordinator's answers: a timer in the app instead of a Kubernetes CronJob (replaces decision 0005), run at 03:00 Dutch time, "may be out of date" after 3 nights with no good reading, no retry in the same night, closure review build |
| 2026-10-14 | implement | technical spec approved: 4 steps, closure review build |
| 2026-10-14 | close | all 4 steps done; build review: R1 (an error that stores nothing was tried again every 5 minutes) and R2 (a timer that stops is not logged) fixed; R3, R4 no change; R5 the documents for `/close` |
| 2026-10-14 | done | closed: product docs read-the-game-list.md, list-of-changes.md; architecture.md, CLAUDE.md, vision, the epic; decision 0017 (replaces 0005); folder moved to changes/done/ |
