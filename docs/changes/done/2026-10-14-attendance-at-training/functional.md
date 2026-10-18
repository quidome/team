# Functional spec: Attendance at training

**Status**: done (approved; changed after the second accept and build review of story A, approved again 2026-10-16 after the sixth spec review)
**Change**: docs/changes/2026-10-14-attendance-at-training
**Date**: 2026-10-15, changed 2026-10-16

## Context
The tool is for one coordinator who runs one youth team in a club. Today it reads the club's
games from the federation's game list and shows the schedule and the changes (released as
v0.2.0). The coordinator still keeps the rest of the team's record in a Google Sheet. The product
is successful when the coordinator stops using the sheet.

The coordinator is also the coach of the team. Attendance at training tells the coach how much
effort and energy each player gives to the team. Today the coordinator marks attendance in the
sheet at each training, for every player who trains: 14 players, of whom 10 play games and 4 only
train. An absence comes with a reason, and the reason has weight: school is a valid reason to
miss a training; other reasons are less valid. (Coordinator, 2026-10-14.)

The team trains every Tuesday from 19:45 to 21:00, from the first Tuesday of September to the
last Tuesday of May. Some Tuesdays are skipped for holidays; now and then there is an extra
training (rare). Some players join later in the season; the trainings before they joined must
not count as missed. When the tool starts, the coordinator wants to enter the attendance of all
trainings since 1 September, from the sheet.

