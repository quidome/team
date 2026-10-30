# Functional spec: Duties

**Status**: approved (change of 2026-10-30)
**Change**: [2026-10-18-duties](./)
**Date**: 2026-10-25 (second change of that day: the duty places of a home game are given to club teams; it replaces the version approved earlier on 2026-10-25, which replaced the one of 2026-10-18); changed 2026-10-27 for story B (facts of 2026-10-27: what is stored is never changed by a reading; changes to past games with duties); changed 2026-10-30 for the messages (vision of 2026-10-30: story C becomes the message of the week, story D the committee message for this week, new story E the duty overviews)

## Context
The coordinator runs one youth team of the club. Besides the games and the trainings, the team
has two kinds of **duty**, and today the coordinator keeps them in a Google Sheet and writes the
messages about them by hand:

- **Table and referee duty.** Every few months (irregular, a few times per half year) the club's referee committee makes a schedule (an untidy sheet) for the **home games of the club**: each home game has 4 duty places, referee 1, referee 2, table 1 and table 2 (the table: the clock and the game administration on the tablet; the players divide those tasks themselves), and the committee gives each place to a **club team** (not to persons), or leaves it empty. One game can have up to 4 different teams; the two table places usually go to one team. In October 2026 the coordinator knows all places until the end of the calendar year. The team then divides the places given to it among its players; to make that fair, the coordinator must see who already did what.
- **Driving.** For each away game of the team, the team needs one car for every 3 players who go. Parents drive, but the duty is given to the player (a principle of the product).

The coordinator also writes two messages by hand: before each game, one for the team (time,
place, who drives, who has table or referee duty); before each weekend, one for the referee
committee (which player does which duty at which game). The tool only produces text to copy; it
never sends (a principle of the product).

The tool already has (see `docs/product/`):
- the games of the club from the game list: the games of the team and the home games of other club teams (the label "Possible duty games"), on the page **"Season"** with labels and a heading per week ([season](../../product/season.md), [schedule](../../product/schedule.md));
- the **start page** with "Open items" (what still needs the coordinator), "Last" (the last training) and "Next" (the games of the team and the trainings of this week and next week) ([start page](../../product/start-page.md)); each open item comes from the rule of the page it belongs to (decision 0022);
- the players of the club and the team of each season, with each player's role ("plays games" or "only trains") from a date; the list of changes to games; attendance at training with "Checked". Attendance at games does not exist.

This is an **epic of five stories**, built in this order (coordinator, 2026-10-18; C changed and E
added 2026-10-30): A duty places, B who does each duty place and the duty totals, C the message of
the week, D the committee message, E the duty overviews. After it, the sheet is no longer needed
for duties. A and B are built and released; C was built as "the message for the next game" and is
not released: it changes into the message of the week first.

The jobs and their places (coordinator, 2026-10-25): **entering the committee's schedule** happens a
few times per half year, on "Season"; **setting and filling the places of one game** on the page
of that game; **seeing what is still open** on the start page (open items); **copying a message**
from the start page; **the totals** on a page "Duties", as "Attendance" has the totals of the
trainings.

Facts from the coordinator (2026-10-18):
- Only players with the role "plays games" get duties.
- When choosing a player, the tool shows the duty totals; it never proposes a player.
- Fairness is judged for each kind apart: drive, table, referee. The totals that help are per kind, for the current season, done and planned.
- A duty counts as done by the player on its duty place, unless the coordinator corrects it.
- The number of drivers is proposed (one per 3 players), and the coordinator can change it; a changed number stays.
- The messages are in Dutch; the screens stay in English.
- The table and referee part of both messages covers the coming week. (Replaced 2026-10-30: both messages cover this week, Monday to Sunday.)
- Hidden players are offered for duties like the others.
- A driver proposal that drops by itself never removes a chosen driver.
- A game whose date has passed was played: its duties stay done, also when it leaves the game list later.
- Another team chosen in the middle of a season: the duties of the old team stay stored; coming ones no longer need duties of the team and do not count; past ones count; choosing the old team again brings them back. (For table and referee places this follows from the team on each place; see the facts of the second change of 2026-10-25.)

Facts from the coordinator (2026-10-25):
- The open items of duties cover the coming **4 weeks**; "Season" shows all games.
- The open items of duties come after the changes not seen, before the trainings.
- The totals are on a page "Duties"; the messages in a part "Messages" on the start page.
- Drivers become an open item only in this week and next (the window of "Next"); table and referee places in the 4 weeks (S16).
- Duties of an earlier season can be changed and corrected, as trainings of an earlier season can (S13).
- The choice of a player shows a clash next to the name, without blocking it (S14). Changed on 2026-10-27: a clash closer than 1 hour 30 minutes blocks the choice (see the facts of 2026-10-27).

Facts from the coordinator (second change of 2026-10-25, after the reviews of story A):
- The coordinator copies the committee's schedule as it is, for **every home game of the club**, also the home games of the team: at each, referee 1, referee 2, table 1 and table 2, each a club team or empty. The tool does not name the tasks at the table.
- The places are chosen from pull-down lists, no typing: the club teams in the game list of the game's season, sorted by name. Reading the committee's sheet into the tool is not part of this.
- A game has two checks: (1) the committee's schedule of the game is entered: a button "Schedule entered", tapped by the coordinator, also with empty places; (2) players of the team are on the places of the team: no button, the tool sees it (story B, open places). A later move of the game or change of a place does not undo check 1.
- Each choice of a place saves at once.
- The **places of the team** are, everywhere in this spec, the table and referee places that name the team plus the driver places of its away games. Drivers stay as they are (the committee does not plan them) (S21).
- The few home games a week (S10) stay acceptable one by one, also with the home games of the team added.

