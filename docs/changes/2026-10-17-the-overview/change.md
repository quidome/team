---
title: The overview
type: feature
stage: implement
closure: review build + accept   # the totals are why the coach keeps attendance; a wrong count does not show itself
created: 2026-10-17
---

# The overview

Story C of the epic [Attendance at training](../2026-10-14-attendance-at-training/). The page
"Attendance" gets the season totals for each player ("there", "absent, valid", "absent, not
valid", "of N trainings"), the group "Left the team", the number of past training days not
opened, the missed trainings of a player, and the choice of an earlier season. The functional spec
is the epic's (`functional.md`, story 5, and the parts of stories 1–3 that the totals need).

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-10-17 | design | created from story C of the epic; coordinator's choices: the missed list shows only absences that count; the start page keeps its links until the new start page (idea of 2026-10-15, extended 2026-10-17) |
| 2026-10-17 | implement | technical spec approved (the coordinator started /implement): 4 steps (the counting rule in the core; `GET /api/attendance`; the choice of season; the totals on the page); closure review build + accept; spec review of the deviations: all serve (notes D1 and D8 for /close) |
