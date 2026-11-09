# 0033 – The report

**Status**: valid
**Date**: 2026-11-09
**Change**: [2026-11-09-the-report](../changes/2026-11-09-the-report/)

## Context
The page "Report" shows, for each player of the current season, attendance at trainings and games
as percentages and the duties as done and planned, with details and a text to copy. The coordinator
uses it to convince people, so its numbers must be the numbers of "Attendance" and "Duties", and
they must never contradict each other. Three questions came up: where the new counts of the
corrections ("Someone else", "Nobody") are made, which players get a row (the two pages use
different rules), and whether the many reads of the page see one moment.

## Decision
- **The corrections are counted in `duty_totals`**, in the same pass as done and planned: for each
  place that counts at a started game, "Someone else" adds `done_for_other` to the player who did
  it and `done_by_other` to the planned player, and "Nobody" adds `not_done` to the planned player.
  So they follow exactly the rules of "done" (decision 0023, with "Not played" of decision 0031),
  also the past places of an earlier team. The rows of "Duties" still need done or planned, so that
  page does not change.
- **The rows of the report are the union** of the players with a training or game that counts
  ("Attendance"), the players with the role "plays games" in the season, and the players in the
  duty totals ("Duties"). The rule is in one function, `report_rows` (output). A player is in the
  table when the page "Team" puts the player in the team (a role today or later); every other
  player is under "Left the team", also a player with a duty and no role in the season.
- **The report reads in one transaction**, read-only with repeatable read, so every read sees the
  database as it was at the first one. The loading helpers of `api/season.rs` take a connection for
  this; the other pages pass a normal connection and keep separate reads.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| Corrections in their own function next to `duty_totals` | `Total` stays a pair | A second count of the same places; its rules can drift from "done" |
| Corrections in `duty_totals` (chosen) | One pass, one rule | `Total` grows; the tests that build it change |
| Rows as on "Attendance" only | One existing rule | Players with duties and no counted training are missing |
| Rows as the union (chosen) | Every player with a number has a row | A third row rule, written in one place |
| Separate reads, as the other pages | No change to the helpers | A save on another screen between two reads can give a row whose numbers disagree |
| One read-only transaction (chosen) | All numbers of one moment | The helpers take a connection; a write in them would fail (none does, checked 2026-11-09) |
| One transaction for all pages | The same guarantee everywhere | Not needed now: each other page shows one kind of number |

## Consequences
- A new rule of "done" in `duty_totals` changes the corrections with it, without extra work.
- A new reader of a season can use the same helpers inside a transaction.
- A write added to `trainings_of`, `games_of` or `duties_of` breaks the report (its tests show it);
  writes belong in their own handlers.
- If the rows of "Attendance" or "Duties" change, `report_rows` must be checked too.
