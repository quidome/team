---
title: The administration panel
type: feature
stage: done
closure: review accept
created: 2026-11-11
---

# The administration panel

Story 1 of the epic [Checked for every event](../../2026-11-11-checked-for-every-event/change.md):
every mark of a training and a game in one panel "Administration", one row each, with its state
(○ / ✓) and one action ("Mark as …", "Count the drives", "Undo"). The behaviour is in the epic's
`functional.md`, story 1; the architecture in its `technical.md`.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-11-11 | design | created from the epic (story 1); front end only |
| 2026-11-11 | implement | technical.md approved: 4 steps (components and training page, game page, lines below "Not checked" and the note, look); closure review accept (the reason is that the marks read clearly; no server change) |
| 2026-11-11 | implement | 4 steps built on the branch administration-panel; acceptance review: A1 (the count and the warning did not follow taps in the list) fixed, the rest as meant |
| 2026-11-11 | done | closed: principles.md (the rule of the marks), game-attendance.md, trainings-and-attendance.md, duties.md, architecture.md; the old words in code comments and API descriptions updated (A7) |
