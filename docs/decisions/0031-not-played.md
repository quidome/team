# 0031 – Not played

**Status**: valid
**Date**: 2026-11-08
**Change**: [2026-11-08-not-played](../changes/done/2026-11-08-not-played/) (story E of [attendance at games](../changes/2026-10-31-attendance-at-games/))

## Context
A game that did not take place still counted: the drivers of a past game counted as having driven,
a game that left the game list before its date counted once its date passed, and a game known to
be off kept its warnings and open items. A reading must never change what the coordinator stored,
so the tool cannot decide this by itself. The coordinator's rules (2026-11-08): everything entered
can always be changed; "in the past" changes only how events are shown; "not played" changes only
what counts; "Checked" locks nothing.

## Decision
- **"Not played" is a fact of the game, owned by events**, in its own table `games_not_played` (decision 0030), not in `games`, which each reading writes. It is read with the games into `StoredGame.not_played_for`, so every reader of the games gets it from the type it already uses.
- **The mark holds for a date** (decision 0029): it stores the date of the game at the tap, and `not_played` compares it with the date of the game now. The row stays when the game moves.
- **What counts goes through one rule.** `game_places` takes a `GameMoment` (in the game list, past, started, not played). Each part is `in_use` as if played (past, or in the game list and required) and `counts` only when in use and not "not played". The totals, the open items, the messages, the overviews and the duty part of the clash rule read `counts`; `has_team_places` reads `in_use`, so the game still shows in "Next" and "Last". Decision 0023 stays valid; this adds the mark to its rule.
- **The clash rule reads the mark apart** for the games of the team (`team_games`), and `game_warnings` gives nothing for a not played game.
- **"The drivers drove"**, a switch in `game_duties` (duties owns it): from the start time of a not played game (without a start time: the start of the game day; `started`), its drivers `drove` and count in `duty_totals` only, never in open items, overviews or clashes.
- **Few refusals**: any game whose page opens can be marked, and the switch is accepted at any time; both are one write with no lock, the last tap wins.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| The mark on the stored game, in its own table of events (chosen) | One read; each reader gets it from the type it uses; a forgotten input cannot count the game | A field more on a type the game list also builds |
| A column in `game_attendance` (the first design of the epic) | The row already existed | A table of attendance would hold a rule that duties obey (decision 0030) |
| A separate list of not played games passed beside the games | The types stay as they were | Each reader needs a new input; one that forgets it counts the game |
| The switch counts in everything from the start time | One rule | On the game day the game came back in open items, overviews and clashes (build review R2) |

## Consequences
- A not played game counts nothing, warns about nothing, is in no message or overview and blocks no player, while all its data can still be changed; "Undo" brings everything back.
- Story C ("Checked") must decide again whether a not played game can be checked (by the coordinator's rules it can) and whether "Checked" needs the start time; story D leaves not played games out of the totals of "Attendance".
- A game that moves back to its marked date is "not played" again without a new tap.
