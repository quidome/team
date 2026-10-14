---
title: The list of changes
type: feature
stage: done
closure: review build + accept   # new table, the save of the choice takes the reading lock, two new routes (build); the daily "what must I act on" screen against functional story 4 (accept)
created: 2026-10-13
---

# The list of changes

Story 4 of the epic [Games from the game list](../2026-10-10-games-from-game-list/). The tool
keeps each change to a coming game in the schedule until the coordinator marks it as seen, so
the coordinator knows when a drive or a duty may need action. The start page shows how many
changes are not seen yet; a page lists them with old and new values, the quick filter, and
"seen" for one or all.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-10-13 | design | created from story 4 of the epic; coordinator's answers: a game that comes back into the schedule after it left outside it is "now in the schedule" (A3 of 3a); a summary line from 5 new games; an own page with one line on the start page (A2 of 3b); a game that moves back shows nothing; only changes to games of today or later (deviation) |
| 2026-10-13 | design | spec review of the deviations: all serve; two notes fixed (team's new games never in the summary line, "Mark these N as seen" with a filter), one for the duties change and `/close` |
| 2026-10-13 | implement | technical spec approved: 4 steps, closure review build + accept |
| 2026-10-13 | close | all 4 steps done; build and accept reviews: A1/R2 (a team game that another club team plays lost the team look), R1 (a version used again), R3/A3, R4, R5, R6, R8 comments fixed; A2, A4 no change; R7 and the documents for `/close` |
| 2026-10-13 | done | closed: product doc list-of-changes.md (new), schedule.md, read-the-game-list.md, choose-club-and-team.md, architecture.md, CLAUDE.md, vision, ideas, the epic; decision 0016 (0012 partly replaced); folder moved to changes/done/ |
