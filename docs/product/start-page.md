# The start page

What it does **now**, for the coordinator. Updated at each /close that changes it.

## Purpose
A look of a few seconds tells the coordinator whether something still needs them, and what comes
next. Everything else of the season is one tap further, on [Season](season.md).

## Behaviour
- **No club chosen:** only the choose step ([choose club and team](choose-club-and-team.md)) and the links.
- **A new season without a team:** the choose step, then "Open items", "Last" and "Next" (with the trainings only), then the links.
- **A team chosen**, from the top: the team header (team, season, "Change"); "Open items"; "Last"; "Next"; "Messages" (only when a message has something to say); the links "Season · Attendance · Duties · Team · Players · Settings".
- **"Open items"**: what still needs the coordinator, one row for each kind, in this order; a tap goes to the place to act:
  1. "The last reading of the game list was held back." (a held-back reading waits) and "The games may be out of date since Thu 22 Oct." (3 nights without a good reading): to the "Game list" card on "Season" ([read the game list](read-the-game-list.md)).
  2. "4 changes not seen" ("1 change not seen"): to the [list of changes](list-of-changes.md).
  3. "Home game Sat 31 Oct: schedule not entered" for one, "5 home games: schedule not entered" for more: the home games of the club from today until 27 days later whose committee schedule is not entered ([duties](duties.md)); gone games, games that are no longer home games of the club, and past games are not in it. For one game a tap opens it; for more, a tap opens the list of those games below the item ("Sat 31 Oct · 14:30 Blue Drakes X14 1 – BV Alphia X14 1"), each a tap to its page, so the item and its list always hold the same games.
  4. "Away game Sat 7 Nov: 1 driver open", "Home game Sat 31 Oct: 1 table place open" (or "1 referee place, 2 table places open") for one, "3 games with open places" for more: the coming games with places of the team that have no player and that count ([duties](duties.md)): table and referee places from today until 27 days later, driver places from today until the Sunday of next week. A driver proposal that rises when a player joins opens a place without a save. For one game a tap opens it; for more, a tap opens the list of those games below the item, as for the schedule.
  5. "Training day Thu 22 Oct not opened" for one, "3 training days not opened" for more: to the oldest of them, which shows "Not opened" with "Open training".
  6. "Training Thu 22 Oct not checked" for one, "2 trainings not checked" for more: to the oldest of them ([trainings and attendance](trainings-and-attendance.md), "Checked").
  Without open items: "Nothing open."
- **Which dates count:** only dates before today (Dutch date); today's training is in "Next". A cancelled training or training day is never an open item. The open items cover the current season and, in July and August, also the season before, so the record of June stays in view after 1 July. In those two months the number of training days not opened can therefore be larger than the line on "Attendance", which counts one season.
- **"Last"**: the newest training before today that is not cancelled, in the same seasons as the open items, also when it is checked; and all games of the newest past date with places of the team (a place that names the team, or drivers), in the current season, or in July and August when it has none yet, the season before. Newest date first, a heading per date ("Thu 22 Oct"), each row as on "Season" ("Training · 19:45 · 2 absent · Checked"; a game with its places). A tap opens it, so a late correction is one tap. With no such training and no such game, "Last" is not shown.
- **"Next"**: the games of the team, the home games of other club teams with places of the team, and the training days and trainings from today until the Sunday of next week (on a Sunday: today and the next week), grouped by date, each row as on "Season" (games with their places). Other possible duty games are not in it. Cancelled trainings show, as "Cancelled" (the switch "Show cancelled" is a setting of "Season" only). A cancelled training does not count as something in the two weeks: when they have nothing else, "Next" also shows the first coming event of the season after them that is not cancelled; when there is none, "No coming events."
- **"Messages"**: below "Next", the button "Message of the week" when this week has a game of the team today or later, or table or referee places of the team ([messages](messages.md)); otherwise the part is not shown.
- A tap on a training or a game in "Last" or "Next", or on a training item or a game of an item, opens it; "Back" there returns to the start page.

## Errors and limits
- The start page cannot be loaded: "The start page could not be loaded. Load the page again."
- "Next" covers only the current season: in the last days of June its two weeks can reach into the new season, whose events do not show (an idea in `docs/ideas.md`).

## Interface
- Screen: the start page (`/`).
- Server address (needs a login): `GET /api/start`: the open items (each with its kind; dates and counts where they apply), the last training as an event, the last games with places of the team (`last_games`, with their season `last_games_season`), the events of "Next" (each with its week and a game or a training row, as on `GET /api/season`), and `messages` (`week`: whether the message of the week has something to say, from the same rule as `GET /api/messages/week`). The item "schedule not entered" has its games (season, id, date, start time, home, away); the item "open_places" has its games with the open places of each kind (`referee`, `table`, `drive`).
- The rules are `team_core::open_items`, `schedule_not_entered`, `open_places_games`, `last_training`, `last_games`, `next_events` and `open_item_seasons`; each open item comes from the rule of its source page (the "Game list" card, the list of changes, the page of a game, "Attendance").
