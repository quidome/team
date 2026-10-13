# Choose club and team

What it does **now**, for the coordinator. Updated at each /close that changes it.

## Purpose
The tool reads the games of one club from the game list. The coordinator tells it once which
club and which team are theirs, so it reads the right games. The coordinator never types a
number: the club is found by name, and the team is chosen from the club's own teams.

## Behaviour
- Before a club and team are chosen, the start page shows only the choose step.
- The coordinator types part of the club name and taps "Search". Upper or lower case is not important. Each result shows the club name and the town (the name only when the game list has no town for the club).
- After a tap on the club, the tool reads the club's games of the current season and shows the club's own teams, not the opponents, with short names (for example "M16 1" for "Blue Drakes M16 1"). A tap on a team saves the choice and reads the game list at once, so the [schedule](schedule.md) shows.
- The start page then shows the club and team (for example "Blue Drakes M16 1") and the season, with "Change". "Change" opens the teams of the same club; "Other club" goes back to the search.
- **New season.** A season starts on 1 July (Dutch time), as the NBB (the Dutch basketball federation) counts it, and is written as "2026-2027". In a new season the tool keeps the club and asks for the team again, because a youth team moves up an age group. It shows last season's team (for example "In 2026-2027 the team was Blue Drakes M16 1"), and below the choose step last season's schedule.
- **No games yet.** In summer the new season can still be empty. The tool then says that the club has no games for the season yet. A club that is not kept yet can be saved without a team ("Save club"); a kept club needs nothing. The team is chosen on the start page when games appear.
- The choice of each season is kept. A change in the same season replaces that season's choice; earlier seasons are not changed.
- Another club or team, or the team of a new season, removes the changes not seen yet ([list of changes](list-of-changes.md)); saving the same club and team again keeps them.

## Errors and limits
- "No club found. Try another name." when the search finds nothing.
- "The game list cannot be reached. Please try again." when the game list does not answer, or answers with an error. "Try again" repeats the last action. Nothing is saved.
- "The game list has data the tool does not understand." when the game list has data in a form the tool does not know. Nothing is saved.
- One club and one team for each season. More teams or more coordinators are not possible.
- The game list is read each time the coordinator searches, opens the teams, or saves; a save with a team also starts a reading of the games. While a team is still needed, each visit to the start page reads it too. Reading the games themselves is in [Read the game list](read-the-game-list.md).

## Interface
- Screens: the start page (`/`) and the page for a change (`/choose`).
- Server addresses (all need a login): `GET /api/selection` (the state of the choice), `GET /api/clubs?search=` (search), `GET /api/clubs/{number}/teams` (the club's own teams this season), `PUT /api/selection` (save club and team).
- Setting: `GAME_LIST_URL`, the address of the game list (default `https://db.basketball.nl`). The cluster uses the default.
