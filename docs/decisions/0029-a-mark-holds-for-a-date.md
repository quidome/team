# 0029 – A mark holds for a date

**Status**: valid
**Date**: 2026-11-04
**Change**: [2026-10-31-attendance-at-games](../changes/2026-10-31-attendance-at-games/) (story [warnings in time](../changes/done/2026-11-04-warnings-in-time/))

## Context
A game can move to another date after players answered. The answers stay stored (nothing the
coordinator stored changes by a reading of the game list), but they were given for the old date,
so "7 coming" can be false for the new date, and the warning "too few players" stays silent. The
list of changes shows the move, but "Seen" (and "Mark all as seen") is one tap on another page,
which never shows the answers. Later marks of a game (stories C and E: "Checked", "Not played")
have the same problem.

## Decision
- **A mark stores the date of the game when it was made**, and the core compares it with the date of the game now. A mark with another date has no effect, or, for the answers, is shown as "from before the move".
- **For the answers** the date is `game_attendance.answers_for`: the first answer of a game sets it to the date of the game (a later answer leaves it, so a new tap never confirms old answers); when all answers are removed, the next first answer sets it again. The answers are "from before a move" when the game has a stored answer and `answers_for` is not its date now (`team_core::answers_moved`).
- **Only the coordinator ends it**: "Answers checked" (`PUT /api/games/{season}/{id}/answers-checked`) stores the date of the game now. "Seen" in the list of changes does not, and nothing changes by itself.
- **The page of the game shows the note until "Answers checked"**; the open item "moved: check the answers" shows from 3 days before the new date until its start time (deviation D1 of the story, accepted by the coordinator).

## Options considered
| Option | For | Against |
|--------|-----|---------|
| Store the date of the mark and compare (chosen) | Nothing changes by itself; survives "Seen"; one comparison in the core | A date column for each kind of mark |
| Clear or flag the answers at a reading that moves the game | No comparison | A reading would change what the coordinator stored (the spec: never by a reading) |
| End the note at "Seen" of the change | No new button | "Mark all as seen" ends it without a look at the answers (spec review S1, S23, S30) |

## Consequences
- Story E ("Not played") and story C ("Checked") store the date of their mark in the same way (`not_played_for`, `checked_for`); a mark with another date has no effect.
- A game that moves back to the date of its answers is no longer "moved", without a tap.
- In release 1 a moved game whose start time has passed has no open item; only its page shows the note.
