# Trainings and attendance

What it does **now**, for the coordinator. Updated at each /close that changes it.

## Purpose
The coordinator, who is also the coach, keeps who was at training and why a player was not, in
seconds on a phone in the hall. A **training** exists, and counts, only when the coordinator opens a
training day (suggested by a weekly training, see [weekly trainings and reasons](weekly-trainings-and-reasons.md))
or adds a training. Every player with a role on its date ([players and team](players-and-team.md))
is "there" at first; the coordinator only marks who is absent, with a reason, and then confirms
with "Checked" that the attendance is complete, so a training nobody looked at can stay in view (the
open items of the start page). The page
"Attendance" shows, for each player, how often the player was there and why not, for the current
or an earlier season. The list of training days and trainings is on [Season](season.md).

## Behaviour
### The page "Attendance"
- The start page links to "Attendance" (second in the line "Season · Attendance · Team · Players · Settings").
- The page shows the chosen season: the current one, or an earlier one from the choice "Season" (shown when there is more than one). The season is in the address (`/attendance?season=2025-2026`), so a reload keeps it. A change of season closes the open lists of missed trainings.
- From the top: the line of training days not opened, then "Totals" (see below).
- The training days and trainings themselves, with "Cancel" on a day that is not opened, "Show cancelled" and "Add training", are on [Season](season.md), with the games of the same weeks.

### The totals
- Under "Totals", one row for each player with at least one training that counts for that player, sorted by first name: the name with "of 10 trainings", and "9 there · 1 absent, valid · 0 absent, not valid". Hidden players have rows too.
- A training counts for a player when it exists (opened or added), is not cancelled, its date is today or earlier, and the player has a role on that date. So a player who joined later has no missed trainings before that date, a training next week with an absence noted in advance does not count yet, and a cancelled training counts again after "Undo cancel". An absence on a date outside the player's roles is kept but does not count. "Absent, valid" and "absent, not valid" follow the reason's setting of now: a reason changed between valid and not valid moves the counts at once. Nothing is counted on the side: the totals are worked out each time the page loads.
- Players who had a role earlier in the season and have none today or later show under "Left the team", with the totals of their trainings. A player after "Change role" is in the team, once. In an earlier season this is measured on its last day, 30 June.
- A tap on a player shows the trainings the player missed that count, newest first, with the date and the reason ("Tue 6 Oct · Ill"), or "No missed trainings."; a second tap closes the list.
- Above the totals: "3 training days not opened" ("1 training day not opened"), when training days before today were never opened. It links to the oldest of them, which shows "Not opened" with "Open training"; "Back" there returns to "Attendance". Today's training day is not in it (it is a coming day on "Season").
- "No trainings yet this season." when no training counts yet. "No player had a role at these trainings." when trainings count but no player had a role at any of them; in the current season with "Add players in Team." and a link.

### On the start page
- The [start page](start-page.md) shows the training days and trainings of this week and next week in "Next", the last training before today in "Last" (also when checked), and as open items the training days not opened ("Training day Thu 22 Oct not opened") and the trainings not checked ("Training Thu 22 Oct not checked"). A tap opens the training; "Back" there returns to the start page.

### A training
- The top shows the date and times ("Tue 20 Oct · 19:45–21:00") and "N absent", "All there" or "Cancelled".
- "Back" returns to the page the training was opened from: the start page ("Last", "Next" or an open item), "Attendance" (the line of training days not opened), else "Season" of the training's season. "Change" to another date keeps that.
- The players with a role on the date (also those who only train, and hidden players in the team) as large rows, sorted by first name, each "there" or the reason of the absence.
- A tap on a player opens the reasons (hidden reasons are not offered); a tap on a reason marks the player absent with it. A tap on an absent player shows the reasons with "There" on top; "There" marks the player there again, another reason changes the reason.
- Each tap is saved at once, with no save button. The row shows the new state at once, with "saving…" until the answer comes. When a tap cannot be saved (no network, server error, or no answer within 10 seconds), the row shows the stored state again and the page says "Not saved. Try again." A mark never looks saved when it was not.
- The taps for one player are sent one after the other, so a late answer never overwrites a newer tap. Between two screens, the last tap wins. When the phone comes back to the page, it shows the stored marks (unless a tap is still being sent).
- "Change" changes the date and times of the training; the attendance stays. "Cancel training" marks it as cancelled: it shows "Cancelled" with "Undo cancel", keeps its absences (which do not count while cancelled) and can still be marked. "Undo cancel" makes it count again with its absences. "Delete training" asks first ("Delete the training of Thu 22 Oct? Its attendance is removed.") and deletes the training with its attendance; on a training day the date shows "Not opened" again. Nothing else deletes a training.
- A date that is not opened (reached by its address) shows "Not opened" with "Open training".
- A training is independent once opened: a change or delete of a weekly training never changes it. A training on a date that is no longer a training day stays, as "extra".

