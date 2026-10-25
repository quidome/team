# Functional spec: Start page and season

**Status**: approved
**Change**: [2026-10-25-start-page-and-season](./)
**Date**: 2026-10-25

## Context
The coordinator runs one youth team of the club and opens the tool on the phone a few times a
week. Today the start page shows, from the top: the team with the season and "Change", the line
"N changes not seen", the card of the next training, the "Game list" card (read the game list,
the result of the last reading, the held-back warning, "may be out of date"), the full schedule of
games (all games of the team and all home games of other club teams, the "possible duty games",
with a filter and the past games on request), and the links "Attendance · Team · Players ·
Settings". The page "Attendance" shows the totals of each player and the list of all training days
and trainings of a season. See `docs/product/` (schedule, list of changes, read the game list,
trainings and attendance).

The coordinator does different jobs at different times (coordinator, 2026-10-25):
- **Often**: see what comes next, and what still needs the coordinator; complete the record of last week.
- **Rarely**: find any game or training of the season, for example to enter the referee committee's list of duty places later, or to look up a game in December.

Today these jobs are mixed on the start page, and nothing on one screen says what is still open:
a training that was never opened, a training whose absences were never marked, changes not seen,
a problem with the game list. The sheet and the coordinator's memory keep that.

This change gives each job its own place:
1. **The page "Season"**: all events of a season (an event is a training or a game), with date separators and labels.
2. **The start page as a landing page**: "Open items" (what still needs the coordinator) and "Next" (the events of this week and the next).
3. **Checked**: a past training is an open item until the coordinator confirms that its attendance is complete. The tool stores only absences, so an opened training with no absences cannot tell "everyone was there" from "not looked at yet". Only attendance is checked; duties will not be (a filled duty place counts as done by its player unless the coordinator corrects it; duties spec of 2026-10-18).

It is the base for duties (`docs/changes/2026-10-18-duties/`, back in Discover): duties will add
their open items (an away game without enough drivers, an open duty place, a possible duty game
not handled yet), the label group "Duties" on "Season", and the last game on the start page (to
correct its duties).

Facts from the coordinator (2026-10-25):
- "Next" covers this week and the next week.
- The last game is left out of the start page until duties give it something to complete.
- The list of trainings moves from "Attendance" to "Season"; "Attendance" keeps the totals.
- "Season" separates the events per week, then per date.
- Changes not seen, problems with the game list and training days not opened become open items.
- The last training always shows on the start page, also after it is checked, so a late correction is one tap (answer to spec review note 1).
- "Season" opens with "Coming" at each visit; labels are not kept (note 2).
- In July and August the open items also cover the season before (note 4).

## User stories

### Story A: The page "Season"
**As the** coordinator, **I want** one page with all events of a season, **so that** I can find
any game or training, and the start page can stay short.

#### Acceptance criteria
- [ ] The start page links to "Season" (first in the line of links, see story B). The page shows "Season 2026-2027" and all events of that season that the tool knows: all games of the team (home and away), all possible duty games (home games of other club teams), and all training days and trainings. Away games of other club teams are not shown, as now.
- [ ] **Date separators**: a heading for each week, Monday to Sunday, with the week number and the dates ("Week 44 · 26 Oct – 1 Nov"). Within a week, the events are grouped by date ("Sat 31 Oct"). A week without events has no heading.
- [ ] **Labels**, in two groups:
  - What: "Games of the team", "Possible duty games", "Trainings".
  - When: "Coming", "Past".
