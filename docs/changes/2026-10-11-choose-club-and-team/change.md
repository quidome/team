---
title: Choose club and team
type: feature
stage: close
closure: review build + accept   # strict reader and browser test setup (build); raised in step 4 after a screen problem only the screenshot showed (accept)
created: 2026-10-11
---

# Choose club and team

Story 2 of the epic [Games from the game list](../2026-10-10-games-from-game-list/). The
coordinator finds the club by name in the source's club list and chooses the team from the club's
own teams, changes it later, and chooses again at a new season. The choice is kept for each
season, so earlier seasons stay.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-10-11 | design | created from story 2 of the epic |
| 2026-10-11 | implement | technical spec approved: 4 steps, closure review build; full game reading moved here from story 3; one row for each season |
| 2026-10-11 | implement | steps 1–4 done; closure raised to review build + accept: step 4 found a wrong API description from step 3 and a screen problem that no test caught |
| 2026-10-11 | close | build review (R1–R8) and accept review (needs correction, A7 = R2): R1–R6 fixed, R7 at commit, R8 for Close; accept notes A5 and A12 not fixed (not asked for); all checks pass |
