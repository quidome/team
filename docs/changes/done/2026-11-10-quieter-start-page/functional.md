# Functional spec: A quieter start page

**Status**: approved
**Change**: docs/changes/done/2026-11-10-quieter-start-page/
**Date**: 2026-11-10

## Context
The start page exists so that a look of a few seconds tells the coordinator whether something
still needs them, and what comes next ([the start page](../../../product/start-page.md)). Each
earlier change added its own part to it. With a team chosen it now shows, from the top: the team
header with "Change", "Open items", "Last" (the newest past training, also when checked, and the
games of the newest past date with places of the team), "Next" (all events from today until the
Sunday of next week, each as a card with hall, places, counts and badges), "Messages" (two
buttons) and seven links. On 2026-11-10 the coordinator looked at the page with real data and said
it is overwhelming: "there is just so much to see". In that screenshot one row needed the
coordinator ("1 change not seen"); everything else was information with nothing to do.

The coordinator then chose from mockups on a design canvas
(https://claude.ai/artifact/9mH5jgbtjyEJwLyXUoQTJ1, private to the coordinator): mockup
"E · Combined" for the page, and "F3 · Busy Saturday, grouped by date" for the events.

**Serves the purpose**: the coordinator opens the start page a few times a week to see what
needs them (journey 0 of the vision). A page that hides that answer among done things slows
the fast capture that both core values depend on. This change serves the purpose by making the
open items and the next events the only things in view.

## Current behaviour
As in [the start page](../../../product/start-page.md), "Behaviour". The parts that this change
touches:
- The team header shows "Your team · 2026-2027", the team, and "Change" (to the choice of club
  and team).
- "Open items": one card for each kind of item (8 kinds), in a fixed order; a tap goes to the
  place to act; an item with more than one game opens the list of its games below it.
- "Last": always shown when such a training or game exists, also when everything in it is
  checked.
- "Next": cards grouped by date. A game of the team has an orange left border, a soft orange
  background and the badge "Team"; each game shows both teams one above the other, the hall and
  town, "· Checked" or "· Not played", the count of who comes, the duty places of the team, and
  warnings. A training shows its start time and its status.
- "Messages": "Message of the week" (when this week has something to announce) and "Committee
  message" (whenever a team is chosen).
- Links: Season · Attendance · Duties · Report · Team · Players · Settings.

## Analysis
- **What works:** the open items are right. They are the only part the coordinator must act on,
  and their rules (which items, their order, where a tap goes) are not in doubt. The range of
  "Next" (until the Sunday of next week) is right: next week's game must be visible before the
  weekend.
- **What does not work:**
  - Done things have the same weight as things that need the coordinator: a checked training, a
    game with all places filled and a game with 10 players without an answer all look alike.
  - "Last" shows mostly things that are done: a past training or game of the team that is not
    checked is already an open item. One thing in it is not: the duty places of a past home game
    of another club team are never checked and never an open item, and "Last" is the one tap to
    that game for a correction after the game ("Someone else", "Nobody"), which counts in the duty
    totals and the report.
  - Each event is a card with up to six lines, so two weeks fill more than one phone screen.
  - Rare actions ("Change", Team, Players, Settings) take the same place as weekly ones, and the
    committee message shows also in weeks without places of the team.
- **Conclusion:** keep the rules of the open items and of "Next"; change what is shown and how
  loud it is.

## User stories

### Story 1: Open items as quiet rows
**As a** coordinator, **I want to** see at the top of the start page only the things that need
me, each as one short row, **so that** I see in a moment whether I must do something.

#### Acceptance criteria
- [ ] The open items have the same kinds, texts, order and tap targets as now ([the start page](../../../product/start-page.md), "Open items").
- [ ] Each item is one row: an orange dot, the text, and "›". No card around it, no heading "Open items".
- [ ] An item with more than one game ("3 games with open places", "2 games: too few players", "5 home games: schedule not entered") opens the list of its games below it with a tap, as now; each game in that list opens its page.
- [ ] Without open items, the page shows one grey row "Nothing needs you." (it replaces "Nothing open.").
- [ ] The rows are at the top of the page, below the header.

#### Behaviour
The coordinator opens the start page and reads the rows with a dot. A tap on "1 change not seen"
opens the list of changes; a tap on "Training Tue 3 Nov not checked" opens that training. "Back"
there returns to the start page, as now.

#### Errors and edge cases
- Many items (all 8 kinds): all rows show; the page grows. The rule of the items does not change.
- The page cannot be loaded: "The start page could not be loaded. Load the page again." (as now).

### Story 2: Coming events as one quiet list
**As a** coordinator, **I want to** see the coming events as a short list in which the games of
the team stand out and done things step back, **so that** I see what comes next without reading
every line.

#### Acceptance criteria
- [ ] The list has the same events as "Next" now: the games of the team, the home games of other club teams with places of the team, and the training days and trainings, from today until the Sunday of next week (on a Sunday: today and the next week). When these have nothing, the list shows the first coming event of the season that is not cancelled; with none, "No coming events." (as now).
- [ ] The heading "Next" goes. The events follow the open items after a small space.
- [ ] The events are grouped by date: one small grey heading per date ("Tue 10 Nov"), then one row per event, in time order. A row shows its start time on the left ("time not known yet" when the start time is not known).
- [ ] No cards: rows are separated by a thin line.
- [ ] **A game of the team** has a soft orange background and a thin orange bar on the left of the row. Its first line says "Home game against <opponent>" or "Away game at <opponent>" (a home game: a game in the club's own hall). The line below has the hall and town, then what applies: the count of who comes ("6 coming · 2 no answer"; from the start time "8 there · 1 absent"), "Not played", "Checked". The duty places of the team (the drivers of an away game, the table and referee places of a home game) follow on the next line, with the same text as the row has now.
- [ ] **A game with places of the team that is not a game of the team** (a home game of another club team) has a thin blue bar on the left and no background. Its first line has both teams ("Blue Drakes X14 1 – BS Leiden X14 4"); the line below has the places of the team, with the same text as now ("all filled", "Table 1, 2: Anna, open"). It has no hall and town, on purpose: such a game is always in the club's own hall.
- [ ] **A training** has no bar and no background: "Training", and its status as now ("Not opened", "2 absent · Checked", "Cancelled", "extra").
- [ ] **An event with nothing left to do is lighter** (grey text, and a lighter bar): a training that is checked or cancelled; a game of the team that is checked or marked "Not played"; a game that is not a game of the team whose places of the team are all filled and whose schedule of the referee committee is entered, or that is marked "Not played". A home game of the club whose schedule is not entered shows "Schedule not entered" on its row and has normal text, so the row agrees with the open item "schedule not entered". All other events have normal text. A game that is no longer in the game list stays crossed out, as now.
- [ ] The warnings of a game (for example a driver who is not coming) still show on its row, as now.
- [ ] A tap on a row opens the training or the page of the game, as now; "Back" returns to the start page. A training day that is not opened opens as now.
- [ ] Each row is at least 44 pixels high. The text of a lighter row, the bars and the background are readable in light and dark mode.

#### Behaviour
On a quiet week the coordinator sees, for example:

```
Tue 10 Nov
  19:45   Training                          (lighter: Checked · 7 absent)
Sat 14 Nov
| 14:30   Blue Drakes X14 1 – BS Leiden X14 4    (lighter: all filled)
Tue 17 Nov
  19:45   Training
Sat 21 Nov
| 14:00   Away game at Break Stars M16 1          (soft orange)
          De Trits, Baarn · 10 no answer
```

On a busy Saturday with a duty game, a game of the team and a second duty game, the three rows
stand under one date heading, in time order; only the game of the team has a background.

#### Errors and edge cases
- A game of the team against another club team in the club's own hall: "Home game against
  Blue Drakes M16 2".
- A game of the team whose home and away teams were swapped (a change): the row follows the game
  as it is now.
- A game with places of the team that also has a warning (an open place in the coming days) has
  normal text, not lighter: a place is open.
- Today's training that is already checked is lighter, like any other done event.

### Story 3: Past events only when they need me
**As a** coordinator, **I want** the start page to show past events only when they still need me,
and the duty games of the last date with duties, **so that** done things do not take my attention
and a correction of a duty after the game stays one tap away.

#### Acceptance criteria
- [ ] The part "Last" is not shown, and no past training and no past game of the team is in the list of events.
- [ ] A past training or game of the team that is not checked shows only as an open item (as now: "Training Thu 22 Oct not checked", "Game Sat 31 Oct not checked").
- [ ] **The last duty games:** the games of the newest date before today that have places of the team and are not games of the team (home games of other club teams) show at the top of the list of events, above the coming events, under their date heading, with the blue bar and their places as in story 2. They are lighter (the date has passed). They are in the same seasons as "Last" is now: the current season, or in July and August, when it has none yet, the season before.
- [ ] They stay until a newer date before today has such games; then that date replaces them.
- [ ] The games of the team are not in the last duty games, on purpose: the drivers and places of a game of the team are corrected in "After the game" before "Checked", and until then the game is an open item. After "Checked", a correction goes through "Season".
- [ ] A tap opens the page of the game, where the places can be corrected; "Back" returns to the start page.
- [ ] A past event that is checked can still be opened from "Season" (as now: the label "Past", then the date).

#### Behaviour
On Monday after a Saturday with a home game of X14 1 where the team had the table, the list of
events starts with "Sat 14 Nov · 15:30 · Blue Drakes X14 1 – BS Leiden X14 4 · Table 1, 2: Anna,
Bram" (lighter, blue bar). A tap opens the game, where the coordinator records "Someone else" for
table 1. A late correction of a checked training goes through "Season" (the label "Past", the
date, the training).

#### Errors and edge cases
- A past duty game that left the game list, or that is marked "Not played", shows as the game
  does on its page: crossed out, or with "Not played".
- No past date with such games in those seasons: nothing shows above the coming events.
- In July and August the open items still cover the season before (as now), so the record of
  June stays in view through its open items.

### Story 4: The messages only when they have something
**As a** coordinator, **I want** the buttons of the messages on the start page only in a week
when they have something to announce, **so that** the start page has only the weekly actions of
this week.

#### Acceptance criteria
- [ ] Below the events, the start page has the link "Message of the week ›" when this week has something to announce (the same rule as now: a game of the team today or later, or table or referee places of the team). The heading "Messages" goes.
- [ ] Next to it, the link "Committee message ›" when this week (Monday to Sunday, the same week as the message) has table or referee places of the team, by the same rule as the part "Tafel en scheidsrechter:" of the message of the week. In other weeks it does not show on the start page.
- [ ] Without either, nothing shows there.
- [ ] The page "Duties" also has the button "Committee message", next to "Driving overview" and "Table and referee overview", for the current season only (like the overviews), whenever a team is chosen, also in a week without places. It opens the same page as now (`/messages/committee`).
- [ ] "Back" on the page of a message returns to the page it was opened from: the start page or "Duties".

#### Errors and edge cases
- No team chosen: no links on the start page, and "Duties" shows no committee message, as it shows no overviews.
- An earlier season on "Duties": no committee message, as no overviews.

### Story 5: A short header and fewer links
**As a** coordinator, **I want** the header and the links to show only what I use each week,
**so that** the page starts with what matters.

#### Acceptance criteria
- [ ] The header has the team name ("Blue Drakes M16 1") and today's date ("Tuesday 10 November"). No "Your team · 2026-2027", no "Change".
- [ ] In July and August, when the open items also cover the season before, the header also shows the season of the team ("Blue Drakes M16 1 · 2026-2027"), so the coordinator can tell the seasons apart.
- [ ] At the bottom of the page: Season · Attendance · Duties · Report · More.
- [ ] "More" opens a new page "More" with the links Team, Players and Settings. "Back" returns to the start page.
- [ ] The page "Settings" has "Change club or team", which opens the choice of club and team that "Change" opened before.
- [ ] Without a club chosen, and in a new season without a team, the choose step shows at the top of the start page, as now.

#### Errors and edge cases
- Without a club chosen, the start page has the choose step and the links, as now (with "More").
- In a new season without a team, the header shows only today's date; the choose step is below it.

## Interface
- Screens: the start page (`/`), changed; the page "More" (new); the page "Duties" (`/duties`),
  with the button "Committee message"; the page "Settings", with "Change club or team".
- Words on the screen: "Nothing needs you.", "Home game against …", "Away game at …",
  "Message of the week ›", "Committee message ›", "More", "Change club or team".
- The messages themselves (their text) do not change.

## Consistency
- The rules of the open items, of the range of the events, of when the message of the week is
  offered, and of the page of a game do not change. The rule of the last duty games is the part of
  the rule of "Last" for games with places of the team that are not games of the team.
- "Season" keeps its current look in this change (an idea in `docs/ideas.md`), so for a while the
  start page and "Season" look different. Both still use the team colour for the games of the
  team.
- "Home game" and "away game" are the words of `docs/language.md`.
- The orange dot of an open item and the orange background of a game of the team share the team
  colour; one is a dot, the other a background.

## What changes for existing users
- "Last" goes, except the duty games of the last date with duties; past checked events are on "Season".
- The committee message shows on the start page only in weeks with table or referee places of the
  team; it is also on "Duties".
- "Change" moves to "Settings"; Team, Players and Settings move under "More".
- No data changes, no migration.

## Warnings from architect and engineer
- The row of an event is shared by the start page and "Season". The new look must not change
  "Season" in this change.
- The start page server address and its product rules still produce the data of "Last" (the
  last training, and the last games also of the team). After this change only a part is read;
  the rest must be decided, not left behind.
- The rule for when the committee message is offered changes: on the start page it follows the
  places of this week; on "Duties" it is always offered for the current season.
- At Close, these product documents must be updated too, because they name the old header, the
  old line of links, "Last" or "Next", or where the buttons of the messages are:
  `choose-club-and-team.md`, `season.md`, `players-and-team.md`, `trainings-and-attendance.md`,
  `weekly-trainings-and-reasons.md`, `duties.md` and `messages.md`.
- "Nothing left to do" decides the look of a row. It must be one rule, with its own tests, not a
  choice made separately in each row.
- Six browser test files use "Last" or reach the committee message from the start page; they
  change with the page.
- Tap targets stay at least 44 pixels high; the lighter text, the bars and the background must
  be readable in light and dark mode.

## Not in scope
- The look of "Season" (an idea for later).
- Changes to the rules of the open items, or new kinds of open items (a duty game is never an
  open item; that would need a new idea).
- Changes to the text of the messages.
- A bottom navigation bar like a phone app (mockup C).

## Open questions
- None for now.

## Spec review
Reviewed 2026-11-10 by `/review spec` (fresh agent). Read: this spec, `change.md`, `docs/vision.md`,
`docs/language.md`, and the product docs start-page, season, duties, messages, game-attendance,
trainings-and-attendance, report, choose-club-and-team, weekly-trainings-and-reasons, principles.
Not read: `technical.md` (none yet), the mockups (private artifact), the code. There was no idea.md
(type `review`).

| Story | What the user is doing here | Verdict | Note |
|---|---|---|---|
| 1 Open items as quiet rows | Opening the tool a few times a week, seeing at a glance "does something need me?" (journey 0) | serves | Only the look changes: the kinds, the order and the taps stay the same. One orange dot per row fits "one look of a few seconds". Checked against the start-page doc, "Open items". |
| 2 Coming events as one quiet list | The same look, for "what comes next?", on a phone, with one hand | serves – note | (a) The rule "nothing left to do" is clear and complete for the kinds in "Next". It sends no thing that needs the coordinator to grey (a game of the team is lighter only when checked or not played; a duty game with an open place stays normal). Checked by tracing each row kind in the start-page and season docs. (b) A home game of another club team drops its hall and town. That is fine (it is the club's own hall), but the spec should say so on purpose, not by leaving it out. (c) The rows lose the badge "Team", which "Season" and the page of a game still use. The spec accepts this under "Consistency". |
| 3 No part "Last" | Correcting the record after the events: the attendance of the last training and the duty places of the last weekend | serves – note | The Analysis says "a past thing that is not done is already an open item". That is **not true for duties**. Duties are never checked (word "checked"). A past home game of **another** club team with table or referee places of the team is not an open item of any kind: "open places" covers only coming games, and "games not checked" covers only games of the team. Today "Last" is the one tap to such a game, for the correction "Someone else" / "Nobody" after the game. Those corrections feed the duty totals and the report (duties.md, "Done and planned"), so they serve core value 2 and the purpose directly. Without "Last" the path is: Season, the label "Past" (Season opens with "Coming" only, season.md "Labels"), find the date, the game. So the Behaviour line "through 'Season' (one tap from the start page), then the training" is also wrong: it is at least three taps plus a search. The coordinator may still choose to drop "Last", but Discover should state this cost and decide it, or keep a quiet way to reach the past weekend's duty games. Checked by a trace through start-page.md (items 5 and 8, "Last"), duties.md and season.md. Whether the coordinator really corrects duties after weekends: not checked (only the user knows). |
| 4 The messages where they belong | Before the weekend, copying the message of the week for the team (journey 4) and the committee message for the referee committee (journey 5) | serves – note | The message of the week stays one tap away, as journey 4 says. But the spec calls the committee message a "rare action". Journey 5 of the vision makes it a weekly one ("Before the weekend, the coordinator copies a message for the referee committee"). The move adds a tap each week with places of the team. That is acceptable, and the vision does not say where the button lives. But the story's reason ("the start page has only the weekly action") is not accurate. Another option for Discover: offer it on the start page only in a week with table or referee places of the team, as the week message already checks. Checked against vision.md journeys 4–5 and messages.md "Where". |
| 5 A short header and fewer links | Opening the page and starting at what matters; now and then changing roles, players, settings, or the team | serves – note | Rare actions one level down fits journey 0. Notes: (a) the header drops the season. In July and August the open items also cover the season before (start-page.md, "Which dates count"), and then the season label helped to tell them apart. (b) "Change club or team" ends three taps deep (More, Settings, Change). That is fine for a choice made once a season, because the new season shows the choose step by itself. (c) choose-club-and-team.md and the "Season" doc ("first in the line …") name the old header and link line. Close must update them. Checked by reading those docs. |

**The purpose.** The claim holds in part. A quieter start page speeds up journey 0 and the fast
capture that both core values need. But this is supporting work (rule 8), and it is not small: four
pages, a new page, and six browser test files. One part of it (story 3) takes away a short path to
correcting duties, which is where the purpose lives. No conflict with the vision.

**Verdict for the spec: serves – note.** Before Design, Discover should settle story 3 (the
correction of the duty games of the past weekend) with the coordinator, and fix the "one tap"
line. The notes on stories 2, 4 and 5 can be handled in the spec text or accepted as they are.

**Answers of Discover (2026-11-10, with the coordinator).** Story 3: the coordinator chose to keep
the duty games of the newest past date at the top of the list (lighter), so a correction after the
game stays one tap away; the Analysis and the line about "one tap" are corrected. Story 4: the
committee message shows on the start page only in a week with table or referee places of the
team, and is always on "Duties". Story 2: the missing hall and town of a home game of another
club team is now said on purpose. Story 5: the header shows the season in July and August; Close
updates `choose-club-and-team.md` and `season.md`. The rows of the spec review above are for the
version before these answers.

### Second spec review (2026-11-10, after the answers of Discover)
Reviewed by `/review spec` (fresh agent), on the version with the answers above. No extra text
in the arguments. Read: this spec, `change.md`, `docs/vision.md`, `docs/language.md`,
`product-workflow`, and the product docs start-page, season, duties, messages,
choose-club-and-team, principles (plus searches in the other product docs). Not read:
`technical.md` (none yet), the mockups (private), the code.

| Story | What the user is doing here | Verdict | Note |
|---|---|---|---|
| 1 Open items as quiet rows | A look of a few seconds: "does something need me?" (journey 0) | serves | Only the look changes; kinds, texts, order and taps stay. Checked against start-page.md, "Open items". |
| 2 Coming events as one quiet list | "What comes next?", on the phone, and which coming thing still needs work | serves – note | (a) **The rule "nothing left to do" misses "Schedule not entered".** A home game of another club team with all places of the team filled is lighter, but its committee schedule can still be not entered: places can be chosen without the tap "Schedule entered" (duties.md, "Schedule entered"; season.md shows "Table 1, 2: … · Schedule not entered"). That game is then open item 4 at the top and a grey row below, two answers to one question. Discover should add "and the schedule is entered" to the rule (or say on purpose that it is not part of it). Checked by a trace through start-page.md item 4, duties.md and season.md, "A game". (b) "The same text as now" for the places of a duty game: say whether "· Schedule not entered" stays on the row (season.md has it; the spec's examples do not). (c) The edge case on a warning ("an open place in the coming days") is covered by the rule itself; fine. |
| 3 Past events only when they need me | Correcting the record after the weekend: the duty places of last Saturday | serves – note | The earlier note is answered for home games of other club teams. Still: **the rule now drops the games of the team** that the old "Last" had (drivers of an away game, places at a home game of the team). While such a game is not checked it is an open item, so one tap (for more than one: the oldest). Once checked, a later driver correction ("Someone else" drove) goes through "Season", "Past", the date, as for a checked training. That is a fair trade, but the story's "so that a correction of a duty after the game stays one tap away" holds only for duty games of other club teams; the spec should say the games of the team are left out on purpose. Checked by comparing start-page.md "Last" (places "that name the team, or drivers") with the new criterion ("are not games of the team"). Whether driver corrections happen after "Checked": not checked (only the user knows). Small: the past rows stand above today's rows under only a date heading; lighter text and the date tell them apart, so no label is needed. |
| 4 The messages only when they have something | Before the weekend, copying the message of the week (journey 4) and the committee message (journey 5) | serves | The committee message stays one tap away in each week that has places, and is always on "Duties". "By the same rule as 'Tafel en scheidsrechter:'" makes it one rule. Side effect, acceptable: that part also covers days of the week that have passed, so on the Sunday after a Saturday with places the link still shows. Checked against messages.md, "The text" and "Where". |
| 5 A short header and fewer links | Starting at what matters; now and then roles, players, settings, the team | serves – note | Fits journey 0; the season in July and August answers the earlier note. "Change club or team" in "Settings" (the page of weekly trainings and reasons) is three taps deep; fine for a choice made once a season. Close must update more docs than the Warnings name: besides `choose-club-and-team.md` and `season.md`, also `players-and-team.md` and `trainings-and-attendance.md` (they quote the line of links), `weekly-trainings-and-reasons.md` ("linked from the start page"), `duties.md` (the page of a game opens from "Next" or "Last") and `messages.md` ("Where", and "Back returns to the start page"). Checked by a search for "Team · Players · Settings", "Last" and "start page" in `docs/product/`. Not stated: the header in a new season without a team. |

**The purpose.** The claim holds: the change speeds up journey 0 and the fast capture both core
values need, and story 3 now keeps the correction of duties, which feeds the duty totals and the
report, one tap away for the duty games of other club teams. It is still supporting work of some
size (rule 8): five stories, a new page, changes to "Duties" and "Settings", six browser test
files. No conflict with the vision.

**Verdict for the spec: serves – note.** Before Design: fix the rule of "nothing left to do" for a
game whose schedule is not entered (story 2a), and state on purpose that the games of the team
are not among the last duty games (story 3). The other notes can go into the spec text or the
list for Close.

**Answers of Discover to the second review (2026-11-10, with the coordinator).** Story 2: a home
game of the club is lighter only when its places of the team are filled and its schedule is
entered; until then the row shows "Schedule not entered". Story 3: the games of the team are not
in the last duty games, on purpose (the coordinator corrects drivers before "Checked"). Story 5:
the header in a new season without a team. Close: the five more product documents are listed in
the warnings. The committee message on the Sunday after the places is accepted.
