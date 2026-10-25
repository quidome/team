# Functional spec: Duties

**Status**: approved
**Change**: [2026-10-18-duties](./)
**Date**: 2026-10-25 (replaces the version approved on 2026-10-18: the rules stay, the screens change)

## Context
The coordinator runs one youth team of the club. Besides the games and the trainings, the team
has two kinds of **duty**, and today the coordinator keeps them in a Google Sheet and writes the
messages about them by hand:

- **Jury and referee duty.** Every few months (irregular, a few times per half year) the club's referee committee gives the **team** (not persons) a set of duty places at **home games of other teams of the club**, never at the team's own games: for each such game 1 or 2 referee places and 1 or 2 jury places (the table: the clock and the game administration on the tablet). The committee's list names each game. In October 2026 the coordinator knows all places until the end of the calendar year. The team then divides those places among its players; to make that fair, the coordinator must see who already did what.
- **Driving.** For each away game of the team, the team needs one car for every 3 players who go. Parents drive, but the duty is given to the player (a principle of the product).

The coordinator also writes two messages by hand: before each game, one for the team (time,
place, who drives, who has jury or referee duty); before each weekend, one for the referee
committee (which player does which duty at which game). The tool only produces text to copy; it
never sends (a principle of the product).

The tool already has (see `docs/product/`):
- the games of the club from the game list: the games of the team and the home games of other club teams (the "possible duty games"), on the page **"Season"** with labels and a heading per week ([season](../../product/season.md), [schedule](../../product/schedule.md));
- the **start page** with "Open items" (what still needs the coordinator), "Last" (the last training) and "Next" (the games of the team and the trainings of this week and next week) ([start page](../../product/start-page.md)); each open item comes from the rule of the page it belongs to (decision 0022);
- the players of the club and the team of each season, with each player's role ("plays games" or "only trains") from a date; the list of changes to games; attendance at training with "Checked". Attendance at games does not exist.

This is an **epic of four stories**, built in this order (coordinator, 2026-10-18): A duty places,
B who does each duty place and the duty totals, C the message for the next game, D the message for
the referee committee. After it, the sheet is no longer needed for duties.

The jobs and their places (coordinator, 2026-10-25): **entering the committee's list** happens a
few times per half year, on "Season"; **setting and filling the places of one game** on the page
of that game; **seeing what is still open** on the start page (open items); **copying a message**
from the start page; **the totals** on a page "Duties", as "Attendance" has the totals of the
trainings.

Facts from the coordinator (2026-10-18):
- Only players with the role "plays games" get duties.
- When choosing a player, the tool shows the duty totals; it never proposes a player.
- Fairness is judged for each kind apart: drive, jury, referee. The totals that help are per kind, for the current season, done and planned.
- A duty counts as done by the player on its duty place, unless the coordinator corrects it.
- The number of drivers is proposed (one per 3 players), and the coordinator can change it; a changed number stays.
- The messages are in Dutch; the screens stay in English.
- The jury and referee part of both messages covers the coming week.
- Hidden players are offered for duties like the others.
- A driver proposal that drops by itself never removes a chosen driver.
- A game whose date has passed was played: its duties stay done, also when it leaves the game list later.
- Another team chosen in the middle of a season: the duties of the old team stay stored; coming ones no longer need duties of the team and do not count; past ones count; choosing the old team again brings them back.

Facts from the coordinator (2026-10-25):
- A possible duty game is **handled** when the coordinator saved its places, also as none.
- The open items of duties cover the coming **4 weeks**; "Season" shows all games.
- The open items of duties come after the changes not seen, before the trainings.
- The totals are on a page "Duties"; the messages in a part "Messages" on the start page.
- A possible duty game without places for the team is handled with a save of 0 and 0, one by one: the club has few home games of other club teams a week (answer to spec review S10).
- Drivers become an open item only in this week and next (the window of "Next"); jury and referee places in the 4 weeks (S16).
- Duties of an earlier season can be changed and corrected, as trainings of an earlier season can (S13).
- The choice of a player shows a clash next to the name, without blocking it (S14).

## User stories

