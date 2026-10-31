# 0023 – Players on the duty places, and what counts

**Status**: valid
**Date**: 2026-10-28
**Change**: [2026-10-18-duties](../changes/done/2026-10-18-duties/) (stories [duty places](../changes/done/2026-10-18-duty-places/) and [who does each duty place, and the totals](../changes/done/2026-10-18-who-does-each-duty/))

## Context
The duties belong to the game: at each home game of the club the referee committee gives the 4
table and referee places to club teams, and at each away game of the team players drive (story A,
tables `game_duties` and `duty_teams`). Story B puts a player on each place of the team, corrects
it after the game, and shows the duty totals, so the coordinator can show that the work is
shared fairly. The game list changes games at any time, also after they were played: a time, a
hall, home and away, or a game that leaves the list. One coordinator, but a phone and a computer
can save the same place at the same moment.

## Decision
- **One row for each filled place of the team** (`duty_assignments`): the planned player and the outcome after the game (as planned, done by another player, not done). The primary key (game, kind, number) keeps one player on a place; a unique key (game, planned player) keeps one place for each player at a game; the players are `ON DELETE RESTRICT`, so "Delete player" cannot remove a record of duties.
- **Every write of a player takes the row lock of the game** (`duties::lock`: insert the row of the game if missing, then `SELECT … FOR UPDATE`), reads the places and the players under the lock, checks in the core, and writes, in one transaction. A table or referee place gets a player only while it names the team; a place with a player keeps its team; the number of drivers never drops below the highest filled driver place.
- **What the coordinator stored is never changed by a reading of the game list** (A1, the coordinator, 2026-10-27). A coming game counts its places while it is in the game list and still needs them. A past game counts what is stored, also after a later swap, a removal from the game list, or another team chosen. A filled place of a past game counts as done by its player unless it is corrected; "Nobody" counts for nobody.
- **The totals are worked out at each request** by one rule, `team_core::duty_totals`, over all stored games of the season (`api::season::SeasonPlaces`). The choice of a game and the page "Duties" use the same function. Nothing is counted on the side (as for attendance, decision 0020, and the open items, decision 0022).
- **Clashes** (the coordinator, 2026-10-27): two games on the same court of the same hall never clash (a court runs games one after the other). On another court, start times less than 1:30 apart block the choice, and less than 2:00 apart give a note, because games often run over 1:30. A start time not known only gives a note; a game without a court counts as another court. A clash never blocks a place of a past game, or "Someone else": that records what happened. A player already on a place stays when a later change makes a clash.
- **The choice comes with the page of the game**, with the reasons a player cannot be chosen worked out by the same checks as the saves (`team_core::offers` from `check_assignment` and `check_outcome`). The front end only shows them.
- **A change to a past game with duties of the team stays in the list of changes until "Seen"** (a place that names the team, drivers above 0, or a player on a place; `duties::with_duties`, read in the transaction of the reading). There is no separate resolver: "Seen" means checked, and a correction on the page of the game changes what counts.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| One row for each filled place, with keys (chosen) | The database refuses what two screens could do | A new table and migration `0012` |
| Players as a list on `game_duties` | One row per game | No keys: two players on one place, or one player twice, only stopped by code |
| Totals worked out at each request (chosen) | Corrections, moves and gone games always right | Each page reads all duties of the season (measured: about 30 ms in a debug build) |
| Stored totals, updated on each save | Cheap to read | Every save, correction and reading must update them |
| A reading never changes what is stored (chosen) | The record of what happened is the coordinator's | A past swapped or gone game still counts; the coordinator sees its change and corrects by hand |
| A reading drops the duties of a game that swapped or left | No old places | A late fix in the federation's list would erase what the players did |
| A clash blocks under 1:30, warns under 2:00 (chosen) | Stops what cannot be done, shows what is tight | Two numbers to explain |
| A clash only warns | Simpler | A player could be planned on two courts at once |
| The reasons of the choice from the checks of the saves (chosen) | The choice never offers a player a save refuses | A list of offers for each player on the page answer |
| Past changes with duties kept until "Seen" (chosen) | A change to a game the players did shows | The list of changes can hold past games |

## Consequences
- A new kind of duty, or a new rule about who may do a place, is added to the core checks once; the choice and the saves follow.
- After another team is chosen in the middle of a season, the past places of the old team still count in the totals, but the page of a game shows only the places of the current team (an idea in `docs/ideas.md`).
- The clash and the role of a player are read before the lock of the game; a duty saved at the same moment at another game is not seen by the check (one coordinator; accepted).
- The messages of stories C and D read the players and the totals from these rules.
