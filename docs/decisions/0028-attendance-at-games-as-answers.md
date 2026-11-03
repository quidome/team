# 0028 – Attendance at games as stored answers

**Status**: valid
**Date**: 2026-11-01
**Change**: [2026-10-31-attendance-at-games](../changes/2026-10-31-attendance-at-games/) (story [who comes to a game](../changes/done/2026-11-01-who-comes-to-a-game/))

## Context
Before a game the coordinator keeps who comes, who does not and who has not answered. A training stores only absences (decision 0020), because "there" is the normal state. At a game no state is normal: most players start with no answer, and roles can change during the week. The tool is used on a phone, tap after tap, and sometimes on a second screen.

## Decision
- **No row means "no answer".** `game_answers` has one row for each player who answered; `game_attendance` has at most one row for each game, with the extra players. The list comes from the roles on the date of the game, so it follows role changes, and an answer of a player off the list does not count.
- **Attendance has its own tables**, not columns of `game_duties`: answers are not duties, and a tap needs no row lock of the game. Each tap is one write and the last tap wins, as a mark at a training (decision 0020).
- **Answers stay.** The player link is `ON DELETE RESTRICT`, so "Delete player" is refused; the player can be hidden. A game with answers is kept by `kept_games`. The page of a game that left the game list keeps its count and buttons (coordinator, 2026-11-01); its row has no count.
- **A hidden reason is allowed when the answer already has it**, as in `check_mark`, so "Coming" and back keeps working.
- **One count.** `game_count` is the only count, and the server sends it after each tap. `row_count` says when a row shows it. The front end keeps the order of the first load and only shows the count.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| No row is "no answer" (chosen) | Follows roles; nothing to fill in or clean up | The list always needs the roles |
| A row for each player from the start | Simple reads | Must follow each role change |
| Own tables (chosen) | No lock for a tap | Two more tables |
| The count in the front end | No answer needed | A second rule; wrong with two screens |

## Consequences
- Each page with a count reads the answers and roles of the season; this is light, and the number of requests is no limit (coordinator).
- A player who answered and then leaves the list keeps the stored answer, which does not count.
- The rule that a tap sets `answers_for` only when the game has no answers gets its record at the close of story B.