### Story A: Duty places
**As the** coordinator, **I want to** record the duty places of the team at each game, **so that**
the tool knows what must be filled, without the sheet.

#### Acceptance criteria
- [ ] **The page of a game** opens with a tap on a game on "Season", in "Next" or "Last" on the start page, on a duty open item for one game, or on a change in the list of changes. It shows the date, start time ("time not known yet" when not known), home team, away team, hall and town, and its duty places. "Back" returns to the page it came from; to "Season" with the same labels and place in the list, as from a training, so the coordinator can walk through a filtered list.
- [ ] At a **possible duty game** (a home game of another club team, coming or past), the coordinator sets the number of referee places and of jury places, each 0, 1 or 2, and saves. After a save the game is **handled**, also with 0 and 0; the page then says "No duty places for the team." Before any save it says "Not handled yet."
- [ ] At an **away game of the team** (a game at another club), the page shows the number of drivers. While the coordinator has not changed it, it is the proposal: one for every 3 players with the role "plays games" on the game's date, rounded up (10 players: 4 drivers; 0 players: 0), marked "proposed", and it follows the team when players join or leave. The coordinator can set another number (0 to 10); from then on that number stays, without "proposed". "Use proposal" goes back to the proposal. When the proposal drops by itself (a role changes) below the driver places already filled, the number stays at the filled places, with "more than proposed"; nobody is removed. An away game is always handled.
- [ ] A **home game of the team** has no duty places; its page says so and offers nothing to set.
- [ ] A number of places below the number of filled places is refused: "2 places are filled. Clear one first."
- [ ] **On "Season"** each game that can have duties shows its places on its row: "Referee: open · Jury: Anna, open", "Drivers (4, proposed): Anna, 3 open", "No duty places", or "Not handled yet". A game whose places are all filled shows "all filled".
- [ ] **"Season" gets the label group "Duties"**: "Handled", "Not handled", "Open places". They follow the rules of the other groups (labels of different groups narrow, labels of one group add). They apply only to possible duty games and away games of the team: with a Duties label on, home games of the team and trainings do not show. To enter the committee's list the coordinator uses "Possible duty games" + "Not handled".
- [ ] **A new open item on the start page**: "Possible duty game Sat 31 Oct not handled" for one, "5 possible duty games not handled" for more: the possible duty games from today to 4 weeks ahead that are not handled. A tap opens the game; for more than one, a tap opens the list of those games below the item (date, start time, the game), each a tap to its page, so the item and its list always hold the same games.
- [ ] **"Next"** on the start page also shows the possible duty games where the team has duty places, with their places, as on "Season".
- [ ] **"Last"** on the start page also shows the last game before today with duty places, as on "Season", so a correction after the weekend is one tap (story B).

#### Behaviour
The coordinator gets the committee's list in the chat and opens "Season" with "Possible duty
games" and "Not handled": a tap on a game, "Referee 1, Jury 2", save, back; the next game. A game
of the list without places for the team: "Referee 0, Jury 0", save. The open item "5 possible
duty games not handled" shrinks while the coordinator works. For an away game the page already
shows "Drivers: 4 (proposed)". Places can be set for past games too, to enter what already happened
this season.

#### Errors and edge cases
- A game that moves (another date, time or hall) keeps its duty places and the players on them. Its change in the list of changes says "has duties" with the filled places (for example "Jury: Anna, open"), so the coordinator can tell the players and the committee.
- A coming game (today or later) that leaves the game list keeps its duty places while it is gone; on its page and on its row on "Season" it shows "no longer in the game list". Its duties do not count, and it is no open item, while it is gone. If it comes back, they count again.
- A game whose date has passed was played: its duties stay done, also when it leaves the game list later. Its page says "no longer in the game list".
- A game that stops being a home game of another club team (for example the home and away teams swap, or it becomes a game of the team) keeps its places, with "This game no longer needs duties of the team." on its page and on its row, and its places do not count and are no open item. Setting the places to 0 removes them.
- Another team chosen in the middle of a season (rare): the duties of the old team stay stored, and the same rule applies: coming ones show "This game no longer needs duties of the team." and do not count; past ones count. Choosing the old team again brings them back.
- A game of an earlier season can be opened (from "Season" of that season, or from "Last" in July and August), and its duty places and players can be changed and corrected, as trainings of an earlier season can. Its duties count in the totals of that season.
- A possible duty game more than 4 weeks ahead is no open item; "Season" shows it as "Not handled yet".

