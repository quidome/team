# Season

What it does **now**, for the coordinator. Updated at each /close that changes it.

## Purpose
The coordinator finds any game or training of a season in one place: the games of the team, the
home games of other club teams ("possible duty games") and the training days and trainings, per
week. It is not a list for every day; it gives access to every event of the season (an event is a
training or a game).

## Behaviour
- The start page links to "Season", first in the line "Season · Attendance · Team · Players · Settings". The page shows "Season 2026-2027".
- **Which events.** All games of the team (home and away), all home games of other club teams, and every training day and training of the season. Away games of other club teams are not shown. The games come from the team chosen for that season; without a chosen team there are no games.
- **Headings.** A heading per week, Monday to Sunday, with its number and dates ("Week 44 · 26 Oct – 1 Nov"), and within it per date ("Sat 31 Oct"). A week without events has no heading.
- **Labels**, in three groups: What ("Games of the team", "Possible duty games", "Trainings"), When ("Coming", "Past") and Duties ("Entered", "Not entered", "Open places"). Labels of different groups narrow the list, labels of one group add to it, and a group with no label on shows all of that group. One tap turns a label on or off, without a new load. The page opens with "Coming" on and no label of What or Duties.
- **Duties.** "Entered" and "Not entered": the home games of the club whose committee schedule is entered, or not; "Open places": the games where the team has a place that no player fills yet (a place of the team without a player). With a label of Duties on, trainings and other games do not show. So "Not entered" with "Coming" is the list to walk through when the committee's schedule comes ([duties](duties.md)).
- **Labels are not kept between visits.** A new visit (for example from the start page) opens with "Coming" again. A return from a training or a game opened on "Season" ("Back", the browser's back, or after "Delete training") shows the same labels and the same place in the list; a game that no longer fits the labels (for example entered meanwhile) is gone from it.
- **Order.** The part "Coming" (today and later, Dutch date) nearest first. With "Past" on, the part "Past", newest week and date first. With both on, the page has the two parts, each with its own headings; the week of today can have a heading in both. Within a date: known start times in time order, then those not known; at the same time a training comes before a game.
- **A game** shows the start time ("time not known yet" when the game list has 00:00), the home team, the away team, the hall and town; a game of the team has a coloured edge and the badge "Team". A game no longer in the game list is shown crossed out, with "no longer in the game list", while its change is in the [list of changes](list-of-changes.md); in an earlier season it is hidden. Below the hall, its duty places: at a home game of the club the places of the team ("Referee 1: open · Table 1, 2: open, open") or "No places for the team", then "· Schedule not entered" while the schedule is not entered (with no place chosen at all: only "Schedule not entered"); at an away game of the team the drivers ("Drivers (4, proposed): Anna, 3 open"); the players by name ("Table 1, 2: Anna, open", "Bram (was Anna)", "not done (was Anna)"); "all filled" when every place of the team has a player; and what changed ("This game is no longer a home game of the club."). A tap on a game opens its page ([duties](duties.md)).
- **A training** shows its start time and "Not opened", "All there", "N absent" or "Cancelled", "· Checked" when it is checked, and "extra" for a training on a date that is not a training day. A tap opens the training (a training day that is not opened becomes a training). See [trainings and attendance](trainings-and-attendance.md).
- **"Cancel"** on a training day that is not opened cancels it without opening it. **"Show cancelled"** (on by default) hides cancelled trainings when off; each phone or computer keeps its own setting.
- **"Add training"** (only in the current season) adds a training on a date of the season without a training: the times are filled in from a weekly training on that day of the week, or typed.
- **Season choice**, when there is more than one season: the current season, and the earlier seasons with a chosen team, roles, weekly trainings or trainings. The season is in the address (`/season?season=2025-2026`), so a reload keeps it.
- **The "Game list" card** is at the top of the current season when a team is chosen (the open items of the game list on the start page link to it, `/season#game-list`): "Read the game list", the result of the last reading, the held-back warning and "may be out of date" ([read the game list](read-the-game-list.md)). After a good reading, the list shows the new games at once.

## Errors and limits
- No team chosen in the current season: the trainings, and "Choose the team to see its games." with a link to the start page.
- A season with no events: "No games or trainings in 2026-2027 yet."
- A season with no training days and no trainings: "No training days or trainings this season. Add a weekly training in Settings."
- A part with no events under the labels: "No coming trainings.", "No past possible duty games.", "No coming events.", and so on; with a label of Duties on: "No coming games with these labels."
- An address with a season that cannot be read (`?season=abc`): "This season cannot be read." with "Current season".
- The new games cannot be loaded after a good reading: "The new games could not be loaded. Load the page again." The reading itself is not affected.

## Interface
- Screen: `/season` (with `?season=` for another season), linked from the start page (`/`).
- Server address (needs a login): `GET /api/season?season=`: the season, the current season, today, the first and last day of the season, the seasons to choose, whether the games show (`shown`, `team_needed`, `none`), the club and team of the season's choice, and the events in two parts (`coming`, `past`), each with its ISO week (number, Monday, Sunday) and a game (with its places) or a training row.
- The order of the events is `team_core::season_events`; which games show is `team_core::schedule`, which training rows show is `team_core::season_list`.
