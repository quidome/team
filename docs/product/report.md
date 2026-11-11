# The report

What it does **now**, for the coordinator. Updated at each /close that changes it.

## Purpose
Now and then the coordinator must steer a conversation with a player, a parent or the team, and
sometimes convince people that the effort or the duties are not in balance. The report puts what
each player gave this season on one screen: attendance at trainings and games, and the duties done
and planned. A tap shows what is behind the numbers, and each column can be copied into the chat.
The numbers are the numbers of [Attendance](trainings-and-attendance.md) and [Duties](duties.md);
the report adds no data and does not rank players.

## Behaviour
### The table
- The [start page](start-page.md) links to "Report" after "Duties". The page shows the current season only ("Report 2026-2027"), with no season choice, and "Back" to the start page.
- One row for each player of the season, sorted by first name, with the short names of "Duties" ("Anna B." when two players have the same first name); hidden players too. A player has a row when a training or game counts for the player (the rows of "Attendance"), or when the player has a row on "Duties" (the role "plays games" in the season, or a duty done or planned).
- Players with a role today or later in the season are in the table; all others are below it under "Left the team", in the same columns (also a player with a duty and no role in the season).
- **Trainings** and **Games**: the percentage "there" of the trainings or checked games that count for the player, whole numbers rounded half up (2 of 3: "67%"; 1 of 8: "13%"). A valid absence lowers it as a not valid one does. "–" when nothing counts for the player, for example Games for a player who only trains.
- **Drive**, **Table**, **Referee**: "done + planned" ("1 + 2"); "0 + 0" for a player with a row on "Duties" and no duty of that kind. "–" for a player without a row on "Duties", and for every player when no team is chosen.
- What counts is the rule of the two pages, without change: a training that exists, is not cancelled, is today or earlier and falls in a role of the player; a game of the team that is checked and played, for a player on its list with an answer; a duty as on "Duties" (done from the start of the game, planned before, "Not played" counts nothing except the drives with "The drives count").
- Above the table, when the numbers are not complete: "2 training days not opened · 1 game not checked" (each part only when not 0). It counts the training days before today that were never opened (as on "Attendance") and the games of the open item "not checked" of the start page in this season. A tap on a part opens the oldest of them; "Back" there returns to "Report".
- On a phone 360 pixels wide the table fits without scrolling to the side, with names of 10 characters; the column headings may break with a hyphen ("Train-ings").

### The details of a player
- A tap on a row (the name or the numbers) opens the details under it; a second tap closes them. The details do not repeat the name. More than one player can be open, also under "Left the team".
- "9 trainings: 6 there (67%) · 2 absent, valid (22%) · 1 absent, not valid (11%)", and the same for games; a line where nothing counts is not shown. Each percentage is rounded on its own, so the three can add up to 99 or 101; the numbers are exact.
- For each duty kind with a duty or a correction: "Table: 2 done · 1 planned", and below it, only when not 0, the corrections after a game:
  - "1 done by another player": planned for this player, done by another player ("Someone else"); not in this player's "done";
  - "1 done for another player": this player did it in place of the planned player; also in this player's "done";
  - "1 not done": planned for this player, corrected to "Nobody"; not in "done".
  The corrections count with the rule of "done": at a started game whose duties count, also the past places of an earlier team.
- "Nothing counts yet this season." for a player with no line.
- "Missed trainings and games" (for a player with trainings or games) opens [Attendance](trainings-and-attendance.md) with the list of that player open; "Back" there returns to "Report".

### Copy
- The table of the team has "Copy" under each column heading. A tap puts the text on the clipboard and shows "Copied" above the table. When the browser refuses, the text shows above the table with "Select the text and copy it.", to select by hand. A column without lines has no "Copy". The tool never sends the text.
- The text is Dutch: a heading with the kind, the club and team, the season and today's date of the tool's clock, then one line for each player in the team (not "Left the team") whose cell is not "–", by first name:

  ```
  Trainingen Blue Drakes M16 1, seizoen 2026-2027 (tot en met 9 nov)
  Anna 90%
  Bram 67%
  ```
  ```
  Rijden Blue Drakes M16 1, seizoen 2026-2027 (tot en met 9 nov)
  Anna 3 (+1 gepland)
  Bram 0
  ```
- The kinds are "Trainingen", "Wedstrijden", "Rijden", "Tafel" and "Scheidsrechter". Trainings and games give the percentage only, the duties the done number with the planned number in brackets when not 0. No valid split and no corrections. Without a chosen team the heading has no club and team ("Trainingen seizoen 2026-2027 (tot en met 9 nov)").

## Errors and limits
- "No trainings, games or duties yet this season." when no player has a row.
- The page cannot be loaded: "The report could not be loaded. Load the page again."
- Only the current season; no other period, and no sorting by a column.
- All numbers are read at one moment, so a save on another screen never gives a row whose numbers disagree.
- Two players with the same first name and no last name have the same short name (an idea in `docs/ideas.md`).
- The copied attendance has no valid split, so a player who was ill looks the same as one who stayed away (an idea in `docs/ideas.md`).

## Interface
- Screen: `/report`. Links from it: `/training/[date]?from=report`, `/game/[season]/[id]?from=report`, `/attendance?player=<id>&from=report`.
- Server address (needs a login): `GET /api/report`: the season; `incomplete` (`not_opened`, `first_not_opened`, `not_checked`, `first_not_checked` with season and id); `team` and `left`, each row with player, name, hidden, `trainings` and `games` (`counted`, `there`, `absent_valid`, `absent_not_valid`, `percent`, `percent_valid`, `percent_not_valid`; `null`: "–") and `duties` (`drive`, `table`, `referee`, each `done`, `planned`, `done_by_other`, `done_for_other`, `not_done`; `null`: "–"); `copies` (`trainings`, `games`, `drive`, `table`, `referee`: the text, or `null`). Names are only in the answer, never in a log.
- The rules are `team_core::report_rows`, `percent`, `Share`, `report_copy` and `ReportColumn` (output), over `season_overview` (attendance), `duty_totals` with its corrections and `has_duty_row` (duties), and `games_not_checked` (start page). Decision 0033 records the choices.