### Checked
- A training of today or earlier that is not cancelled has the button "Checked": a tap saves at once, and the page then shows "Checked" with a check mark (a state, not a button) and "Undo check", which also saves at once. A training after today, or a cancelled one, has no "Checked" (an absence noted in advance does not make it complete; a cancelled training has no attendance).
- A save that fails shows "Not saved. Try again." and the stored state; a training deleted on another screen meanwhile shows "This training no longer exists. Load the page again."
- A check stays when the coordinator marks a player, changes the times or the date (also to a date after today: it then shows "Checked" with "Undo check"), cancels the training or undoes the cancel. "Delete training" removes it; opened again, the training is not checked.
- Two screens: a check and a mark at the same moment are both kept; "Checked" and "Undo check": the last one wins.
- A training of an earlier season can be checked and unchecked as well.
- On "Season" a checked training shows "· Checked" after its status.
- When checked trainings came (release of story C of start page and season), every training before that day was checked once.

### Earlier seasons
- Trainings and attendance of an earlier season can still be opened, marked, changed and cancelled, for example the last training of May found in July: choose the season on "Season" and tap the row. "Back", and the jump after "Delete training", on a training return to "Season" of that training's season.
- "Add training" is only for the current season, and "Change" cannot move a training to another season. So a forgotten extra training (not on a training day) of an earlier season cannot be entered.
- The choice offers the seasons with a role, a weekly training or a training, and the current season.

## Errors and limits
- "There is already a training on this date." – "Add training" or "Change" to a date that has a training.
- "This date is outside the season." – "Change" to a date outside the training's season, or "Add training" outside the current season.
- "The end time must be after the start time."
- "This date is not a training day." – "Open" of a date without a weekly training (only from an address).
- "This player is not in the team on that date." – a mark for a player with no role on the date (the page never offers it).
- "This reason is hidden." – a mark with a hidden reason (the page never offers it).
- "A training after today cannot be checked yet." and "A cancelled training cannot be checked." – a check the page never offers.
- "This date cannot be read." – an address with a date that is not a date.
- "This season cannot be read." – an address with a season that is not a season (`?season=abc`), with a button "Current season".
- "No player has a role on this date." – a training with no player in a role.
- "No reasons. Add one in Reasons." – all reasons hidden: only "There" is offered.
- A tap that reaches the server after its 10 seconds is saved, but shown as not saved until the page loads again.
- "Add training" is only for the current season. One training per date. Logs carry dates and player ids, never names. The data exists only in the tool. A nightly encrypted copy of the database is kept on the home storage (TrueNAS), not yet outside the house.

## Interface
- Screens: `/attendance` (with `?season=` for another season), `/training/[date]` (with `?from=start` or `?from=attendance` for "Back"), the list on `/season`; "Last", "Next", the open items and the link on the start page (`/`). The start page has the line "Season · Attendance · Team · Players · Settings"; "Attendance" has no links to the other pages.
- Server addresses (all need a login): `GET /api/attendance?season=` (the totals, "Left the team", the training days not opened and the seasons to choose from), `GET /api/season?season=` (the list, with the games; see [Season](season.md)), `GET /api/trainings/{date}` (one training with its players), `PUT /api/trainings/{date}/open`, `POST /api/trainings` ("Add training"), `PUT /api/trainings/{date}` ("Change"), `POST /api/trainings/{date}/cancel`, `POST /api/trainings/{date}/uncancel`, `DELETE /api/trainings/{date}`, `PUT` and `DELETE /api/trainings/{date}/absences/{player}` (mark absent, mark there), `PUT` and `DELETE /api/trainings/{date}/checked` ("Checked", "Undo check").
