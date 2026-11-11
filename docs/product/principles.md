# How the tool treats what is stored

What holds for every page **now**, for the coordinator. Updated at each /close that changes it.

## Purpose
The record of the season must stay right. The coordinator often learns later what really
happened, so the tool never locks a record and never changes it by itself.

## Behaviour
- **Changes are always possible.** Everything the coordinator entered can be changed later: the attendance of a past training or game, the places and players of a past game, the answers of a game that was not played, the records of an earlier season.
- **"In the past" changes only how events are shown**: which part of a list an event is in, some words (from the start time of a game "There" and "Absent", and a driver who is not coming shows "· absent"), and which buttons make sense (for example "Checked", "Someone else" and "Nobody" from the start time of a game). It does not lock anything.
- **"Not played" and "cancelled" change only what counts**: a game that was not played ([attendance at games](game-attendance.md)) and a cancelled training ([trainings and attendance](trainings-and-attendance.md)) keep all that is stored, which counts again after "Undo".
- **"Checked" means that the admin of an event is complete.** It locks nothing: a checked training or game can still be marked and changed ([trainings and attendance](trainings-and-attendance.md), [attendance at games](game-attendance.md), "Checked").
- **Marks.** A mark is a fact the coordinator sets on an event and can undo: "Checked", "Cancelled", "Not played", "The drives count", "Schedule entered". The marks of an event are rows of one panel "Administration" at the top of its page, shown only while a mark can be set. Each row is a fact with ○ (not set) or ✓ (set), sometimes one line below, and one button: the action when not set ("Mark as checked", "Mark as entered", "Count the drives"), "Undo" when set; a button never repeats the words of the state. Orange is only for the next step the coordinator must do ("Mark as checked", "Mark as entered"); a panel without orange means the administration of the event is done. Rare marks ("Cancel training", "Mark as not played") are a plain button until they are set, then a row. A choice between options ("Coming" / "Not coming", the reasons) is not a mark.
- **A reading of the game list never changes what the coordinator stored** ([read the game list](read-the-game-list.md)): it changes only the games themselves.

## Errors and limits
- Two screens: the last save wins, as each page says.
