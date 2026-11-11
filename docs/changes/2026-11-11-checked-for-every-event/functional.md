# Functional spec: Checked for every event

**Status**: approved
**Change**: docs/changes/2026-11-11-checked-for-every-event/
**Date**: 2026-11-11

## Context
The coordinator records each event of the season: who was at a training or a team game, and who
did the duties of a game. Today a training and a team game have "Checked" (the coordinator
confirmed the record); a duty game has no such end. After the quieter start page (v0.11.0) the
coordinator found three gaps in real use (2026-11-11):
- a duty game never stops asking for attention in the administration, so the start page kept "the
  last duty games" in view until a newer date with duty games passed (Sat 31 Oct was still shown
  on Wed 11 Nov), and a past duty game and a coming one looked the same;
- the team games before 4 November 2026 were never an open item, so the coordinator had to find
  the ones that were not checked by hand (the report counts a game only when it is checked);
- the marks on the page of a game look and read differently: "Schedule entered" shows the same
  words as text and as button, and is grey when set; "The drivers drove" is a switch that is
  orange when set.

The words are in `docs/language.md` (2026-11-11): **event** (a training or a game), **team game**
(a game the team plays; its only duty is drive, at an away game), **duty game** (a home game of
another club team where the team has table or referee places) and **checked** (an event that the
coordinator marked as done in the administration; it no longer asks for attention, and can still
be opened and corrected).

**Serves the purpose**: core value 2 (share the duties fairly) needs a duty record that the
coordinator confirmed, and both core values need correct data. The open items (journey 0) then
show every event whose record is not done, and nothing that is done.

## Current behaviour
- **Checked.** A training of today or earlier, and a team game from its start time, have the button
  "Checked"; set, the page shows "Checked ✓" (a game) or "Checked" with a check mark (a training)
  and "Undo check". "Checked" on a team game also makes each player with "no answer" absent. A
  check of a game holds for its date (decision 0029). A duty game has no "Checked".
- **The other marks** on the page of a game: "Not played" (a button; set: "Not played" with
  "Undo"), "The drivers drove" (a pressed switch on a not played away game with drivers, orange when
  set, with "The drives count." / "No drive counts."), "Schedule entered" (a home game of the club:
  the text "Schedule entered" / "Schedule not entered" next to a button "Schedule entered", orange
  when not set and grey when set), "Answers checked" (a button in the note of a moved game). On the
  page of a training: "Checked", and "Cancel training" (set: "Cancelled" with "Undo cancel").
- **Open items** ([start page](../../product/start-page.md)): training days not opened and
  trainings not checked over the seasons of the open items; "Game Sat 31 Oct not checked" / "2
  games not checked" for the team games before today that are not checked, not "not played", still
  in the game list, and on or after 4 November 2026; a tap goes to the oldest of them.
- **The start page list**: the last duty games (the duty games of the newest date before today,
  always lighter), then the events from today until the Sunday of next week.

## Analysis
- What works: "Checked" of trainings and team games, its meaning, and the open items of trainings.
- What does not work: a duty game has no "Checked", so it cannot leave the attention; the date
  4 November hides older team games that are not checked; the marks have three different forms,
  and two of them read as a state where they are an action.
- Conclusion: one form for every mark, "Checked" for a duty game, and an open item for every past
  event that is not checked, from the start of the season. Then the start page can list only today
  and later.

## User stories

### Story 1: One administration panel for each event
**As a** coordinator, **I want** every mark of an event in one place, each with its state and one
action, **so that** I see at once what is done and what still needs me, and never mistake a state
for a button.

