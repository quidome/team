---
title: Attendance at training
type: feature
stage: done
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
| 2026-10-16 | discover | review of the accept findings A1 and A2 of story A: functional spec changed (training days apply from a chosen date, all training days of the season shown; "Change" for a role); A3 (a new season fills in last season's training days) goes to story B; spec review pending |
| 2026-10-16 | discover | third spec review (S12–S16): coordinator replaced "Applies from" by weekly trainings (a day of the week with times in a window, more than one at a time; they only suggest trainings; opened trainings are independent; any training can be deleted by hand); roles of earlier seasons cannot be changed; word "weekly training" for /vision; fourth spec review pending |
| 2026-10-16 | discover | fourth spec review (S17–S22): coordinator split story 1 into the pages "Players" (all players, "Hidden", the history with "Correct") and "Team" ("Add player", "Change role" and "Remove from the team" with a date); "Change" on a training; trainings of an earlier season stay open; words player, role, hidden and weekly training for /vision; fifth spec review pending |
| 2026-10-16 | discover | fifth spec review: all stories serve (notes S23–S26, fixed; coordinator: the proposed start is the first training date until a training of the season is opened); waiting for approval, then /vision for the words player, role, hidden, weekly training and training day |
| 2026-10-16 | design | functional spec approved again (coordinator); before design: /vision for the words player, role, hidden, weekly training and training day; then the epic's technical spec and story A are designed again |
| 2026-10-16 | implement | technical spec designed again and approved: weekly trainings replace the season's training days, pages Players and Team, one training per date with "Change" and delete by hand; story A rewrites its migration 0005; story A to /design again |
| 2026-10-16 | implement | /discover review of story A's second accept and build review (A6–A10, R13, R17): story 1 changed and approved again after the sixth spec review (all stories serve; S27–S31 fixed). "Remove from the team" from a date (no roles from then, "Undo"), "In the team from" 1 September with "Today", hidden only shortens lists; the epic's technical spec drops the first training date rule at /design of story A |
| 2026-10-17 | implement | story A closed: players, the team with roles ("Remove from the team" from a date with "Undo"), weekly trainings and reasons; decision 0018 (so the record of story B becomes 0019); product documents players-and-team and weekly-trainings-and-reasons; story B next |
| 2026-10-17 | implement | story B closed: trainings (open, add, change, cancel, delete), the list on "Attendance", the card, marking with one tap per absence; decision 0020 (0019 is not used); product document trainings-and-attendance; story C next, before 1 July 2027 |
| 2026-10-17 | implement | story C designed and approved in its own folder (`2026-10-17-the-overview`): the start page keeps its links (coordinator); notes for /close: the Interface line about links from "Attendance", and "Add training" only in the current season |
| 2026-10-17 | implement | story C closed: the totals, "Left the team", the training days not opened, the missed trainings and the choice of season on "Attendance"; product document trainings-and-attendance; functional.md changed (Navigation, "Add training" only in the current season); the epic review accept is next |
| 2026-10-18 | done | epic review accept: accepted with notes (A1–A4; A4 an idea line); closed: the living documents were updated at the close of each story; vision "where we are" (built and reviewed, a release next); links of decisions 0018 and 0020 and of the stories to `done/`; folder moved to `docs/changes/done/`; the release stays as a line in the backlog |