- [ ] How the labels combine: labels of different groups narrow the list; labels of one group add to it; a group with no label on shows all of that group. For example "Possible duty games" + "Coming" shows only the coming possible duty games; "Games of the team" + "Trainings" shows both. One tap turns a label on or off, without a new load.
- [ ] The page opens with "Coming" on and no label of "What", at each visit: labels are not kept between visits (a kept label could hide events without a sign). The labels that are on are always visible at the top of the list.
- [ ] **Order**: coming events (today and later, Dutch date) first, nearest first. Then, when "Past" is on, the past events of the season, newest week and date first. With both on, the list has two parts with their own headings, "Coming" and "Past"; the week of today can then have a heading in both parts. Within a date: games and trainings with a known start time in time order, then those whose start time is not known.
- [ ] **A game** shows what the schedule shows now: the start time ("time not known yet" when the game list has 00:00), the home team, the away team, the hall and town, and the badge "Team" with the team colour for a game of the team. A game no longer in the game list is shown crossed out with "no longer in the game list" while its change is in the list of changes, as now.
- [ ] **A training** shows what "Attendance" shows now: the start time, and "Not opened", "All there", "N absent" or "Cancelled"; "extra" for a training on a date that is not a training day; and "Checked" (story C). A tap opens the training, as from "Attendance" now (a training day not opened becomes a training).
- [ ] What moves here from "Attendance": "Add training" (only in the current season), "Cancel" on a training day not opened, and the switch "Show cancelled" (on by default; each phone or computer keeps its own setting).
- [ ] **Season choice**, as on "Attendance": the current season, and the earlier seasons with games or trainings. The season is in the address (`/season?season=2025-2026`), so a reload keeps it. The current season is the default.
- [ ] The **"Game list" card** moves from the start page to the top of "Season" of the current season, with its behaviour of now (read, result line, held back with "These games are really gone", "may be out of date"). After a good reading, the list shows the new games at once.
- [ ] From the training page, "Back" goes to the page it came from (the start page or "Season"). After "Delete training", the training page goes to "Season" of that training's season.

#### Behaviour
To look up a game in December, the coordinator taps "Season", scrolls down the weeks, and finds
"Week 51 · 14 Dec – 20 Dec". To prepare for the referee committee's list, the coordinator turns on
"Possible duty games": only the coming home games of other club teams are left, per week. To
correct a training of three weeks ago, the coordinator turns on "Trainings" and "Past".

#### Errors and edge cases
- A season with no events: "No games or trainings in 2026-2027 yet."
- A combination of labels with no events: "No coming trainings.", "No past possible duty games.", "No coming events.", and so on, as the schedule does now.
- No team chosen for the current season (a new season): the page shows the trainings of the season and "Choose the team to see its games." with a link to the choice. The games of an earlier season are found through the season choice.
- An earlier season with no kept games, or a club not chosen in it: only its trainings.
- An address with a season that cannot be read (`?season=abc`): "This season cannot be read." with "Current season", as on "Attendance".

### Story B: The start page as a landing page
**As the** coordinator, **I want** the start page to show what still needs me and what comes next,
**so that** a look of a few seconds tells me if I must do something.

#### Acceptance criteria
- [ ] With a club and team chosen, the start page shows, from the top: the team header (team, season, "Change"); "Open items"; "Last"; "Next"; the line of links "Season · Attendance · Team · Players · Settings".
- [ ] **"Open items"** shows one row for each open item, oldest problem first within its kind, in this order of kinds:
  1. The game list: "The last reading of the game list was held back." while a held-back reading waits for the coordinator; "The games may be out of date since Thu 22 Oct." while the game list has not been read well for 3 nights (the rule of now). A tap goes to the "Game list" card on "Season".
  2. Changes: "4 changes not seen" ("1 change not seen"). A tap opens the list of changes. This replaces the line under the team name.
  3. Training days not opened: "Training day Thu 22 Oct not opened" for one; "3 training days not opened" for more than one. A tap opens the oldest of them (which shows "Not opened" with "Open training"). The same days and the same number as the line on "Attendance".
  4. Trainings not checked (story C): "Training Thu 22 Oct not checked" for one; "2 trainings not checked" for more than one. A tap opens the oldest of them.
- [ ] Without open items, "Open items" shows "Nothing open."
- [ ] **"Last"** shows the last training before today (the newest training before today that is not cancelled), as on "Season" (for example "Thu 22 Oct · Training 19:45 · 2 absent · Checked"), also after it is checked. A tap opens it, so a late correction is one tap. With no training before today in the season, "Last" is not shown.
- [ ] Open items cover the current season and, in July and August, also the season before (so the record of June is not dropped on 1 July); only dates before today (Dutch date). Today's training is in "Next", not in "Open items". A cancelled training, or a cancelled training day, is never an open item.
- [ ] **"Next"** shows the events of the team from today until the Sunday of next week (today is a Sunday: today and the next week), grouped by date: the games of the team and the training days and trainings, each as on "Season". Possible duty games are not in "Next". A tap on a training opens it.
- [ ] When "Next" has no events in those two weeks, it shows the first coming event of the season after them; when there is none, "No coming events."
- [ ] The schedule, the card of the next training and the "Game list" card leave the start page (the schedule and the card of the game list are on "Season"; the next training is in "Next").

