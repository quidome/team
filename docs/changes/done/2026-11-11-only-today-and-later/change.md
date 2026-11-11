---
title: Only today and later on the start page
type: feature
stage: done
closure: checks      # set by /design; only the user lowers it
created: 2026-11-11
---

# Only today and later on the start page

Story 4 of the epic [Checked for every event](../../2026-11-11-checked-for-every-event/change.md):
the list of events on the start page shows only today and later. The last duty games go: a past
event shows only through the open items (story 3), so a past row and a coming row cannot be mixed
up. The behaviour is in the epic's `functional.md`, story 4; the architecture in its
`technical.md`.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-11-11 | design | created from the epic (story 4) |
| 2026-11-11 | implement | technical.md approved: 2 steps (core and server, front end and checks); closure checks (a removal; the open items already do the job, the tests prove it; the epic ends with its acceptance review) |
| 2026-11-11 | implement | 2 steps built on the branch only-today-and-later; closure checks: all checks pass |
| 2026-11-11 | done | closed: start-page.md, game-attendance.md, duties.md, architecture.md (no last duty games) |
