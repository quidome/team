---
title: After the game
type: feature
stage: implement
closure: review build + accept   # from the epic; "Checked" changes the record, and duties change from the start time
created: 2026-11-08
---

# After the game

Story C of the epic [Attendance at games](../2026-10-31-attendance-at-games/). From the start time
of a game the coordinator records who was there: the words become "There" and "Absent", and one tap
on "Checked" makes each "no answer" absent; "Undo check" brings back only those players. A past
game of the team that is not checked is an open item. From the start time the duty places also
offer "Someone else" and "Nobody", and count as done.

Spec: story C in [functional.md](../2026-10-31-attendance-at-games/functional.md) of the epic.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-11-08 | design | created from `/design story C of attendance at games`; the moment is the start time for "Checked", the words and the duties; a not played game can be checked; a moved game: the check and the settled answers have no effect (coordinator) |
| 2026-11-08 | implement | technical.md approved: 5 steps (duties from the start time; "Checked" stored; the page; the open item and the rows; the note on the players page); deviations D1 (own lock in `game_attendance`), D2 (the start time; a not played game can be checked; a moved game), D3 (the release day in `game_check_start`); spec review of the deviations: all serve, S40 kept (rare), S41 fixed; decision record 0032 at close |
