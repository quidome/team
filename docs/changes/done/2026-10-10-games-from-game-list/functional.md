# Functional spec: Games from the game list

**Status**: approved
**Change**: docs/changes/2026-10-10-games-from-game-list
**Date**: 2026-10-10

## Context
One coordinator runs one youth team in a club. Today the coordinator keeps the schedule by hand
in a Google Sheet. Typing games costs time and can be wrong, and games move during the season. A
moved game can break an arrangement that is already made, such as a drive or a jury duty, so the
coordinator must notice each change.

The federation (the Nederlandse Basketball Bond, the Dutch basketball federation) publishes the
game list: all games of a club for a season, open to anyone, with no login or key. This change
makes the tool read the game list, show the games that matter to the coordinator, and show what
changed since the coordinator last looked. The coordinator never types a game.

This is priority 1 in the vision: attendance, duties and messages all depend on correct games.
It is also the first change of the new product. The product has no screens yet (see Warnings).

The games are a source for several uses. The tool keeps all games of the club that the game list
gives. Each use chooses which games it needs and how it shows them. This change has the first
use: the schedule (stories 3 and 4). Later uses (duties, the message for the next game, the
message for the referee committee, the overview for the parents) choose their own games and
layout from the same source.

For the schedule, the coordinator needs two groups of games:
- **all games of the team**, home and away;
- **the home games of other club teams**, because players of the team do jury and referee duties at those games. Below, these are called "possible duty games" (plain words, not a new term).

### Facts from the real game list (checked 2026-10-10)
- One request per club and season gives all games of the club: home and away games of all club teams. Example: the club Blue Drakes, season 2026-2027, 71 games now; last season 118 games.
- The same source has a list of all clubs, with name, town and address. The club numbers in the game list are the source's own numbers, not the numbers on the federation's official pages.
- The source appears to be a site run by a private person, built on the federation's data. It may not be an official federation service (see Warnings).
- Each game has a game number of its own, a date, a start time, a home team, an away team, a place (hall name and town), and the competition.
- Most youth teams show games only until the end of the calendar year. The second half of the season appears later as new games. Last season, the teams kept the same team number in both halves.
- An unknown start time is shown as 00:00 (two games this season).
- An unplayed game shows the result "0 - 0". This is a placeholder, not a result.
- The game list does not say what changed. To find changes, the tool must compare with what it read before.

## User stories

### Story 1: Choose the club and the team
**As a** coordinator, **I want to** tell the tool once which club and which team are mine, **so that** it reads the right games.

#### Acceptance criteria
- [ ] Before a club and team are chosen, the tool shows only this step.
- [ ] The coordinator searches for the club by name (part of the name is enough) and chooses it from the results. Each result shows the club name and town.
- [ ] The tool reads the game list of that club and shows the club's own teams that it found (not the opponents). The coordinator chooses the team from these teams.
- [ ] If the search finds no club, the tool says so, and the coordinator can search again.
- [ ] The coordinator can change the club or the team later. After a change, the games shown follow the new choice.
- [ ] After a change of club or team, the games of the new choice appear with no changes recorded, the same as the first reading. Changes of the old choice that were not seen yet are removed.
- [ ] At the start of a new season, the tool keeps the club and asks the coordinator to choose the team again, because a youth team moves up an age group (for example M16 1 becomes M18 1). Until the coordinator chooses, the tool shows the games of the last season and says that a new choice is needed.

#### Behaviour
The coordinator types "drakes" and chooses "Blue Drakes, Woerden". The tool shows the teams M16 1, M16 2, M18 1,
MSE 1, X12 1 and X14 1. The coordinator chooses M16 1. The tool reads the game list and shows the
games (story 3).

#### Errors and edge cases
- The game list cannot be reached: the tool says so. The coordinator can try again. Nothing is saved.
- In summer, the new season can still be empty. The tool says that it found the club but no games for this season yet. The club comes from the list of clubs, so the coordinator can still save it. The team is then chosen when games appear.

### Story 2: Read the game list when the coordinator asks
**As a** coordinator, **I want to** read the game list with one tap, **so that** I see the newest schedule now and do not have to wait for the nightly reading.

#### Acceptance criteria
- [ ] One action reads the game list of the chosen club for the current season.
- [ ] After the reading, the tool shows a short result, for example: "3 new, 1 moved, 1 no longer in the game list." The counts are for the games in the schedule (story 3). They count what this reading found; the list of changes (story 4) shows what changed since the coordinator last looked, so the two can differ (changed 2026-10-14, epic review accept A1).
- [ ] A reading with no changes in the game list adds or changes nothing, and says "no changes".
- [ ] A reading never makes the same change appear twice, also when the nightly reading (story 5) runs at the same time.
- [ ] The tool never changes the data of a game by itself or by hand. The game list is the truth. When the game list is wrong, the coordinator asks the club or the federation to fix it. (Confirmed by the coordinator on 2026-10-10. If real use shows that the coordinator needs own corrections, that becomes a new idea.)

