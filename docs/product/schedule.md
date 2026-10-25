# The schedule

What it does **now**, for the coordinator. Updated at each /close that changes it.

## Purpose
The coordinator sees the games that matter, from the game list, and stops keeping the schedule in
the sheet: all games of the team, and the home games of other club teams, where players of the
team can have jury and referee duties ("possible duty games"). The games show on
[Season](season.md), together with the trainings; the games of the team of the next two weeks also
show in "Next" on the [start page](start-page.md).

## Behaviour
- **Which games.** All games of the team, home and away, and the home games of other club teams. Away games of other club teams are kept, but not shown. The games of a season are those of the team chosen for that season; without a chosen team there are no games.
- On "Season" the labels "Games of the team" and "Possible duty games" choose the games; "Coming" (today and later, Dutch date) and "Past" choose the parts.
- **Order.** Coming games nearest first, past games newest first; within a date, games with a known start time first in time order, then those without one.
- Each game shows the start time, the home team, the away team, and the hall and town. A game of the team has a coloured edge and the badge "Team".
- A start time of 00:00 in the game list shows "time not known yet". No results are shown.
- A game that is no longer in the game list is shown crossed out, with "no longer in the game list", while its change is in the [list of changes](list-of-changes.md). After "seen", or when its date has passed, it is hidden. A gone game with no change (for example gone in the first reading after a choice) is hidden at once. In an earlier season gone games are hidden.
- After a good reading on the "Game list" card ("Season"), the new games show at once.
- Choosing a team reads the game list at once.
- An earlier season's games are on "Season" through its season choice. A renamed hall shows its newest name, also for earlier seasons.

## Errors and limits
- A part of "Season" with no games under the labels says so ("No coming games of the team.", "No past possible duty games.").
- The new games cannot be loaded after a reading: "The new games could not be loaded. Load the page again." The reading itself is not affected.

## Interface
- Screens: [Season](season.md) (`/season`) and "Next" on the [start page](start-page.md) (`/`).
- Server addresses (need a login): `GET /api/season` (the games with the trainings of a season) and `GET /api/start` ("Next"). Each game has its reason ("team" or "duty") and whether it is still in the game list.
- The rule for which games show is `team_core::schedule`.