#### Behaviour
On Monday evening the coordinator opens the tool. "Open items": "Training Thu 22 Oct not checked".
A tap opens that training (it is also under "Last"); the coordinator marks Eva absent with "Ill", taps "Checked", and goes
back: "Nothing open." "Next" shows the training of Tuesday, the training of Thursday, the away game
on Saturday, and the trainings and the home game of next week. On Wednesday a parent says that Bram was ill on Tuesday: a tap on "Last" opens Tuesday's training, already checked, and the coordinator changes Bram's mark.

#### Errors and edge cases
- No club chosen: the start page shows only the choose step, as now.
- A new season without a team (the club is chosen): the start page shows the choose step, then "Open items" and "Next" with the trainings of the season only. The view "Last season" with last season's games leaves the start page; the earlier season is on "Season".
- Open items or "Next" cannot be loaded: "The start page could not be loaded. Load the page again."
- A training day that is no longer a training day (its weekly training changed) and was never opened is no longer an open item, as it no longer shows on "Attendance".

### Story C: Checked trainings
**As the** coordinator, **I want to** confirm that the attendance of a training is complete,
**so that** a training I did not look at stays in view until I do.

#### Acceptance criteria
- [ ] A training (opened or added) whose date is today or earlier has "Checked" on its page. A tap saves at once; the page then shows "Checked" with "Undo check". "Undo check" saves at once and shows "Checked" again.
- [ ] A training before today that is not checked and not cancelled is an open item (story B).
- [ ] A checked training stays checked when the coordinator marks a player, changes its times or date, cancels it or undoes the cancel: these are the coordinator's own changes. A checked training moved to a date after today stays checked, shows "Checked" with "Undo check", and is not an open item when its date passes.
- [ ] A cancelled training is never an open item; "Undo cancel" of a past training that is not checked makes it an open item again.
- [ ] "Delete training" removes the training with its check; on a training day the date is "not opened" again, and so an open item when it is before today.
- [ ] At the release, every training before today counts as checked, so the first day does not bring a long list. Trainings of today and later start not checked.
- [ ] "Season" shows "Checked" on the row of a checked training, next to "All there" or "N absent".
- [ ] A training of an earlier season can be checked and unchecked as well; it is not an open item (story B).
- [ ] A save that fails shows "Not saved. Try again." and the page shows what is stored, as for marking.

#### Behaviour
During training the coordinator marks who is absent. Afterwards, in the car park or at home, a tap
on "Checked" ends it. A forgotten one shows the next time the start page opens.

#### Errors and edge cases
- Two screens: "Checked" on one while the other marks a player: both are kept. "Checked" and "Undo check" on two screens: the last one wins.
- A training whose date is after today and that is not checked has no "Checked" yet (an absence noted in advance does not make it complete).

## Interface
- Screens:
  - `/` the start page: team header, "Open items", "Last", "Next", the line of links "Season · Attendance · Team · Players · Settings".
  - `/season` (with `?season=` for another season): the "Game list" card (current season), the labels, "Add training", "Show cancelled", the events per week and date.
  - `/attendance`: the season choice, the line of training days not opened, and "Totals". The list "Trainings" and its actions are no longer here.
  - `/training/[date]`: gets "Checked" / "Undo check".
- Example of the start page:

```
Blue Drakes M16 1 · 2026-2027              Change

Open items
  Training Thu 22 Oct not checked
  4 changes not seen

Last
  Thu 22 Oct   Training 19:45 · All there

Next
  Tue 27 Oct   Training 19:45 · Not opened
  Thu 29 Oct   Training 19:45 · Not opened
  Sat 31 Oct   14:30 B.Z. '72 M16 1 – Blue Drakes M16 1 · Woerden
  Tue 3 Nov    Training 19:45 · Not opened
  Thu 5 Nov    Training 19:45 · Not opened
  Sat 7 Nov    11:00 Blue Drakes M16 1 – Lions M16 2 · Utrecht

Season · Attendance · Team · Players · Settings
```

- Example of "Season" with "Possible duty games" + "Coming":

