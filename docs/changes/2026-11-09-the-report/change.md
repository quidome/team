---
title: The report
type: feature
stage: close
closure: review build + accept   # the numbers are used to convince people; the loading helpers of three pages change; a new count in the only source of the duty totals
created: 2026-11-09
---

# The report

A new page "Report" for the current season: a table with one row for each player
(attendance at trainings and games as a percentage, drive, table and referee as done + planned),
and the details of a player on a tap (valid and not valid absences, duties taken over by someone
else). Each column can be copied as text for the chat, to show an imbalance.

## Log
| Date | Stage | Note |
|------|-------|------|
| 2026-11-09 | idea | created from `/vision` (line of 2026-10-30 in `ideas.md`): a table with the details of each player on one new page; journey 8 and the word "report" changed |
| 2026-11-09 | discover | idea ready for discovery |
| 2026-11-09 | design | functional.md approved: three stories (A the table, B the details of a player, C copy a column); spec review: all "serves – note", notes 1 to 4 answered (a line for training days not opened and games not checked, "today or later", the link to "Attendance" with the player open and "Back", copy stays percentage only) |
| 2026-11-09 | implement | technical.md approved: one story in 6 steps (corrections in `duty_totals`, `output/report.rs`, `GET /api/report` in one read-only transaction, the page, the details, copy); decision 0033; functional.md story B: the corrections count with the rule of "done"; closure review build + accept |
| 2026-11-09 | close | all 6 steps built; `just check` passes; build review R1–R7 and acceptance review A1–A6: the phone layout (headings, "Copied" above the table), tests that can fail, `has_duty_row` and `overview_of` as shared rules fixed; R6/A3 under Deviations; R7 for close; A6 to `ideas.md` |