#### Behaviour
The coordinator taps the read action. A few seconds later, the result line is shown, and the
schedule (story 3) and the list of changes (story 4) are up to date.

#### Errors and edge cases
- The game list cannot be reached, or gives an error: the games stay as they were. The tool shows that the reading failed and when the last good reading was.
- The game list comes back empty, or with far fewer games than before: the tool does not treat this as "all these games are gone". It keeps the games, does not record changes, and warns the coordinator. The warning has the action "these games are really gone" (for example when a club team was withdrawn). After that action, the tool records each of those games as "no longer in the game list" (story 4). If the next reading gives the games again, the warning goes away.
- The game list has data that the tool does not understand (the format changed): the tool keeps the games, and shows an error. It never shows wrong games without a warning.

### Story 3: See the schedule
**As a** coordinator, **I want to** see the coming games of my team and the club's home games on my phone, **so that** I can stop keeping the schedule in the sheet.

This is the first use of the games (see Context). It decides only what the schedule shows. Later
uses choose their own games and layout.

#### Acceptance criteria
- [ ] The schedule holds all games of the team (home and away) and the home games of other club teams (possible duty games). It holds no away games of other club teams.
- [ ] A quick filter has three choices: all, games of the team, possible duty games. One tap changes it. The same filter applies to the list of changes (story 4).
- [ ] Coming games are shown first, in time order, grouped by date. Past games of this season can be opened on request.
- [ ] The games of the team look clearly different from the games of other club teams.
- [ ] Each game shows day, date, start time, home team, away team and place (hall name and town).
- [ ] A game with start time 00:00 in the game list shows "time not known yet", not "00:00".
- [ ] No results are shown. A "0 - 0" placeholder is never shown as a result.
- [ ] A game that is no longer in the game list is shown crossed out until the coordinator marks that change as seen (story 4). After that, it is hidden.
- [ ] The screen shows when the last good reading was, and if the last reading failed.
- [ ] The screen works well on a phone, with one hand.
- [ ] Only the coordinator can see and use the screens.

#### Behaviour
The coordinator opens the tool. The first thing shown is the next games, for example:

```
Last read: today 03:00

Sat 17 Oct
  12:15  Quick Runners M16 1 – Blue Drakes M16 1      Sporthal Wolfskamer, Huizen    (team)
  14:00  Blue Drakes M18 1 – Green Giants M18 1       Bulwijk, Woerden
Sat 24 Oct
  10:00  Blue Drakes M16 1 – Green Giants M16 1       Bulwijk, Woerden               (team)
  time not known yet
         Blue Drakes X12 1 – Quick Runners X12 1      Bulwijk, Woerden
```

The example shows content, not layout. Design decides the layout.

#### Errors and edge cases
- No games yet (start of the season): the screen says that the game list has no games for this season yet, and when it was last read.
- A start time is in Dutch local time. Summer time and winter time never shift a shown start time.

### Story 4: See what changed
**As a** coordinator, **I want to** see every change in the schedule until I have looked at it, **so that** I can fix any arrangement that a change breaks.

#### Acceptance criteria
- [ ] The tool shows a change for each of these, for the games in the schedule (story 3):
  - a new game;
  - a game that moved: new date, new start time, or new place (old and new value shown);
  - a start time that became known;
  - home and away swapped (old and new home team shown), because this decides if a game needs drivers or brings jury and referee places;
  - a team of the game changed (old and new team shown);
  - a game that is no longer in the game list.
- [ ] When a change moves a game into or out of the schedule (for example an away game of another club team becomes a home game), the change says why: "now a home game of another club team" or "no longer a home game of another club team", not "new game" or "no longer in the game list".
- [ ] Each change stays visible until the coordinator marks it as seen.
- [ ] The coordinator can mark one change, or all changes at once, as seen.
- [ ] When the tool finds many new games in one reading (for example the second half of the season in January), it shows one summary line ("18 new games"), which the coordinator can open.
- [ ] A change to a game of the team is clearly visible next to changes to other club teams' games.
- [ ] The very first reading after choosing the club and team (story 1) does not list every game as "new".

#### Behaviour
On the schedule, a short line shows how many changes are not seen yet. The coordinator opens
it and sees, for example:

