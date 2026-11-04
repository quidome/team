---
title: Attendance at games
type: feature
stage: implement
closure: review accept   # epic: each story has its own closure (review build + accept); a fresh agent checks the whole result after story D
created: 2026-10-31
---

# Attendance at games

During the week the coordinator keeps who comes to a coming game of the team, who does not, and who
has not answered, and after the game confirms who was there. The tool warns from 3 days before the
game when fewer than 7 players come, and when a player on a driver place does not come.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-10-31 | idea | created from `/vision attendance at games`: from "a record whenever there is time" to the week before the game; journey 7, priority 7 and the words "attendance" and "checked" changed |
| 2026-10-31 | discover | idea ready for discovery |
| 2026-10-31 | design | functional.md approved: five stories (A who comes, B warnings, C after the game, D totals, E not played, which also stops duties); three spec reviews: all "serves – note", S1–S39 answered; the first goal needs only A and B; words extra player and not played added |
| 2026-11-01 | implement | technical.md approved: an epic of five stories in two releases (A who comes, B warnings; then E not played, C after the game, D totals), each with closure review build + accept; epic closure review accept |
| 2026-11-01 | implement | story A, who comes to a game: technical.md approved (`changes/2026-11-01-who-comes-to-a-game/`), 4 steps |
| 2026-11-04 | implement | story B, warnings in time: technical.md approved (`changes/2026-11-04-warnings-in-time/`), 4 steps; deviation D1: the item of a moved game ends at the start time |