#### Acceptance criteria
- [ ] The page of a training and the page of a game have one panel "Administration", with one row for each mark that can be set now. Without such a mark the panel is not shown.
- [ ] Each row shows its state as a fact, with "○" when not set and "✓" when set, and one button: the action of a tap when not set, **"Undo"** when set. A button never repeats the words of the state.
- [ ] **Checked** (a training from its date, a game from its start time): "Not checked ○" with **"Mark as checked"**; set: "Checked ✓" with "Undo". Below the row: on a training the attendance ("2 absent"); on a team game the count ("7 there · 3 absent"), on an away game also its drivers ("7 there · 3 absent · Drivers: Anna, Bram"), so one "Mark as checked" confirms the attendance and the drive and both are in view; after the tap the message as now ("2 players with no answer are absent.").
- [ ] **Schedule entered** (a home game of the club, also before the game, while the schedule is required or entered, as now): "Schedule not entered ○" with **"Mark as entered"**; set: "Schedule entered ✓" with "Undo".
- [ ] **The drives count** (a not played away game with drivers, from its start time, as now): "No drive counts ○" with **"Count the drives"**; set: "The drives count ✓" with "Undo".
- [ ] **Rare marks** stay a plain outlined button until they are set, then they are a row of the panel with "Undo": **"Cancel training"** → "Cancelled" (a training); **"Mark as not played"** → "Not played", with "Its duties and attendance do not count." (a game).
- [ ] **Answers from before a move** keeps its note, with the button **"Mark answers as checked"**; after the tap the note goes away, as now. It stays a note outside the panel: it is a warning about the answers, not a mark on the record of the event.
- [ ] **Orange only for the next step the coordinator must do:** "Mark as checked" and "Mark as entered" while not set. "Count the drives", the rare marks and "Undo" are outlined. A panel without orange means the administration of the event is done.
- [ ] **A team game with no answer at all** (no player has an answer, and no player is marked there or absent): the row "Not checked" shows "Nobody has an answer yet: marking as checked makes all N players absent." before the tap. The tap works as now.
- [ ] Nothing else changes: what each mark does and counts, when it can be set, the choice "Coming" / "Not coming" and the reasons.

#### Behaviour
On Monday the coordinator opens the training of Thursday: "Administration · Not checked ○ · 2
absent · [Mark as checked]". A tap: "Checked ✓ · 2 absent · [Undo]". The page has no orange left.

#### Errors and edge cases
- A save that fails shows "Not saved. Try again." and the stored state, as now.
- Two screens: the last tap wins, as now.
- A rare mark that is set (for example "Not played") and "Checked" can both be rows of one panel.

### Story 2: "Checked" for a duty game
**As a** coordinator, **I want to** mark a duty game as checked after I confirmed its places,
**so that** it no longer asks for my attention.

#### Acceptance criteria
- [ ] From its start time (without a start time, from the start of the game day), the page of a duty game has the row "Not checked ○" with "Mark as checked" in the administration panel, below "Schedule entered". The line below the row shows the places of the team ("Table 1: Anna · Table 2: Cas (was Bram)"). Set: "Checked ✓" with "Undo".
- [ ] Before the start time the row is not shown; a check is refused: "A game can be checked from its start time." (as for a team game).
- [ ] The check holds for the date of the game (decision 0029): when the game moves, the check has no effect; when it moves back, it holds again.
- [ ] The check holds only while the game is a duty game: when another team is chosen and the game becomes a team game (or is no longer a game with places of the team), its duty check has no effect; the check of a team game likewise holds only while it is a team game.
- [ ] A duty game marked "Not played" has no "Checked" row and never asks for attention.
- [ ] "Checked" changes only the attention: a filled place still counts as done from the start time ([duties](../../product/duties.md)), the totals and the report do not change.
- [ ] On "Season" a checked duty game shows "· Checked", as a checked team game.

#### Errors and edge cases
- A duty game that left the game list after it was played can still be checked; it is never an open item, on purpose, the same as a team game that left the game list (its places still count as done, as now).
- Two screens: the last check or undo wins.

### Story 3: Every past event that is not checked is an open item
**As a** coordinator, **I want** the start page to ask for every past event of the season that is
not checked, **so that** I never have to search for it.

#### Acceptance criteria
- [ ] New open item, after "games not checked": "Duty game Sat 31 Oct not checked" for one, "12 duty games not checked" for more: the duty games before today that are not checked, not "not played" and still in the game list.
- [ ] "Games not checked" counts all team games of the season before today (not only from 4 November 2026), with the same other rules as now. The coordinator entered the older games by hand and wants to find the gaps.
- [ ] With the date 4 November 2026 gone, the games before it are like all others: their list on the page of the game is no longer closed behind "Show the list", and "Attendance" no longer says "The start page asks for games from Wed 4 Nov.".
- [ ] The report's line "N games not checked" uses the same rule, so it counts the older team games too. Duty games are not in that line: they do not change the numbers of the report.
- [ ] Both cover the seasons of the open items: the current season, and in July and August also the season before.
- [ ] For one game a tap opens it. For more, a tap opens the list of those games below the item ("Sat 31 Oct · 14:30 Blue Drakes X14 1 – BV Alphia X14 1"), each a tap to its page, as for "schedule not entered"; this replaces "to the oldest of them" for "games not checked".
- [ ] After "Mark as checked" (or "Not played") the game leaves the item; an item with no games is not shown.

