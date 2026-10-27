# Duties

What it does **now**, for the coordinator. Updated at each /close that changes it.

## Purpose
The coordinator records the duty places of each game in the tool, not in the sheet: at each home
game of the club the 4 table and referee places that the referee committee gives to the club
teams, and at each away game of the team the number of drivers. So the tool knows which places
are the team's, and later which players fill them. (Players on the places, the totals and the
messages come in the next stories.)

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

## Errors and limits
- A save that fails: "Not saved. Try again.", and the page shows what is stored.
- "Save" of the drivers cannot be tapped while the field is empty or not a whole number from 0 to 10.
- A team that is no longer a team of the club: "This team is not a team of the club. Load the page again."
- A game that is not a game of the chosen team's schedule (or an address that cannot be read): "This game cannot be found.", with a link to "Season".
- The table and referee places of the club's home games are entered by hand, one game at a time; the committee's sheet is not read.
- Players on the places, the totals for each player, the page "Duties" and the messages come in the next stories of duties.

## Interface
- Screen: the page of a game, `/game/[season]/[id]` (with `?from=start` or `?from=changes` for "Back").
- Server addresses (all need a login):
  - `GET /api/games/{season}/{id}`: the game with its places, the season, the current season, the team and the club teams of the pull-down lists.
  - `PUT /api/games/{season}/{id}/places/{kind}/{number}` (`referee` or `table`, 1 or 2) with `{ team }` (`null`: "—").
  - `PUT` and `DELETE /api/games/{season}/{id}/schedule-entered`.
  - `PUT /api/games/{season}/{id}/drivers` with `{ drivers }` (`null`: the proposal).
  - Each write answers the game page. Refusals: `not_a_home_game`, `no_drivers_needed`, `out_of_range`, `unknown_team`, `invalid_place`, `invalid_season`, `invalid_game`.
  - `GET /api/season`, `GET /api/start` and `GET /api/changes` give each game its places (`places`: the 4 places with their teams, the places of the team, "schedule entered", the drivers, whether each part is still needed and counts, `not_entered` and `open_places`).
- The rules are `team_core::required_duties`, `game_places`, `driver_proposal`, `club_teams`, `kept_games` and the checks `check_team`, `check_entered` and `check_drivers`.
