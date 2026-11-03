---
title: Who comes to a game
type: feature
stage: done
closure: review build + accept   # from the epic; /design may only raise it
created: 2026-11-01
---

# Who comes to a game

Story A of the epic [Attendance at games](../2026-10-31-attendance-at-games/). The page of a game of
the team gets a list of who comes, who does not, and who has not answered, with the count and the
extra players. The functional spec is the epic's (`functional.md`, story A); the architecture is in
the epic's `technical.md`.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-11-01 | design | created from story A of the epic; confirmed by the coordinator: a past game keeps the words "coming" and "not coming" until story C |
| 2026-11-01 | implement | technical.md approved: one story of 4 steps (rules and storage; routes; the list on the page of a game; the count on rows); closure review build + accept |
| 2026-11-01 | implement | all 4 steps built, `just check` passes; build review R1 to R10 handled, accept review: accepted with notes; A2 confirmed by the coordinator: the page of a gone game keeps its count and buttons |
| 2026-11-01 | done | closed: `docs/product/game-attendance.md`, decision 0028, architecture, vision; the next story of the epic is B |
