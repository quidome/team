---
title: The page "Season"
type: feature
stage: implement
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