### Story B: Who does each duty place, and the duty totals
**As the** coordinator, **I want to** put a player on each duty place and see what each player has
done, **so that** I can divide the places fairly and the totals are right.

#### Acceptance criteria
- [ ] On the page of a game, a tap on a duty place opens the choice of a player: the players with the role "plays games" on the game's date, hidden players too (hidden only keeps lists short, as at a training), sorted by first name. Next to each name, that player's totals for that kind in the current season: "2 done · 1 planned". The tool marks no player as proposed.
- [ ] A tap on a player puts the player on the duty place and saves at once. "Clear" makes the place open again.
- [ ] A player who already has a duty place at the same game is not offered for another place at that game. A player with a clash at the same time is offered with a note next to the name: "also Jury 13:00" (a duty place at another game that overlaps) or "plays 14:30" (a game of the team that overlaps); the coordinator can still choose the player.
- [ ] A filled duty place of a game before today (Dutch date) counts as **done** by its player. A filled duty place of today or later counts as **planned**. Open places count for nobody.
- [ ] On a past game, the coordinator can correct a duty place: another player ("Someone else": the same choice), or "Nobody" (the place was not done; it counts for nobody and shows "not done (was Anna)", so the planned player stays visible). The correction replaces what counts.
- [ ] **A new open item on the start page**: "Away game Sat 31 Oct: 2 drivers open", "Possible duty game Sun 1 Nov: 1 jury place open", or for more than one game "3 games with open places": the possible duty games from today to 4 weeks ahead with open jury or referee places, and the away games of the team in this week and next (the window of "Next") with open driver places, that still need duties of the team. A tap opens the game; for more than one, a tap opens the list of those games below the item, each a tap to its page.
- [ ] **The page "Duties"** (new; the links of the start page become "Season · Attendance · Duties · Team · Players · Settings") has a table "Totals": one row for each player with the role "plays games" in the season or with a duty in it (also when the role starts later), sorted by first name, and the columns Drive, Jury and Referee, each "done · planned" (for example "3 · 1"). "Done" is the duty total of the word list; "planned" is shown apart from it. Players who left the team stay in the table while they have duties in the season; hidden players have rows like the others. A season choice, as on "Attendance".
- [ ] What counts: duties of coming games that are in the game list and still need duties of the team, and duties of past games (also when they left the game list later); see story A. The totals next to the names in the choice and the table on "Duties" give the same numbers.
- [ ] Two players with the same first name show the first letter of the last name in the choice, on "Season", on the start page and in the message for the next game ("Anna B.", "Anna V.").

#### Behaviour
When a new set of places from the committee is entered, the open item "4 games with open places"
shows. The coordinator taps it, gets the list of those games, opens each game, and for each
place taps a player, looking at the totals next to the names. The team usually agrees in the chat
first; the coordinator records the result. After a weekend nothing needs to be done unless
something went otherwise: then a tap on the game under "Last", a tap on the place, "Someone else"
or "Nobody".

#### Errors and edge cases
- A player is no longer offered for duty places on dates when the player has no role "plays games" (left the team, or only trains). A player already on such a place stays on it, with "Not in the team on this date" next to the name; the coordinator can clear or change it.
- A player can be deleted only when the player has no absences (existing rule) and no duties; else "Delete player" is not offered (as now for absences).
- Two screens: the last save wins. A duty place never holds two players.
- A save that fails shows "Not saved. Try again." and the place shows what is stored.
- A driver proposal that rises by itself (a player joins the team) opens new driver places; the game can become an open item without a save.
- "Duties" with no duties in the season: "No duties yet this season."

### Story C: The message for the next game
**As the** coordinator, **I want to** copy one message about the team's next game, **so that** the
team and the parents know when and where, and who drives or has jury or referee duty.