#### Errors and edge cases
- At the first release, all past duty games of the season (and older team games not checked) ask at once; the item stays one row with its list.
- A game that moves to a date after today leaves the item; it asks again after its new date.

### Story 4: The start page lists only today and later
**As a** coordinator, **I want** the list of the start page to show only what comes, **so that** a
past row and a coming row cannot be mixed up.

#### Acceptance criteria
- [ ] The part "last duty games" is not shown: a past event is shown only through the open items (story 3).
- [ ] The list has the events from today until the Sunday of next week, as now, with the same rows and marks. An event of today that is done (for example a checked training) is lighter, as now.

#### Errors and edge cases
- Without coming events: "No coming events.", as now.

## Interface
- Screens: the page of a training (`/training/[date]`), the page of a game (`/game/[season]/[id]`), "Season" (the row "· Checked" of a duty game), the start page (open items).
- Words: "Administration", "Not checked", "Mark as checked", "Schedule not entered", "Mark as entered", "No drive counts", "Count the drives", "The drives count", "Mark as not played", "Mark answers as checked", "Undo", "Nobody has an answer yet: marking as checked makes all N players absent.", "Duty game Sat 31 Oct not checked", "N duty games not checked".

## Consistency
- One form for every mark, the same on a training and a game. The rule goes into `docs/product/principles.md` at Close.
- "Checked" keeps its meaning of the word list; a duty game gets it too, with the same moment (the start time) and the same rule of a check for a date.
- The open item of duty games works as the other items with a list ("schedule not entered", "open places").

## What changes for existing users
- New button words on the pages of a training and a game (the marks themselves work as before).
- The first time after the release, the open items can be long: every past duty game of the season, and every team game before 4 November 2026 that is not checked.
- The last duty games leave the start page.
- No data is lost; a check stored before stays.

## Warnings from architect and engineer
- A duty game needs its own stored check: the check of a team game sits in its attendance and makes players absent; a duty game has no attendance. Both must follow "a check holds for a date" and hold only while the game is of that kind.
- The release date of "games not checked" is stored (`game_check_start`); removing the rule needs a migration and a decision record, as does the check of a duty game.
- The answer of the start page changes shape again (story 3 adds an item, story 4 removes the last duty games).
- Story 1 renames buttons that many browser tests use.
- "Mark as checked" on a team game without answers makes every player absent: the warning of story 1 must show before the tap.

## Not in scope
- "Someone else" before the game (a separate change, `docs/ideas.md`, 2026-11-08).
- A way to check several games at once.
- Any change to the totals, or to the numbers of the report (only its line "games not checked" follows story 3).
- "Season" in the quiet look.

## Open questions
- None.

## Spec review
Reviewed 2026-11-11 by `/review spec` (fresh agent). Read: this spec, `idea.md`, `change.md`,
`docs/vision.md`, `docs/language.md`, `docs/product/` (start page, duties, game attendance,
trainings and attendance, report, season, principles, read the game list). Not read:
`technical.md` (none yet), `docs/changes/done/`.

| Story | What the user is doing here | Verdict | Note |
|---|---|---|---|
| 1 Administration panel | On the page of one event, after it (or before a home game), finishing its record and seeing at a glance whether anything is still to do. | serves – note | S1, S2 |
| 2 Checked for a duty game | After a weekend, looking at the places of a duty game once, correcting them ("Someone else", "Nobody"), and closing it. | serves – note | S3, S4 |
| 3 Every past event not checked is an open item | Opening the start page and finding everything of the season that is not done, without searching; at the first release, catching up the whole season once. | serves – note | S5, S6, S7 |
| 4 Only today and later | A look of a few seconds at what comes, with no past row to mix up with a coming one. | serves | The job of the last duty games (a correction one tap away) moves to the open item of story 3, which holds a duty game until it is checked. Checked by reading start-page.md ("Past events"). |

**Serves the purpose**: yes, with one limit. The claim "core value 2 needs a duty record that the
coordinator confirmed" holds only for attention: story 2 says "Checked" does not change the
totals or the report, so a filled place still counts as done without a confirmation, and the
report does not say which duty numbers are not confirmed (S4). What the change really brings is
that every past event asks once to be looked at, so the corrections get made: that serves
"correct data", under both core values. No conflict with the vision or its principles found
(read vision.md: journeys 0, 3, 7, principles).

