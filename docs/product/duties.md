# Duties

What it does **now**, for the coordinator. Updated at each /close that changes it.

## Purpose
The coordinator records the duty places of each game in the tool, not in the sheet: at each home
game of the club the 4 table and referee places that the referee committee gives to the club
teams, and at each away game of the team the number of drivers. On the places of the team they
put players, correct them after the game, and see the duty totals of each player, so the work is
shared fairly. From them the tool makes the [messages](messages.md) to copy.

## Behaviour
- **The page of a game** (`/game/2026-2027/<id>`) opens with a tap on a game on [Season](season.md), in "Next" or "Last" on the [start page](start-page.md), on a game of the open item "schedule not entered", or on the teams of a change in the [list of changes](list-of-changes.md). It shows the date and start time ("time not known yet" when not known), the home and away team, the hall and town, and the badge "Team" for a game of the team.
- **"Back"** returns to the page it came from: the start page, the list of changes, or "Season" of the game's season with the same labels and the same place in the list (also with the browser's back).
- **A home game of the club** (of any club team, also of the team; coming or past) has 4 places: Referee 1, Referee 2, Table 1, Table 2. Each has a pull-down list with "—" (empty) and the club teams of the season (each team with a game in the game list of that season, also a game that left it later, and a team that a place still names), sorted by name. A choice saves at once. One place holds one team; when two phones save the same place, the last save wins.
- Below the places, the places of the team: "M16 1: Referee 1, Table 1, Table 2", or "No places for the team."
- **"Schedule entered"**: a tap marks that the committee's schedule of this game is entered, also with empty places; the page says "Schedule not entered" before and "Schedule entered" after; a second tap undoes it. A later change of a place, or a move of the game, does not undo it.
- **An away game of the team** at another club shows the number of drivers. Until the coordinator sets a number it is the proposal, one driver for every 3 players with the role "plays games" on the game's date, rounded up, at most 10 (10 players: 4 drivers; 0 players: 0): "Drivers: 4 (proposed)". The proposal follows the team when players join or leave.
- The coordinator types a number from 0 to 10 and taps "Save"; from then on that number stays: "Drivers: 6", with "Proposal: 4 (one for every 3 players)" below. "Use proposal" goes back to the proposal. A saved number equal to the proposal shown stays "proposed", so it keeps following the team.
- An away game of the team in the club's own hall is a home game of the club: it has the 4 places and no drivers.
- **Games that change keep their duties.** A moved game keeps its places, their teams and "Schedule entered". A game that leaves the game list keeps them, with "no longer in the game list"; while it is gone its duties do not count and it is no open item, and when it comes back they count again. A game whose date has passed was played: its duties count, also when it leaves the game list later.
- A game that is no longer a home game of the club (home and away swapped) keeps its places, with "This game is no longer a home game of the club."; setting them to "—" removes them. An away game that no longer needs drivers of the team keeps its drivers, with "This game no longer needs drivers of the team."; 0 removes them. While it is coming, such a game does not count and is no open item.
- A game with a place that names a team, or with drivers set above 0, stays on "Season" also when it is gone or no longer in the schedule.
- Another team chosen in the middle of a season: each place keeps its team, so the places of the old team are never places of the new team.

### Players on the places
- **The places of the team** are the table and referee places that name the team, and the driver places of an away game of the team. Only these get players. The page of a game lists them under "Players on the places", one line for each place: "Table 2 · Yara", "Driver 1 · open".
- **The choice.** A tap on a line opens the choice: the players with the role "plays games" on the game's date (hidden players too) and the players already at the game, sorted by first name. Next to each name the player's totals for that kind in the game's season ("2 done · 1 planned"), and the clashes as notes ("also Table 13:00", "plays 16:00", "plays (time not known)"). The tool proposes no player. A tap on a player saves at once; "Clear" makes the place open again.
- **Who cannot be chosen** is shown without a tap, with the reason: "on this place", "already at this game" (one place for each player at a game, also as "Someone else"), "Not in the team on this date", "at another game at almost the same time".
- **Clashes.** A clash is a duty place at another game or another game of the team; the game itself is never a clash. Two games on the same court of the same hall never clash. On another court (in the same hall or another hall), start times less than 1:30 apart block the choice, and from 1:30 to less than 2:00 apart give a note. A game without a known start time gives a note for every game on another court on the same date and never blocks; a game without a court counts as another court. A clash never blocks a place of a past game: that records what happened. A player already on a place stays when a later change makes a clash; the place shows the note.
- **After the game** (a game before today, Dutch date), a place with a player offers "Someone else" (the same choice) and "Nobody". The place then shows "Bram (was Anna)" or "not done (was Anna)", so the planned player stays visible. Choosing the planned player again under "Someone else" undoes the correction; another planned player resets it. The note under a corrected place is of the player who did it.
- **Done and planned.** A filled place of a game before today counts as done by its player (or by "Someone else"); from today it counts as planned; an open place and "Nobody" count for nobody.
- **What counts.** What the coordinator stored is never changed by a reading of the game list. A coming game counts its places while it is in the game list and still needs them. A past game counts what is stored, also after a later swap, a removal from the game list, or another team chosen. Driver places with a player stay, also when the game no longer needs drivers.
- **Not in the team on this date**: a player on a place without the role "plays games" on that date stays, with that note, and still counts; the coordinator can clear or change it.
- **Names.** Two players with the same first name show the first letter of the last name ("Anna B.", "Anna V."), in the choice, on "Season", on the start page and on "Duties"; a player without a last name shows the first name only. The set is the players with a role in that season, plus the players on its places.
- **Two screens.** The row of the game is locked for each save, so one place never holds two players and one player never has two places at a game; the last save wins.
- **"Delete player"** is not offered for a player with duties ([players and team](players-and-team.md)).