#### Acceptance criteria
- [ ] The start page has a part "Messages" below "Next". It shows "Message for the next game" when the team has a coming game (today or later), or jury or referee duties in the coming week. A tap opens the message: Dutch text, with a "Copy" button that puts it on the clipboard and says "Copied".
- [ ] The message has the date and start time of the team's next game, home ("Thuis") or away ("Uit"), the opponent, the hall and the town; for an away game the drivers by first name, with open places as "nog N open"; and the jury and referee duties of the team in the coming week (from today until the next Sunday, today itself when it is a Sunday), whether or not the team plays that week, each with the date, start time, the game, and the players by first name. Two players with the same first name show the first letter of the last name ("Anna B.", "Anna V.").
- [ ] A start time not known yet shows "tijd nog niet bekend".
- [ ] The text is built from what is stored at that moment; the coordinator may edit it after pasting in the chat.

#### Behaviour
Example for an away game:

```
Volgende wedstrijd: zaterdag 31 oktober, 14:30
Uit tegen B.Z. '72 M16 1
Sporthal Bulwijk, Woerden

Rijden: Anna, Bram, Cas (nog 1 open)

Jury en scheidsrechter deze week:
za 31 okt 16:00 Blue Drakes X12 1 – De Dunckers X12 1
  Jury: Eva, Fleur
  Scheidsrechter: Gijs
```

For a home game the line "Rijden" is left out. Without jury or referee duties in the coming week,
the part "Jury en scheidsrechter deze week" is left out. Without a coming game of the team (a free
week, or after the last game), the message has only that part.

#### Errors and edge cases
- No coming game of the team and no duties in the coming week: the button is not shown.
- The next game is no longer in the game list: the message is for the next game that is.
- Copying fails (the browser refuses): the text stays shown and selectable, with "Select the text and copy it."

### Story D: The message for the referee committee
**As the** coordinator, **I want to** copy one message with the team's jury and referee duties of
the coming week, **so that** the referee committee knows which player does which duty.

#### Acceptance criteria
- [ ] The part "Messages" on the start page has "Message for the referee committee". It opens the message for the coming week: Monday to Sunday of the week of the coming weekend (on a Saturday or Sunday: this week), so duties on a weekday are in it too. "Next week" and "Previous week" move one week.
- [ ] The message lists each game of that week with jury or referee places of the team, in time order: the date, start time, home team – away team, then for each kind the players by **first and last name**; open places show "nog open".
- [ ] A week without duties: "Geen jury- of scheidsrechtersdiensten deze week." and the button "Copy" still works.
- [ ] Copying works as in story C.

#### Behaviour
Example:

```
Jury en scheidsrechters Blue Drakes M16 1, week 26 oktober – 1 november

za 31 okt 13:00 Blue Drakes X12 1 – De Dunckers X12 1
  Scheidsrechter: Eva de Vries
  Jury: Fleur Jansen, Gijs Bakker

zo 1 nov 11:00 Blue Drakes M14 2 – Lions M14 1
  Scheidsrechter: nog open
  Jury: Hugo Smit
```

#### Errors and edge cases
- A player with no last name shows the first name only.
- A game that is no longer in the game list, or no longer needs duties of the team, is left out of the message.
- Copying fails: as in story C.

## Interface
- **The page of a game** (new, `/game/...`): the game, its places, the choice of a player, "Use proposal", "Someone else", "Nobody".
- **"Season"**: the places on each row of a game that can have duties; the label group "Duties" ("Handled", "Not handled", "Open places"); "Back" from a game keeps the labels and the place.
- **The start page**:
  - "Open items", in this order: the game list; changes not seen; possible duty games not handled; games with open places; training days not opened; trainings not checked;
  - "Last" also with the last game with duty places;
  - "Next" also with the possible duty games where the team has places;
  - a part "Messages" below "Next", with "Message for the next game" and "Message for the referee committee";
  - the links "Season · Attendance · Duties · Team · Players · Settings".
