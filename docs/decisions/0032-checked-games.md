# 0032 – "Checked" of a game

**Status**: valid
**Date**: 2026-11-08
**Change**: [2026-11-08-after-the-game](../changes/done/2026-11-08-after-the-game/) (story C of [attendance at games](../changes/2026-10-31-attendance-at-games/))

## Context
After a game the coordinator must record who was there with almost no work. Most players answer
before the game; the rest have "no answer" (no row, decision 0028). The coordinator's rules
(2026-11-08): everything entered can always be changed; "in the past" changes only how events are
shown; "not played" changes only what counts; "Checked" means the admin is complete and locks
nothing. The game list can move a game at any time, also after it was checked. From the start of
a game the real attendance and drivers can already be filled in.

## Decision
- **One moment, "started"** (`team_core::started`: the start time has passed; without a start time the game day has begun, Dutch time), holds for "Checked", for the words "There" and "Absent", and for the duties: from it a place offers "Someone else" and "Nobody", a filled place counts as done, and a clash does not block a place. Decision 0023 stays valid; this replaces its moment ("before today") for those rules. The week message and the duty overviews still drop a game only after its date.
- **"Checked" stores the date of the game** in `game_attendance.checked_for` (decision 0029): `checked` compares it with the date of the game now. After a move the check has no effect until the game moves back; a check on the new date replaces it.
- **"Checked" makes each "no answer" absent with a settled row**: `game_answers.settled`, a "not coming" without a reason. A settled row counts only while the check holds (`answers_in_effect`, read by every reader of the answers), so a moved game never shows answers nobody gave. A tap clears `settled`; "Undo check" deletes only the settled rows. A settled row does not hold back the date of the first real answer (`answers_for`), so a later move is still noticed.
- **Its own lock**: "Checked" and "Undo check" lock the row of the game in `game_attendance` (`SELECT … FOR UPDATE`), not the row lock of duties (decision 0030), and read the answers under that lock. The settled rows are inserted with `ON CONFLICT DO NOTHING`, so a tap at the same moment wins. Taps take no lock (decision 0020).
- **The day of the release** is stored in `game_check_start`, one row filled by its migration. From that date a past game of the team that is not checked, not "not played" and in the game list is an open item (`games_not_checked`, last in `open_items`). Older games are never an open item and show their list closed; they can still be marked and checked.
- **A not played game can be checked**, and so can a game with no player on its list. A second "Checked" changes nothing.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| Settled rows that count only while the check holds (chosen) | "Undo check" brings back exactly those players; a move shows nothing nobody said | Every reader must go through `answers_in_effect` |
| Plain "not coming" rows | No new column | "Undo check" cannot tell them from real answers; after a move they count on the new date |
| "Checked" only stored, "no answer" counted as absent when checked | No rows to insert | A player who gets a role later would count as absent without a tap; every count needs the rule |
| The row lock of duties | One lock for the game | Attendance would lock the tables of duties (decision 0030) |
| The release day from the date of the first check, or as in migration `0010` (all older trainings checked) | No new table | Older games would show "Checked" without anyone checking them, or need a guess |
| "Checked" from the day after the game (the first spec) | Matches trainings | The coordinator fills in the drivers and attendance already during the game |

## Consequences
- The coordinator checks a game with one tap from its start time; the open item reminds of every game since the release.
- Story D counts games in the totals of "Attendance" only when checked and played, and reads the release day from `game_check_start`.
- A player with a later role has "no answer" at earlier checked games and does not count there until tapped; the note after a role change names those games.
- A game without a start time starts "Checked" at 00:00 while its warnings run until the end of the day (accepted as rare).