This change brings the players into the tool for the first time. It must be faster than the
sheet: in the hall, on a phone, with one hand (core value 1 of the vision: "attendance at
training takes seconds, on a phone, with one hand").

Words (`docs/language.md`): a **player** is a person of the club, in the team of a season through a
**role** (plays games or only trains);
**attendance** is whether a player was at a training, and if not, the **reason**; a reason is
valid or not valid.

## User stories

### Story 1: Keep the players and the team
**As a** coordinator, **I want to** keep the players and who is in the team when, **so that** I
can mark their attendance without typing names, and the totals are fair to players who join,
change role or leave during the season.

A player is the person: a name, with no dates. What has a start and an end is the player's role
in the team of a season: "plays games" or "only trains" (coordinator, 2026-10-15). Both roles come
to training. The roles of a player, one after the other, are the player's history. The
coordinator works on two pages (coordinator, 2026-10-16): **Players**, all players the coordinator
ever entered, and **Team**, the team of the current season.

#### Acceptance criteria
Players:
- [ ] The page "Players" lists all players, sorted by first name, each with "Plays games" or "Only trains" when the player is in the team today, else "Not in the team". "Add player" adds a player with a first name (required) and a last name (optional).
- [ ] A tap on a player opens the player: the name (change and save), the switch "Hidden", the history, and "Delete player".
- [ ] "Hidden" only keeps lists short; it is not a status of the player (coordinator, 2026-10-16). A hidden player is left out of three lists: the list on "Players" (unless the switch "Show hidden players" is on), the choices of "Add player" on the team page, and "Players of last season" (so "Add all" does not add it). Nothing else changes: the history, the team and the totals stay, and a hidden player under "Left the team" still has "Add to the team" on its own row. For example a player who left the club.
- [ ] The history lists the player's roles, newest first, for example "Plays games · from Sun 1 Nov" and "Only trains · Tue 1 Sep – Sat 31 Oct". A role of the current season has "Correct": the kind, the start date and the end date (empty while the role continues), filled in with the stored values. A correction applies to the whole role, with the same checks as a new role. For example a player added from today who joined on 1 September, or added as "Plays games" by mistake. Roles of an earlier season are shown but cannot be corrected.

Team:
- [ ] The page "Team" has a heading with the name of the team chosen from the game list for the season and the season, for example "U14-1 · 2026-2027"; before the team is chosen, "Team not chosen yet" with a link to choose it. Players can be added to the team before the team is chosen (coordinator, 2026-10-16, during /design of story A).
- [ ] The page "Team" shows, for the current season, the players in the team today or later in the season (sorted by first name, with "Plays games" or "Only trains"), then "Left the team" with the players who had a role earlier in the season and have none today or later in the season. A player after "Change role" stays in the team.
- [ ] "Add player" on the team page adds a player to the team: a player from the list (not hidden, not in the team; last season's team first), or a new player (a first name, and a last name if wanted), with a role and the date from which the player is in the team ("In the team from"). For "Add player" and "Add all", "In the team from" is filled with 1 September of the season, with a button "Today" next to the date field; any other date is typed or picked (coordinator, 2026-10-16, after the sixth spec review). So the first setup in October, with the trainings since 1 September, needs no typed date, and a newcomer in January takes one tap on "Today". "Add to the team" and "Change role" are filled with today.
- [ ] A player in the team whose last day in the team is today (a role that ends today, with no role after it) shows "Last day today" in the row, so a removal from tomorrow is visible at once. Not on the last day of the season (30 June), when every role ends.
- [ ] A tap on a player of the team gives "Change role" and "Remove from the team".
- [ ] "Change role" (the new role and "From", the first day of the new role, proposed today) acts on the role on that date: that role ends on the day before, and the new role starts on the date and keeps the planned end of the old role. So the history keeps both, for example "Only trains · Tue 1 Sep – Sat 31 Oct" and "Plays games · from Sun 1 Nov". A planned later role stays as it is.
- [ ] "Remove from the team" asks "Not in the team from" (the first day without the team), filled with today, with a button "Tomorrow" next to the date field, always (for a player whose last day is today). From that date the player has no roles in the season (coordinator, 2026-10-16): the role on the day before ends on that day, and every role that starts on the date or later is removed, also a planned "Change role". For example on Thu 15 Oct "Not in the team from" today: "Only trains · Tue 1 Sep – Wed 14 Oct", and the player is under "Left the team" at once with "Left · last day Wed 14 Oct". A player whose roles of the season all start on the date or later has no role in the season after it: not under "Left the team", and back in the choices of "Add player". So "Add all" in August, followed by "Remove from the team" from today for a player who did not come back, leaves nothing of that player in the season. When the removal also removes a role that starts after the date (for example a planned "Change role"), the tool first asks for a confirmation that names it: "This also removes "Plays games · from Sun 1 Nov"." A removal that only ends the current role is saved at once.
- [ ] After "Remove from the team", the player's row shows "Undo" until the coordinator leaves the team page. "Undo" puts back the player's roles exactly as they were before the removal. After that, "Correct" (end date empty) restores the role that was ended, and "Add to the team" a role that was removed; the history shows only the roles that are still there.
- [ ] A player under "Left the team" can be added again ("Add to the team", with a role and "In the team from", proposed today): a new role from the date of return; the time between does not count.
- [ ] A role ends at the latest at the end of its season (30 June). A new season starts with no players in the team. While no player of last season's team has a role in the season, the team page shows "Players of last season" (last season's team, not hidden) with "Add all". "Add all" asks "In the team from" (1 September of the season, with "Today", as for "Add player"); each player gets the same role as at the end of last season, from that date. A newcomer added first does not make the group go; a player of last season's team with a role in the season does. Players of last season are still first in the choices of "Add player".

Everywhere:
- [ ] A player shows at a training, and counts for it, only when the date of the training is inside one of the player's roles. So a player who joined in November has no missed trainings in September and October.
- [ ] Absences outside a player's roles (for example after the end of a role was moved earlier) are kept, but do not count; they count again when the dates are changed back.
- [ ] Players are shown by name only: first name and last name. Two players with the same first name are told apart by the last name.

#### Behaviour
At the start of the season the coordinator opens "Team", taps "Add player" and types the 14
players once, each with "Plays games" or "Only trains". For a player who joined later, the
coordinator sets the date the role started. When a player moves from "only trains" to "plays
games", the coordinator taps the player, "Change role", and the date. When a player leaves, the
coordinator taps the player, "Remove from the team", and the first day without the team (today,
or "Tomorrow" when the player still trains today). Next season, the coordinator taps "Add all" (or
adds players one by one) and removes those who did not come back ("Not in the team from" today:
nothing of them stays in the season). A removal by mistake is undone with "Undo". A player who
left the club is set to "Hidden" on the page "Players", so the lists stay short.

#### Errors and edge cases
- No first name: the player cannot be saved; the tool says "A first name is needed."
- The end of a role before its start: not saved ("The end must be after the start.").
- "Change role" on a date that is not after the start of the current role: not saved ("The change must be after the start of the role. To change the role from its start, use "Correct" in the history.").
- "Change role" on a date before a role that starts later (for example a role typed from Mon 2 Nov): not saved ("This player's role starts on Mon 2 Nov. To change it, use "Correct" in the history.").
- "Change role" or "Remove from the team" on a date on which the player has no role, and none later in the season: not saved ("This player is not in the team on that date.").
- "Change role" to the role the player already has on that date: not saved ("The player already has this role.").
- "Undo" after another screen changed the player's roles: not saved ("This player was changed in the meantime. Use "Correct" in the history.").
- A role removed by "Remove from the team" had absences (story B): they are kept and do not count, as for any date outside a role; "Undo" or a new role on those dates makes them count again.
- Two roles of one player that overlap in time: not saved ("This player already has a role on that date.").
- Two players with the same first and last name: allowed (the coordinator knows who is who), but the tool warns "There is already a player with this name."
- A player with no absences can be deleted at any time ("Delete player", after a confirmation), for example a player added by mistake. "There" is not stored for each player, so nothing is lost. A player with absences can only be removed from the team or hidden.
- A role starts (or its start is moved earlier, also with "Correct") while trainings on or after that date are already entered: "there" is the default, so the player now counts as there at those trainings. On save the tool says "This player now counts as there at N earlier trainings." and lists them, so the coordinator can mark absences.

### Story 2: Weekly trainings and trainings
**As a** coordinator, **I want** the tool to know when the team trains, **so that** today's
training is ready to mark without setting it up each time.

A **weekly training** (word for `/vision`, see Open questions) is a day of the week with a start
and an end time, in a window from a date until a date, for example "Tuesday 19:45–21:00, from Tue
1 Sep until Tue 25 May". In its window it suggests a training on each of its days: these dates
are the **training days** of the season. A weekly training only suggests: a training exists only
when the coordinator opens it (coordinator, 2026-10-16). Once opened, a training has its own date
and times, and a later change of the weekly training does not touch it.

#### Acceptance criteria
- [ ] The coordinator can add a weekly training: a day of the week, a start and an end time, and a window from a date until a date, inside the season. A team that trains twice a week has two weekly trainings.
- [ ] For a new weekly training the tool proposes the window from that day of the week on or after 1 September until that day of the week on or before 31 May. The coordinator can change both dates.
- [ ] The coordinator can change everything of a weekly training ("Change": the day, the times and the window), and delete it. A mistake is corrected by a change, for the whole window. A real change of schedule is the end of one weekly training (its window ends) and a new one (for example Thursday from Thu 7 Jan).
- [ ] A change or delete of a weekly training never changes or deletes an opened training: only the training days that are not opened follow it.
- [ ] The settings page lists the weekly trainings of the season, sorted by the start of the window, for example "Tuesday 19:45–21:00 · Tue 1 Sep – Tue 25 May".
- [ ] The start page shows a card "Training" for the next training: today's training until midnight (so attendance can still be entered at home after the training), else the next training day or training. A cancelled training shows on the card too, as "Training · Tue 20 Oct · Cancelled". For example "Training · Tue 20 Oct · 19:45".
- [ ] With no coming training day or training in the season, the start page shows no training card. The coordinator can still open trainings from the page "Attendance".
- [ ] The coordinator can open the next training at any time, also days before it, to note an absence in advance.
- [ ] Opening a training day makes the training exist, with the date and times of its weekly training at that moment and with every player there (coordinator, 2026-10-15). From then on it counts. A training day that the coordinator never opened does not exist and does not count anywhere.
- [ ] The page "Attendance" lists every training day and every training of the season: "Coming" (today and later, nearest first) above "Past" (newest first). Each shows "Not opened", "All there", "N absent" or "Cancelled". A date has at most one row: a training on a training day is that day's row. So the coordinator can enter past trainings (for example all trainings since 1 September) one after the other, and prepare coming ones: note an absence weeks ahead, or cancel a holiday break in advance.
- [ ] "Cancel training" marks a training as cancelled (coordinator, 2026-10-15), for example a holiday or a training cancelled at the last moment. A training day can be cancelled from the list without opening it. A cancelled training stays in the list, like a normal training, with the label "Cancelled". It does not count in the totals.
- [ ] The list of training days and trainings has a switch "Show cancelled", on by default. Off, the list hides cancelled trainings (coordinator, 2026-10-15).
- [ ] "Undo cancel" makes a cancelled training a normal training again. Absences noted before the cancel are kept while it is cancelled (not counted), and count again after the undo.
- [ ] "Add training" makes a training on any date of the season that has no training yet (an extra training), with the times of a weekly training on that day of the week, or times the coordinator types.
- [ ] "Change" on a training changes its date and its times; the attendance stays (coordinator, 2026-10-16). For example trainings opened while a weekly training said Wednesday or 19:00 by mistake.
- [ ] The coordinator can delete any training by hand ("Delete training", after a confirmation: "Delete the training of Thu 22 Oct? Its attendance is removed."), for example one opened or added on a wrong date (coordinator, 2026-10-16). Nothing else ever deletes a training. When the date is a training day, it shows as "Not opened" again. A holiday is cancelled, not deleted: a deleted training day counts as not opened.
- [ ] Trainings and attendance of an earlier season can still be opened, entered, changed and cancelled (coordinator, 2026-10-16), for example the last training of May found in July. Its weekly trainings and roles are fixed. "Add training" is only for the current season, so an extra training of an earlier season cannot be added (2026-10-17, story C).
- [ ] Weekly trainings, training days and "today" are in Dutch local time, the same as the games.
- [ ] A new season (from 1 July, as for the games) has no weekly trainings. The settings page proposes the weekly trainings of last season with new proposed windows (story B of the epic builds this).

#### Behaviour
On Tuesday at 19:40 the coordinator opens the tool. The start page shows "Training · Tue 20 Oct ·
19:45" with a short line: "Not opened yet", "All there" or "2 absent". One tap opens the training. On Friday
the coordinator reads in the chat that a player cannot come next Tuesday, opens the card
"Training · Tue 27 Oct" and marks that player.

Setting up in October: the coordinator adds the weekly training "Tuesday 19:45–21:00" with the
proposed window Tue 1 Sep – Tue 25 May, and then enters the trainings since 1 September. A
Wednesday typed by mistake is corrected with "Change"; the list of training days follows at once.

#### Errors and edge cases
- The training is cancelled after the coordinator already opened it (for example to note an absence in advance): the coordinator taps "Cancel training"; the noted absence is kept, and counts again if the cancel is undone.
- The coordinator opens a past training day by mistake while it was a holiday: it now counts with everyone there; "Cancel training" makes it not count. ("Delete training" would make it "Not opened" again, so it would count as not opened above the totals.)
- The end of a window before its start: the tool does not save it and says "The last training date must be after the first."
- A window outside the season: not saved ("This date is outside the season.").
- Two weekly trainings on the same day of the week with windows that overlap: not saved ("There is already a weekly training on Tuesday in these dates."). Weekly trainings on different days may overlap.
- A start time after the end time: the tool does not save it ("The end time must be after the start time.").
- "Add training", or "Change" of a training, to a date that already has a training: not saved ("There is already a training on this date."). "Change" to a date outside the season: not saved ("This date is outside the season.").
- A training on a date before every player's role (for example a window moved earlier): the training screen says "No player has a role on this date."
- A weekly training is changed (or deleted) while some of its training days are opened or cancelled: those trainings stay, with their times and attendance. A training on a date that is no longer a training day shows in the list like an extra training.
- A weekly training is changed so that a date with an extra training becomes a training day (for example extra trainings on the real Tuesdays, while the weekly training said Wednesday by mistake): the date has one row, the training, with its attendance kept.
- The times of a weekly training change after a coming training was opened (for example to note an absence in advance): that training keeps its times.

### Story 3: Mark attendance at a training
**As a** coordinator, **I want to** mark in seconds who is not at training and why, **so that**
I can keep attendance in the hall with one hand.

#### Acceptance criteria
- [ ] A training shows all players who are in the team on its date (also those who only train), sorted by first name. Each player shows as "there" at first.
- [ ] When the coordinator opens a training, it is stored at once with every player there. No extra tap is needed for a training where everyone came.
- [ ] A cancelled training opens with "Cancelled" at the top and "Undo cancel"; its players and noted absences show, and can still be changed.
- [ ] A tap on a player who is there opens the list of reasons. A tap on a reason marks the player as absent with that reason. The row then shows the reason.
- [ ] A tap on an absent player shows the list of reasons again, with "There" on top; a tap on "There" marks the player as there again; a tap on another reason changes the reason.
- [ ] Each tap is saved at once; there is no save button. The training shows "N absent" or "All there".
- [ ] When a tap cannot be saved (no network, server error), the row shows the old state again and the tool says "Not saved. Try again." It never shows a mark as saved when it was not.
- [ ] A training in the past can be opened from the page "Attendance" and marked or corrected in the same way, also weeks later.
- [ ] Which players show follows story 1: the players with a role on the training's date.
- [ ] The rows are large enough to tap with a thumb on a phone (390 pixels wide), and the page has no sideways scroll.

#### Behaviour
The players walk in. The coordinator opens today's training: 14 names, all "there", and "All
there" at the top. Two players are missing: a tap on the first, "Ill"; a tap on the second, "No
reason". The top says "2 absent". Done. On a Tuesday where everyone came, opening the training is
enough.

Entering the past: the coordinator opens "Attendance", which lists Tue 1 Sep (the first training
date in the example) up to today. For each Tuesday: open it and tap the absent players, or tap
"Cancel training" for a holiday.

#### Errors and edge cases
- A player arrives late after being marked absent: a tap on the player, "There".
- The same training is open on the phone and on a computer: the last tap wins. When the page is opened or shown again, it shows the stored marks.

### Story 4: Reasons
**As a** coordinator, **I want** a short list of reasons, each valid or not valid, **so that** I
can mark an absence with one tap and the totals show which absences matter.

#### Acceptance criteria
- [ ] The tool starts with these reasons, in this order: School (valid), Ill (valid), Injured (valid), Holiday (not valid), Work (not valid), No reason (not valid).
- [ ] The coordinator can add a reason with a name and "valid" or "not valid". It comes last in the list.
- [ ] The coordinator can rename a reason, and change it between valid and not valid. Earlier absences with that reason follow the change: the totals always use the current setting.
- [ ] The coordinator can hide a reason. A hidden reason is not offered for new marks, but earlier absences keep it and still show it. The coordinator can show it again.
- [ ] Two reasons cannot have the same name ("There is already a reason with this name.").

#### Behaviour
A player misses training for a family party. The coordinator adds the reason "Family" (not
valid) once, in the list of reasons, and uses it from then on.

#### Errors and edge cases
- A reason with no name cannot be saved ("A name is needed.").
- All reasons hidden: the training shows only "There", and the tool says "No reasons. Add one in Reasons."

### Story 5: Season totals
**As a** coach, **I want to** see for each player how often they were at training this season,
and why they were not, **so that** I understand the effort each player gives to the team.

#### Acceptance criteria
- [ ] The page "Attendance" shows, for the current season, one row for each player who could be at one or more trainings of the season: the name, "there" (count), "absent, valid" (count), "absent, not valid" (count), and "of N trainings".
- [ ] Only trainings up to and including today count. A training in the future with an absence noted in advance does not count yet.
- [ ] For each player, only the trainings (opened or added) at which the player had a role count (story 1). A player who joined later has no missed trainings before that date.
- [ ] Players who had a role earlier in the season and have none today or later in the season show in a separate group "Left the team", with the totals of their trainings.
- [ ] The page also lists the training days and trainings of the season (story 2), "Coming" above "Past"; a tap opens the training (story 3).
- [ ] The season runs from 1 July to 30 June (Dutch time), the same as for the games. An earlier season can be chosen on the same page.
- [ ] With no trainings in the season yet: "No trainings yet this season."
- [ ] When past training days are still "Not opened", the page says so above the totals: "3 training days not opened", with a link to the first of them. So the coach sees when the totals are not complete.

#### Behaviour
After a few weeks the coach opens "Attendance": "Avery – 9 there, 1 absent valid, 0 absent not
valid, of 10 trainings". The rows are sorted by first name. A tap on a player shows the trainings
the player missed, with the date and the reason.

#### Errors and edge cases
- A reason changed from valid to not valid: the counts change with it (story 4).
- A training was cancelled: it no longer counts.

## Interface
Screens (phone first, also usable on a computer):
- **Start page** (`/`): below the team header, a compact card "Training · Tue 20 Oct · 19:45" with "Not opened yet", "All there", "N absent" or "Cancelled". A tap opens the training (and so makes it exist, story 2). It shows only when the season has a coming training day or training, and takes one line so the next games stay high on the first screen.
- **Training** (for one date): the date and time at the top, "N absent", "All there" or "Cancelled"; the players as large rows (name, and the reason when absent). A tap on a row opens the list of reasons with "There" on top. At the bottom: "Change" (date and times), "Cancel training" or "Undo cancel", and "Delete training".
- **Attendance**: the season (with a choice of earlier seasons), the totals for each player, the group "Left the team", and the list of training days and trainings of the season, "Coming" above "Past" ("Not opened", "All there", "N absent", "Cancelled"), with "Cancel training" on a day that is not opened and the switch "Show cancelled", and above the totals the number of training days not opened. "Add training" (with a date; only in the current season). A tap on a player shows the trainings that player missed.
- **Players**: all players, sorted by first name, with the role today or "Not in the team", "Add player" and the switch "Show hidden players". A tap opens the player: first name, last name, "Hidden", the history (the roles, newest first, "Correct" for a role of the current season), and "Delete player" when the player has no absences.
- **Team** (current season): the heading with the team's name and the season, or "Team not chosen yet"; the players in the team with "Plays games" or "Only trains", then "Left the team"; "Add player" (from the list or new, with a role and a date); while no player of last season's team has a role in the season, those players with "Add all" (with "In the team from"). A tap on a player gives "Change role" ("From") and "Remove from the team" ("Not in the team from", with "Tomorrow"), or "Add to the team" under "Left the team"; after a removal, "Undo" on the row. "In the team from" of "Add player" and "Add all" is filled with 1 September, with "Today"; the other dates with today, and "Not in the team from" with "Tomorrow" next to it.
- **Weekly trainings and reasons** (settings): the weekly trainings of the season (day, start and end time, window), "Add weekly training", "Change" and "Delete" for each, and the list of reasons with valid / not valid, "Add reason", rename, hide.
- Navigation: the start page links to "Attendance", "Team", "Players" and the settings (coordinator, 2026-10-17, story C; until the start page becomes one page of past and coming events, `docs/ideas.md`). "Attendance" has no links to the other pages.

Example of the training screen:

```
Tue 20 Oct · 19:45–21:00 · 2 absent
  Avery Jansen            there
  Blake de Vries          Ill
  Casey                   there
  Dana Smit               No reason
  …
  [ Change ]  [ Cancel training ]  [ Delete training ]
```

## Consistency
- Dates and times are Dutch local time, and the season starts on 1 July, as for the games (`docs/product/schedule.md`, `docs/product/choose-club-and-team.md`).
- Dates are written as on the schedule ("Tue 20 Oct"; with the year where the season is not clear).
- The layout follows the start page and the list of changes: cards, the same colours and fonts, phone first, no sideways scroll at 320 pixels.
- Only the coordinator can see and change this (login, as for all of the tool).
- Like the list of changes, a tap acts at once and the screen shows the result; errors are short sentences.

## What changes for existing users
No breaking changes. The start page gets one more card (when the season has a coming training day
or training). Until the coordinator adds players and weekly trainings, nothing else changes. The schedule,
the "Game list" card and the list of changes stay as they are.

The coordinator types in the players once (no import from the sheet).

## Warnings from architect and engineer
- **[Steve]** Players, trainings and attendance are the first data that only exist in the tool: unlike games, they cannot be read again from a source. The backup of the database must cover them, and every mistake must be correctable in the app.
- **[Steve]** (2026-10-16) A change of a weekly training can make a date that was a training day stop being one, also for trainings that were already opened or cancelled. The result for the user must be the one in story 2 (they stay), never a training that disappears or loses its attendance.
- **[Steve]** (2026-10-16) Story A of the epic already stores the training days in another form (sets of days, valid from a date, and a first and last date of the season). It is not merged or released, so its stored form can still change without loss.
- **[Bruce]** (2026-10-16) The training days are worked out from the weekly trainings each time, not stored. Two weekly trainings on the same day must never overlap, also when two screens save at the same time; opening a training day twice, or from two screens, must still make one training.
- **[Bruce]** (2026-10-16, second accept review) "Remove from the team" ends one role and can remove others in one save, and "Undo" puts several roles back: each is saved whole or not at all, under the same rule as "Change role". "Undo" must notice a change from another screen and never overwrite it.
- **[Steve]** (2026-10-16, second accept review) With today as the proposal everywhere, the "first training date" is no longer used by story 1; story B no longer completes that rule ("until a training of the season is opened"). The epic's technical spec must drop it at design.
- **[Bruce]** (2026-10-16) "Change role" changes two roles at once (the end of one, the start of the next): both or neither are saved. "Change role", "Remove from the team" and "Correct" on two screens: the last save wins, and the check for overlap must use the stored roles.
- **[Steve]** (2026-10-16) `docs/language.md` said a player is "a member of the team"; with the page "Players" a player can also be outside the team. Settled by `/vision` on 2026-10-16 (Open questions).
- **[Steve]** Training days, "today" and the season must use Dutch local time, the same rules as the games, also on the nights the clock changes.
- **[Bruce]** "Saved at once" on a phone in a sports hall: the network can be slow or gone. A tap that was not saved must never look saved, and a late answer of an earlier tap must not overwrite a newer tap.
- **[Bruce]** The same training open on two screens: the last tap wins; a screen that is shown again must show the stored marks.
- **[Bruce]** The totals depend on dates that change: the day a player was added or left, a training added or cancelled later, a reason changed between valid and not valid. Each total must follow from the stored data at the moment it is shown, not from counts kept on the side.
- **[Bruce]** Opening a training stores it (coordinator, 2026-10-15). So opening is a change, not only a look: two screens that open the same day at the same time must make one training, not two; and opening a training in the past, only to look at it, makes it count as "all there". The coordinator must see clearly that a day is "Not opened" before opening it.
- **[Bruce]** A cancelled training keeps its absences (not counted). Cancel and undo must never lose or double them, and the totals must leave cancelled trainings out every time.
- **[Bruce]** Entering all trainings since 1 September means many taps in one sitting. Each must be quick, and the list must show at once which days are still "Not opened".

## Not in scope
- Attendance at games (the next change).
- Jersey number, birthday and association ID of a player.
- Reading the players from a sheet or another source.
- Totals for recent weeks, trends, or charts.
- Messages to the team or to parents about attendance.
- Being late, or leaving early: a player is there or not there.
- The place of a training.
- More teams or more coordinators.
- A start page with one list of coming trainings, games and duties, with a filter for cancelled items (an idea for after duties, `docs/ideas.md`).

## Open questions
- Settled 2026-10-15 (Vision, with the coordinator): **player** now "has a first name, and usually a last name" (`docs/language.md`); 3 players in the coordinator's sheet have no last name.
- Settled 2026-10-16 (Vision, with the coordinator): **team** is the youth team of one season, the team chosen from the game list for that season; the **club** has the players and the teams; a **player** is a person of the club; the **coordinator** runs one team in each season (`docs/language.md`).
- Settled 2026-10-16 (Vision, with the coordinator): **player** is a person the coordinator keeps, in the team through a **role**; new words **role**, **hidden**, **weekly training** and **training day**; **training** exists, and counts, only when opened or added (`docs/language.md`).

## Spec review
Reviewed 2026-10-15 by a fresh agent (`/review spec`). Read: this spec, `idea.md`, `change.md`,
`docs/vision.md`, `docs/language.md`, `docs/product/schedule.md`, `choose-club-and-team.md`,
`list-of-changes.md`. Each note is checked by reading these documents (trace through the spec
text); nothing was run.

| Story | What the user is doing here | Verdict | Note |
|---|---|---|---|
| 1 Keep the players | Typing the 14 players once at the start of the season, and keeping joins and leaves right, so that totals are fair to a player who joined late. | serves – note | See S1, S2. The choice to keep players apart from the season's team differs from `idea.md` ("the players belong to the team of the current season's choice") and is the better fit: the team gets a new name each season, the players stay. |
| 2 Training days and trainings | Having this Tuesday's training ready without setup, skipping holidays, and entering all trainings since 1 September from the sheet. | serves – note | See S3, S4, S5. "Opening makes it exist" fits the hall (an all-there Tuesday is one tap) but makes a mistake the main risk; the spec's own fix for it (S3) loses the day. |
| 3 Mark attendance at a training | In the hall, phone in one hand, tapping the two players who are missing and a reason for each. | serves | Two taps per absence, no save button, a failed save never looks saved: this is core value 1 as written. |
| 4 Reasons | Saying why a player was absent with one tap, with a weight (valid or not) the coach chose. | serves | Renaming and changing valid/not valid follow through to past absences, which matches "totals follow from the stored data". |
| 5 Season totals | As coach, seeing who comes and who misses without a valid reason, to judge effort. | serves – note | See S6. |

No story conflicts with the vision. `/vision` must still settle the open question on **player**
(last name optional) before Design: the word list now says "Has a first and a last name", and
story 1 makes it optional. "Coach" (story 5) is not in the word list; the vision uses it the same
way (journey 2), so it reads clearly, but `/vision` may want to name it.

Notes:
- **S1 – story 1, edge cases.** "A player cannot be deleted." followed by "… can be deleted" contradicts itself on first reading. Suggest: "A player with attendance cannot be deleted, only marked as left." (Trace of the text.)
- **S2 – story 1, the twin of the "in the team since" rule.** Moving "in the team since" later asks to remove marks; moving the date a player left *earlier*, past trainings where the player has marks, is not described (kept and hidden? removed? asked?). Also, because "there" is the default, a player added with the proposed date (first training of the season) after past trainings were entered counts as "there" at all of them. That is likely right for the normal order (players first, then the past), but Design should know presence is derived, not stored. (Trace: story 1 criteria 3–4, story 3 criterion 1.)
- **S3 – story 2, "Delete training" on a training day.** The fix for "opened by mistake" (story 2 edge case, and Bruce's warning about opening a past day only to look) sets the day to "No training". That day then no longer shows as "Not opened", so a real training that was opened by mistake *before* its attendance was entered drops out of the list the coordinator works through, and out of the totals, with no sign. For the coordinator entering the past from the sheet, the expected result of undoing a mistaken open is "Not opened" again. Suggest: on a training day, deleting returns the day to "Not opened" (the coordinator taps "No training" separately for a holiday), or the delete offers both. (Trace: story 2 criteria 6, 8, 10; edge cases 1–2.)
- **S4 – story 2, which training the start-page card shows.** "today's training until its end time, else the next training day" does not say that it skips a day marked "No training" (a holiday week marked in advance), nor whether an extra training can be the next training. Both are cases the coordinator meets in the first months. (Trace: story 2 criteria 3, 8, 9.)
- **S5 – story 2, after the training.** The card moves to next week at 21:00. A coach who marks attendance after the training, at home, must go through "Attendance". This may be intended (the hall is the main case), but it is a choice worth making on purpose, e.g. "today's training until the end of the day". (Trace: story 2 criterion 3, idea "User view".)
- **S6 – story 5, days never opened.** A Tuesday where everyone came but the coordinator forgot to open the training does not count, and the totals give no sign of it: "of N trainings" is quietly lower for everyone. Core value 1 says the stored data must be right. Suggest the totals show "N training days not opened" (with a link to the list), so the coach sees that the totals are incomplete. (Trace: story 2 criterion 6, story 5 criteria 1–3.)
- **Not a finding.** "Plays games" / "Only trains" is stored but used by nothing in this change. It is cheap, it is in the word list, and attendance at games (the next change) needs it, so it does not go against "small and correct beats complete".

### Second review (2026-10-15)
A second fresh agent (`/review spec`) read the spec as changed after the answers below. Read:
this spec, `idea.md`, `change.md`, `docs/vision.md`, `docs/language.md` (with its uncommitted
change), and all of `docs/product/`. Each note is checked by a trace through these documents;
nothing was run. The arguments held only the mode and the change folder.

| Story | What the user is doing here | Verdict | Note |
|---|---|---|---|
| 1 Keep the players | Typing the players once, and keeping joins, leaves and mistakes right, so that the totals are fair. | serves – note | See S7, S8. |
| 2 Training days and trainings | Having the next training ready without setup, cancelling holidays, entering the past from the sheet. | serves – note | See S9, S10. The S3 answer (cancel, never remove; keep absences) fits the hall and makes every mistake correctable. |
| 3 Mark attendance at a training | In the hall, phone in one hand, two taps per absent player. | serves | See S11 (outside this spec). |
| 4 Reasons | One tap for a reason, with a weight the coach chose. | serves | |
| 5 Season totals | As coach, seeing who comes and who misses without a valid reason. | serves | "N training days not opened" (S6) makes incomplete totals visible. |

No story conflicts with the vision. The word **player** is settled in `docs/language.md`; the open
question on **training** is still for `/vision` (see S10).

Notes:
- **S7 – story 1, "Delete player" is stricter than the mistake it is for.** A player can be deleted only with no absences *and* in no entered training. Because "there" is the default (edge case 5), a player added by mistake in the middle of the season (a duplicate, a typo) is at once "in" every entered training, so can never be deleted: it can only be marked as left, and then shows forever in "Left the team" with "there" counts for trainings it never had. Its presence is derived, so deleting it loses nothing stored. Suggest: a player with no absences can be deleted. The Interface also says another rule ("when the player has no attendance"); the two must agree. (Trace: story 1 edge cases 4–5, story 5 criterion 4, Interface "Players".)
- **S8 – story 1, removing marks when a date moves.** Moving "in the team since" later, or the left date earlier, asks to *remove* the absences outside the new dates. But criterion 4 already says that a player does not show or count outside those dates, so the marks would not count anyway. Removing them makes "undo" of a left date, or a correction of a wrong date, lose data, which is the opposite of the S3 answer for cancelled trainings (absences kept, count again after undo) and of Steve's warning ("every mistake must be correctable"). Suggest: keep the marks and ignore them outside the player's dates, with no question. The two questions also use two words ("absences", "marks") for the same thing. (Trace: story 1 criteria 3–4, edge case 2; story 2 criterion 10.)
- **S9 – story 2, only the next training is reachable ahead.** The list goes "up to today" and the card shows only the next training. So the coordinator cannot note an absence two weeks ahead ("Avery is away the next three Tuesdays", a school camp), cannot cancel the autumn or Christmas break in advance, and cannot reach a future extra training to delete it unless it is the next one. In a sheet they can be filled in ahead (not checked how the coordinator does it now). The idea's own case ("a player said so in the chat") often reaches further than one week. Suggest: the list also shows the coming training days of the season (or the next few weeks), so they can be opened or cancelled; or say on purpose that only the next training can be prepared. (Trace: story 2 criteria 3, 5, 7, 11–12; idea "Attendance".)
- **S10 – story 2, "training day" has two meanings.** It means a day of the week in the settings ("training days of the week", "Tuesday 19:45–21:00") and a dated day in the list ("a training day that the coordinator never opened", "3 training days not opened"). With edge case 4 (the days change in the middle of the season) this matters for the user: the list and the count of days not opened must follow the days that were valid on each date, or a change from Tuesday to Thursday in January shows the Thursdays of September as "Not opened" and drops the Tuesdays. "From then on" says the intent; Design should see it as a rule of the count. `/vision` may want a word for the dated day next to **training**. (Trace: story 2 criteria 1, 7, edge case 4; story 5 criterion 8.)
- **S11 – outside this spec: the login in the hall.** A login lasts 8 hours (`docs/product/login.md`), and trainings are a week apart, so each Tuesday in the hall starts with a login before the first tap. With a passkey this may take seconds, but core value 1 measures the whole path, not only the taps. Not a change to this spec; worth a line in `docs/ideas.md` or a check in real use. (Trace: `login.md` "A login lasts 8 hours"; story 2 behaviour.)

### Third review (2026-10-16)
A third fresh agent (`/review spec`) read the spec as changed after the accept review of story A
(A1, A2), with its uncommitted diff against the approved version. Read: this spec, `idea.md`,
`change.md`, `docs/vision.md`, `docs/language.md`, `docs/product/schedule.md`. Not read: any
`technical.md`. Each note is checked by a trace through these documents; nothing was run. The
arguments held only the mode and the change folder. Stories 3–5 did not change; they are checked
again only for effects of the changes.

| Story | What the user is doing here | Verdict | Note |
|---|---|---|---|
| 1 Keep the players | Typing the players once, then fixing a role typed wrong (wrong start date, wrong kind) so the totals are fair. | serves – note | "Change" closes the gap A2 found: a proposed start that was not true can now be moved. See S14, S15. |
| 2 Training days and trainings | Setting the training days once, fixing a mistake in them later for the whole season, or recording a real change of schedule from a date. | serves – note | "Applies from" with a preview line and the full list of saves makes every mistake visible and correctable, which A1 lacked. See S12, S13, S16. |
| 3 Mark attendance at a training | In the hall, two taps per absent player. | serves | No change. |
| 4 Reasons | One tap for a reason with a weight. | serves | No change. |
| 5 Season totals | As coach, seeing who comes and who misses. | serves | A correction of the training days moves which dates are "Not opened"; the count above the totals follows it (story 2 edge case), so incomplete totals stay visible. |

No story conflicts with the vision. Both changes serve Steve's warning "every mistake must be
correctable in the app" and core value 1 ("the stored data is the truth and must be right").

Notes:
- **S12 – story 2, a correction erases a later real change.** "Every earlier save that applied from that date or later is replaced." So the coordinator who in January fixes a wrong time of the autumn (a save from Tue 1 Sep) also removes a change of schedule saved from Tue 5 Jan, and must type it again. The rule is simple and the user can recover, but only if the preview line names *every* save it replaces, not only one as in the example. Suggest: the criterion says the line lists all saves that the save replaces ("This replaces Wednesday 19:00–21:00 from Tue 1 Sep and Thursday 19:00–20:30 from Tue 5 Jan."). (Trace: story 2 criteria "Applies from" and the preview line.)
- **S13 – story 2, an extra training on a date that becomes a training day.** The spec covers one direction of a correction (a date that stops being a training day keeps its training, as an extra training). The twin is not covered: while the training days are wrong (Wednesday typed for Tuesday), the coordinator who enters the past may use "Add training" on the real Tuesdays. After the correction those Tuesdays are training days too. Does the list then show one training or two on that date, and can the former extra training still be deleted ("a training on a training day cannot be deleted")? For the user the expected result is one training on that date, with its attendance kept. Suggest one sentence in the edge case. (Trace: story 2 criteria "Add training", "Only an extra training can be deleted", edge case "corrects a mistake".)
- **S14 – story 1, "Change" only in the current season.** The players page can show an earlier season ("for the chosen season") and "Attendance" shows its totals, but a role of an earlier season cannot be corrected. A wrong start found after 30 June stays wrong in last season's totals, and "Add all" copies last season's kind into the new season (where it can be changed). Low weight in the first season; worth saying on purpose, as the spec does for the screen of a real role change. (Trace: story 1 criteria "Change", "Add all", players page; story 5 "An earlier season can be chosen".)
- **S15 – story 1, the real change of role has a way, but no name.** A player who moves from "only trains" to "plays games" in November can be recorded now by ending the role ("Left the team", 31 Oct) and giving a new role from 1 Nov. The spec allows this but does not say it, and the button is called "Left the team" for a player who did not leave. With "Change" right next to the role, the coordinator will likely use it, and the history (only trained in September and October) is lost. Harmless in this change (nothing uses the kind yet), but attendance at games will read it. Suggest: the behaviour says how a real change is entered for now, or a line in `docs/ideas.md` keeps the later screen from being forgotten. (Trace: story 1 note under the criteria, behaviour, Interface "Players".)
- **S16 – story 2, what "Applies from" covers.** The same settings form holds the training days, their times, and the first and last training date of the season. The spec does not say whether a change of the first or last training date also goes through "Applies from" (it reads as if not: those are dates of the season, not of a period), nor what moving the first training date later does to the save that applied from the old first date. Also open: whether a coming training that is already opened (an absence noted in advance) keeps its time when the times of its training day change from today; the card and the training screen show that time. Small, but Design must choose, and the coordinator sees the result. (Trace: story 2 criteria 1, "Applies from", edge case "changes … from today"; Interface "Training days and reasons".)

### Fourth review (2026-10-16)
A fourth fresh agent (`/review spec`) read the spec as changed after the third review (weekly
trainings, "Change" of a role, roles of earlier seasons), with its uncommitted diff against the
approved version. Read: this spec, `idea.md`, `change.md`, `docs/vision.md`, `docs/language.md`,
`docs/ideas.md`, all of `docs/product/`. Not read: any `technical.md`. Each note is checked by a
trace through these documents; the weekdays of the example dates were checked with `date` (Tue 1
Sep 2026, Tue 25 May 2027, Tue 20 Oct, Tue 5 Jan: all right). The arguments held only the mode and
the change folder. Stories 3 and 4 did not change; they are checked only for effects of the
changes.

| Story | What the user is doing here | Verdict | Note |
|---|---|---|---|
| 1 Keep the players | Typing the players once, fixing a role typed wrong with "Change", and recording a real change of role through "Left the team" and back. | serves – note | "Change" with its form sentence answers A2 and S15 well. The path the form sentence prescribes fails with the proposed dates, and its button has no place in the Interface. See S17, S18, S19. |
| 2 Weekly trainings and trainings | Setting up the weekly training once, then entering the trainings since 1 September from the sheet, and fixing a mistake in the setup. | serves – note | "A weekly training only suggests" is simpler than "Applies from" and keeps every opened training safe: no correction can lose attendance. The other side of that rule: a mistake that already reached opened trainings cannot be corrected without losing their attendance. See S20, S21, S22. |
| 3 Mark attendance at a training | In the hall, two taps per absent player. | serves | No change. "Delete training" now sits at the bottom of every training next to "Cancel training"; the confirmation guards it. |
| 4 Reasons | One tap for a reason with a weight. | serves | No change. |
| 5 Season totals | As coach, seeing who comes and who misses. | serves – note | The real change of role now goes through "Left the team", so a player in the team can have an ended role. See S18. |

No story conflicts with the vision. "A weekly training only suggests; any training can be deleted
by hand" fits core value 1 and Steve's warning that every mistake must be correctable. The new
word **weekly training** (and **training day** with one meaning) is used throughout the spec but is
not in `docs/language.md`; under basic rule 6 `/vision` must settle it before Design uses it, as it
did for **player** (Open questions say so).

Notes:
- **S17 – story 1, the prescribed path for a real change of role fails with the proposed dates.** The "Change" form tells the coordinator: use "Left the team" and then "Back in the team" with the new role. "Left the team" proposes today as the end; a new role proposes today as the start once trainings have started (criterion 2). With both proposals taken, the two roles share today, and the save fails: "This player already has a role on that date." The note under the criteria says the new role starts "on the next day". So the one path the tool itself recommends gives an error unless the coordinator changes a date. Suggest: for a player whose role ended, the new role proposes the day after the end (or the end proposes yesterday). (Trace: story 1 criteria 2, 3, 4 (form sentence), edge case "Two roles … overlap"; note under the criteria.)
- **S18 – story 1 and 5, "Back in the team" and the group "Left the team".** The form sentence names a button "Back in the team", but the Interface "Players" lists only "Left the team" and "Change" for each role; nothing says where the coordinator gives a second role to a player of the current season (only "Add player", and "Add to this season" for last season's players). Also, with S15 answered, a player who moves from "only trains" to "plays games" in November has an ended role and a role now. The players page puts players "whose role in the season ended" under "Left the team", and story 5 shows them "in a separate group 'Left the team'". Read as written, that player shows in both groups, or in "Left the team" while still in the team. For the coach, a player who never left must not show as left. Suggest: "Left the team" holds only players with no role now (on the players page and in the totals), and the Interface names "Back in the team" for a player in that group. (Trace: story 1 criteria 4, 6, 11, Interface "Players"; story 5 criterion 4.)
- **S19 – story 1, what "an earlier season can only be looked at" covers.** The sentence is in the criterion on roles, but reads wider. Attendance lets the coordinator choose an earlier season and tap a training (story 5), and story 3 lets a past training be "marked or corrected … also weeks later". Not said: in July, can the coordinator still correct a mark of a May training, cancel or delete a training of last season, or open a "Not opened" day of last season (which makes it exist and count)? If attendance of an earlier season can still be changed while its roles cannot, a wrong start of a role found after 30 June stays in the totals while the marks around it can be fixed. Either answer is fine for one coordinator; the user should know which, and opening a day of last season only to look must not create a training. Suggest: one sentence saying whether trainings and marks of an earlier season can be changed. (Trace: story 1 criterion 5; story 3 criterion 8; story 5 criteria 5, 6; Bruce's warning on opening.)
- **S20 – story 2, a mistake that reached opened trainings cannot be corrected without losing attendance.** The behaviour says: set up in October, enter the trainings since 1 September, and "A Wednesday typed by mistake is corrected with 'Change'; the list of training days follows at once." But the Wednesdays that were opened while entering the past stay, as extra trainings with their attendance, and the real Tuesdays show as "Not opened". A wrong time works the same way: the opened trainings keep the wrong time on the training screen and on the card (edge case "The times … change after a coming training was opened"). The only way to correct an opened training's date or times is "Delete training", which removes its attendance, and entering it again. The order in the behaviour (set up, then enter seven weeks) is the order in which this happens, and the list of a season then shows seven extra Wednesday trainings next to seven "Not opened" Tuesdays. Suggest: the training screen lets the coordinator change the date and times of a training (with the same check "There is already a training on this date."), so a mistake is corrected with its attendance kept; or the behaviour says plainly that opened trainings must be deleted and entered again. (Trace: story 2 intro "Once opened … does not touch it", criteria 4, 10, 16; edge cases 8, 10; behaviour "Setting up in October"; Steve's warning "every mistake must be correctable".)
- **S21 – story 2 with story 1, a training before the roles' start shows no players.** A role's start is a date, proposed as the first training date of the season when the player is added (story 1). When later the window of a weekly training is moved earlier (a preseason from 25 Aug), or an extra training is added before the first training date, no player has a role on that date: the training shows no players and counts for no one, and the coordinator must "Change" all 14 roles. The roles were not wrong; only the proposal was taken when the window was different. This was possible before (first training date moved), but the new rules invite it: "Add training" on any date of the season, and every window can be changed at any time. Suggest at least one edge case that says what the coordinator sees: for example a training with no players says "No players in the team on this date." with a link to "Players". (Trace: story 1 criteria 2, 7, 9; story 2 criteria 3, 6, 15; story 3 criteria 1, 9.)
- **S22 – story 2, "Cancel" or "Delete" for a past holiday opened by mistake.** The edge case offers both as equal. They are not equal for the user: after "Delete training" the day is "Not opened" again and stays in "N training days not opened" above the totals for the rest of the season, so the count never reaches zero. Suggest the edge case says "Cancel training" for a holiday and "Delete training" for a day opened instead of another. (Trace: story 2 criterion 16, edge case 2; story 5 criterion 8.)

### Fifth review (2026-10-16)
A fifth fresh agent (`/review spec`) read the spec as changed after the fourth review (pages
"Players" and "Team", "Change role", "Remove from the team", "Change" on a training, trainings of
an earlier season open). Read: this spec (with its uncommitted diff against the approved
version), `idea.md`, `change.md`, `docs/vision.md`, `docs/language.md`, `docs/ideas.md`, all of
`docs/product/`. Not read: any `technical.md`. Each note is checked by a trace through these
documents; the weekdays of the example dates were checked with `date`. The arguments held only
the mode and the change folder. Stories 3 and 4 did not change; they are checked only for effects
of the changes.

| Story | What the user is doing here | Verdict | Note |
|---|---|---|---|
| 1 Keep the players and the team | Typing the 14 players once (in October, for a season that started on 1 September), then recording joins, a move to "plays games", and leaves, each from a date. | serves – note | "Change role" and "Remove from the team" with the proposals "day before" / "day after" answer S17 and S18: the real change of role no longer fails on its own proposals. See S23, S24, S25. |
| 2 Weekly trainings and trainings | Setting up the weekly training once, entering the trainings since 1 September, fixing a wrong day or time on trainings already entered. | serves | "Change" on a training (S20) makes the Wednesday-for-Tuesday mistake correctable with attendance kept: change the weekly training, then move each opened Wednesday to its Tuesday, which becomes that day's row. Small text point in S26. |
| 3 Mark attendance at a training | In the hall, two taps per absent player. | serves | No change in the marking. S26 (example screen). |
| 4 Reasons | One tap for a reason with a weight. | serves | No change. |
| 5 Season totals | As coach, seeing who comes and who misses. | serves – note | See S24 (a player who changed role must not show as left) and S26. |

No story conflicts with the vision. Still open under basic rule 6: `docs/language.md` says a player
is "a member of the team" and has no **role**, **hidden**, **weekly training** or **training day**
(checked by reading the word list). The spec uses all of them throughout; `/vision` must settle
them before Design, as the Open questions say.

Notes:
- **S23 – story 1, the proposed start date at the first use.** The Context names the first use: in October the coordinator enters all trainings since 1 September from the sheet. "Add player" proposes "the first training date of the season, or today when the season's trainings have started". In October the season's trainings have started (a weekly training with the proposed window starts on Tue 1 Sep), so each of the 14 players is proposed from today. Taken as proposed, every training since 1 September then says "No player has a role on this date", and the coordinator must change the date 14 times, or "Correct" 14 roles. If the players come before the weekly training, there is no first training date, and the spec does not say what is proposed. The rule fits a player who joins mid-season, not the one setup the spec is written for. Suggest: "started" means that a training of the season has been opened (so the first setup proposes the first training date, a later joiner today), and say what is proposed when the season has no weekly training yet. (Trace: Context, story 1 criterion "Add player" on the team page, story 2 criterion "first training date", edge case "No player has a role on this date"; behaviour "Setting up in October".)
- **S24 – story 1 and 5, "Left the team" after "Change role".** The team page and story 5 still put in "Left the team" "the players whose role in the season ended". After "Change role" on 1 November, the player's first role ended on 31 October, so read as written the player shows in "Left the team" (and in the totals' group "Left the team") while still in the team. The answer to S18 meant "players with no role now", but the criteria were not changed to say so. For the coach a player who moved up must not look like a player who left. Suggest in both places: "the players with no role today or later in the season". (Trace: story 1 Team criterion 1, story 5 criterion 4, answers S17/S18.)
- **S25 – story 1, "Add all" and hidden players.** "Hidden" is for "a player who left the club", and a hidden player "is not offered when a player is added to the team". "Add all" in a new season adds "the players of last season's team"; it does not say that it skips hidden players. A player hidden in June comes back in July unless the coordinator sees and removes them. Suggest: the list under "Add all" and the button leave out hidden players (or show them apart). Also not said: whether a player who is hidden while still in a role shows at the trainings (as read: yes, "hiding changes nothing else"); that is consistent, but then hiding is not a way to take a player out of the team. (Trace: story 1 criteria "Hidden", "Add all", behaviour last sentence.)
- **S26 – text that disagrees with the rest.** (a) Story 5 says the page lists the training days "newest first"; story 2 and the Interface say "Coming" (nearest first) above "Past" (newest first). (b) Story 2 criterion 3 gives "Thursday from 5 Jan"; Tue 5 Jan 2027 is a Tuesday (checked with `date`), so the window of that example starts on a day that is not its day of the week. (c) The example training screen shows only "[ Cancel training ]"; the Interface puts "Change", "Cancel training" or "Undo cancel", and "Delete training" at the bottom. None changes what the user gets, but Design reads the text literally. (Trace: story 5 criterion 5, story 2 criteria 3 and "Attendance", Interface "Training" and the example.)

### Sixth review (2026-10-16)
A sixth fresh agent (`/review spec`) read the spec as changed after the second accept and build
review of story A ("Remove from the team" with "Not in the team from", "Undo", every proposed
date today, "Change role" rules, "Players of last season", "Hidden", "Last day today"), with its
uncommitted diff against the approved version. Read: this spec, `idea.md`, `change.md`,
`docs/vision.md`, `docs/language.md`, `docs/ideas.md`, `docs/product/choose-club-and-team.md`.
Not read: any `technical.md`. Each note is checked by a trace through these documents; the
weekdays of the example dates were checked with `date` (Thu 15 Oct, Wed 14 Oct, Sat 31 Oct, Sun 1
Nov, Mon 2 Nov 2026: all right); leftover text was found with `grep -n -i "first training
date\|proposed\|member of the team\|last day\|what was there"`. The arguments held only the mode
and the change folder. Stories 2 to 5 changed only by the removal of the "first training date";
they are checked for effects of the changes.

| Story | What the user is doing here | Verdict | Note |
|---|---|---|---|
| 1 Keep the players and the team | Setting up the team once (in October for a season that started on 1 September, or with "Add all" in a new season), then recording a player who leaves, from today or from tomorrow, and undoing a removal tapped by mistake. | serves – note | "Not in the team from" with "Tomorrow" and "Undo" serve the hall well: a removal shows at once, a player who never started leaves nothing behind, and a mistake is one tap. The cost moved to the setup. See S27, S28, S29, S30. |
| 2 Weekly trainings and trainings | Setting up the weekly training, entering the past, fixing mistakes. | serves | Only the "first training date" went; nothing else in story 2 used it (grep above: the remaining hit, "Tue 1 Sep (the first training date in the example)" in story 3, is an example, not a rule). |
| 3 Mark attendance at a training | In the hall, two taps per absent player. | serves | No change. A removal from today takes the player off today's training; "Tomorrow" is there for a player who trained today. |
| 4 Reasons | One tap for a reason with a weight. | serves | No change. |
| 5 Season totals | As coach, seeing who comes and who misses. | serves | "Left the team" now follows a removal at once, and a player whose roles were all removed is in no group and no total, which is what the coach expects. Kept absences outside a role do not count (story 1 edge case). |

No story conflicts with the vision. Core value 1 ("keep it correct") and Steve's warning ("every
mistake must be correctable in the app") are served by "Undo"; S29 is about what is left after it.

Notes:
- **S27 – story 1, the first setup now takes 14 typed dates, and a missed one is silent.** The Context names the first use: in October the coordinator enters all trainings since 1 September. With every start proposed as today, the coordinator types 1 September for each of the 14 players (the criterion says so on purpose). The cost is one sitting; the risk is the one player where today is kept by mistake. The order in the behaviour (players first, then the past) then gives no sign: the warning "now counts as there at N earlier trainings" fires only when trainings were entered first, and "No player has a role on this date." only when no player has one. The September trainings just show 13 names, and that player's totals start in October. Suggest one of: the date field keeps the date typed last while the coordinator adds players in one visit (14 adds, one typed date); or the team page says so for a player whose role starts after trainings of the season that exist. Either keeps "today" for the mid-season joiner, which the coordinator chose. (Trace: Context; story 1 criterion "Add player" on the team page; edge cases "No player has a role on this date" (story 2) and "A role starts … while trainings … are already entered"; behaviour "At the start of the season".)
- **S28 – story 1, "Players of last season" goes after the first added player, and "Add all" may have no date.** (a) The group shows only while no player has a role in the season. A new season often starts with a newcomer (the age group moves up). A coordinator who adds that newcomer first loses "Add all" and adds the 13 others one by one, each with a date. Suggest: the group shows while players of last season's team (not hidden) have no role in the season, and "Add all" adds those; or the behaviour says "Add all" first. (b) The criterion says "Add all" gives each player a role "from today", but the list of proposed dates names "Add all" as "proposed as today", which means a date field. If "Add all" has no date, a coordinator who taps it on 15 September, after two trainings, must "Correct" 13 roles. Say which; a date field ("In the team from", proposed today) is the same as for every other start. (Trace: story 1 criteria "Add player" (the list of proposed dates) and "A role ends at the latest …"; Interface "Team"; `docs/product/choose-club-and-team.md` "a youth team moves up an age group".)
- **S29 – story 1, after "Undo" is gone, the history does not show "what was there".** The criterion says that after leaving the page, "Correct" and "Add to the team" restore the roles "with the history showing what was there". But roles that start on the date or later are removed, and there is no stored "removed" state (change section, "Undo (A7)"). So a planned "Change role", or the whole role a player got from "Add all", is gone from the history; only the ended role shows its new end. The coordinator must remember the removed role to restore it. The weight is low ("Undo" covers the moment of the mistake, and a removal is a deliberate step), but the sentence promises more than the tool keeps, and on a phone a tap on another page ends "Undo" too. Suggest the sentence says what is left: "the history shows the role that ended; roles that were removed are gone." (Trace: story 1 criteria "Remove from the team" and "After 'Remove from the team'"; change section "Undo (A7)".)
- **S30 – story 1, "Last day today" on 30 June.** Every role ends at the latest at the end of its season, so on 30 June every player in the team has a role that ends today with no role after it, and every row says "Last day today". Harmless for one day, but it reads as if the whole team was removed. Suggest: "Last day today" only for a role that ends before the end of the season. (Trace: story 1 criteria "Last day today" and "A role ends at the latest at the end of its season (30 June)".)
- **S31 – text that disagrees with the rest.** (a) The Context (paragraph "Words") still says a **player** is "a member of the team who plays games or only trains"; `docs/language.md` now says a person of the club, in the team through a role. Steve's warning "`docs/language.md` says a player is 'a member of the team'" is no longer true (settled in Open questions). (b) "Add player" on the team page makes a new player with "first name and last name"; the page "Players" and the word list make the last name optional. (c) "with a button 'Tomorrow' for a player whose last day is today" reads as if the button shows only for some players; the Interface and the behaviour read as if it always shows, for the coordinator to use when the player still trains today. Say which. (Trace: Context; story 1 criteria "Add player" (both pages) and "Remove from the team"; Interface "Team"; behaviour; warnings.)

## Answers to the spec review (2026-10-15)
Decided with the coordinator; the stories above are changed to match.
- **S1:** fixed. (The rule is widened by S7: a player with no absences can be deleted.)
- **S2:** fixed. (The first answer, "asks before it removes later absences", is replaced by S8: absences are kept.) A player added (or "in the team since" moved earlier) while trainings on or after that date are entered counts as there at them; the tool says so and lists those trainings.
- **S3:** replaced by the coordinator's rule: a training is never removed but **cancelled**. A cancelled training stays in the list like a normal training, with the label "Cancelled", does not count, keeps its absences, and can be undone ("Undo cancel"). A training day can be cancelled without opening it. Only an extra training (added on a wrong date) can be deleted. "No training" is gone.
- **S4:** fixed. The start page card shows the next training day or extra training, also when it is cancelled ("Training · Tue 20 Oct · Cancelled").
- **S5:** fixed. The card shows today's training until midnight, so attendance can be entered at home after the training.
- Added after the review (coordinator, 2026-10-15): the switch "Show cancelled" on the list of trainings; a combined start page of coming events is an idea for later.
- **S6:** fixed. The page "Attendance" shows "N training days not opened" above the totals, with a link to the first of them.
- **Last name:** settled in `docs/language.md` (see Open questions).

## Answers to the second spec review (2026-10-15)
Decided with the coordinator; the stories above are changed to match.
- **S7:** fixed. A player with no absences can be deleted at any time; story 1 and the Interface say the same.
- **S8:** fixed. Moving "in the team since" or the date a player left keeps the absences outside the dates (not counted); nothing is asked or removed.
- **S9:** fixed. The list shows the whole season: "Coming" up to the last training date above "Past", so absences and cancels can be noted weeks ahead.
- **S10:** fixed. A change of training days applies from the day it is made; earlier dates keep the days set then. No new word: "training day" in the settings is a day of the week, in the list a date; the spec says which.
- **S11:** not in this spec; a line in `docs/ideas.md` (a longer login, so a training does not start with logging in).


## Change after approval (2026-10-15, during /design)
Decided with the coordinator, without a third spec review: a player is the person and has no
dates; a **role** ("plays games" or "only trains") in the team of a season has a start and an end.
A player can leave and come back (a new role). A role ends at the latest at the end of its season;
a new season starts with no players in a role, and "Add to this season" / "Add all" brings last
season's players back. A change of role within a season must be possible in the stored data; its
screen comes later. Story 1 and the parts of stories 3 and 5 and the Interface that named "in the
team since" are changed. The answers to S2, S7 and S8 still hold, now for roles.

## Change after the accept review of story A (2026-10-16)
The accept review of story A (`docs/changes/2026-10-15-players-and-settings/technical.md`, A1 and
A2) found two mistakes the coordinator could not correct in the app. Decided with the coordinator,
after the third spec review (S12–S16):
- **Weekly trainings (A1, S12, S13, S16).** Before, the season had training days of the week with
  a first and a last training date, and a change applied from today, so a mistake corrected later
  stayed in force for all earlier dates and was hidden. Now the coordinator keeps **weekly
  trainings**: a day of the week with times and a window, more than one at the same time. A
  weekly training only suggests trainings; a change corrects it for its whole window, and a real
  change of schedule is the end of one and a new one. Opened trainings have their own date and
  times and are never changed or deleted by a weekly training; any training can be deleted by
  hand. The season's first and last training date go; each window has its own. (A first proposal,
  "Applies from" on a save, was replaced by this before it was approved.) Changed: story 2 (all),
  Interface, What changes, warnings.
- **Players and team (A2, S14, S15, S17, S18).** Story 1 now has two pages: "Players" (all
  players, with "Hidden" and the history, where "Correct" fixes a role typed wrong) and "Team"
  (the team of the current season, with "Add player", "Change role" and "Remove from the team",
  each with a date). "Change role" records a real change of role, so the history stays right.
  Roles of an earlier season cannot be corrected. Changed: story 1 (all), Interface, warnings.
- **Trainings (S19–S22).** "Change" on a training fixes its date and times and keeps its
  attendance; trainings of an earlier season stay open; a training with no player in a role says
  so; a holiday is cancelled, not deleted. Changed: story 2, Interface.
- **New season (A3).** "A new season proposes last season's weekly trainings" (story 2) is not
  built in story A; story B takes it.

The sentence on the settings page "A change applies from today; earlier trainings keep their
days." goes (A5).

## Answers to the third spec review (2026-10-16)
Decided with the coordinator; the stories above are changed to match.
- **S12:** replaced: there is no "Applies from"; a change of a weekly training corrects only that weekly training.
- **S13:** fixed. A date has at most one row; an extra training on a date that becomes a training day is that day's training, with its attendance kept. Any training can be deleted by hand, nothing else deletes one.
- **S14:** on purpose: roles of an earlier season cannot be changed (story 1).
- **S15:** fixed. The "Change" form says how a real change of role from a date is entered; its own screen still comes later.
- **S16:** replaced: the season's first and last training date go (each window has its own); an opened training keeps its times when its weekly training changes.

## Answers to the fourth spec review (2026-10-16)
Decided with the coordinator; the stories above are changed to match.
- **S17, S18:** replaced: "Change role" (new role and a date) ends the current role the day before and starts the new one; "Remove from the team" ends the role; the page "Team" holds both. "Left the team" is only the group of players whose role ended.
- **S19:** trainings and attendance of an earlier season stay open; its roles and weekly trainings are fixed.
- **S20:** fixed. "Change" on a training changes its date and times; the attendance stays.
- **S21:** fixed. The training screen says "No player has a role on this date."
- **S22:** fixed. A holiday is cancelled; "Delete training" is for a training opened or added on a wrong date.

## Answers to the fifth spec review (2026-10-16)
Decided with the coordinator; the stories above are changed to match.
- **S23:** fixed. The proposed start in the team is the first training date of the season until a training of the season is opened, then today; with no weekly training, today. "Add all" uses the same date.
- **S24:** fixed. "Left the team" (story 1 and 5) is a player with a role earlier in the season and none today or later.
- **S25:** fixed. "Add all" leaves out hidden players.
- **S26:** fixed. Story 5 lists "Coming" above "Past"; the example is "Thursday from Thu 7 Jan"; the example training screen shows "Change", "Cancel training" and "Delete training".

## Change after approval (2026-10-16, during /design of story A)
Decided with the coordinator, without a new spec review (two small additions on one page): the
page "Team" shows the name of the team chosen from the game list for the season, or "Team not
chosen yet" with a link to choose it; players can be added before the team is chosen. The model
behind it: the club has the players; a team exists for one season and is the team chosen from the
game list. The words go to `/vision` (Open questions).

## Change after the second accept and build review of story A (2026-10-16)
The accept review (A6–A10) and the build review (R13, R17) of story A, second form, found rules
missing at the edges of a role. Decided with the coordinator (`/discover review:`):
- **Removing from the team (A6, A7, R13).** "Remove from the team" means: from a date, the player
  has no roles in the season. The field is the first day without the team ("Not in the team
  from", proposed today, with "Tomorrow"), as "Change role" and "In the team from" also name the
  first day of what is new. The role on the day before ends then; roles from the date on are
  removed. A player who was in the team for part of the season is under "Left the team" at once;
  a player who never started (for example after "Add all" in August) has nothing left in the
  season. Before, the field was the last day (so a removal from today showed nothing until
  tomorrow), a later planned role stayed, and a role that had not started could not be removed.
- **Undo (A7).** Every mistake must be correctable: after "Remove from the team", "Undo" on the
  row until the coordinator leaves the page; later by hand with "Correct" and "Add to the team".
  No "Delete" for a role in the history, and no stored "removed" state.
- **Proposed dates.** "In the team from" of "Add player" and "Add all" is 1 September of the
  season, with "Today" (after the sixth spec review, S27); "Add to the team" and "Change role"
  propose today. This replaces the answer to S23 (the first training date until a training is
  opened). "Change role" on a role that starts later says to use "Correct" (A8).
- **"Change role" to the same role (R17)** is refused.
- **"Players of last season" (A10)** shows only while no player of last season's team has a role
  in the season, and "Add all" asks "In the team from" (S28).
- **A removal that also removes a later role** asks for a confirmation that names it (S29).
- **Hidden (A9).** "Hidden" only keeps lists short: the list on "Players", the choices of "Add
  player" and "Players of last season". A hidden player under "Left the team" keeps "Add to the
  team" on its own row; before, the spec said a hidden player "is not offered when a player is
  added to the team".
- A player whose last day in the team is today shows "Last day today" in the row, not on 30 June
  (S30).

## Answers to the sixth spec review (2026-10-16)
Decided with the coordinator; the stories above are changed to match.
- **S27:** fixed. "In the team from" of "Add player" and "Add all" is filled with 1 September of
  the season, with a button "Today" and the date field for any other date. "Add to the team" and
  "Change role" stay on today.
- **S28:** fixed. "Players of last season" stays while no player of last season's team has a role
  in the season (a newcomer does not count); "Add all" asks "In the team from".
- **S29:** fixed. A removal that also removes a later role first asks for a confirmation that names
  that role; the criterion no longer says the history shows what was there.
- **S30:** fixed. No "Last day today" on the last day of the season.
- **S31:** fixed. The Context uses the word list's meaning of player; Steve's warning is marked
  settled; the last name is optional in "Add player" on the team page; "Tomorrow" is always next
  to the date field of "Remove from the team".

## Change during story C (2026-10-17)
At the design and close of story C, with the coordinator, after the spec review of its deviations
(all serve; notes D1 and D8):
- Navigation (D1): the start page keeps "Attendance · Team · Players · Settings"; "Attendance" gets no links. The coordinator's choice until the start page becomes one page of past and coming events (`docs/ideas.md`). Changed: Interface, "Navigation".
- "Add training" only in the current season (D8, a limit from story B, accept review A5): an extra training of an earlier season cannot be added. Changed: story 2 (trainings of an earlier season), Interface "Attendance". An idea line keeps the case.