- **"Duties"** (new, `/duties`): the table "Totals", with a season choice.
- **Two message pages**, each with Dutch text and "Copy"; the committee message with "Next week" and "Previous week".
- Server addresses follow in the design.

Example of the start page (a Monday, one week after a committee list):

```
Open items
  3 possible duty games not handled
  Away game Sat 31 Oct: 2 drivers open
  Training Thu 22 Oct not checked

Last
  Sat 24 Oct   13:00 Blue Drakes X12 1 – Lions X12 1 · Jury: Eva, Fleur
  Thu 22 Oct   Training 19:45 · 2 absent

Next
  Tue 27 Oct   Training 19:45 · Not opened
  Sat 31 Oct   14:30 B.Z. '72 M16 1 – Blue Drakes M16 1 · Drivers: Anna, Bram, 2 open
               16:00 Blue Drakes X12 1 – De Dunckers X12 1 · Jury: Eva, open

Messages
  Message for the next game
  Message for the referee committee

Season · Attendance · Duties · Team · Players · Settings
```

## Consistency
- The rules of "Season" stay: which games, the order, gone games, the labels and how they combine, labels not kept between visits (a return from a game, as from a training, keeps them once).
- The open items follow decision 0022: each comes from the rule of its source (here: the rule of "handled" and of open places that the page of a game and "Season" use); nothing is counted on the side.
- The choice of a player works like marking at a training: a tap saves at once, "Not saved. Try again." when it fails, the stored state shown again.
- Players: only the role "plays games" on the date counts, as roles decide who is at a training; hidden players are offered.
- Totals are worked out each time from what is stored, as the attendance totals are; "Duties" is to duties what "Attendance" is to trainings.
- The list of changes keeps its behaviour; a change to a game with duties also says "has duties".

## What changes for existing users
- New: the page of a game, the page "Duties", the duty labels on "Season", the duty places on the rows of "Season" and in "Next" and "Last", two kinds of open items, the part "Messages" on the start page.
- The links of the start page become "Season · Attendance · Duties · Team · Players · Settings".
- A tap on a game (on "Season", in "Next", in the list of changes) now opens its page; before, it did nothing.
- "Delete player" is also not offered for a player with duties.
- No data to move: there are no duties yet. The coordinator enters this season's duties (also past ones) by hand. All possible duty games start as "not handled"; only those of the next 4 weeks are open items.

## Warnings from architect and engineer
- **Games move and disappear.** Duty places are entered weeks or months ahead; the game list can move a game, swap home and away, change the team, or drop the game in that time. Every such case must keep the duty places and say clearly what happened (story A), and the totals and open items must never count a duty of a game that is gone or no longer needs duties.
- **One rule everywhere (decision 0022).** The totals next to the names, the table on "Duties", and later an overview for the parents must give the same numbers from the same rule. "Not handled" and "open places" must be the same rule on "Season", on the page of a game and in the open items. The horizons (4 weeks for jury and referee places, this week and next for drivers) belong only to the open items, and the list an item opens holds the same games.
- **The driver proposal moves.** While not changed, it follows the players with the role "plays games" on that date; a role change can change the number after drivers were chosen, and can make a game an open item without a save. Filled places above a lower proposal must not disappear.
- **The start page gets heavier**: games with places, the players on them, and the roles for the proposal. It must stay one request and fast.
- **Two screens.** One coordinator, but a phone and a computer can save the same duty place; one place never holds two players, and a choice must not be lost without a sign.
- **Dates, weeks and weekends** follow Dutch time.

