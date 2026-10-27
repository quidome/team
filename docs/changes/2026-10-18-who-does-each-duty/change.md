---
title: Who does each duty place, and the duty totals
type: feature
stage: implement
closure: review build + accept   # from the epic; /design may only raise it
created: 2026-10-27
---

# Who does each duty place, and the duty totals

Story B of the epic [Duties](../2026-10-18-duties/). The coordinator puts a player on each duty
place of the team, corrects it after the game when needed, and sees the duty totals of each player
for drive, table and referee, on the page of a game and on a new page "Duties". A new open item
shows the games with open places of the team. A change to a past game with duties stays in the list
of changes until seen. The functional spec is the epic's (`functional.md`, story B, the shared
rules and the facts of 2026-10-27); the architecture is in the epic's `technical.md`. Released
together with story A ([Duty places](../done/2026-10-18-duty-places/)) as release 1.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-10-27 | design | created from story B of the epic. Before the design the coordinator answered A1 (a reading never changes what is stored; a past game counts what is stored), asked to keep changes to past games with duties in the list of changes, and set the rule for clashes (same court: never; other court: under 1:30 blocks, under 2:00 warns); `functional.md` changed for these (facts of 2026-10-27) |
| 2026-10-27 | implement | technical.md approved: one story of 8 steps (the table and the rules of one place; totals, names and clashes in the core; store and routes with the race test; the choice on the page of a game; the open item of open places; the page "Duties"; changes to past games with duties; measure and finish); closure review build + accept |
