# Idea: The report

**Status**: ready for discovery
**Purpose fit**: The purpose itself: "the tool shows what each player gives to the team: the effort at training and games, and a fair share of the duties." Journey 8, priority 7. Both core values: see who makes an effort, and let people see that the duties are shared fairly.
**Date**: 2026-11-09

## The problem
Now and then the coordinator must steer a conversation, for example with a player, a parent or
the team, and needs facts for it. Sometimes people must be convinced that the work or the effort
is not in balance. Today the numbers are on two pages ("Attendance" and "Duties"), in a form made
for entering data, not for comparing players. The sheet shows attendance as percentages and
nothing about duties.

## The idea
As agreed with the coordinator (2026-11-09):

- A new page "Report" for the current season. Only the current season for now.
- **The table**: one row for each player, sorted by name:

  | name | trainings | games | drive | table | referee |
  |---|---|---|---|---|---|
  | Jopie | 60% | 50% | 0 + 0 | 1 + 2 | 1 + 0 |

  Attendance at trainings and games as a percentage, as in the sheet. Drive, table and referee as
  done + planned ("1 + 2": 1 done, 2 planned). Both duty views are useful: what a player did, and
  what is still coming.
- **The details of a player**: a tap on the name shows them on the same page, as an expanded row
  or as a pop-up (an expanded row keeps the table in view and lets the coordinator open two
  players to compare; a pop-up suits long details). They show:
  - trainings and games: how many there, how many absent with a valid reason, how many absent
    without one ("3 of 5 there, 1 absent valid, 1 absent not valid"), so a low percentage from
    illness is visible and not hidden in a formula;
  - duties: for each kind, done and planned, and the corrections after a game: how many duties
    planned for this player another player did, how many this player did in place of another
    player (both from the correction "Someone else"), and how many planned duties of this player
    nobody did (the correction "Nobody").
- **Copy**: each column of the table can be copied as text for the chat, with the names and the
  numbers, to show an imbalance. This can be any duty kind, and also attendance.
- The tool shows the numbers side by side and does not rank players; the coordinator draws the
  conclusion.
- "Attendance" and "Duties" stay as they are. They do more than totals (links to training days not
  opened, each missed training with its reason, the totals while a place is given).
- **Later, a second story**: the list behind the numbers in the details: which trainings and games
  the player missed, with the date and the reason, and which duties the player did or will do.

Two earlier sketches were dropped: one table with "3/5" in five columns (it fits on a phone, but
has no room for valid absences), and two tables on one page (more numbers, but less easy to read
at a glance).

## Why now
Attendance at games is built (v0.9.0), so all numbers exist. The vision calls the report the main
goal of the tool, and "Where we are" says effort stays "halfway" until the report exists.

## How it fits
- It adds no new data: the numbers are the same as the totals on "Attendance" and "Duties", and must
  stay the same (one source for each total).
- The copied text is a message in the sense of the word list: the tool produces text, the
  coordinator carries it to the chat.
- The phone is the main screen. Estimate (not measured): about 350 pixels for the table, about 100
  for the name, so 5 narrow columns of 40 to 50 pixels fit ("100%", "12 + 3").

## User view
The coordinator, a few times a season, before or during a conversation: open "Report" on the
phone, look at the table, tap a name for the details of one player, and maybe copy one column into
the chat. It must be
quick to read and the numbers must be right, because they are used to convince people.

## Open questions
For `/discover`:
- Which players have a row: the team of today, players who left, hidden players? (The two pages
  differ today: "Duties" includes players with a duty, "Attendance" players with a training or game
  that counts.)
- The percentage: of what exactly, how rounded, and what shows when nothing counts yet (0 of 0)?
- The details: expanded row or pop-up?
- The corrections in the details: the words for "done by another player", "done for another
  player" and "nobody did it"; for each kind or in total?
- The copied text: what does it look like, which players are in it, and is it one column, one
  table, or one kind? Must the text say which season or date it covers?
- Where is the link to "Report" (start page, or next to "Attendance" and "Duties")?
- Does the page need a season choice later, or is a period other than the season ever needed? (For
  now: no.)
- After real use: do the totals tables on "Attendance" and "Duties" still have a purpose, or does the
  report replace them? Not for this change.