## Not in scope
- Attendance at games (it may later make the driver proposal more exact).
- A departure time for away games in the message (the coordinator adds it by hand when needed).
- The overview for the parents with duty totals (journey 6; later).
- Recording which team of the club covers which places (the club's own division; idea of 2026-09-27).
- Proposing a player for a duty place.
- Messages in other languages, and templates the coordinator can edit (idea of 2026-10-10).
- Duties at the team's own home games, and duties other than drive, jury and referee.
- Totals over more than one season.
- A date "committee list entered until" (the horizon of 4 weeks is used instead; coordinator, 2026-10-25).

## Open questions
- For `/vision`: a word for the set of duty places the referee committee gives the team for a period (the coordinator says "shifts" and "a wave"); this spec writes it out in plain words.
- For `/vision`: the phrase "possible duty games" (a label on "Season" now) now that "duty place" exists; and "handled" as a word for a possible duty game whose places were saved.

## Spec review
<Filled in by `/review spec` (a fresh agent). One row per user story. Design does not start
without this table. A "does not serve" row sends the spec back to Discover.>

| Story | What the user is doing here | Verdict | Note |
|---|---|---|---|
| A Duty places | A few times per half year: walking the committee's list through "Season" and saving each game's places (also "none"); now and then: setting the drivers of an away game; each visit: seeing on the start page which coming games are not handled yet | serves – note | S10, S11, S12, S13 |
| B Who does it, totals | Putting the player the chat agreed on onto each place, with the totals per kind in view; after a weekend that went otherwise, one correction from "Last"; now and then: showing that the division is fair ("Duties") | serves – note | S11, S14, S15, S16 |
| C Message for the next game | Around training: copying one message for the team and parents (when, where, who drives, who has jury or referee duty this week) | serves – note | S17 |
| D Message for the referee committee | Before the weekend: copying the week's jury and referee duties with full names for the committee | serves | – |

Reviewed 2026-10-25 by a fresh agent (`/review spec`), against `docs/vision.md`,
`docs/language.md`, `idea.md` and `docs/product/` (start page, season, schedule, list of changes,
players and team, trainings and attendance), decision 0022, and the version of this spec approved
on 2026-10-18 (`git show HEAD:`) for what the rewrite kept or lost. The arguments held only the
mode and the change folder; nothing was ignored. The notes are numbered from S10 so they do not
mix with S1–S9 of the first review, which the log of `change.md` names. No story conflicts with the
vision: journey 0 gets its duty open items, journeys 3–5 are served, the principles hold (duties to
players, text only, one coordinator). S10 is the one to settle before design; the rest are gaps.

- **S10 – "Handled" asks a save of "0, 0" on every possible duty game the list does not name.** The committee gives the team places at a few games, but "possible duty games" are all home games of all other club teams, and each one stays "Not handled yet" on its row on "Season" and counts in the open item until the coordinator opens it and saves 0 and 0. So entering a list of, say, 8 places means also opening every other home game of the club in that period, and again each time the game list adds one. If that is many games, the walk-through of story A costs far more than copying the list, against core value 1 ("capture fast"), and the open item "N possible duty games not handled" stays on the start page as noise the coordinator learns to ignore. The spec has no way to handle several games at once (for example "No places for the team" on all not-handled games of a week). Discover should know how many possible duty games a typical week has, and decide whether that is acceptable. Checked: trace of story A (criteria 2, 6, 7, 8 and "Behaviour") against `docs/product/season.md` (which games show) and `list-of-changes.md` ("5 or more new possible duty games show as one line", which suggests many); the real number per week is not checked (the local database is empty).
  - 2026-10-25: Answered (coordinator): one by one is fine; the club has few home games of other club teams a week. A save of 0 and 0 handles a game (facts of 2026-10-25).
- **S11 – The open item counts 4 weeks; the list it opens shows the whole season.** "5 possible duty games not handled" opens "Season" with "Possible duty games" and "Not handled", and "3 games with open places" opens "Season" with "Open places" and "Coming"; neither list stops at 4 weeks. "Open places" also holds every coming away game whose drivers are not chosen yet, which in October is every away game until May. The coordinator taps "3 games…" and gets a list of perhaps 20, which is the "two numbers for one fact" that decision 0022 wants to avoid. Either the address also carries the horizon, or the list shows where the 4 weeks end, or the item's text says "in the next 4 weeks". Checked: trace of story A criterion 8 and story B criterion 6 against criterion 7 of story A and the warning "The horizon of 4 weeks belongs only to the open items"; decision 0022, Context.
  - 2026-10-25: Fixed: a tap on an item for more than one game opens the list of those games below the item, each a tap to its page; the item and its list hold the same games (stories A and B).
- **S12 – The walk-through needs "Season" to keep its labels and place on "Back" from a game.** Story A's behaviour is "tap a game, save, back; the next game" on a filtered list. `season.md` keeps labels and place only on a return from a training, and labels are otherwise not kept between visits. The spec says "Back returns to the page it came from" but not that "Season" comes back with the same labels and place; without that, each game costs three taps to set the filter again. Checked: search `docs/product/season.md` for "Labels are not kept"; trace of story A "Behaviour" and the Consistency section.
  - 2026-10-25: Fixed: "Back" from a game to "Season" keeps the labels and the place, as from a training (story A, Consistency).
- **S13 – Duties of an earlier season cannot be changed, unlike trainings.** In July and August the start page still covers the season before (open items, "Last"), and a training of an earlier season can still be marked and changed ("the last training of May found in July"). For duties, "Last" in early July can show the last game of June, but a correction there is refused ("Duties of an earlier season cannot be changed."). That is rare, but it breaks the one-tap correction of story B and differs from trainings without a reason given. Checked: search `docs/product/trainings-and-attendance.md` ("Earlier seasons") and `start-page.md` ("in July and August"); story A last-but-one edge case.
  - 2026-10-25: Answered (coordinator): duties of an earlier season can be changed and corrected, as trainings (story A).
- **S14 – A player in two places at once is not flagged.** Only "the same game" is excluded. The club's hall has several courts, so two home games of other club teams can be at the same time, and a jury place can be at the hour the team plays an away game; the choice offers that player with no sign. The spec does not have to block it, but the coordinator would then find the clash only in the chat. Whether the choice shows "also Jury 13:00" or "plays away 14:30" next to the name is for Discover to decide; this note only says the spec is silent. Not checked against the committee's real lists (they may never overlap).
  - 2026-10-25: Answered (coordinator): the choice shows "also Jury 13:00" or "plays 14:30" next to the name, without blocking (story B).
- **S15 – A player who joins later can be planned but has no row in "Totals".** The choice offers players with "plays games" on the game's date, so a player who joins on 1 December can be put on a place in December. The table on "Duties" has a row only for players with the role "today or earlier in the season", so that player's "planned" shows in the choice but not in the table, against "the same numbers" in criterion 7. Checked: trace of story B criteria 1, 6 and 7.
  - 2026-10-25: Fixed: the table has a row for each player with "plays games" in the season or a duty in it, also when the role starts later (story B).
- **S16 – Who is asked to drive.** A small check, not a gap: the driver proposal counts all players with "plays games", not those who go (attendance at games is out of scope), and an away game 4 weeks ahead with drivers not chosen yet becomes an open item at once. If drivers are usually agreed in the week of the game, the item "N games with open places" is on the start page nearly always and stops meaning "needs you now". The 4 weeks are a fact from the coordinator; it is not clear that it was meant for drivers as well as for the committee's places. Checked: trace of story B criterion 6 with story A criterion 3; the facts of 2026-10-25.
  - 2026-10-25: Answered (coordinator): drivers are an open item only in this week and next; jury and referee places in 4 weeks (story B).
- **S17 – The message for the next game lost the first letter of the last name.** The approved spec said two players with the same first name show "Anna B." "in the choice and in the message for the next game" (answer to S8). The rewrite lists "the choice, "Season" and the start page", and story C says "by first name" only, so two drivers called Anna read as "Rijden: Anna, Anna". Checked: search for "Anna B" in this file and in `git show HEAD:docs/changes/2026-10-18-duties/functional.md`.
  - 2026-10-25: Fixed: the message for the next game shows "Anna B." for two players with the same first name again (story C, story B).
- Smaller, no action needed from the review: on "Season" the drivers' row is "Drivers: 4 (proposed) · 3 open" (no names), while the start page example shows "Drivers: Anna, Bram, 2 open" and "Next" is "as on "Season"" — one of the two should change. The label "Handled" mixes games with places and games saved as "none"; to see "where do we have duties" the coordinator needs "Open places" or the rows. "Handled" is a new word, already listed under Open questions for `/vision`. Checked: trace of story A criteria 6, 7, 9 and the start page example.