```
Blue Drakes M16 1 – Green Giants M16 1    moved: Sat 24 Oct 10:00 → Sun 25 Oct 13:30   (team)
Blue Drakes X12 1 – Quick Runners X12 1   start time known: Sat 24 Oct 11:30
Blue Drakes M18 1 – Hoops M18 1           no longer in the game list (was Sat 31 Oct 14:00)
```

The coordinator reads them, fixes what must be fixed outside the tool (for now), and marks them
as seen.

#### Errors and edge cases
- When the federation deletes a game and makes it again with a new game number, the coordinator sees "no longer in the game list" for the old game and "new game" for the new one. Both show date, time and teams, so the coordinator can see what happened.
- A game that changes twice before the coordinator looks shows the change from what the coordinator last saw to what is true now.
- A game that leaves the game list and later comes back shows as a game that came back, not as two unrelated games.

### Story 5: Nightly reading
**As a** coordinator, **I want** the tool to read the game list by itself every night, **so that** the schedule is up to date without me thinking of it.

#### Acceptance criteria
- [ ] The tool reads the game list once a night, without the coordinator.
- [ ] The nightly reading records changes in the same way as story 2.
- [ ] When the nightly reading fails, the schedule shows this the next time the coordinator opens it, with the time of the last good reading.
- [ ] Before a club and team are chosen, there is no nightly reading.

#### Behaviour
The coordinator does nothing. In the morning, the schedule shows "Last read: today 03:00"
(the exact time is a Design choice), and any new changes.

#### Errors and edge cases
- Several nights of failure in a row: the screen shows clearly that the games may be out of date, and since when.

## Interface
There are three screens or parts of a screen. The layout is for Design:
- **Choose club and team** (story 1): a search for the club by name, then a choice of team.
- **Schedule** (story 3): the list of games, the quick filter, the time of the last reading, the read action (story 2), and the number of changes not seen yet.
- **Changes** (story 4): the list of changes, with "seen" for each one and for all.

All screens work on a phone first.

## Consistency
This is the first behaviour of the new product, so there is no existing behaviour to match. It
follows the vision's principles:
- the schedule comes from the game list, not from typing;
- one user changes data;
- small and correct beats complete.

The old app (being replaced) had a preview step before changes were applied. This spec drops
that on purpose. The game list is the truth, so approving a change adds a step without adding
control. The coordinator needs to *see* each change, which story 4 gives.

## What changes for existing users
No existing users of the new product. The coordinator's Google Sheet is not read or changed.
Players, duties and attendance in the old app or the sheet are not touched.

## Warnings from architect and engineer
- **The product has no base yet.** There are no screens, no login, no storage and nothing that runs a nightly job. This change cannot reach the coordinator without them. Design must choose: (a) build only as much of the base as this change needs, inside this change, or (b) a separate technical change for the base first. Logging in is an open question in the vision ("to decide in the first design").
- **Outside service with no agreement.** The game list has no published terms or format promise, and it appears to be run by a private person on the federation's data, not by the federation. The format can change, or the service can stop, without notice. The tool must notice data it does not understand and say so (story 2).
- **Two addresses.** The same data is at `www.basketballstats.nl` and `db.basketball.nl`. Which one is the official one is not known.
- **Read politely.** Once a night plus readings on demand is fine. Frequent automatic reading is not.
- **Season change.** The tool must know which season is current. In summer, the new season can be empty, and the old season must not look like "all games are gone".
- **Readings at the same time.** A reading on demand and the nightly reading can overlap. They must not record the same change twice or lose a change.
- **A game is known by its game number only.** A game made again with a new number looks like one gone and one new (story 4).
- **Local time.** All dates and start times are Dutch local time, with summer and winter time.
- **Placeholders in the data.** 00:00 means "start time not known", and "0 - 0" on an unplayed game is not a result.

## Not in scope
- Changing, adding or deleting games in the tool.
- Results, standings and statistics (longer horizon in the vision).
- Showing away games of other club teams. The tool keeps them, but the schedule does not show them.
- Short names for teams, and travel times to places (they belong with drives, later).
- Messages to the phone about changes. Add only if real use shows that changes are missed.
- More than one club or team, or more than one coordinator.
- Showing which duties a change affects. That comes with the duties change, which must then mark changes to games with duties as more important.
- A full first-run setup (a separate idea in `docs/ideas.md`).

## Open questions
- None for the user at this time. Design decides the base (warning 1), logging in, and the exact time of the nightly reading.
- For `/vision` (minor): the word list defines "game list" as "the federation's list", but the source may be a private site built on the federation's data. Check if the meaning must change.
- For `/vision` (minor): the word list defines "home game" for the team only. This spec says "home games of other club teams" and "possible duty games" in plain words. Check if a word is needed.

