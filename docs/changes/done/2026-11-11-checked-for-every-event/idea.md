# Idea: Checked for every event

**Status**: ready for discovery
**Purpose fit**: serves the purpose directly. Core value 2 (share the duties fairly) needs a duty
record that the coordinator confirmed, and both core values need correct data. Journey 0 says the
start page shows what still needs the coordinator; a past event that is not done is exactly that.
**Date**: 2026-11-11

## The problem
The coordinator looked at the start page and the report after v0.11.1 (2026-11-11) and found:
- **A duty game has no end in the administration.** Trainings and team games have "Checked"; a
  duty game (a home game of another club team where the team has table or referee places) does
  not. Its places count as done from the start time, also when nobody confirmed them. So the start
  page keeps "the last duty games" (the duty games of the newest past date) in view, until a newer
  date with duty games passes: Sat 31 Oct was still there on Wed 11 Nov.
- **Past and coming rows look the same.** A past duty game is lighter because its date has passed;
  a coming duty game is lighter because its places are filled and its schedule is entered. Both are
  grey rows with a thin blue bar, so the start page did not show that Sat 14 Nov was still to come.
- **Old team games did not ask for attention.** The open item "games not checked" counts only
  team games on or after 4 November 2026 (the release of answers at games). The coordinator had to
  find the older team games that were not checked by hand, and the report counts a game only when
  it is checked.

## The idea
- **"Checked" for every event** (word list, 2026-11-11): the coordinator marks an event as done in
  the administration. It then no longer asks for attention: it is not an open item and is not
  shown as something to do. It can still be opened, looked back at and corrected. For a training
  (its attendance), a team game (its attendance and its drive) and a duty game (its places: done,
  or corrected with "Someone else" or "Nobody").
- **Every past event that is not checked is an open item**, grouped by kind as now ("3 duty games
  not checked"); a tap on an item of more events opens their list.
- **The start page lists only today and later.** The part "last duty games" goes; the open items
  take over its job.
- **From the start of the season.** All past events of the current season that are not checked
  ask for attention at once, also the duty games and the team games before 4 November 2026: the
  coordinator checks them one time, and then the record of the season is complete.

## The administration panel (decided with the coordinator, 2026-11-11)
How a mark (a fact the coordinator sets on an event and can undo) looks on the page of an event.
Mockups: the rows "Marks" and "One panel per event" on the design canvas
(https://claude.ai/artifact/9mH5jgbtjyEJwLyXUoQTJ1, private to the coordinator).
- **One panel "Administration" for each event**, with one row for each mark that can be set now.
  The panel shows only from the moment a mark can be set (a training from its date, a game from
  its start time; "Schedule entered" also before the game).
- **Each row is a fact with one action:** "Not checked ○" with **[Mark as checked]**; "Checked ✓"
  with **[Undo]**. The state is always visible; the button never repeats the words of the state.
- **Buttons are actions:** "Mark as …" for every mark ("Mark as checked", "Mark as entered",
  "Mark as not played", "Mark answers as checked"), except "Count the drives" (the drivers drove)
  and "Cancel training", which are clear verbs already. "Undo" alone when set.
- **Orange only for the next step the coordinator must do;** a panel without orange means the
  administration of the event is done. Optional marks ("Count the drives") are outlined.
- **Rare marks** ("Cancel training", "Mark as not played") stay a plain button until they are set;
  then they are a row of the panel ("Cancelled", "Not played", with [Undo]).
- **Not a mark:** a choice between options ("Coming" / "Not coming", the reasons) stays as it is.
- Today two marks break this: "Schedule entered" (the state text next to a button with the same
  words; grey when set) and "The drivers drove" (a switch that stays pressed; orange when set).
  This change brings all marks into the panel. At Close the rule goes into
  `docs/product/principles.md`.

## Why now
The quieter start page (v0.11.0) made the open items the place for what needs the coordinator.
In real use the gaps above showed up within a day, and the duty record is half of the purpose.

## How it fits
- Words: team game, duty game and checked are in `docs/language.md` (2026-11-11).
- It builds on "Checked" of trainings and team games ([trainings and attendance](../../product/trainings-and-attendance.md), [attendance at games](../../product/game-attendance.md)), the open items of [the start page](../../product/start-page.md) and the corrections after a game on [duties](../../product/duties.md).
- It undoes a part of v0.11.0 (the last duty games), on purpose.

## User view
After a weekend the coordinator opens the start page and sees, for example, "Game Sat 14 Nov not
checked" and "Duty game Sat 14 Nov not checked". A tap opens the game; the coordinator corrects a
place if needed ("Someone else", "Nobody") and taps "Checked". The item is gone, and the list
below shows only what comes.

## Open questions
For `/discover`:
- Does "Checked" change the duty totals, or only the attention? (Today a filled place counts as
  done from the start time.)
- Is "Checked" of a team game one mark for its attendance and its drive, as the word list says, also
  when the attendance is complete but a drive is not corrected yet?
- "Someone else" before the game (`docs/ideas.md`, 2026-11-08): the coordinator asked to add it to
  this idea; the panel suggested a separate change after this one, because "Checked" works without
  it. To decide at the start of Discover.
- The open item for many duty games at the first release ("12 duty games not checked"): is one
  row with a list enough, or is a way to check several at once needed?
- A duty game marked "Not played": is it checked, or does it need no check?
