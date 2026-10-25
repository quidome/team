# Idea: Start page and season

**Status**: ready for discovery
**Vision fit**: New priority 3 and journey 0 ("Open the tool"). Serves the third reader of the vision, the coordinator ("what happened, and what must still be arranged"), the north star (a few minutes in the tool each week) and core value 1 (capture fast). It is the base for duties: their open items and labels show on these two pages.
**Date**: 2026-10-25

## The problem
When the coordinator opens the tool, the start page shows the full schedule of games, also the
home games of other club teams where the team has no part, and the next training on a card. What
the coordinator wants to know is not on one screen: what comes next, what is still open, and
whether the record of last week is complete. Today the sheet and the coordinator's memory keep
that. Different jobs happen at different times (coordinator, 2026-10-25): the weekly look at what
comes and what is open; completing the record of the last training and game; looking up any game
or training of the season.

## The idea
Two pages, each for its own job (coordinator, 2026-10-25):

1. **The start page as a landing page**: quick access to what is done often.
   - The last game and the last training, to complete their record.
   - The next few events (games and trainings).
   - The open items: what still needs the coordinator. In this change: a past training that is not checked. Duties add theirs later (an away game without enough drivers, an open jury or referee place, a possible duty game not handled yet).
   - A home game of another club team shows on the start page only when the team has a part in it (that comes with duties; until then, not at all).
2. **The page "Season"**: all events of the season that the tool knows, also all possible duty games. Not a list for every day, but access to any game or training of the season.
   - Date separators, so a date is found quickly.
   - Labels that combine: labels of different groups narrow the list, labels of one group add to it. In this change the groups What (games of the team, possible duty games, trainings) and When (coming, past). Duties add the group Duties (handled, not handled, open places).
   - A tap on an event opens its page (the training page exists; a page of a game comes with duties).
3. **Checked.** A past training is an open item until the coordinator checks it ("Checked"), or it is cancelled. Only absences are stored, so an opened training with no absences cannot tell "everyone was there" from "not looked at yet". Checked belongs to attendance only: duties need no confirmation (a filled duty place counts as done by its player unless corrected; duties spec, 2026-10-18). Later, attendance at games is checked in the same way.

## Why now
Duties (the next priority) were in design when the coordinator found that the spec put several
jobs on the same screens (2026-10-25). Building these two pages first gives duties a place for
their open items and labels, and avoids a page "Duties" that would be removed later.

## How it fits
- The schedule (`docs/product/schedule.md`) moves from the start page to "Season", with its rules (which games, coming and past, gone games). The current filter ("All", "Games of the team", "Possible duty games") becomes the labels of the group What.
- The card of the next training on the start page (`docs/product/trainings-and-attendance.md`) becomes part of the next events.
- "Attendance" keeps the totals; whether it keeps its own list of trainings is a question for Discover. The idea of 2026-10-18 (on "Attendance", correcting last week's training needs a long scroll) is answered by the last training on the start page.
- The "Game list" card, the line "N changes not seen" and the links stay on the start page unless Discover finds a better place.
- No new data from the game list. One new stored fact: a training is checked.

## User view
The coordinator opens the tool on the phone a few times a week. The start page answers: what comes
next, and is anything still open? A tap on last Thursday's training marks the absences, then
"Checked". Before entering the referee committee's list, or to look up a game in December, the
coordinator opens "Season" and combines labels ("Possible duty games" + "Coming").

## Open questions
- How many are "the next few events": a number, or a period (for example this week and the next)?
- Does "Attendance" keep its own list of trainings, now that "Season" shows them?
- At the release, the past trainings of this season are not checked. Do they count as checked (proposal), so the coordinator does not get many open items at once?
- Can a checked training be changed, and does a change make it open again? (Proposal from the design of duties: no, because it is the coordinator's own change.)
- Is a cancelled training ever an open item? A training day that was never opened?
- The last game: what does it show before duties and attendance at games exist?
- Which date separators: per week ("Week 44 · 26 Oct – 1 Nov"), per weekend, or per date?
- Where do the "Game list" card, "N changes not seen" and the links go?