Facts from the coordinator (2026-10-27, at the design of story B):
- What the coordinator stored is never changed by a reading of the game list. A past game counts what is stored: a filled place counts as done by its player unless corrected, also after a later swap, a removal from the game list or another team chosen (A1 of the reviews of story A).
- Clashes (replaces S14 for close clashes): two games on the same court of the same hall never clash, because the next game on a court waits for the game before it. Two games on different courts (in the same hall or in another hall) whose start times are less than 1 hour 30 minutes apart block the choice; from 1 hour 30 minutes to less than 2 hours apart they give a warning, because games do not always finish in 1 hour 30 minutes. A game without a known start time gives a warning for every game on another court on the same date and never blocks. A game without a court counts as another court.
- A change to a past game with duties of the team stays in the list of changes until the coordinator has seen it, so the coordinator can check that game. There is no separate conflict resolver: "Seen" means checked, and a correction on the page of the game changes what counts.

Facts from the coordinator (2026-10-30, with the vision of that day):
- Messages announce what comes. A week is **Monday to Sunday**; the messages cover **this week** only (the week that holds today), also on a Saturday or Sunday. Table and referee places on days of the week that have passed stay in the messages until Sunday; a game of the team before today leaves the message of the week, because the poll is only for games still to come (S24). One week is enough for now, also on a Sunday; two weekends may come later (S25).
- Trainings are never in a message: the team knows them.
- The message of the week is the start of a poll in the chat: the coordinator puts a short poll under it, one for each game of the team. Two games in a week: both are in the message.
- The table and referee lines of a message have no hall: these places are always in the club's own hall.
- A week with only table and referee places has a message of the week with only those places. A week with neither a game of the team nor places of the team has no message of the week.
- "Scheidsrechter" comes before "Tafel" in every message and overview. Open places of the team are written one way everywhere: the names, a comma, then "nog N open" ("Anna, Bram, nog 1 open"; "nog 2 open") (S26). A message names only the players of the team on the places of the team, and open places of the team; it never mentions places given to other club teams.
- Before the game a message shows the planned player; after a correction it shows the player who did it ("Someone else"), or "niemand" ("Nobody").
- The duty overviews show all coming duties of the current season, without counts. They are on the page "Duties". The counts of what each player did become a separate report later (vision journey 8).

## User stories

### Story A: Duty places
**As the** coordinator, **I want to** copy the committee's schedule into the tool and record the
drivers of the team, **so that** the tool knows which duty places the team must fill, without the
sheet.

#### Acceptance criteria
- [ ] **The page of a game** opens with a tap on a game on "Season", in "Next" or "Last" on the start page, on a duty open item for one game, or on a change in the list of changes. It shows the date, start time ("time not known yet" when not known), home team, away team, hall and town, and its duty places. "Back" returns to the page it came from; to "Season" with the same labels and place in the list, as from a training, so the coordinator can walk through a filtered list.
- [ ] At a **home game of the club** (of any club team, also of the team; coming or past) the page shows 4 places: Referee 1, Referee 2, Table 1, Table 2. Each has a pull-down list with "—" (empty) and the club teams in the game list of the game's season, sorted by name. A choice saves at once.
- [ ] Below the places the page lists the table and referee places of the team apart: "M16 1: Referee 1, Table 1, Table 2", or "No places for the team."
- [ ] A button **"Schedule entered"** marks that the committee's schedule of this game is entered, also with empty places. Before the tap the page says "Schedule not entered"; after it "Schedule entered", and a second tap undoes it. Changing a place later, or a move of the game, does not undo it.
- [ ] At an **away game of the team** (a game at another club), the page shows the number of drivers. While the coordinator has not changed it, it is the proposal: one for every 3 players with the role "plays games" on the game's date, rounded up (10 players: 4 drivers; 0 players: 0), marked "proposed", and it follows the team when players join or leave. The coordinator can set another number (0 to 10); from then on that number stays, without "proposed". "Use proposal" goes back to the proposal. When the proposal drops by itself (a role changes) below the driver places already filled, the number stays at the filled places, with "more than proposed"; nobody is removed. An away game needs no "Schedule entered".
- [ ] A number of drivers below the number of filled driver places is refused: "2 places are filled. Clear one first." A place of the team with a player (story B) cannot get another team or "—": "Clear the player first."
- [ ] **On "Season"** a home game of the club shows on its row the places of the team ("Referee 1: open · Table 1, 2: Anna, open") or "No places for the team", followed by "· Schedule not entered" while the button is not tapped (with no places chosen: only "Schedule not entered") (S18); an away game of the team its drivers ("Drivers (4, proposed): Anna, 3 open"). Places whose players are all chosen show "all filled". The page of the game shows the teams of all 4 places.
- [ ] **"Season" gets the label group "Duties"**: "Entered", "Not entered", "Open places". They follow the rules of the other groups (labels of different groups narrow, labels of one group add). "Entered" and "Not entered" apply to home games of the club; "Open places" to home games of the club and away games of the team; with a Duties label on, trainings and games they do not apply to do not show. To enter the committee's schedule the coordinator uses "Not entered" (with "Coming").
- [ ] **A new open item on the start page**: "Home game Sat 31 Oct: schedule not entered" for one, "5 home games: schedule not entered" for more: the home games of the club from today to 4 weeks ahead without "Schedule entered". A tap opens the game; for more than one, a tap opens the list of those games below the item (date, start time, the game), each a tap to its page, so the item and its list always hold the same games.
- [ ] **"Next"** on the start page also shows the home games of other club teams where the team has places, with its places, as on "Season" (the games of the team are in it already).
- [ ] **"Last"** on the start page also shows the games of the last date before today that have places of the team (table, referee or drivers), as on "Season", so a correction after the weekend is one tap (story B) (S21).

