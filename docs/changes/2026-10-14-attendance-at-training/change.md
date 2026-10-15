---
title: Attendance at training
type: feature
stage: implement
closure: review accept   # epic: each story has its own closure; a fresh agent checks the whole result after story C
created: 2026-10-14
---

# Attendance at training

The coordinator, who is also the coach, keeps for each training which players were there and,
for each absence, the reason. Season totals for each player show the effort each player gives to
the team. This is the first change with players in the tool.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-10-14 | idea | created from priority 2 of the vision ("attendance at training") |
| 2026-10-14 | discover | vision check done, ready for discovery; words player, attendance and reason changed or added; attendance at games is a separate, next change |
| 2026-10-15 | design | functional spec approved after two spec reviews (all stories serve); coordinator's decisions: opening a training makes it exist with everyone there, cancel instead of delete (with undo, shown as cancelled, filter "Show cancelled"), "in the team since" for players who join later, the whole season in the list, last name optional (word player) |
| 2026-10-15 | design | spec changed after approval (coordinator, no third review): a player has no dates; a role (plays games or only trains) in the team of a season has a start and an end; a new season starts with no roles, "Add all" brings last season's players back |
| 2026-10-15 | implement | technical spec approved: epic with 3 stories (A players and settings, B trainings and attendance, C the overview); attendance stored as absences only; no backup yet (coordinator's choice) |
