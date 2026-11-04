---
title: Warnings in time
type: feature
stage: implement
closure: review build + accept   # from the epic; the start page sends player names, and the warning period has edges a fresh look can test
created: 2026-11-04
---

# Warnings in time

Story B of the epic [Attendance at games](../2026-10-31-attendance-at-games/). The start page warns
in time when a coming game of the team has fewer than 7 players, when a driver does not come, and
when a game moved with answers from before the move; the page of the game shows the same facts.
The functional spec is the epic's (`functional.md`, story B, and the move in story A); the
architecture of the epic is in its `technical.md`.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-11-04 | design | created from story B of the epic; confirmed by the coordinator: Q1 the line "Too few players" on the page only in the warning period; Q2 the item "moved" ends at the start time (deviation); Q3 a cancelled game still warns until story E |
| 2026-11-04 | design | spec review of the deviations: D1 serves; Q1 serves – note: Q1 was asked wrongly (story A gives the period of the line); the coordinator chose the spec, so the line shows from 3 days and earlier once the game has an answer |
| 2026-11-04 | implement | technical.md approved: one story of 4 steps (the rules; the three open items; the page of a game: too few and drivers; a moved game); closure review build + accept; deviation D1 accepted by the spec review |
