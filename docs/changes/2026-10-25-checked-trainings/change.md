---
title: Checked trainings
type: feature
stage: close
closure: review build   # the migration changes stored data once; the check races with marks
created: 2026-10-25
---

# Checked trainings

Story C of the epic [Start page and season](../2026-10-25-start-page-and-season/). The coordinator
confirms that the attendance of a training is complete ("Checked", with "Undo check"); a past
training that is not checked will be an open item on the start page (story B). At the release
every training before today counts as checked. "Season" shows "Checked" on the row. The functional
spec is the epic's (`functional.md`, story C); the architecture is in the epic's `technical.md`.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-10-25 | design | created from story C of the epic |
| 2026-10-25 | implement | technical.md approved: a story of 4 steps (the migration; core and the store; the routes; the page and the row on "Season"); closure review build; the routes answer the training page (coordinator) |
| 2026-10-25 | close | all 4 steps built, `just check` passes (86 browser tests); review build R1–R5 fixed (R3, R4 by the coordinator: a row lock for a check, no "Checked" on a cancelled training); `docs/architecture.md` for `/close` |
