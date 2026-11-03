# Attendance at games

**Status**: who comes to a game is built. The warnings for too few players, "Not played" and "Checked" are not built yet.

## Purpose
During the week before a game the coordinator keeps track of who comes, from the chat with the players and parents. The coordinator sees at once how many players there are and whom to ask.

## Behaviour
### The list
- The page of a game of the team has at the top the players with the role "plays games" on the date of the game, also hidden players. Players who only train are not on it.
- Each player is "coming", "not coming" (with or without a reason) or "no answer". A player starts as "no answer".
- Each row has "Coming" and "Not coming". One tap saves at once. A tap on the chosen state goes back to "no answer".
- From the start page this takes two taps: a game in "Next", then a button.
- The groups are "No answer", "Coming" and "Not coming", in that order, each sorted by first name. An empty group is not shown. The rows do not move while the page stays open; a row stays in its group until the page is opened again.
- A past game keeps the words "coming" and "not coming", and can still be changed.

### The count
- Above the list the count stays in view, for example "6 coming · 1 not coming · 2 no answer". A part with 0 is left out. "Coming" is the players coming plus the extra players.
- The row of a coming game of the team in "Next" shows the same count, also with no answers ("9 no answer"). On "Season" it shows with at least one answer or extra player. The row of a game that left the game list has no count.

### Reasons
- A row with "not coming" has the link "Reason", or "Reason: School" with a chosen reason. The reasons do not open by themselves. They are the same reasons as at a training ([weekly trainings and reasons](weekly-trainings-and-reasons.md)): those that are not hidden, and a hidden reason the row already has.
- One tap adds or changes the reason; a tap on the chosen reason removes it. The reasons close at a tap on another row, and at a tap on "Coming" or "Not coming".
- The reason stays when the state goes to "Coming" and back. It goes when the player goes back to "no answer".

### Extra players
- Below the list "Extra players" has [−] and [+], from 0 to 10, saved at each tap. The tool keeps only the number, not who they are.

## Errors and limits
- A tap that fails shows the stored state and "Not saved. Try again." The taps of one player are sent one after the other; the last tap wins.
- "No players on this date." when the list is empty; "Extra players" still works.
- A game that left the game list stays on "Season" while it has answers, and its page keeps the count and the buttons. A game with only extra players is not kept.
- A game with answers that is no longer a game of the team (another team was chosen) can be opened, but shows no list. Its answers stay stored.
- A player with an answer cannot be deleted, only hidden or removed from the team ([players and team](players-and-team.md)).
- On a small phone a long name is cut off in its row.

## Interface
- Server addresses (need a login): `PUT` and `DELETE /api/games/{season}/{id}/answers/{player}`, `PUT /api/games/{season}/{id}/extra-players`. The list comes with `GET /api/games/{season}/{id}` (`attendance`).
- The rules are `team_core::game_list`, `game_count`, `row_count` and `check_answer`.
