---
title: The page "Season"
type: feature
stage: done
closure: review build + accept   # from the epic; /design may only raise it
created: 2026-10-25
---

# The page "Season"

Story A of the epic [Start page and season](../2026-10-25-start-page-and-season/). A page with all
events of a season (games of the team, possible duty games, training days and trainings), with a
heading per week and per date, labels that combine, a season choice and the "Game list" card. The
list of trainings with "Add training", "Cancel" and "Show cancelled" moves here from "Attendance".
The start page gets the link "Season" and keeps its schedule and cards until story B. The
functional spec is the epic's (`functional.md`, story A); the architecture is in the epic's
`technical.md`.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-10-25 | design | created from story A of the epic |
| 2026-10-25 | implement | technical.md approved: a story of 5 steps (core order and `not_opened`; `GET /api/season`; the page read only; the actions and "Back"; "Attendance" without its list); closure review build + accept; `?from=start` moved here from story B |
| 2026-10-25 | close | all 5 steps built, `just check` passes; review build (R1–R7) and review accept (accepted with notes, A1–A7): all fixed or answered (A1, R4, R5 by the coordinator); living documents for `/close`: architecture, trainings and attendance, CLAUDE.md |
| 2026-10-25 | done | closed: new `docs/product/season.md`; trainings and attendance, schedule, players and team, architecture (the page Season, patterns, debt), CLAUDE.md, vision and ideas updated; no decision record (0022 at the close of the epic) |
