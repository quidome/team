---
title: Start page and season
type: feature
stage: implement
closure: review accept   # epic: each story has its own closure (A and B review build + accept, C review build); a fresh agent checks the whole result after story B
created: 2026-10-25
---

# Start page and season

The start page becomes a landing page for what the coordinator does often (the last game and the
last training, the next events, the open items), and a new page "Season" shows all games and
trainings of the season, with date separators and labels that combine. A past training stays an
open item until the coordinator checks it. The base on which duties show their open items.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-10-25 | idea | created from `/vision` (the line of 2026-10-15 in `ideas.md`, extended on 2026-10-25 in the design of duty places); new priority 3 and journey 0 of the vision; words event, checked, open item, Season |
| 2026-10-25 | discover | idea ready for discovery |
| 2026-10-25 | design | functional.md approved: three stories (A the page "Season", B the start page as a landing page with "Open items", "Last" and "Next", C checked trainings); spec review: all three "serves – note", notes 1–7 answered (1, 2, 4 by the coordinator) |
| 2026-10-25 | implement | technical.md approved: an epic of three stories in the order A the page "Season" (review build + accept, release 1), C checked trainings (review build, release 1 or 2), B the start page (review build + accept); epic closure review accept; "Last" uses the seasons of the open items (coordinator, fixed in functional.md) |
