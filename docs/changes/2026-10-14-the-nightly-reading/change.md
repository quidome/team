---
title: The nightly reading
type: feature
stage: implement
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
