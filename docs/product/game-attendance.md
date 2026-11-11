# Attendance at games

**Status**: who comes to a game, the warnings in time, "Not played" and "Checked" are built, and checked games count in the totals of [Attendance](trainings-and-attendance.md).

## Purpose
During the week before a game the coordinator keeps track of who comes, from the chat with the players and parents. The coordinator sees at once how many players there are and whom to ask.

## Behaviour
### The list
- The page of a game of the team has at the top the players with the role "plays games" on the date of the game, also hidden players. Players who only train are not on it.
- Each player is "coming", "not coming" (with or without a reason) or "no answer". A player starts as "no answer".
- Each row has "Coming" and "Not coming". One tap saves at once. A tap on the chosen state goes back to "no answer".
- From the start page this takes two taps: a game in the list of events, then a button.
- The groups are "No answer", "Coming" and "Not coming", in that order, each sorted by first name. An empty group is not shown. The rows do not move while the page stays open; a row stays in its group until the page is opened again.
- From the start time of the game (without a start time: from the start of the game day) the words are "There" and "Absent": the buttons, the groups and the count ("8 there · 1 absent"). Everything can still be changed ([how the tool treats what is stored](principles.md)).

### The count
- Above the list the count stays in view, for example "6 coming · 1 not coming · 2 no answer". A part with 0 is left out. "Coming" is the players coming plus the extra players.
- The row of a coming game of the team on the start page shows the same count, also with no answers ("9 no answer"). On "Season" it shows with at least one answer or extra player. From the start time the row says "there" and "absent". The row of a game that left the game list has no count.

### Reasons
- A row with "not coming" has the link "Reason", or "Reason: School" with a chosen reason. The reasons do not open by themselves. They are the same reasons as at a training ([weekly trainings and reasons](weekly-trainings-and-reasons.md)): those that are not hidden, and a hidden reason the row already has.
- One tap adds or changes the reason; a tap on the chosen reason removes it. The reasons close at a tap on another row, and at a tap on "Coming" or "Not coming".
- The reason stays when the state goes to "Coming" and back. It goes when the player goes back to "no answer".

### Extra players
- Below the list "Extra players" has [−] and [+], from 0 to 10, saved at each tap. The tool keeps only the number, not who they are.

### Warnings
- **Too few players.** Above the count: "Too few players: 6 of 7" when fewer than 7 come (players coming plus extra players), from 3 days before the date (00:00, Dutch time), and earlier once the game has an answer, until the start time (without a start time: the end of the date). From 3 days before, the same game is an open item on the [start page](start-page.md); both come from one rule.
- **A driver who is not coming.** A player on a driver place whose answer is "not coming" shows "Anna · not coming" at the place; from the start time "Anna · absent", only while the place is "as planned" (after "Someone else" or "Nobody" there is no note). In the choice of a driver such a player stays a choice, with the note "Not coming". The tool never removes a driver. Until the start time it is an open item on the start page, at once.
- **Answers from before a move.** When the game moved after players answered: "Answers from before the move to Wed 11 Nov." with the button "Mark answers as checked". Only that tap ends it; "Seen" in the [list of changes](list-of-changes.md) does not, and a new answer does not (decision 0029). From 3 days before the new date until its start time it is an open item on the start page.
- No warnings for a game that left the game list, is no longer a game of the team, or is "not played" (below); none for table and referee places, or for a driver with "no answer".

### Not played
- **Every game whose page opens** has the button "Not played" as its last button: a game of the team, a home game of another club team, a game kept for its duties or answers, coming or past, also a game that left the game list. A reading of the game list never sets or clears it.
- **One tap** saves at once. The game then counts nothing: no duty in the totals (drive, table, referee), no open places and no "schedule not entered", no warning and no open item, no line in the messages or the duty overviews, and it blocks no player at another game ([duties](duties.md)). It does not count in the totals of "Attendance" either. Nothing stored is deleted.
- **The page** shows the row "Not played ✓" with "Undo" in the panel "Administration" at the top ([how the tool treats what is stored](principles.md), "Marks"); a game that is played has the button "Mark as not played" last on the page. The list of who comes is closed behind "Show the list", with the count beside it; opened, it works as before. Answers, extra players and duty places can still be changed ([how the tool treats what is stored](principles.md)). A driver's "· not coming" is not shown. "Undo" saves at once and makes everything count as before.
- **"The drivers drove."** From the start time (without a start time: from the start of the game day), a not played game with driver places of the team has the row "No drive counts ○" with "Count the drives" in the panel, and after the tap "The drives count ✓" with "Undo"; it is never orange (optional). When the drives count, its drives count in the duty totals as at a played game (done by the player on the place); the game is still no open item and in no overview. "Undo" of "Not played" keeps the mark of the drives; it has effect again only after a new "Not played".
- **The rows** on "Season" and on the start page show "· Not played" after the hall and town (a game of another club team: "Not played" below its teams); on a game of the team it replaces the count, and on the start page the row is lighter. A not played game stays in the list of events and among the last duty games.
- **A mark holds for a date** (decision 0029): the mark stores the date of the game. When the game gets another date, it is a normal game of its new date again, with its answers and duties; the mark stays stored, so a game that moves back to the marked date is "not played" again without a new tap. A new tap stores the new date.
- Two screens: the last tap wins.

