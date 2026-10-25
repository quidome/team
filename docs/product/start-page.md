# The start page

What it does **now**, for the coordinator. Updated at each /close that changes it.

## Purpose
A look of a few seconds tells the coordinator whether something still needs them, and what comes
next. Everything else of the season is one tap further, on [Season](season.md).

## Behaviour
- **No club chosen:** only the choose step ([choose club and team](choose-club-and-team.md)) and the links.
- **A new season without a team:** the choose step, then "Open items", "Last" and "Next" (with the trainings only), then the links.
- **A team chosen**, from the top: the team header (team, season, "Change"); "Open items"; "Last"; "Next"; the links "Season · Attendance · Team · Players · Settings".
- **"Open items"**: what still needs the coordinator, one row for each kind, in this order; a tap goes to the place to act:
  1. "The last reading of the game list was held back." (a held-back reading waits) and "The games may be out of date since Thu 22 Oct." (3 nights without a good reading): to the "Game list" card on "Season" ([read the game list](read-the-game-list.md)).
  2. "4 changes not seen" ("1 change not seen"): to the [list of changes](list-of-changes.md).
  3. "Training day Thu 22 Oct not opened" for one, "3 training days not opened" for more: to the oldest of them, which shows "Not opened" with "Open training".
  4. "Training Thu 22 Oct not checked" for one, "2 trainings not checked" for more: to the oldest of them ([trainings and attendance](trainings-and-attendance.md), "Checked").
  Without open items: "Nothing open."
- **Which dates count:** only dates before today (Dutch date); today's training is in "Next". A cancelled training or training day is never an open item. The open items cover the current season and, in July and August, also the season before, so the record of June stays in view after 1 July. In those two months the number of training days not opened can therefore be larger than the line on "Attendance", which counts one season.
- **"Last"**: the newest training before today that is not cancelled, in the same seasons as the open items, also when it is checked. Its date shows above the row ("Thu 22 Oct"), and the row as on "Season" ("Training · 19:45 · 2 absent · Checked"). A tap opens it, so a late correction is one tap. With no training before today, "Last" is not shown.
- **"Next"**: the games of the team and the training days and trainings from today until the Sunday of next week (on a Sunday: today and the next week), grouped by date, each row as on "Season". Possible duty games are not in it. Cancelled trainings show, as "Cancelled" (the switch "Show cancelled" is a setting of "Season" only). A cancelled training does not count as something in the two weeks: when they have nothing else, "Next" also shows the first coming event of the season after them that is not cancelled; when there is none, "No coming events."
- A tap on a training in "Last" or "Next", or on a training item, opens the training; "Back" there returns to the start page.

## Errors and limits
- The start page cannot be loaded: "The start page could not be loaded. Load the page again."
- "Next" covers only the current season: in the last days of June its two weeks can reach into the new season, whose events do not show (an idea in `docs/ideas.md`).
- The last game is not on the start page; it comes with duties.

## Interface
- Screen: the start page (`/`).
- Server address (needs a login): `GET /api/start`: the open items (each with its kind; dates and counts where they apply), the last training as an event, and the events of "Next" (each with its week and a game or a training row, as on `GET /api/season`).
- The rules are `team_core::open_items`, `last_training`, `next_events` and `open_item_seasons`; each open item comes from the rule of its source page (the "Game list" card, the list of changes, "Attendance").