```
Week 44 · 26 Oct – 1 Nov
  Sat 31 Oct
    13:00 Blue Drakes X12 1 – De Dunckers X12 1 · Utrecht
    16:00 Blue Drakes M14 2 – Lions M14 1 · Utrecht
Week 45 · 2 Nov – 8 Nov
  Sun 8 Nov
    time not known yet  Blue Drakes V18 1 – Rivals V18 1 · Utrecht
```

- Server addresses follow in the design.

## Consistency
- The rules of the schedule stay: which games, Dutch dates, the order within a date, "time not known yet", gone games crossed out while their change shows, the team colour and badge. Only the place changes (start page → "Season") and the filter becomes labels; the words "Games of the team" and "Possible duty games" stay.
- The rules of trainings stay: what a training day is, opening, cancelling, "extra", "Show cancelled", "Add training" only in the current season.
- Training days not opened: the open item uses the same days and number as the line on "Attendance".
- "Checked" saves at once with "Not saved. Try again." on failure, as marking does.
- "Show cancelled" is kept on each phone or computer, as now. The labels are not kept: "Season" opens with "Coming", as the schedule opens with its coming games now.
- The list of changes keeps its behaviour; only its entry on the start page moves to "Open items".

## What changes for existing users
- The schedule moves from the start page to "Season"; its filter becomes labels; "Show past games" becomes the label "Past".
- The card of the next training goes; the next trainings are in "Next".
- The line "N changes not seen" and the warnings of the game list become open items; the "Game list" card moves to "Season".
- The list of trainings, "Add training", "Cancel" on a row and "Show cancelled" move from "Attendance" to "Season".
- The view "Last season" on the start page in a new season goes; the earlier season is on "Season".
- New: open items, "Last", "Next", "Checked". At the release, all trainings before today count as checked.
- The start page links become "Season · Attendance · Team · Players · Settings".

## Warnings from architect and engineer
- **One rule for each open item.** The count of training days not opened on "Attendance" and the open item must give the same days; the held-back and out-of-date warnings must give the same answer on the start page as on the "Game list" card; changes not seen the same number as the list of changes. Two rules would give two numbers.
- **Worked out each time.** Open items come from what is stored at that moment; only "checked" is a new stored fact. Nothing is counted on the side.
- **The start page is opened often, on a phone.** It must load fast, also while a reading of the game list runs.
- **The release marks the past trainings as checked**, once, and must give the same result when it runs twice.
- **Dutch dates and weeks**, also around the clock change (the night of 24 to 25 October 2026): "today", "this week", "next week" and the week numbers follow Dutch time.
- **Two screens**: "Checked" and a mark at the same time must both be kept; neither may be lost without a sign.

## Not in scope
- Duties: their open items, the label group "Duties" (handled, not handled, open places), the page of a game, and the last game on the start page (all with duties).
- Attendance at games, and checking a game.
- Open items of an earlier season, except the season before in July and August.
- A list of past events on the start page beyond the open items.
- Notifications or messages about open items (the tool never sends).
- A tap on a game (no page of a game yet).
- The overview for the parents.

## Open questions
- For `/vision`: the phrase "possible duty games" (open question in the vision) now also names a label on "Season".

## Spec review
Reviewed 2026-10-25 by `/review spec` (fresh agent). Read: this spec, `idea.md`, `change.md`,
`docs/vision.md`, `docs/language.md`, `docs/product/` (schedule, trainings and attendance, list
of changes, read the game list, choose club and team), and the line of 2026-10-18 in
`docs/ideas.md`. All findings are traces against these documents unless they say otherwise. No
extra text was given in the arguments.