### After the game: "Checked"
- **From the start time** (without a start time: from the start of the game day) a game of the team has the row "Not checked ○" with the orange "Mark as checked" in the panel "Administration" at the top of its page. Below the row: the count of who comes ("7 there · 3 absent", following each tap in the list), on an away game the drivers ("Drivers: Anna, 1 open"), and on a game where nobody has an answer the warning "Nobody has an answer yet: marking as checked makes all 6 players absent.". Before the start time it is refused: "A game can be checked from its start time." A not played game can be checked too, and so can a game with no player on its list.
- **One tap** makes each player with "no answer" absent, without a reason, and stores that the game is checked. The row then shows "Checked ✓", the message "2 players with no answer are absent." ("1 player with no answer is absent."; none for 0), and "Undo". The list is closed behind "Show the list", with the count beside it. A second check changes nothing.
- **A checked game can still be changed.** A tap moves a player between "there" and "absent"; a tap on the chosen state does nothing. A reason added after the check counts as a tap. A player who gets a role on the date later shows as "no answer" and does not count until tapped; after the role change the players page says so ([players and team](players-and-team.md)).
- **"Undo"** makes the game not checked and brings back to "no answer" only the players that "Checked" made absent and that were not tapped after that.
- **A check holds for a date** (decision 0029): it stores the date of the game. When the game moves, the check has no effect and the players it made absent are "no answer" again; when the game moves back, both hold again. A check on the new date replaces the old one. Nothing stored is deleted.
- **The open item** "Game Sat 31 Oct not checked" ("2 games not checked") is on the [start page](start-page.md) for each game of the team before today that is not checked, not "not played", still in the game list, and on or after 4 November 2026 (release 1, from which answers were entered). Older games are never an open item, and show their list closed behind "Show the list"; they can still be marked and checked.
- **The totals.** A checked game that was played counts on [Attendance](trainings-and-attendance.md) for each player on its list with "there" or "absent".
- **The rows** on "Season" and on the start page show "· Checked" after the hall and town; on the start page a checked game of the team is lighter. A past game of the team is not in the list of the start page.
- Two screens: the last check or undo wins. A tap at the same moment as "Checked" wins over it.

## Errors and limits
- A tap that fails shows the stored state and "Not saved. Try again." The taps of one player are sent one after the other; the last tap wins.
- "No players on this date." when the list is empty; "Extra players" still works.
- A game that left the game list stays on "Season" while it has answers, and its page keeps the count and the buttons. A game with only extra players is not kept.
- A game with answers that is no longer a game of the team (another team was chosen) can be opened, but shows no list. Its answers stay stored.
- A player with an answer cannot be deleted, only hidden or removed from the team ([players and team](players-and-team.md)).
- On a small phone a long name is cut off in its row.

## Interface
- Server addresses (need a login): `PUT` and `DELETE /api/games/{season}/{id}/answers/{player}`, `PUT /api/games/{season}/{id}/extra-players`, `PUT /api/games/{season}/{id}/answers-checked`. The list comes with `GET /api/games/{season}/{id}` (`attendance`, with `too_few`, `enough_players`, `answers_moved`, `checked` and `before_release`; each tap answers the same fields).
- "Checked": `PUT /api/games/{season}/{id}/checked?kind=team` answers `{ settled, page }` (the same address checks a duty game with `?kind=duty`, [duties](duties.md); a page that is out of date gets `game_changed`) (`settled`: the players made absent); `DELETE` ("Undo check") answers the page. Refusals: `checked_too_early`, `not_a_team_game`. The rows of `GET /api/season` and `GET /api/start` have `checked`, and a coming game of the team has `started`.
- "Not played": `PUT` and `DELETE /api/games/{season}/{id}/not-played` ("Not played", "Undo") and `PUT` and `DELETE /api/games/{season}/{id}/drivers-drove` (the switch), each answering the page of the game, which has `not_played`, `drivers_drove` and `started`. The rows of `GET /api/season` and `GET /api/start` have `not_played`.
- The rules are `team_core::game_list`, `game_count`, `row_count`, `check_answer`, and for the warnings `game_warnings` (with `in_warning_period`, `before_start`, `drivers_not_coming`, `answers_moved`, `move_note`, `ENOUGH_PLAYERS` 7 and `WARNING_DAYS` 3); for "Not played" `not_played` and `started`, and what counts in `game_places` (decision 0031); for "Checked" `check_game_checking`, `checked`, `answers_in_effect`, `to_settle`, `games_not_checked` and `now_no_answer` (decision 0032).
