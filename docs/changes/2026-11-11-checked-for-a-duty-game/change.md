---
title: Checked for a duty game
type: feature
stage: close
closure: review build + accept
created: 2026-11-11
---

# Checked for a duty game

Story 2 of the epic [Checked for every event](../2026-11-11-checked-for-every-event/change.md):
a duty game (a home game of another club team where the team has table or referee places) can be
marked as checked from its start time, for its date, only while it is a duty game. It changes only
the attention, not the totals. The behaviour is in the epic's `functional.md`, story 2; the
architecture in its `technical.md`.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-11-11 | design | created from the epic (story 2) |
| 2026-11-11 | implement | technical.md approved: 4 steps (core rules, data and routes, the row on the page, look and checks); closure review build + accept (a new table and migration, a changed route, a decision record) |
| 2026-11-11 | implement | 4 steps built on the branch duty-game-checked; build and acceptance reviews: A5, R1–R3, R5, R6 fixed (`?kind=` on the check, one rule `game_checked`, `mark_holds` in shared), R7 and A10 for Close |