## Answers to the first spec review (2026-10-10)
- Story 1: season change and change of club or team added (notes 1 and C2).
- Story 2: (a) the coordinator confirmed that the tool never changes a game; (b) the "far fewer games" warning now has a way out.
- Story 4: home and away swapped, a changed team, and games that move into or out of the schedule added.
- C3: listed under open questions for `/vision`.
- Also added, from the coordinator: the games are a source for several uses; the schedule is the first use (Context, story 3).

## Answers to the second spec review (2026-10-10)
- Story 1: the club is now chosen by name from the source's list of clubs (the coordinator agreed). No number is typed.
- Story 2: the result counts match the list of changes.
- Story 3: a quick filter (all, games of the team, possible duty games), chosen by the coordinator; it also applies to the list of changes.
- Minor notes: story 1 now says the club's own teams, and where the club comes from in summer.
- Words: one name, "schedule", for the screen; "home games of other club teams" instead of "home games of the club".
- Who runs the source: added to Context, Warnings, and the open questions for `/vision`.

## Spec review
Reviewed 2026-10-10 by `/review spec` (fresh agent). Read: this file, `idea.md`, `change.md`,
`docs/vision.md`, `docs/language.md`. `docs/product/` is empty (first change), so there were no
product docs to compare with. All checks are by reading unless the note says otherwise.

| Story | What the user is doing here | Verdict | Note |
|---|---|---|---|
| 1 Choose club and team | Setting up once, so the tool can read the right games without typing a schedule. | serves – note | Youth teams move up an age group each season (M16 1 this year is a different federation team next year), so the chosen team stops matching at each season change. The spec only covers "choose the team when games appear" for the first setup. Say what the coordinator sees and does when a new season starts. See also note C2 (changing the team later). |
| 2 Read on demand | Hearing in the chat that a game moved and wanting the tool to show it now. | serves – note | (a) The idea marked "the tool never changes a game" as *not yet confirmed* (its open question 3: a game with no start time, or a game the club moved before the federation updated). The spec makes it an acceptance criterion and lists no open user questions, with no record that the coordinator confirmed it. If the coordinator does need to note a correction, it goes back into the sheet, against the success measure "the coordinator stops using the sheet". Confirm with the coordinator and write down the answer. (b) "Far fewer games" keeps the old games and warns, but gives no way out: if the drop is real (a club team withdrawn), the warning and the stale games stay forever, and there is no approval step to accept it. Say how this state ends. |
| 3 See the games | Looking at the coming games on a phone instead of in the sheet: where the team plays and which club home games need jury and referee places. | serves | Covers what the sheet holds for games: date, time, teams, hall and town, team games marked. Handles the 00:00 and "0 - 0" placeholders and summer and winter time. Fits journey 1 and core value 1. |
| 4 See what changed | Finding out a game moved, checking what arrangement it breaks, and marking it as seen. | serves – note | The list of recorded changes misses a change of **home and away** (the two teams swap) or of a team. That decides whether a game needs drivers or brings jury and referee places, so it breaks arrangements as much as a new date does. A swap also moves a game of another club team into or out of the story 3 set; then the tool would show it as "new game" or "no longer in the game list", which is misleading. Add these as recorded changes. |
| 5 Nightly reading | Nothing: the coordinator expects the schedule to be current when they open the tool for attendance. | serves | Once a night plus on demand matches the weekly rhythm in the idea, and the failure cases show the age of the data. |

**Cross-story notes**
- C1 – Vision fit: no conflict. The spec follows "the schedule comes from the game list", "one user changes data" and "small and correct beats complete", and keeps notifications, results and duties out of scope as the vision and idea say.
- C2 – Changing the team or club later (story 1) changes which games are in the story 3 set. The spec does not say whether the new team's games then all appear as "new" (story 4 only protects the *first* reading), or what happens to unseen changes of the old team. Say it.
- C3 – Words: the spec uses "game number", "club number" and "team number", which are not in `docs/language.md`. They are federation terms, not new product concepts, so this is minor; Vision may add them.

**Verdict:** no story fails to serve the user, and nothing conflicts with the vision. The notes on
stories 1, 2 and 4 and C2 should go back to Discover before Design. Note 2(a) needs the
coordinator's answer.

