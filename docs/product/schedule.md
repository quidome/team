# The schedule

What it does **now**, for the coordinator. Updated at each /close that changes it.

## Purpose
The coordinator sees the games that matter on the phone, from the game list, and stops keeping
the schedule in the sheet: all games of the team, and the home games of other club teams, where
players of the team can have jury and referee duties ("possible duty games").

## Behaviour
- When a team is chosen, the start page shows the team (with the season, "Change", and the line "N changes not seen" when there are changes), the "Game list" card ([Read the game list](read-the-game-list.md)), and then the schedule.
- **Which games.** All games of the team, home and away, and the home games of other club teams. Away games of other club teams are kept, but not shown.
- **Filter.** "All", "Games of the team" or "Possible duty games". One tap changes it; one choice is always on.
- **Coming games** (today and later, Dutch date) come first, in time order, grouped by date ("Sat 31 Oct"). Within a date, games with a known start time come first; games without one come last.
- **Past games** of the season open with "Show past games", newest date first, and close with "Hide past games".
- Each game shows the start time, the home team, the away team, and the hall and town. A game of the team has a coloured edge and the badge "Team".
- A start time of 00:00 in the game list shows "time not known yet". No results are shown.
- A game that is no longer in the game list is shown crossed out, with "no longer in the game list", while its change is in the [list of changes](list-of-changes.md). After "seen", or when its date has passed, it is hidden. A gone game with no change (for example gone in the first reading after a choice) is hidden at once.
- After a good reading on the start page, the schedule shows the new games at once.
- Choosing a team reads the game list at once, so the schedule shows without a tap.
- **New season.** Until the team is chosen again, the start page shows the choose step and below it "Last season · 2026-2027" with last season's team and its games (open, newest first). This view has no read action. A renamed hall shows its newest name, also for earlier seasons.

- All games of the season, also past ones, are on [Season](season.md), together with the trainings, per week.

## Errors and limits
- No games and no reading yet: "The game list has not been read yet."
- No games after a reading: "The game list has no games for this season yet." The "Game list" card shows when it was last read.
- A filter with no games: "No coming games of the team.", "No past possible duty games.", and so on. Only past games: "No coming games."
- Last season with no kept games: "The tool has no games of 2025-2026."
- The schedule cannot be loaded again after a reading: "The new schedule could not be loaded. Load the page again." The reading itself is not affected.

## Interface
- Screen: the start page (`/`).
- Server address (needs a login): `GET /api/schedule`: the schedule to show, or `null` when nothing is chosen, or a team is needed and no earlier season had one. It gives the season, whether it is last season's, the club, the team, the coming games and the past games; each game has its reason ("team" or "duty") and whether it is still in the game list; and the number of changes not seen yet (`unseen_changes`, 0 for last season).
- The rule for which games show is `team_core::schedule`; which team's schedule shows is `team_core::schedule_target`.