### Notes
- **S1 – The drive in "Checked" of a team game.** The word list (2026-11-11) says checked covers a team game's "attendance and its drive", and `idea.md` left this as an open question; the spec says "Open questions: None" but answers it nowhere. Story 1 shows under the row of a team game only the attendance count ("7 there · 3 absent"), not the drivers, so on an away game the coordinator can tap "Mark as checked" without seeing the drive. Suggest: say in story 1 that "Checked" of a team game is one mark for both, and show the driver places on the line below the row of an away game, as story 2 does for the places of a duty game. Checked by reading language.md line "checked", idea.md "Open questions", story 1.
- **S2 – Two marks stay outside the panel.** "Mark answers as checked" stays in its own note, and the idea says "This change brings all marks into the panel". Probably right (it is a warning about a move, not the end of the record), but say so in story 1 so Design does not have to guess. Also a team game where the team has its own table or referee places (the warning "This team plays this game.", v0.11.2) is neither a duty game nor covered by the word "checked" of a team game; say which check covers those places, or that none does. Checked by reading duties.md and language.md ("team game", "duty game").
- **S3 – Played duty games that leave the game list.** Story 2 keeps "never an open item" for a duty game that left the game list after it was played, as for team games. For a team game the cost is only a game that does not count; for a duty game its places do count as done ("A game whose date has passed was played: its duties count", duties.md), so it counts without ever asking for a look. Probably rare; a known limit is enough, but write it down. Checked by reading duties.md ("Games that change keep their duties") and story 2.
- **S4 – The report and "Not in scope".** "Not in scope: any change to the totals or the report" is not quite true. The report's line "1 game not checked" counts "the games of the open item 'not checked' of the start page" (report.md); story 3 drops the date 4 November from that item, so the line grows, and the duty games not checked are a new item the report does not mention. The coordinator then reads duty numbers in the report that were never confirmed, with no hint. Suggest: say in the spec what the report's line does (follow the item, and add "· 3 duty games not checked", or stay as is on purpose). Checked by a search in the code: `games_not_checked` is used in `crates/server/src/api/report.rs:205`.
- **S5 – Old team games without answers (the main risk).** Before 4 November 2026 no answers were entered, so each old team game not checked has every player at "no answer". Story 3 puts them all in the open items; the only ways out are to tap each player there or absent, "Mark as checked" (which makes all players absent, not valid, and lowers the game percentage of everyone in the report), or "Mark as not played" (false). The warning of story 1 helps, but the open item pushes towards the one tap that writes wrong data, against "the stored data is the truth and must be right" (vision, core values). Whether the coordinator has the attendance of those games (in the sheet?) and wants to enter it is a claim about the user's work: **not checked, ask the coordinator**. If not, the spec needs a way to close an old game without counting it, or to keep the date 4 November for team games only.
- **S6 – The closed list of older games.** Today a game before 4 November shows its list closed behind "Show the list" (game-attendance.md; code: `before_release` in `web/src/routes/game/[season]/[id]/+page.svelte:286`). When story 3 asks the coordinator to complete those games, that closed list costs an extra tap on each and reads as "nothing to do here". The spec should say whether it opens. Checked by a search for `before_release`.
- **S7 – Text that becomes false.** "Attendance" shows "Games count once they are checked. The start page asks for games from Wed 4 Nov." (`web/src/lib/trainings.ts:58`); after story 3 the start page asks for all games. Add it to "What changes for existing users" and to the interface words. Checked by a search for "asks for games from".

**Answers of Discover (2026-11-11, with the coordinator).** S5: the coordinator entered all team
games that could be found and wants the whole season treated the same, to find the gaps; story 3
stays, and the warning of story 1 covers a game without answers. S1: on an away game the line
below "Not checked" also shows the drivers, so one check confirms the attendance and the drive.
S2: "Mark answers as checked" stays a note, with the reason in story 1. S3: a played duty game
that left the game list is never an open item, on purpose (story 2). S4: the report's line "games
not checked" follows story 3, duty games are not in it (story 3, Not in scope). S6, S7: the closed
list of games before 4 November and the sentence on "Attendance" go (story 3).
