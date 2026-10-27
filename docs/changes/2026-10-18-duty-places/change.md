---
title: Duty places
type: feature
stage: close
closure: review build + accept   # from the epic; /design may only raise it
created: 2026-10-18
---

# Duty places

Story A of the epic [Duties](../2026-10-18-duties/). The coordinator records the duty places of
the team at each game: referee and jury places at home games of other club teams, and drivers at
away games of the team (proposed as one per 3 players). A page for each game, the page "Duties",
the mark "duties" in the schedule, and "has duties" in the list of changes. The functional spec
is the epic's (`functional.md`, story A and the shared rules); the architecture is in the epic's
`technical.md`. Released together with story B.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-10-18 | design | created from story A of the epic |
| 2026-10-25 | design | stopped after Phase 1 (architecture): the coordinator wants the screens split by job (landing page, season overview, open items; `ideas.md` 2026-10-15, extended 2026-10-25). Back to Discover of the epic after the landing page. Phase 1 answers: drivers only when the home club is another club; every away game with drivers counts as having duty places; the duties of a club no longer chosen stay stored and out of view; the needs come from the choice of the game's own season; a row with all 0 places means "handled". |
| 2026-10-25 | design | the epic design was approved again for the new screens (start page, "Season", decision 0022); it replaces the Phase 1 of 2026-10-18 above. Next: `/design` of this story again |
| 2026-10-25 | implement | technical.md approved: one story of 7 steps (migration and core `duties`; store and routes of a game; the places on the answers; the page of a game; "Season"; the start page; the list of changes); closure review build + accept |
| 2026-10-25 | design | all 7 steps built and both reviews run (R1–R9, A1–A7), not committed; then back to design: the epic goes back to discover because the duty places of a home game are given to club teams (`/vision` 2026-10-25). The new design reuses most of the code on disk; findings R2, R3, R7, R8 and A6 stay to fix. |
| 2026-10-26 | implement | technical.md approved again for places given to club teams: two tables (`game_duties` with "Schedule entered" and drivers, `duty_teams` one row per place that names a team), two parts per game that count on their own, one route per place, games with places kept on "Season" (D5); the first build copied from `origin/duty-places-v1` and reworked; 7 steps; closure review build + accept |
| 2026-10-27 | implement | 7 steps built on the branch `duty-places` (commit `e6147aa`); review build (R1–R5) and accept (accepted with notes, A1–A6): R1 accepted, R2 no defect (D3), R3, R4 and A1 for story B, R5 and A3 for `/close`; technical.md done |
