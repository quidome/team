---
title: Players and settings
type: feature
stage: implement
closure: review build + accept   # stored roles deleted by a removal, new transactions with locks, the user behaviour changed after the second accept review
created: 2026-10-15
---

# Players and settings

Story A of the epic [Attendance at training](../2026-10-14-attendance-at-training/). The
coordinator types in the players with their roles (plays games or only trains, from and until),
sets the training days of the season with their times and the first and last training date, and
keeps the list of reasons for an absence.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-10-15 | design | created from story A of the epic |
| 2026-10-15 | implement | technical spec approved: 5 steps ("Add all" last), closure review build |
| 2026-10-16 | implement | build review found bugs R1 and R3 (in tests too weak to see them); all findings to be fixed in 4 steps; closure raised to review build + accept |
| 2026-10-16 | implement | accept review: needs correction (A1–A5); /discover review: changes the epic's functional spec for A1 and A2; this story gets an extra step after the new spec is approved and designed |
| 2026-10-16 | design | the epic's functional spec was changed and approved again (pages Players and Team, weekly trainings); this story goes back to design after /vision |
| 2026-10-16 | implement | designed again and approved: 4 steps (weekly trainings end to end, core rules for players, the team page, the players page and the player page); migration 0005 rewritten; the team page shows the chosen team; closure review build + accept |
| 2026-10-16 | design | second build and accept review: not ready (R13–R17, A6–A10); the epic's story 1 changed and approved again for A6–A10, R13 and R17; this story goes to /design for the new rules, with R14, R15, R16 and the stale comments |
| 2026-10-17 | implement | steps 5–9 designed and approved: core rules and "Change role", "Remove from the team" from a date with the confirmation, "Undo", "Add all" with a date and "Today", the settings fix R15; closure review build + accept |