### Second spec review (2026-10-10)
Reviewed by `/review spec` (fresh agent, no memory of the first review's session). Input: mode
`spec` and this change folder, no other arguments. Read: this file, `idea.md`, `change.md`,
`docs/vision.md`, `docs/language.md`, `docs/ideas.md`, `docs/backlog.md`, `.claude/team.md`.
`docs/product/` is empty. Checks are by reading unless the note says otherwise. Two read-only
fetches of the source's help pages were run (see story 1 and C2).

**Answers to the first review:** all are in the spec. Checked by reading: story 1 criteria 5–7
(season change, change of club or team, unseen changes of the old choice), story 2 criterion 5
and the second edge case (coordinator's answer, "these games are really gone"), story 4
criteria 1–2 (home and away swapped, changed team, moving into or out of the schedule), and the
open question for `/vision`.

| Story | What the user is doing here | Verdict | Note |
|---|---|---|---|
| 1 Choose club and team | Setting up once, and again each season, so the tool reads the right games. | serves – note | **The club number may not be one the coordinator has.** The spec says the coordinator types "the federation's club number" that "the federation website shows". The source's own help page says the opposite: "the IDs used in the database are not the numbers used in Foys (the NBB's official program) … the NBB database has its own unique ids", to be looked up per club and season at `basketballstats.nl/db/service/howtolink.pl`. Checked: fetched `basketballstats.nl/help/koppelingen` and `howtolink.pl` on 2026-10-10 (that page picks the club by name from a list). A coordinator who types the number from the federation will get no club, or the wrong club, and the error in criterion 4 only says "try a different number". Say where the coordinator finds the right number, or let the coordinator find the club by name. Minor: "the teams it found" should say the club's own teams, not opponents (the example implies it); and in the summer edge case, say where the club name comes from when the season has no games. |
| 2 Read on demand | Hearing in the chat that a game moved, and wanting the tool to show it now. | serves – note | The result line "71 games read: 3 new, 1 moved, 1 no longer in the game list" mixes two sets: 71 is all games of the club, but story 4 records changes only for games in the schedule. Say that the counts are the changes in the list of changes (story 4), so the numbers the coordinator sees match what is there to mark as seen. The rest serves: the confirmed "game list is the truth" and the way out of the "far fewer games" warning close the first review's notes. |
| 3 See the schedule | Opening the phone to answer "when and where do we play next, and which club home games need our jury and referee places". | serves – note | **The team's own games may get lost.** The schedule holds every home game of every club team (last season 118 club games, roughly half of them home games, over at least six teams). The vision says the club's games matter "when we play at home" (journey 1), and journey 3 ties jury and referee places to the team's home games. If that is how duties work, club home games on days the team does not play at home are noise on a small screen; if players also do duties on other days, all are needed. The spec does not say which. Ask the coordinator, and either narrow the set or say how the coordinator quickly finds the team's next game (the "clearly different" look is not enough when it is one row among many). Not checked against the sheet (not in the repository). |
| 4 See what changed | Finding out a game moved, checking which arrangement it breaks, and marking it as seen. | serves | Covers every change that breaks an arrangement (date, time, place, home and away, team, gone, back). The summary line for many new games and "mark all as seen" keep the list usable. The same volume question as story 3 applies: changes to club home games that do not concern the team will also show up here. |
| 5 Nightly reading | Nothing: the coordinator expects the schedule to be current when opening the tool for attendance. | serves | Once a night plus on demand fits the weekly rhythm, and the failure cases show how old the data is. |

**Cross-story notes**
- C1 – Vision fit: no conflict. The spec follows "the schedule comes from the game list", "one user changes data" and "small and correct beats complete", and keeps results, notifications and duties out of scope.
- C2 – Who publishes the game list. `docs/language.md` and the spec call it "the federation's list" and say "the federation publishes" it. The help page fetched above is signed "© 2026 Jaap Voets" and describes a database with its own ids, built from the federation's data. It may be a private site, not a federation service. This does not change any story, but it matters for "keep it correct": the warning "outside service with no agreement" is stronger than written. For Design (which address to use, warning "two addresses") and for `/vision` (the meaning of "game list"). Checked: fetch of the help page; who runs `db.basketball.nl` not checked.
- C3 – Words. Story 2's behaviour says "the game list screen (story 3)", but "game list" is the federation's list in `docs/language.md`; stories 4 and 5 say "game screen"; the Interface says "Schedule". Use one name, "schedule". Checked: search for `game list screen\|game screen\|schedule` in this file. Also, "home games of the club" and "away game of another club team" stretch "home game / away game", which `docs/language.md` defines for the team only; minor, for `/vision` with the federation numbers.

**Verdict:** every story serves the user and nothing conflicts with the vision. Before Design:
the club number (story 1) and the size of the schedule (story 3, a question for the coordinator)
should go back to Discover. The result line (story 2) and the screen name (C3) are small text
fixes. C2 is for Design and `/vision`.
