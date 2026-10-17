# Weekly trainings and reasons

What it does **now**, for the coordinator. Updated at each /close that changes it.

## Purpose
The settings page tells the tool when the team trains and which reasons there are for an absence,
so that a training is ready to mark without setting it up each time. Trainings themselves, and
marking attendance, come with the next part of attendance at training.

## Behaviour
### Weekly trainings
- A **weekly training** is a day of the week with a start and an end time, in a window from a date until a date inside the season, for example "Tuesday 19:45–21:00 · Tue 1 Sept – Tue 25 May". In its window it suggests a training on each of its days: the **training days** of the season. A weekly training only suggests; a training exists only when the coordinator opens it.
- "Add weekly training": the day, the start and end time, and the window ("First training", "Last training"). When a day is chosen, the window is filled with the proposal: that day on or after 1 September until that day on or before 31 May (for example Tue 1 Sept 2026 – Tue 25 May 2027; Thu 3 Sept 2026 – Thu 27 May 2027). Both dates can be changed; the date fields stay inside the season.
- A team that trains twice a week has two weekly trainings. Two weekly trainings on the same day of the week may not have windows that overlap; on different days they may.
- "Change" changes everything of a weekly training (the day, the times and the window), for its whole window: this is how a mistake is corrected. When the day changes, a date of the window that is the old day's proposal moves to the new day's proposal; a typed date stays. A real change of schedule is the end of one weekly training (its window ends) and a new one (for example Thursday from Thu 7 Jan).
- "Delete" removes a weekly training, after a confirmation ("Delete the weekly training on Tuesday?").
- The list shows the weekly trainings of the current season, sorted by the start of the window.
- A new season (from 1 July) starts with no weekly trainings. (Proposing last season's weekly trainings comes with the next part of attendance at training.)

### Reasons
- The tool starts with these reasons, in this order: School, Ill, Injured (valid), Holiday, Work, No reason (not valid).
- "Add reason" adds a reason with a name and "Valid"; it comes last in the list.
- A reason can be renamed ("Save"), and changed between valid and not valid ("Valid"). A name typed but not saved is not saved by a tap on "Valid" or "Hidden".
- "Hidden": the reason is no longer offered for new marks; it stays in the list with "hidden" and can be shown again.
- Two reasons cannot have the same name; upper and lower case and outer spaces do not count.

## Errors and limits
- "The end time must be after the start time."
- "The last training date must be after the first."
- "This date is outside the season." – a window outside the season.
- "There is already a weekly training on Tuesday in these dates." – an overlap on the same day.
- "A name is needed." / "There is already a reason with this name." – reasons.
- Only the weekly trainings of the current season are shown and changed. Two saves at the same time run one after the other.
- Weekly trainings, training days and "today" are in Dutch local time, as the games.

## Interface
- Screen: `/settings`, linked from the start page.
- Server addresses (all need a login): `GET /api/weekly-trainings`, `GET /api/weekly-trainings/proposal?weekday=` (the proposed window, 1 = Monday), `POST /api/weekly-trainings`, `PUT /api/weekly-trainings/{id}`, `DELETE /api/weekly-trainings/{id}`; `GET /api/reasons`, `POST /api/reasons`, `PUT /api/reasons/{id}`.
