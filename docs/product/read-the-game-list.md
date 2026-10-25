# Read the game list

What it does **now**, for the coordinator. Updated at each /close that changes it.

## Purpose
The coordinator never types a schedule. With one tap, and by itself every night, the tool reads
the club's game list, keeps the games of the club and season, and says in one short line what
changed since the last reading. The game list is the truth: the tool never changes a game itself.

## Behaviour
- Choosing a team reads the game list at once (no result line shows for this reading; the [schedule](schedule.md) is the result).
- When a club and team are chosen, [Season](season.md) of the current season has a "Game list" card with the button "Read the game list". While a reading runs, the button shows "Reading…" and cannot be tapped. A tap while another reading runs waits for it.
- The tool reads all games of the club in the current season and keeps them. Games of other seasons are never changed.
- **Which games count.** Only games of today or later in the coordinator's schedule are counted: all games of the team (home and away), and the home games of other club teams (possible duty games). Away games of other club teams, and past games, are kept but never counted.
- **The result line** shows after a tap. It names only the kinds that changed, in this order: "N new", "N moved" (another date, start time or place), "N start time known", "N home and away swapped", "N team changed", "N no longer in the game list", "N came back", "N now in the schedule", "N no longer in the schedule". With nothing changed: "No changes". A game that changed in more than one way counts once. A game that left the game list while outside the schedule and comes back into it counts as "now in the schedule".
- **First reading.** The first reading of a club and team in a season counts nothing and says "N games read". After a change of team, the next reading is a first reading again. Saving the same club and team again does not make the next reading a first reading.
- A good reading also keeps each change in the [list of changes](list-of-changes.md) until the coordinator has seen it. The result line counts what this reading found; the list shows what changed since the coordinator last looked, so the two can differ (for example a game that moved and moved back).
- After a good reading, the games of "Season" below the card show the new games at once.
- "Last read: Mon 12 October 2026 14:00" shows the time of the last good reading, in Dutch time.
- A start time of 00:00 in the game list means the start time is not known yet.
- **The nightly reading.** Every night at 03:00 Dutch time the tool reads the game list by itself, in the same way as a tap: the same counts, the same guard, the same list of changes. It reads only when a team is chosen for the current season. When the tool was not running at 03:00, it reads as soon as it runs again. A reading after 03:00 (a tap, or choosing the team) makes the nightly reading of that night unnecessary. In the morning "Last read" shows the time of the nightly reading; no result line shows for it, the list of changes shows what changed.

## Errors and limits
- **Game list not reachable or not understood.** The games stay as they were. The card says "The last reading failed: The game list cannot be reached. Please try again." or "… The game list has data the tool does not understand.", and "Last read" still shows the last good reading.
- **Far fewer games.** When the game list has no games, or fewer than half the games of the last good reading of the season, the games are kept and nothing is counted. The card says "The game list has no games now." or "The game list has far fewer games than before (N instead of M).", then "The games are kept.", with the button "These games are really gone". A failed reading after it does not hide this warning. A later normal reading removes it.
- **"These games are really gone"** reads the game list again. When it has at least as many games as the warning showed, the missing games count as "no longer in the game list". When it has even fewer, the warning shows again with the new number, so no more games go than the coordinator confirmed.
- The first reading of a season accepts an empty game list (in summer the new season can still be empty).
- **Several nights without a good reading.** When 3 nightly run moments passed without a good reading (failed, or held back), the card starts with "The games may be out of date since Mon 12 October 2026 03:00." (the time of the last good reading). A good reading removes it.
- A held-back reading, and "may be out of date", are also open items on the [start page](start-page.md) ("The last reading of the game list was held back.", "The games may be out of date since Thu 22 Oct."); a tap goes to the card. A failed or held-back nightly reading shows on the card the next time the coordinator opens "Season", as for a tap. A nightly reading that fails is not tried again the same night; a tap can always read.
- One club and one team. The button shows only when a team is chosen, so in a new season the team must be chosen first; until then there is no nightly reading either. The game list is read when the coordinator taps, when a team is chosen, and every night.

## Interface
- Screens: "Season" (`/season#game-list`), card "Game list"; the open items on the start page (`/`).
- Server addresses (all need a login):
  - `POST /api/readings`: read now. The answer is the reading (good, failed or held back), never an error for a failed source.
  - `GET /api/readings/latest`: the newest reading, the last good reading, the held-back reading to confirm, and `out_of_date_since` (the time of the last good reading after 3 nights without one, else `null`), for the current club and season.
  - `POST /api/readings/{id}/confirm-gone`: "These games are really gone" for a held-back reading.
  - Errors: 400 `no_choice` (no club chosen), 400 `not_held_back` (that reading can no longer be confirmed), 404 (no such reading).
- The rules are in decisions 0014 (seasons as start years), 0015 (readings are recorded results) and 0017 (the nightly reading by a timer in the app; one reading at a time).
