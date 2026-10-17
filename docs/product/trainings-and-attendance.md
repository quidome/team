# Trainings and attendance

What it does **now**, for the coordinator. Updated at each /close that changes it.

## Purpose
The coordinator, who is also the coach, keeps who was at training and why a player was not, in
seconds on a phone in the hall. A **training** exists, and counts, only when the coordinator opens a
training day (suggested by a weekly training, see [weekly trainings and reasons](weekly-trainings-and-reasons.md))
or adds a training. Every player with a role on its date ([players and team](players-and-team.md))
is "there" at first; the coordinator only marks who is absent, with a reason. The totals for each
player come with the next part of attendance at training.

## Behaviour
### The page "Attendance"
- The start page links to "Attendance" (first in the line "Attendance · Team · Players · Settings").
- The page lists every training day and every training of the current season, one row per date (a training on a training day is that day's row): "Coming" (today and later, nearest first) above "Past" (newest first). Each row shows the date, the start time and "Not opened", "All there", "N absent" or "Cancelled". A training on a date that is not a training day shows "extra".
- A tap on a row opens the training: a day that is not opened becomes a training (with the times of its weekly training at that moment, and every player there) and counts from then on. Opening the same day twice, or from two screens, gives one training.
- "Cancel" on a row that is not opened cancels that training day without opening it (for example a holiday).
- The switch "Show cancelled", on by default, hides cancelled rows when off. Each phone or computer keeps its own setting.
- "Add training" adds a training on a date of the season without a training (an extra training, also in the past, to enter the trainings since 1 September): the times are filled in from a weekly training on that day of the week, or typed.
- A season with no training days and no trainings says "No training days or trainings this season. Add a weekly training in Settings."

### The card on the start page
- Below the team header, one line: the next training of the season, today's training until midnight, else the next training day or training, also when cancelled. For example "Training · Tue 20 Oct · 19:45 · Not opened yet", "All there", "2 absent", or "Training · Tue 20 Oct · Cancelled".
- A tap opens the training, as a row of the list does. With no coming training day or training in the season, there is no card.

### A training
- The top shows the date and times ("Tue 20 Oct · 19:45–21:00") and "N absent", "All there" or "Cancelled".
- The players with a role on the date (also those who only train, and hidden players in the team) as large rows, sorted by first name, each "there" or the reason of the absence.
- A tap on a player opens the reasons (hidden reasons are not offered); a tap on a reason marks the player absent with it. A tap on an absent player shows the reasons with "There" on top; "There" marks the player there again, another reason changes the reason.
- Each tap is saved at once, with no save button. The row shows the new state at once, with "saving…" until the answer comes. When a tap cannot be saved (no network, server error, or no answer within 10 seconds), the row shows the stored state again and the page says "Not saved. Try again." A mark never looks saved when it was not.
- The taps for one player are sent one after the other, so a late answer never overwrites a newer tap. Between two screens, the last tap wins. When the phone comes back to the page, it shows the stored marks (unless a tap is still being sent).
- "Change" changes the date and times of the training; the attendance stays. "Cancel training" marks it as cancelled: it shows "Cancelled" with "Undo cancel", keeps its absences (which do not count while cancelled) and can still be marked. "Undo cancel" makes it count again with its absences. "Delete training" asks first ("Delete the training of Thu 22 Oct? Its attendance is removed.") and deletes the training with its attendance; on a training day the date shows "Not opened" again. Nothing else deletes a training.
- A date that is not opened (reached by its address) shows "Not opened" with "Open training".
- A training is independent once opened: a change or delete of a weekly training never changes it. A training on a date that is no longer a training day stays, as "extra".

### Earlier seasons
- Trainings and attendance of an earlier season can still be opened, marked, changed and cancelled, for example the last training of May found in July. For now such a training is opened by its address (for example `/training/2027-05-25`); the choice of season on the page comes with the totals.

## Errors and limits
- "There is already a training on this date." – "Add training" or "Change" to a date that has a training.
- "This date is outside the season." – "Change" to a date outside the training's season, or "Add training" outside the current season.
- "The end time must be after the start time."
- "This date is not a training day." – "Open" of a date without a weekly training (only from an address).
- "This player is not in the team on that date." – a mark for a player with no role on the date (the page never offers it).
- "This reason is hidden." – a mark with a hidden reason (the page never offers it).
- "This date cannot be read." – an address with a date that is not a date.
- "No player has a role on this date." – a training with no player in a role.
- "No reasons. Add one in Reasons." – all reasons hidden: only "There" is offered.
- A tap that reaches the server after its 10 seconds is saved, but shown as not saved until the page loads again.
- "Add training" is only for the current season. One training per date. Logs carry dates and player ids, never names. The data exists only in the tool, and there is no backup yet.

## Interface
- Screens: `/attendance`, `/training/[date]`; the card and the link on the start page (`/`).
- Server addresses (all need a login): `GET /api/trainings?season=` (the list), `GET /api/trainings/next` (the card), `GET /api/trainings/{date}` (one training with its players), `PUT /api/trainings/{date}/open`, `POST /api/trainings` ("Add training"), `PUT /api/trainings/{date}` ("Change"), `POST /api/trainings/{date}/cancel`, `POST /api/trainings/{date}/uncancel`, `DELETE /api/trainings/{date}`, `PUT` and `DELETE /api/trainings/{date}/absences/{player}` (mark absent, mark there).