### The page "Duties"
- **"Duties"** (`/duties`, a link on the [start page](start-page.md)) has the table "Totals" of a season: one row for each player with the role "plays games" in the season or with a duty in it, also players who left and hidden players, sorted by first name. The columns Drive, Table and Referee show "done · planned" ("3 · 1"). The numbers are the same as in the choice of a game of that season.
- A season choice, as on "Attendance": the seasons with a chosen team, the current season and the chosen season (`?season=` in the address). A season without a team has no rows. Without any duty: "No duties yet this season."

## Errors and limits
- A save that fails: "Not saved. Try again.", and the page shows what is stored.
- "Save" of the drivers cannot be tapped while the field is empty or not a whole number from 0 to 10.
- A team that is no longer a team of the club: "This team is not a team of the club. Load the page again."
- A game that is not a game of the chosen team's schedule (or an address that cannot be read): "This game cannot be found.", with a link to "Season".
- The table and referee places of the club's home games are entered by hand, one game at a time; the committee's sheet is not read.
- After another team is chosen in the middle of a season, the past table and referee places of the old team still count in the totals, but the page of the game, "Season" and "Last" show only the places of the current team (an idea in `docs/ideas.md`).
- The totals cover one season. The committee message and the duty overviews come in the next stories of duties.
- A choice that is refused says why ("This player already has a place at this game.", "This player is at another game at almost the same time.", "Driver place 3 has a player. Clear it first."); a save that fails: "Not saved. Try again." and what is stored.

## Interface
- Screens: the page of a game, `/game/[season]/[id]` (with `?from=start` or `?from=changes` for "Back"); the page "Duties", `/duties` (with `?season=`).
- Server addresses (all need a login):
  - `GET /api/games/{season}/{id}`: the game with its places (each place of the team with its player, the outcome and the player of "Someone else"), the season, the current season, the team, the club teams of the pull-down lists, `past`, and the choice (`players`: each with the name, hidden, the totals of the season for each kind, the clashes, and for each place of the team whether the player can be chosen as planned or as "Someone else", and why not).
  - `PUT /api/games/{season}/{id}/places/{kind}/{number}` (`referee` or `table`, 1 or 2) with `{ team }` (`null`: "—").
  - `PUT` and `DELETE /api/games/{season}/{id}/schedule-entered`.
  - `PUT /api/games/{season}/{id}/drivers` with `{ drivers }` (`null`: the proposal).
  - `PUT` and `DELETE /api/games/{season}/{id}/assignments/{kind}/{number}` (`drive`, `table` or `referee`) with `{ player }`: a player on a place of the team, or "Clear".
  - `PUT /api/games/{season}/{id}/assignments/{kind}/{number}/outcome` with `{ outcome, player }` (`as_planned`, `done_by` with a player, `not_done`): a correction after the game.
  - Each write answers the game page. Refusals: `not_a_home_game`, `no_drivers_needed`, `out_of_range`, `unknown_team`, `player_on_place`, `places_filled` (with `filled` and `highest`), `not_a_place_of_the_team`, `player_at_game`, `not_in_team`, `clash`, `not_past`, `no_player_on_place`, `invalid_outcome`, `unknown_player`, `invalid_place`, `invalid_season`, `invalid_game`.
  - `GET /api/duties?season=`: the season, the current season, the seasons to choose, and the rows (player, name, hidden, the totals for each kind, each `{ done, planned }`); `invalid_season`.
  - `GET /api/season`, `GET /api/start` and `GET /api/changes` give each game its places (`places`: the 4 places with their teams, the places of the team, "schedule entered", the drivers, whether each part is still needed and counts, `not_entered` and `open_places`).
- The rules are `team_core::required_duties`, `game_places`, `driver_proposal`, `club_teams`, `kept_games` and the checks `check_team`, `check_entered` and `check_drivers`; for the players `check_assignment`, `check_outcome`, `who_did`, `duty_totals` (the only source of the totals), `clashes`, `display_names`, `choice_players`, `offers` and `duty_rows`. Decision 0023 records the choices.
