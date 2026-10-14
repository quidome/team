---
title: Read the game list on demand
type: feature
stage: done
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
| 2026-10-12 | implement | build and accept reviews: fix A1/R1 (first reading from the last good reading's team), A2, R3, R4 and one "gone" rule in core before Close (coordinator); migration 0003 changed before its first release; closure stays review build + accept |
| 2026-10-12 | close | all 4 steps done, review findings answered in technical.md (A1, A2, R1, R3, R4 fixed; R2, R5, R8 for Close; A3, A4 for stories 4 and 3b) |
| 2026-10-12 | done | closed: product doc read-the-game-list.md, architecture.md, decisions 0014 and 0015 (0012 and 0013 partly replaced), vision, ideas; folder moved to changes/done/ |
