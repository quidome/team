# How the tool treats what is stored

What holds for every page **now**, for the coordinator. Updated at each /close that changes it.

## Purpose
The record of the season must stay right. The coordinator often learns later what really
happened, so the tool never locks a record and never changes it by itself.

## Behaviour
- **Changes are always possible.** Everything the coordinator entered can be changed later: the attendance of a past training or game, the places and players of a past game, the answers of a game that was not played, the records of an earlier season.
- **"In the past" changes only how events are shown**: which part of a list an event is in, some words (a driver who is not coming shows "· absent" after the date), and which buttons make sense (for example "Someone else" and "Nobody" after a game). It does not lock anything.
- **"Not played" and "cancelled" change only what counts**: a game that was not played ([attendance at games](game-attendance.md)) and a cancelled training ([trainings and attendance](trainings-and-attendance.md)) keep all that is stored, which counts again after "Undo".
- **"Checked" means that the admin of an event is complete.** It locks nothing: a checked training can still be marked and changed ([trainings and attendance](trainings-and-attendance.md), "Checked").
- **A reading of the game list never changes what the coordinator stored** ([read the game list](read-the-game-list.md)): it changes only the games themselves.

## Errors and limits
- Two screens: the last save wins, as each page says.
