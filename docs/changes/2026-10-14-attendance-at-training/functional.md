# Functional spec: Attendance at training

**Status**: approved
**Change**: docs/changes/2026-10-14-attendance-at-training
**Date**: 2026-10-15

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

Words (`docs/language.md`): a **player** is a member of the team who plays games or only trains;
**attendance** is whether a player was at a training, and if not, the **reason**; a reason is
valid or not valid.

## User stories

### Story 1: Keep the players
**As a** coordinator, **I want to** keep the players and when each one is in the team, **so
that** I can mark their attendance without typing names, and the totals are fair to players who
join or leave during the season.

A player is the person: a name, with no dates. What has a start and an end is the player's role
in the team of a season: "plays games" or "only trains" (coordinator, 2026-10-15). Both roles come
to training.

#### Acceptance criteria
- [ ] The coordinator can add a player with a first name (required) and a last name (optional), and change both later. A player has no dates.
- [ ] The coordinator can give a player a role in the current season: "plays games" or "only trains", from a date (the tool proposes the first training date of the season, or today when the season's trainings have started).
- [ ] The coordinator can end a player's role with a date (the tool proposes today), and undo the end. A role ends at the latest at the end of its season (30 June).
- [ ] A player who leaves and comes back in the same season gets a new role from the date of return; the time between does not count.
- [ ] A player shows at a training, and counts for it, only when the date of the training is inside one of the player's roles. So a player who joined in November has no missed trainings in September and October.
- [ ] Absences outside a player's roles (for example after the role end was moved earlier) are kept, but do not count; they count again when the dates are changed back.
- [ ] A new season starts with no players in a role. The players page shows the players of last season with "Add to this season" for each, and "Add all": each gets the same role as at the end of last season, from the first training date of the new season (or today).
- [ ] Players are shown by name only: first name and last name. Two players with the same first name are told apart by the last name.
- [ ] The players page shows, for the chosen season, the players with a role now (sorted by first name, with "Plays games" or "Only trains"), then "Left the team" with the players whose role in the season ended.

A change of role within a season (from "only trains" to "plays games") is the end of one role and
the start of the other on the same day. The stored data must allow it; a screen for it comes later.

#### Behaviour
At the start of the season the coordinator opens "Players", taps "Add player" and types the 14
players once, each with "Plays games" or "Only trains". For a player who joined later, the
coordinator sets the date the role started. When a player leaves, the coordinator opens the player
and taps "Left the team" (with the date). Next season, the coordinator taps "Add all" (or adds
players one by one) and ends the roles of those who did not come back.

#### Errors and edge cases
- No first name: the player cannot be saved; the tool says "A first name is needed."
- A role end before its start: not saved ("The end must be after the start.").
- Two roles of one player that overlap in time: not saved ("This player already has a role on that date.").
- Two players with the same first and last name: allowed (the coordinator knows who is who), but the tool warns "There is already a player with this name."
- A player with no absences can be deleted at any time ("Delete player", after a confirmation), for example a player added by mistake. "There" is not stored for each player, so nothing is lost. A player with absences can only end the role.
- A role starts (or its start is moved earlier) while trainings on or after that date are already entered: "there" is the default, so the player now counts as there at those trainings. On save the tool says "This player now counts as there at N earlier trainings." and lists them, so the coordinator can mark absences.

### Story 2: Training days and trainings
**As a** coordinator, **I want** the tool to know when the team trains, **so that** today's
training is ready to mark without setting it up each time.

#### Acceptance criteria
- [ ] The coordinator sets the training days of the week (one or more), each with a start and an end time (for example Tuesday 19:45–21:00), and the first and the last training date of the season.
- [ ] When the training days are set for the first time, the tool proposes the first training day in September as the first date and the last training day in May as the last date. The coordinator can change both.
- [ ] Between the first and the last training date, the start page shows a card "Training" for the next training: today's training until midnight (so attendance can still be entered at home after the training), else the next training day or extra training. A cancelled training shows on the card too, as "Training · Tue 20 Oct · Cancelled". For example "Training · Tue 20 Oct · 19:45".
- [ ] Outside that period, and when no training days are set, the start page shows no training card. The coordinator can still open trainings from the page "Attendance".
- [ ] The coordinator can open the next training at any time, also days before it, to note an absence in advance.
- [ ] Opening a training makes it exist, with every player there (coordinator, 2026-10-15). From then on it counts. A training day that the coordinator never opened does not exist and does not count anywhere.
- [ ] The page "Attendance" lists every training day of the season, from the first to the last training date: "Coming" (today and later, nearest first) above "Past" (newest first). Each day shows each with "Not opened", "All there", "N absent" or "Cancelled", and the extra trainings. So the coordinator can enter past trainings (for example all trainings since 1 September) one after the other, and prepare coming ones: note an absence weeks ahead, or cancel a holiday break in advance.
- [ ] "Cancel training" marks a training as cancelled (coordinator, 2026-10-15), for example a holiday or a training cancelled at the last moment. A training day can be cancelled from the list without opening it. A cancelled training stays in the list, like a normal training, with the label "Cancelled". It does not count in the totals.
- [ ] The list of training days and trainings has a switch "Show cancelled", on by default. Off, the list hides cancelled trainings (coordinator, 2026-10-15).
- [ ] "Undo cancel" makes a cancelled training a normal training again. Absences noted before the cancel are kept while it is cancelled (not counted), and count again after the undo.
- [ ] "Add training" makes a training on any date (an extra training), with the start and end time of the training day, or times the coordinator types.
- [ ] Only an extra training can be deleted ("Delete training", after a confirmation: "Delete the training of Thu 22 Oct? Its attendance is removed."), for example one added on a wrong date. A training on a training day cannot be deleted; it can only be cancelled.
- [ ] Training days and "today" are in Dutch local time, the same as the games.
- [ ] A new season (from 1 July, as for the games) asks for its training days and dates again, with the days of last season and new proposed dates filled in.

#### Behaviour
On Tuesday at 19:40 the coordinator opens the tool. The start page shows "Training · Tue 20 Oct ·
19:45" with a short line: "Not opened yet", "All there" or "2 absent". One tap opens the training. On Friday
the coordinator reads in the chat that a player cannot come next Tuesday, opens the card
"Training · Tue 27 Oct" and marks that player.

#### Errors and edge cases
- The training is cancelled after the coordinator already opened it (for example to note an absence in advance): the coordinator taps "Cancel training"; the noted absence is kept, and counts again if the cancel is undone.
- The coordinator opens a past training day by mistake while it was a holiday: it now counts with everyone there; "Cancel training" makes it not count.
- The last training date is before the first: the tool does not save it and says "The last training date must be after the first."
- The coordinator changes the training days in the middle of the season: the change applies from that day on. Earlier dates keep the training days that were set then, so the list and "N training days not opened" do not change for the past. Opened and cancelled trainings stay.
- A start time after the end time: the tool does not save it ("The end time must be after the start time.").

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
- [ ] Players whose role in the season ended show in a separate group "Left the team", with the totals of their trainings.
- [ ] The page also lists the training days and trainings of the season (story 2), newest first; a tap opens the training (story 3).
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
- **Start page** (`/`): below the team header, a compact card "Training · Tue 20 Oct · 19:45" with "Not opened yet", "All there", "N absent" or "Cancelled". A tap opens the training (and so makes it exist, story 2). It shows only between the first and the last training date, and takes one line so the next games stay high on the first screen.
- **Training** (for one date): the date and time at the top, "N absent", "All there" or "Cancelled"; the players as large rows (name, and the reason when absent). A tap on a row opens the list of reasons with "There" on top. At the bottom: "Cancel training" or "Undo cancel", and "Delete training" for an extra training only.
- **Attendance**: the season (with a choice of earlier seasons), the totals for each player, the group "Left the team", and the list of training days and trainings of the season, "Coming" above "Past" ("Not opened", "All there", "N absent", "Cancelled"), with "Cancel training" on a day that is not opened and the switch "Show cancelled", and above the totals the number of training days not opened. "Add training" (with a date). A tap on a player shows the trainings that player missed.
- **Players**: the list of players, "Add player", the players of last season with "Add to this season" and "Add all" (in a new season), and for each player: first name, last name, the roles of the season ("Plays games" or "Only trains", from and until), "Left the team" (with a date), and "Delete player" when the player has no absences.
- **Training days and reasons** (settings): the training days of the week with start and end time, the first and the last training date of the season, and the list of reasons with valid / not valid, "Add reason", rename, hide.
- Navigation: the start page links to "Attendance"; "Attendance" links to "Players" and the settings.

Example of the training screen:

```
Tue 20 Oct · 19:45–21:00 · 2 absent
  Avery Jansen            there
  Blake de Vries          Ill
  Casey                   there
  Dana Smit               No reason
  …
  [ Cancel training ]
```

## Consistency
- Dates and times are Dutch local time, and the season starts on 1 July, as for the games (`docs/product/schedule.md`, `docs/product/choose-club-and-team.md`).
- Dates are written as on the schedule ("Tue 20 Oct"; with the year where the season is not clear).
- The layout follows the start page and the list of changes: cards, the same colours and fonts, phone first, no sideways scroll at 320 pixels.
- Only the coordinator can see and change this (login, as for all of the tool).
- Like the list of changes, a tap acts at once and the screen shows the result; errors are short sentences.

## What changes for existing users
No breaking changes. The start page gets one more card (between the first and last training
date). Until the coordinator adds players and training days, nothing else changes. The schedule,
the "Game list" card and the list of changes stay as they are.

The coordinator types in the players once (no import from the sheet).

## Warnings from architect and engineer
- **[Steve]** Players, trainings and attendance are the first data that only exist in the tool: unlike games, they cannot be read again from a source. The backup of the database must cover them, and every mistake must be correctable in the app.
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
- For `/vision`: the meaning of **training** ("a training session of the team") does not say that a training counts only when it was opened or added. Probably no change is needed; the spec says it.

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
