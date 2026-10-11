---
title: Read the game list on demand
type: feature
stage: implement
closure: review build + accept   # lock, transaction and comparison (build); result line, guard warning and "really gone" (accept)
created: 2026-10-11
---

# Read the game list on demand

Story 3a of the epic [Games from the game list](../2026-10-10-games-from-game-list/). With one
tap the coordinator reads the club's game list; the tool stores the games of the club and season,
compares them with what it had, and shows a short result line. Readings never count a change
twice, never lose games on a bad answer, and hold back a reading with far fewer games.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-10-11 | design | created from story 3 of the epic, split into 3a (this) and 3b (the schedule) |
| 2026-10-11 | implement | technical spec approved: 4 steps, closure review build + accept; season stored as start year (decision 0014 at Close) |
