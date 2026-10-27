# The list of changes

What it does **now**, for the coordinator. Updated at each /close that changes it.

## Purpose
When a game of the team changes, or a home game of another club team where players may have a
duty, the coordinator must know, so the players can still do their drive or duty. The tool keeps
each change until the coordinator has seen it.

## Behaviour
- **Which changes.** Changes to the games in the [schedule](schedule.md) (all games of the team, and the home games of other club teams) of today or later. A change to a past game is not kept.
- **Kinds.** "new"; "moved" (another date, start time or place, with the old and the new value, for example "Sat 31 Oct 16:00 → Sat 31 Oct 17:30"); "start time known"; "home and away swapped" (old and new home team); "team changed" (the old teams); "no longer in the game list" ("was Sat 31 Oct 14:00"); "came back" (a game that left the game list while it was in the schedule).
- **Into or out of the schedule.** A change that moves a game into or out of the schedule says why: "now a home game of another club team", "now a game of the team", "no longer a home game of another club team", "no longer a game of the team". Also a game that left the game list while outside the schedule and comes back into it is "now in the schedule".
- **One change for each game**, from what the coordinator last saw to what is true now. A game that changes twice shows the first old value and the newest value. A game that is again as last seen (for example it moved and moved back) shows no change. A game that leaves the game list before the coordinator saw its change shows nothing while it is gone; if it comes back, it is compared with what the coordinator last saw, so a new game the coordinator never saw is still "new", not "came back".
- **Start page.** The open item "1 change not seen" / "N changes not seen" on the [start page](start-page.md) opens the list. It shows only when there are changes.
- **The page "Changes"** (`/changes`) has the same filter as the schedule ("All", "Games of the team", "Possible duty games"), starting at "All". A change to a game the team plays, or played when last seen, has the team colour and the badge "Team".
- **Many new games.** 5 or more new possible duty games show as one line "N new games" that opens. New games of the team always show one by one.
- **Seen.** "Seen" on a change marks that change. "Mark all as seen" marks every change; with another filter the button says "Mark these N as seen" and marks only what the filter shows, also the games inside a closed "N new games" line.
- A change that a reading changed again after the page was loaded stays after "Seen", with its new values.
- After "seen", a game that is no longer in the game list is hidden in the schedule, unless it is kept for its duties ([duties](duties.md)).
- **Duties.** A change of a game where the team has places (a place that names the team, or drivers), or of a game that no longer needs a part of its duties, says "has duties" with the places of the team ("has duties · Table 1, 2: open, open"), and what changed ("This game is no longer a home game of the club."), so the coordinator can tell the players and the committee.
- A tap on the teams of a change opens the page of the game; "Back" there returns to the list. A game that left the schedule without duty places has no page, so its teams are no link.
- The first reading after choosing the club and team records no changes. Choosing another club or team, or the team of a new season, removes the changes not seen yet; saving the same club and team again keeps them.

## Errors and limits
- No changes: "No changes not seen." With a filter: "No changes of the team not seen." or "No changes of possible duty games not seen."
- A change whose game has passed (the old and the new date before today) drops off the list without a tap.
- The changes come from readings of the game list ([Read the game list](read-the-game-list.md)): each tap and the nightly reading.
- No history of seen changes. Messages to players come later.

## Interface
- Screens: the start page (`/`), the open item "N changes not seen"; the page `/changes`.
- Server addresses (all need a login):
  - `GET /api/changes`: the season of the list (`null` without a choice) and the changes not seen yet of the chosen team of the current season, in the order of the games. Each change has its kind, its differences, its reason ("team", "duty" or "kept"), the game as last seen (`before`, `null` for a new game), the game now (`after`, `null` for a game no longer in the game list), its places (as on `GET /api/season`), and a version.
  - `POST /api/changes/seen` with `{ changes: [{ id, version }] }`: marks these changes as seen; a change with another version stays. The answer is the number of changes still not seen.
  - `GET /api/start` has the open item `changes` with the number of changes not seen.
- The rules are in decision 0016 (an open change keeps the game as last seen).