#### Behaviour
The coordinator gets the committee's schedule (a sheet) and opens "Season" with "Not entered" and
"Coming": a tap on the first game, four choices ("M16 1", "X12 2", "M16 1", "M16 1"), "Schedule
entered", back; the next game. A game where the schedule leaves places open: the empty places stay
"—", "Schedule entered". The open item "5 home games: schedule not entered" shrinks while the
coordinator works. For an away game the page already shows "Drivers: 4 (proposed)". Places can be
entered for past games too, to enter what already happened this season: there only the places of
the team matter; the other places and "Schedule entered" may stay as they are, and a past game is
never an open item for the schedule (S19).

#### Errors and edge cases
- A game that moves (another date, time or hall) keeps its places, their teams, the players on them and "Schedule entered". Its change in the list of changes says "has duties" with the places of the team (for example "Table 1, 2: Anna, open"), so the coordinator can tell the players and the committee.
- A coming game (today or later) that leaves the game list keeps its places while it is gone; on its page and on its row on "Season" it shows "no longer in the game list". Its duties do not count, and it is no open item, while it is gone. If it comes back, they count again.
- A game whose date has passed was played: its duties stay done, also when it leaves the game list later. Its page says "no longer in the game list".
- A game that stops being a home game of the club (home and away swap) keeps its places, with "This game is no longer a home game of the club." on its page and on its row; while it is coming its places do not count and it is no open item; once its date has passed they count as stored (facts of 2026-10-27). Setting the places to "—" removes them. Its change in the list of changes says "has duties" too.
- An away game of the team that stops being one keeps its drivers the same way, with "This game no longer needs drivers of the team."
- A team named on a place that is no longer in the game list keeps its name on the place, and stays in that pull-down list as long as a place names it.
- Another team chosen in the middle of a season (rare): each place keeps its team, so the places of the old team are never places of the new team; where the schedule names the new team, those places are its places. Drivers of the old team: coming ones show "This game no longer needs drivers of the team." and do not count; past ones count. Choosing the old team again brings its places back.
- A game of an earlier season can be opened (from "Season" of that season, or from "Last" in July and August), and its places, drivers and players can be changed and corrected, as trainings of an earlier season can. Its duties count in the totals of that season.
- A home game more than 4 weeks ahead is no open item; "Season" shows it as "Schedule not entered".
- Two screens set the same place: the last save wins; a place never holds two teams.
- A save that fails shows "Not saved. Try again." and the page shows what is stored.

### Story B: Who does each duty place, and the duty totals
**As the** coordinator, **I want to** put a player on each duty place and see what each player has
done, **so that** I can divide the places fairly and the totals are right.

#### Acceptance criteria
- [ ] On the page of a game, a tap on a duty place of the team (a place that names the team, or a driver place) opens the choice of a player: the players with the role "plays games" on the game's date, hidden players too (hidden only keeps lists short, as at a training), sorted by first name. Next to each name, that player's totals for that kind in the game's season (for a game of this season: the current season; coordinator, 2026-10-27): "2 done · 1 planned". The tool marks no player as proposed.
- [ ] A tap on a player puts the player on the duty place and saves at once. "Clear" makes the place open again.
- [ ] A player who already has a duty place at the same game is not offered for another place at that game. A clash is a duty place at another game ("also Table 13:00") or another game of the team ("plays 14:30"); the game itself is never a clash (S22). A clash on another court less than 1 hour 30 minutes apart shows next to the name and the player cannot be chosen; from 1 hour 30 minutes to less than 2 hours apart it shows as a warning and the player can be chosen (facts of 2026-10-27). A player already on a place stays on it when a later change makes a clash; the place shows the clash as a warning. "Someone else" on a past game is never blocked.
- [ ] A filled duty place of a game before today (Dutch date) counts as **done** by its player. A filled duty place of today or later counts as **planned**. Open places count for nobody.
- [ ] On a past game, the coordinator can correct a duty place: another player ("Someone else": the same choice), or "Nobody" (the place was not done; it counts for nobody and shows "not done (was Anna)", so the planned player stays visible). The correction replaces what counts.
- [ ] **A new open item on the start page**: "Away game Sat 31 Oct: 2 drivers open", "Home game Sun 1 Nov: 1 table place open", or for more than one game "3 games with open places": the home games of the club from today to 4 weeks ahead with open places of the team, and the away games of the team in this week and next (the window of "Next") with open driver places, that still need duties of the team. A tap opens the game; for more than one, a tap opens the list of those games below the item, each a tap to its page.
- [ ] **The page "Duties"** (new; the links of the start page become "Season · Attendance · Duties · Team · Players · Settings") has a table "Totals": one row for each player with the role "plays games" in the season or with a duty in it (also when the role starts later), sorted by first name, and the columns Drive, Table and Referee, each "done · planned" (for example "3 · 1"). "Done" is the duty total of the word list; "planned" is shown apart from it. Players who left the team stay in the table while they have duties in the season; hidden players have rows like the others. A season choice, as on "Attendance".
- [ ] What counts: duties of coming games that are in the game list and still need duties of the team, and duties of past games (also when they left the game list later); see story A. The totals next to the names in the choice and the table on "Duties" give the same numbers.
- [ ] Two players with the same first name show the first letter of the last name in the choice, on "Season", on the start page and in the message of the week and in the duty overviews ("Anna B.", "Anna V.").