| Story | What the user is doing here | Verdict | Note |
|---|---|---|---|
| A: The page "Season" | Rarely: looking up any game or training of the season (a game in December, the possible duty games for the referee committee's list, a training of weeks ago). | serves – note | Gives the rare job its own place and frees the start page; the labels fit the three examples. See notes 1, 2 and 3. |
| B: The start page as a landing page | Often, on the phone, in a few seconds: "must I do something, and what comes next?"; during training, opening today's training. | serves – note | This is journey 0 and priority 3 of the vision. Open items come from one rule each and replace the scattered lines and cards. Today's training is the first row of "Next", so journey 2 keeps its one tap. See notes 1, 4 and 5. |
| C: Checked trainings | Closing the record of a training, so a forgotten one comes back by itself. | serves – note | Answers a real gap: only absences are stored, so "all there" and "not looked at" looked the same. One extra tap per training, and it can be done on the day, so core value 1 holds. See notes 6 and 7. |

**Notes**

1. **A checked training is hard to correct (A, B).** The idea gives the start page "the last
   training, to complete its record", and says this answers the idea of 2026-10-18 (correcting
   last week's training needs a long scroll on "Attendance", because "Coming" is above "Past").
   The spec keeps the last training on the start page only while it is not checked, and the facts
   name only the last game as left out. A late correction (a parent reports a reason on Wednesday
   for Monday's checked training) now goes: "Season", turn on "Trainings", then either turn off
   "Coming" and turn on "Past", or scroll past all coming trainings of the season, because "Season"
   also puts coming above past. That is the same long scroll, one page further. Ask the coordinator
   whether the last training stays reachable from the start page after it is checked, or whether
   "Season" should make the recent past easy to reach. Checked by trace: functional.md lines 19,
   57, 199; idea.md lines 20 and 38; ideas.md line 19.
   - 2026-10-25: Answered (coordinator): "Last" on the start page always shows the last training, also after it is checked (story B).
2. **Kept labels can hide events without a sign (A).** Each phone keeps its labels, so after
   "Trainings" + "Past" to correct a training, the next visit to look up a game shows no games.
   The spec does not say that the page shows which labels are on in a way that makes this clear,
   or how to get back to the default. A small point for Design, but the job of story A is "find
   any game", and a hidden filter works against it. Checked by trace: lines 56, 174.
   - 2026-10-25: Answered (coordinator): labels are not kept; "Season" opens with "Coming" at each visit, and the labels that are on are always visible (story A).
3. **The week of today with "Coming" and "Past" both on (A).** Coming events are nearest first and
   past events newest first, so the current week (for example Mon–Wed past, Thu–Sun coming) is
   split: does its heading show twice, once at the top and once at the start of "Past"? The spec
   does not say. Checked by trace: lines 51, 57.
   - 2026-10-25: Fixed: with both on, the list has two parts, "Coming" and "Past", each with its own headings; the week of today can have a heading in both (story A).
4. **The end of a season drops open items silently (B).** Open items cover only the current
   season, so a training of the last week of June that is not checked stops being an open item on
   1 July, without being checked. The spec puts open items of an earlier season out of scope on
   purpose; the coordinator should know that the record of June can stay incomplete. Checked by
   trace: lines 90, 119, 198; the season start in `docs/product/choose-club-and-team.md`.
   - 2026-10-25: Answered (coordinator): in July and August the open items also cover the season before (story B).
5. **Small differences in the examples (B).** The start page example shows "Not opened yet",
   while story A and "Attendance" say "Not opened" and "Next" shows each event "as on Season". The
   example of "Next" for Monday 26 Oct shows the trainings of Tuesday and Thursday this week, but
   only Tuesday 3 Nov next week, while the behaviour says "the trainings ... of next week". The
   dates and week numbers themselves are right. Checked by a search (`Not opened yet`: lines 146,
   147, 149) and a run of Python's ISO calendar for 22 Oct, 26 Oct, 1 Nov, 8 Nov, 14 Dec and
   20 Dec 2026.
   - 2026-10-25: Fixed: "Not opened", and Thu 5 Nov added to the example.
6. **"Checked" is both the state and the button (C).** "the page then shows 'Checked' with 'Undo
   check'. 'Undo check' ... shows 'Checked' again": the same word is the action before and the
   state after. On a phone in a car park this may read as "it is already checked". The word is
   fixed by `docs/language.md`; how the state and the action look different is for Design.
   Checked by trace: line 112.
   - 2026-10-25: For Design (the word stays, as in `docs/language.md`).
7. **A checked training moved to a later date (C).** "Change" keeps the check, and a training after
   today has no "Checked" yet. A checked past training moved to next week: does it show "Checked",
   and when that date passes, is it an open item or not? Rare, but the two rules meet here.
   Checked by trace: lines 114, 128.
   - 2026-10-25: Fixed: a checked training moved after today stays checked, shows "Checked" with "Undo check", and is not an open item when its date passes (story C).

No story conflicts with the vision: the tool still never sends (open items are worked out, not
reminders), and the change follows the new priority 3 and journey 0. Not checked: how the pages
feel on a phone (there is nothing built yet).
