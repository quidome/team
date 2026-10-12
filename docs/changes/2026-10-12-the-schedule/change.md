---
title: The schedule
type: feature
stage: close
closure: review build + accept   # new route, shifted clock, changed choose flow (build); the first daily screen against functional story 3 (accept)
created: 2026-10-12
---

# The schedule

Story 3b of the epic [Games from the game list](../2026-10-10-games-from-game-list/). The start
page shows the coordinator's schedule from the stored games: all games of the team and the home
games of other club teams, with a quick filter, past games on request, and games that left the
game list crossed out. Choosing a team reads the game list at once; in a new season, last
season's schedule shows until the team is chosen again. The sample page goes away.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-10-12 | design | created from story 3b of the epic; coordinator's answers: crossed out until story 4, read at once after a choice (A4), last season's schedule in a new season (A4), newest hall name for earlier seasons (R5) |
| 2026-10-12 | implement | technical spec approved: 5 steps, closure review build + accept |
| 2026-10-12 | implement | step 4: found a stale start page after a change of team on `/choose` (preloaded data, already in story 2), fixed with `invalidateAll`; closure stays review build + accept (already both reviews) |
| 2026-10-12 | close | all 5 steps done; build and accept reviews: A1, A3, R1–R5 fixed; A2 for story 4; R6 for Close |