#### Behaviour
When a new schedule from the committee is entered, the open item "4 games with open places"
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
- A change to a past game where something is stored for the team (a place that names the team, a number of drivers set, or a player on a place of the team) is kept in the list of changes with "has duties", as a change to a coming game is, until the coordinator taps "Seen"; it does not drop off because its date has passed. A change to a past game without duties of the team is not kept, as now (facts of 2026-10-27).
- A game where players are on driver places keeps those places, also when it no longer needs drivers of the team; while it is coming they do not count, once its date has passed they count as stored.

### Story C: The message of the week
**As the** coordinator, **I want to** copy one message about this week, **so that** the team and
the parents know the games, who drives and who has table or referee duty, and I can put a poll
under it.

(Changed 2026-10-30. Built before as "the message for the next game", not released.)

#### Acceptance criteria
- [ ] The start page has a part "Messages" below "Next" with "Message of the week" when this week (Monday to Sunday, Dutch time) has a game of the team today or later, or table or referee places of the team. Otherwise the button is not shown. A tap opens the message: Dutch text, with a "Copy" button that puts it on the clipboard and says "Copied".
- [ ] The message starts with the week ("Week 26 oktober – 1 november"). Then each game of the team from today to Sunday, in time order (a game today stays all day): "Wedstrijd:" with the date and start time, "Thuis" (a game in the club's own hall) or "Uit", the opponent, the hall and the town; for an away game that needs drivers of the team and has more than 0 driver places, "Rijden:" with the drivers by first name and open places ("Rijden: Anna, Bram, Cas, nog 1 open"; "Rijden: nog 3 open").
- [ ] Then "Tafel en scheidsrechter:" with each game of the week with table or referee places of the team, in time order: the short date, the start time and the two teams, then "Scheidsrechter:" and "Tafel:" with the players by first name and open places as "nog N open". A kind without a place of the team at that game has no line. Places of other club teams are not mentioned.
- [ ] Table and referee places on days of the week that have passed stay in the message until Sunday; games of the team before today do not (S24). A past place shows the player who did it after a correction, or "niemand" for "Nobody".
- [ ] A start time not known yet shows "tijd nog niet bekend". Two players with the same first name show the first letter of the last name ("Anna B.", "Anna V.").
- [ ] The text is built from what is stored at that moment; the coordinator may edit it after pasting in the chat.

#### Behaviour
Example (a Wednesday; one away game of the team, one home game of another club team with places
of the team):

```
Week 26 oktober – 1 november

Wedstrijd: zaterdag 31 oktober, 14:30
Uit tegen B.Z. '72 M16 1
Sporthal Bulwijk, Woerden
Rijden: Anna, Bram, Cas, nog 1 open

Tafel en scheidsrechter:
za 31 okt 16:00 Blue Drakes X12 1 – De Dunckers X12 1
  Scheidsrechter: Gijs
  Tafel: Eva, nog 1 open
```

For a home game, and for an away game with 0 drivers, the line "Rijden" is left out. Two games of the team: two blocks "Wedstrijd:",
with one empty line between them; the coordinator makes a poll for each. Without table or referee
places of the team in the week, the part "Tafel en scheidsrechter:" is left out. Without a game of
the team from today to Sunday (a free week, or after the weekend's game), the message has the week
and that part only.

#### Errors and edge cases
- Nothing to announce this week: the button is not shown. Opened anyway (an old link): "No message this week."
- A coming game that is no longer in the game list, or is no longer a home game of the club, is not in the message (the rule of the totals, story B). A past game of the week keeps what is stored.
- Copying fails (the browser refuses): the text stays shown and selectable, with "Select the text and copy it."

### Story D: The committee message
**As the** coordinator, **I want to** copy one message with the team's table and referee duties of
this week, **so that** the referee committee knows which player does which duty.

#### Acceptance criteria
- [ ] The part "Messages" on the start page has "Committee message" whenever a team is chosen for the current season, also in a week without places. It opens the message for this week, Monday to Sunday.
- [ ] The message starts with the team and the week. Then each game of the week with table or referee places of the team, in time order: the short date, start time, home team – away team, then "Scheidsrechter:" and "Tafel:" with the players by **first and last name**; open places of the team as "nog N open" ("Eva de Vries, nog 1 open"; "nog 2 open"). A kind without a place of the team at that game has no line. Places of other club teams are not mentioned.
- [ ] A week without places of the team: the text is the heading and "Geen tafel- of scheidsrechtersdiensten deze week.", and "Copy" still works.
- [ ] Past days of the week stay, as in story C (the player who did it, or "niemand").
- [ ] Copying works as in story C.

#### Behaviour
Example:

```
Tafel en scheidsrechters Blue Drakes M16 1, week 26 oktober – 1 november

za 31 okt 13:00 Blue Drakes X12 1 – De Dunckers X12 1
  Scheidsrechter: Eva de Vries
  Tafel: Fleur Jansen, Gijs Bakker

zo 1 nov 11:00 Blue Drakes M14 2 – Lions M14 1
  Scheidsrechter: nog 1 open
  Tafel: Hugo Smit
```

#### Errors and edge cases
- A player with no last name shows the first name only.
- A coming game that is no longer in the game list, or is no longer a home game of the club, is left out of the message (the rule of the totals).
- Copying fails: as in story C.

### Story E: The duty overviews
**As the** coordinator, **I want to** copy an overview of all coming duties, after a new or changed
schedule or whenever I want, **so that** the team and the parents see who does which duty.

#### Acceptance criteria
- [ ] The page "Duties" has two buttons, "Driving overview" and "Table and referee overview", for the current season. For an earlier season (the season choice) they are not shown. Each opens a page with Dutch text and "Copy" (as in story C) and "Back" to "Duties".
- [ ] The driving overview has each coming away game of the team (today and later) that needs drivers of the team and has more than 0 driver places, in time order: the short date, start time, "Uit tegen" the opponent and the town, then the drivers by first name and open places ("Anna, Bram, nog 1 open"; "nog 3 open").
- [ ] The table and referee overview has each coming game (today and later) with table or referee places of the team, in time order, in the form of the week part of story C ("Scheidsrechter:" before "Tafel:", first names, "nog N open").
- [ ] No counts and no totals are in the overviews.
- [ ] Without coming duties of that kind: "Geen ritten gepland." or "Geen tafel- of scheidsrechtersdiensten gepland.", and "Copy" still works.
- [ ] What is in an overview follows the rule of the totals (story B): a coming game no longer in the game list, or no longer needing duties of the team, is not in it.

#### Behaviour
Examples:

```
Rijden Blue Drakes M16 1

za 31 okt 14:30 Uit tegen B.Z. '72 M16 1, Woerden
  Anna, Bram, Cas, nog 1 open
za 14 nov 12:00 Uit tegen Lions M16 1, Utrecht
  nog 3 open
```

```
Tafel en scheidsrechter Blue Drakes M16 1

za 31 okt 16:00 Blue Drakes X12 1 – De Dunckers X12 1
  Scheidsrechter: Gijs
  Tafel: Eva, nog 1 open
```

The overviews can be long (all coming duties until the end of the season); places the committee
has not given out yet are not in them, so an overview grows with each new schedule.

#### Errors and edge cases
- A start time not known yet shows "tijd nog niet bekend", as in story C.
- Copying fails: as in story C.

## Interface
- **The page of a game** (new, `/game/...`): the game; at a home game of the club the 4 places with their pull-down lists, the places of the team and "Schedule entered"; at an away game of the team the drivers and "Use proposal"; the choice of a player, "Someone else", "Nobody" (story B).
- **"Season"**: the places of the team on each row of a game that can have duties; the label group "Duties" ("Entered", "Not entered", "Open places"); "Back" from a game keeps the labels and the place.
- **The start page**:
  - "Open items", in this order: the game list; changes not seen; home games with the schedule not entered; games with open places; training days not opened; trainings not checked;
  - "Last" also with the games of the last date before today with places of the team;
  - "Next" also with the home games of other club teams where the team has places;
  - a part "Messages" below "Next", with "Message of the week" (when this week has something to announce) and "Committee message";
  - the links "Season · Attendance · Duties · Team · Players · Settings".
- **"Duties"** (new, `/duties`): the table "Totals", with a season choice; for the current season the buttons "Driving overview" and "Table and referee overview" (story E).
- **Four text pages**, each with Dutch text, "Copy" and "Back": the message of the week and the committee message (back to the start page), the driving overview and the table and referee overview (back to "Duties").
- Server addresses follow in the design.

Example of the start page (a Monday, one week after a committee schedule):

```
Open items
  3 home games: schedule not entered
  Away game Sat 31 Oct: 2 drivers open
  Training Thu 22 Oct not checked

Last
  Sat 24 Oct   13:00 Blue Drakes X12 1 – Lions X12 1 · Table 1, 2: Eva, Fleur
  Thu 22 Oct   Training 19:45 · 2 absent

Next
  Tue 27 Oct   Training 19:45 · Not opened
  Sat 31 Oct   14:30 B.Z. '72 M16 1 – Blue Drakes M16 1 · Drivers: Anna, Bram, 2 open
               16:00 Blue Drakes X12 1 – De Dunckers X12 1 · Table 1, 2: Eva, open

Messages
  Message of the week
  Committee message

Season · Attendance · Duties · Team · Players · Settings
```

## Consistency
- The rules of "Season" stay: which games, the order, gone games, the labels and how they combine, labels not kept between visits (a return from a game, as from a training, keeps them once).
- The open items follow decision 0022: each comes from the rule of its source (here: the rule of "schedule entered" and of open places that the page of a game and "Season" use); nothing is counted on the side.
- The choice of a player works like marking at a training: a tap saves at once, "Not saved. Try again." when it fails, the stored state shown again.
- Players: only the role "plays games" on the date counts, as roles decide who is at a training; hidden players are offered.
- Totals are worked out each time from what is stored, as the attendance totals are; "Duties" is to duties what "Attendance" is to trainings.
- The list of changes keeps its behaviour; a change to a game with duties also says "has duties", and a change to a past game with duties of the team is kept until seen (story B).
- A choice of a place saves at once, as a tap at a training; "Schedule entered" works like "Checked" at a training (a tap by the coordinator, also undone with a tap), but it says only that the schedule is entered, not that the duties were done.

## What changes for existing users
- New: the page of a game, the page "Duties", the duty labels on "Season" ("Entered", "Not entered", "Open places"), the duty places on the rows of "Season" and in "Next" and "Last", two kinds of open items, the part "Messages" on the start page, the duty overviews on "Duties".
- The message for the next game (built, not released) becomes the message of the week before its release; the coordinator never sees the older form.
- The links of the start page become "Season · Attendance · Duties · Team · Players · Settings".
- A tap on a game (on "Season", in "Next", in the list of changes) now opens its page; before, it did nothing.
- "Delete player" is also not offered for a player with duties.
- No data to move: there are no duties yet. The coordinator enters this season's schedule and duties (also past ones) by hand. All home games of the club start as "Schedule not entered"; only those of the next 4 weeks are open items.

## Warnings from architect and engineer
- **Games move and disappear.** Duty places are entered weeks or months ahead; the game list can move a game, swap home and away, change the team, or drop the game in that time. Every such case must keep the duty places, their teams and "Schedule entered", and say clearly what happened (story A), and the totals and open items must never count a duty of a game that is gone or no longer needs duties.
- **One rule everywhere (decision 0022).** The totals next to the names, the table on "Duties", and the messages and duty overviews must give the same numbers from the same rule. "Schedule not entered" and "open places" must be the same rule on "Season", on the page of a game and in the open items. The horizons (4 weeks for table and referee places, this week and next for drivers) belong only to the open items, and the list an item opens holds the same games.
- **The driver proposal moves.** While not changed, it follows the players with the role "plays games" on that date; a role change can change the number after drivers were chosen, and can make a game an open item without a save. Filled places above a lower proposal must not disappear.
- **The start page gets heavier**: games with places, the players on them, and the roles for the proposal. It must stay one request and fast.
- **Two screens.** One coordinator, but a phone and a computer can save the same duty place; one place never holds two players, and a choice must not be lost without a sign.
- **Dates, weeks and weekends** follow Dutch time.
- **The schedule is about the club; the drivers and the players are about the team.** A place keeps the name of its team also when that team leaves the game list, or old places become empty without a sign. Which places are "of the team" follows from the team chosen for the game's season, so another team chosen changes them at once.
- **More games to enter.** "Schedule not entered" now counts every home game of the club in 4 weeks, also those of the team; a busy weekend makes a long list. Each choice saves at once, so a walk through 4 pull-down lists on a phone is 4 saves; each must be quick and must say when it fails.
- **Two screens** can set the same place or the same "Schedule entered": the last save wins; one place never holds two teams.

## Not in scope
- Attendance at games (it may later make the driver proposal more exact).
- A departure time for away games in the message (the coordinator adds it by hand when needed).
- Counts or totals in the duty overviews; the report of what each player did (vision journey 8; later).
- Messages for another week than this one ("Next week", "Previous week", two weekends): later, if needed.
- Reading the committee's sheet into the tool (it is entered by hand with pull-down lists).
- Naming the tasks at the table (clock, administration): the players divide them.
- Proposing a player for a duty place.
- Messages in other languages, and templates the coordinator can edit (idea of 2026-10-10).
- Duties other than drive, table and referee. (The schedule of the team's own home games is entered; the places there name other club teams.)
- Totals over more than one season.
- A date "committee list entered until" (the horizon of 4 weeks is used instead; coordinator, 2026-10-25).

## Open questions
- For `/vision`: a word for the set of duty places the referee committee gives the team for a period (the coordinator says "shifts" and "a wave"); this spec writes it out in plain words.
- For `/vision`: the phrase "possible duty games" (a label on "Season" now, for the home games of other club teams) now that every home game of the club has duty places; and "schedule entered" (check 1 of a home game) as a word.

## Spec review
<Filled in by `/review spec` (a fresh agent). One row per user story. Design does not start
without this table. A "does not serve" row sends the spec back to Discover.>

| Story | What the user is doing here | Verdict | Note |
|---|---|---|---|
| A Duty places | A few times per half year: copying the committee's schedule row by row (4 pull-down lists and "Schedule entered" per home game of the club), walking "Season" with "Not entered" + "Coming"; now and then: setting the drivers of an away game; each visit: seeing which home games in the next 4 weeks are not entered | serves – note | S18, S19, S20 |
| B Who does it, totals | Putting the player the chat agreed on onto each place of the team, with the totals per kind in view; after a weekend that went otherwise, one correction from "Last"; now and then: showing that the division is fair ("Duties") | serves – note | S21, S22 |
| C Message for the next game (before 2026-10-30) | Around training: copying one message for the team and parents (when, where, who drives, who has table or referee duty this week) | serves – note | S23 |
| D Message for the referee committee (before 2026-10-30) | Before the weekend: copying the week's table and referee duties of the team with full names for the committee | serves | – |
| C The message of the week (2026-10-30) | Before the weekend (journey 4): one tap on the start page, copying one message for the team and the parents about this week (each game of the team, drivers, the team's table and referee places), then putting a poll under it for each game | serves – note | S24, S25, S26 |
| D The committee message (2026-10-30) | Before the weekend (journey 5): copying the team's table and referee places of this week with full names for the referee committee, also "none this week" | serves – note | S25 |
| E The duty overviews (2026-10-30) | After a new or changed schedule, or when asked (journey 6): from "Duties", copying all coming drives, or all coming table and referee places, so the team sees who does what | serves – note | S26 |

Reviewed 2026-10-25 by a fresh agent (`/review spec`), the third review, of the second change of
that day (places given to club teams). Read: `docs/vision.md`, `docs/language.md`, `idea.md`
(also its change of 2026-10-25), `change.md`, `docs/product/` (season, schedule, start page, list
of changes, players and team, trainings and attendance), decision 0022, and the version of this
spec approved earlier that day (`git show HEAD:`) for what the change kept or lost. The arguments
held only the mode and the change folder; nothing was ignored. The notes go on from S18. No story
conflicts with the vision: journey 3 is now what the spec builds (copy the committee's schedule
with pull-down lists, then mark the game ok); a place that names a club team fixes the place
that could not follow another chosen team; the principles hold. The words of the spec match the
changed word list (search for `jury|slot|spot|handled|reminder` in this file: no hits outside
the history). S21 is the one to settle before design; the rest are gaps that Design can close
with one line each.

- **S18 – A row with some places chosen but "Schedule entered" not tapped.** The row on "Season" shows "the places of the team", "No places for the team", or "Schedule not entered", as three options. The two checks are independent (facts of the second change), so a game can have places of the team (the coordinator stopped halfway, or entered only the team's places for a past game) and still be "not entered". The spec does not say which text wins. If "Schedule not entered" wins, the row hides that the team has a place there, while that place already counts in "Next", the open item of open places and the totals (story B). The row should show both ("Schedule not entered · Table 1: open"). Checked: trace of story A criteria 3, 4 and 7 against the fact "A game has two checks" and story B criterion 6.
  - 2026-10-25: Fixed: the row shows the places of the team followed by "· Schedule not entered" (story A). (coordinator)
- **S19 – Past games entered to catch up on this season.** The behaviour says places can be entered for past games "to enter what already happened this season". To get the totals right, the coordinator only needs the places of the team; copying all 4 places of every past home game since September would be work with no use. The spec allows that (leave the rest "—"), but then "Schedule entered" on such a game is either not tapped (the row says "Schedule not entered" for ever; see S18) or tapped while the schedule is not really entered. Both are fine for the coordinator once S18 is settled; this note only asks the spec to say that past games need not be entered in full. Checked: trace of story A "Behaviour" and "What changes for existing users" (all home games start as not entered; only 4 weeks are open items).
  - 2026-10-25: Fixed: for past games only the places of the team matter; a past game is never an open item for the schedule (story A, Behaviour). (coordinator)
- **S20 – Small: the cost of the walk-through grew, and the team's own home games give no places.** Before this change a game without places of the team took one save; now every home game of the club, also those of the team, takes 4 pull-down lists and a tap, and the lists hold every club team. The coordinator accepted this (fact "The few home games a week (S10) stay acceptable one by one"). At the team's own home games the places name other club teams (Not in scope), so the coordinator enters them only to clear the open item. No action needed; if real use shows the open item "N home games: schedule not entered" stays long (for example in the weeks before the committee's next schedule, when nothing can be entered yet), that is a line for `ideas.md`, not a change now. Checked: trace of story A criteria 2 and 9 with the facts of the second change and "Not in scope"; the number of home games per week is not checked.
  - 2026-10-25: No action (accepted by the coordinator). (coordinator)
- **S21 – "Last" may lose the drivers, and holds one game only.** The approved spec said "Last" shows "the last game before today with duty places"; the change made it "with places of the team", and the facts now say "The places of the team are the places that name the team. Drivers stay as they are". Read that way, an away game of the team with drivers is not in "Last", and the correction of story B ("a tap on the game under "Last", a tap on the place, "Someone else"") does not work for a driver who did not drive. The Interface still says "the last game with duty places". Also, "Last" holds one game, while a weekend often has the team's away game and a table or referee place at a home game of the club: the other one needs "Season". The spec should say whether "Last" includes driver places (it should, for story B), and whether it holds the games of the last date with duties or one game. Checked: compare `git show HEAD:` line "also shows the last game before today with duty places" with story A criterion 11 now; search "places of the team" in this file (narrow in story A criterion 3 and the facts, broad in story B criterion 1: "a place that names the team, or a driver place").
  - 2026-10-25: Fixed: "places of the team" means the table and referee places that name the team plus the driver places, everywhere; "Last" shows all games of the last date before today with places of the team (story A, facts, Interface). (coordinator)
- **S22 – Small: the clash note at the team's own away game.** "plays 14:30" marks a game of the team that overlaps. In the choice of a driver for an away game of the team, every player plays that same game, so the note would show next to every name. The spec should say that the game itself is not a clash. Checked: trace of story B criterion 3 with story A criterion 5.
  - 2026-10-25: Fixed: the game itself is never a clash (story B). (coordinator)
- **S23 – The message for the next game is silent on open table and referee places.** For drivers story C says "nog N open"; for table and referee duties it says only "the players by first name", and the example has no open place. Story D shows "nog open". The team message is where an open place matters most (someone in the chat can still take it), so it should show it too, for example "Tafel: Eva, nog 1 open". Checked: search for "nog" in this file (story C drivers, story D); the same gap is in the version approved earlier that day (`git show HEAD:`), so it is not new.
  - 2026-10-25: Fixed: the message for the next game shows open places as "nog N open" (story C, example). (coordinator)

Reviewed 2026-10-30 by a fresh agent (`/review spec`), the fourth review, of the change of that
day (stories C and D rewritten for this week, story E added). Read: `docs/vision.md` and its change
of 2026-10-30 (`git show a93b705`), `docs/language.md`, `idea.md`, `change.md`, `docs/product/`
(messages, duties, start page), and the uncommitted diff of this file against `HEAD` for what the
change kept or lost. The arguments held only the mode and the change folder; nothing was ignored.
Stories A and B were not judged again; of B only the changed criterion on names was read (it now
covers the message of the week and the duty overviews; the committee message uses full names, so
it needs no rule). No story conflicts with the vision: C, D and E are journeys 4, 5 and 6 as
changed that day; the tool only makes text; the overviews have no counts, as journey 6 and
priority 8 now say. The words match the word list (search for `next game|overview for the
parents|coming week|jury|slot|reminder|weekend message` in this file: hits only in history, in
the "Replaced" note of the facts and in "What changes for existing users"). S24 is the one to
settle before design; S25 and S26 are one line each.

- **S24 – Past games of the team in the message of the week.** The message is for the team and the parents and is "the start of a poll": one poll for each game (facts of 2026-10-30, journey 4 "Before the weekend", "Messages announce what comes"). But "days of the week that have passed stay in the message until Sunday" (facts; story C criterion 4) also keeps a game of the team that was already played: copied on a Friday, a Tuesday game shows as "Wedstrijd: dinsdag …" with its "Rijden" line, exactly like a game still to come, and nothing marks it as played. The coordinator then deletes that block by hand each time, or a poll "who comes?" is put under a past game. The same rule shows the button on a Friday for a week whose only game was on Tuesday, and on a Sunday evening for a weekend that is over. For the committee message (story D) past days do little harm; in the team message they work against its purpose. The spec should say which is meant: past games of the team left out of the "Wedstrijd" part (the table and referee part may keep them), or marked as played ("gespeeld"), or kept on purpose (then say why, for example to repost an updated message). Checked: trace of the facts of 2026-10-30 (week, past days, poll) against story C criteria 1 and 4 and its example. Whether the team plays on weekdays: not checked (the local database was not running).
  - 2026-10-30: Fixed: a game of the team before today leaves the message of the week (a game today stays all day); table and referee places of past days of the week stay until Sunday (facts of 2026-10-30, story C). (coordinator)
- **S25 – The coming week cannot be announced on a Sunday.** Both messages cover only the week that holds today, and "Next week" and "Previous week" of the committee message are gone (`git diff` of this file, story D criterion 1; Not in scope "Messages for another week"). On a Sunday the message of the week holds only past days, and the names for a Monday or Tuesday table or referee place can reach the committee only on that day. The old message still had the next game in it on a Sunday; the idea line of 2026-10-29 in `docs/ideas.md` (made on Sunday evening, accept review A8) is marked "Replaced 2026-10-30", but the need it named is not met: it is now larger. The coordinator chose "one week is enough for now", so the spec may stay; the idea line should then stay open (not "replaced"), so real use can show whether Sunday evening matters. Checked: compare story D of `git show HEAD:docs/changes/2026-10-18-duties/functional.md` with story D now; read `docs/ideas.md` line 2026-10-29.
  - 2026-10-30: No change: one week is enough for now, also on a Sunday; the idea line of 2026-10-29 in `docs/ideas.md` stays open. (coordinator)
- **S26 – Small: drivers at 0, and three forms of "open".** (a) Story C ("for an away game that needs drivers of the team") and story E ("each coming away game … that needs drivers of the team") do not say what an away game whose number of drivers is 0 gives; the built message leaves out "Rijden" when the number is not above 0 (`docs/product/messages.md`). The spec should say: no "Rijden" line, and not in the driving overview. (b) The same open driver place reads "(nog 1 open)" in the message of the week and ", nog 1 open" in the driving overview, and one open table or referee place reads "nog 1 open" in C and E but "nog open" in D. Parents see the message and the overview side by side; one form, or a line that the difference is meant, saves a question in Design. Checked: search for `nog .*open` in this file (story C criteria 2 and 3 and its example, story D criterion 2 and its example, story E criterion 2 and its example); read `docs/product/messages.md` ("only when … above 0").
  - 2026-10-30: Fixed: an away game with 0 drivers has no line "Rijden" and is not in the driving overview; open places are written one way everywhere, the names, a comma, then "nog N open" (facts of 2026-10-30, stories C, D, E). (coordinator)

Earlier reviews of this spec: S1–S9 (2026-10-18) and S10–S17 (2026-10-25, first change); see the
log in `change.md` and `git log` of this file. The answers of S10–S17 are in the facts and the
stories above.
