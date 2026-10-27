# Product vision: team

## The user's problem space
One coordinator runs one youth team in a club. Each week the same work comes back: which games
are there, who is at training, who drives to away games, and which players cover the table and
referee places that the referee committee gives to the team. The team talks in a WhatsApp group; the coordinator
keeps the record. Today the record is a Google Sheet, because the first version of this tool was
not good enough. Keeping it up to date by hand, and writing the same kinds of messages again and
again, costs time.

Three groups read what the coordinator produces:
- **The team and the parents**: when their child plays, when they must drive or do table or referee duty, and whether the work is shared equally.
- **The club's referee committee**: which of our players do which duty at the coming weekend's games.
- **The coordinator**: what happened, and what must still be arranged.

## What this product is
A small tool for one coordinator. It keeps the games, attendance and duties of one team correct,
and turns them into text that the coordinator copies into the chat.

It is not:
- a tool for the whole club, or for several coordinators;
- a place where parents or players log in;
- connected to WhatsApp or any other messenger;
- a replacement for the federation's schedule. It reads that schedule.

## Core values
1. **Capture fast, keep it correct.** For example, attendance at training takes seconds, on a phone, with one hand. The stored data is the truth and must be right.
2. **The division of duties is visible and fair, and anyone can check it.** Parents and players can see who did what, so they trust the division.

## Where we are
- The first version (SvelteKit and TypeScript, built September 2026) is not good enough in daily use. Its history is in `docs/changes/done/2026-09-20-mvp/`.
- The coordinator uses a Google Sheet as a workaround.
- The base of the new product runs on the cluster (v0.1.0, October 2026): login through Pocket ID, a start page, and a sample schedule with example games.
- Games from the game list runs on the cluster (v0.2.0, released 14 October 2026):
  - the coordinator chooses the club by name and the team from the club's own teams, again at each new season; the choice of each season is kept;
  - the tool reads the club's games on a tap and by itself every night at 03:00, keeps them for each season, and says in one line what changed; a list with far fewer games is held back until the coordinator confirms; after 3 nights without a good reading the start page says the games may be out of date;
  - the start page shows the team's games and the club's home games (possible duty games), with a quick filter and past games on request; in a new season last season's schedule shows until the team is chosen;
  - each change to a coming game of the team or a possible duty game stays until the coordinator marks it as seen, so drives and duties can be fixed in time.
- Attendance at training runs on the cluster (v0.3.0, released 18 October 2026): the players of the club, the team of each season with each player's role ("plays games" or "only trains") from a date, the weekly trainings and the reasons for an absence; opening a training (or adding one), marking who is absent and why with one tap each on the phone, the next training on the start page, and the totals for each player in a season (there, absent with a valid reason, absent without one), also for an earlier season.
- A nightly encrypted copy of the team database is kept on the home storage (since 18 October 2026, built in `gitops-lab`); a copy outside the house comes later.
- The player names are stored encrypted (v0.3.1 and v0.3.2, released 18 October 2026): a copy of the database or a backup does not show who the players are.
- The start page and the page "Season" run on the cluster (v0.4.0, released 25 October 2026): the start page shows the open items (problems with the game list, changes not seen, training days not opened, trainings not checked), the last training and the events of this week and next week; "Season" shows all games and trainings of a season per week, with labels that combine; the coordinator confirms with "Checked" that the attendance of a training is complete. Duties come next, again from Discover.
- The first part of duties is built and reviewed, not yet released (story A, duty places, 27 October 2026): each game has its own page; the coordinator copies the referee committee's schedule into the tool (the 4 table and referee places of each home game of the club, each given to a club team, and "Schedule entered") and sets the drivers of the team's away games (proposed as one per 3 players); "Season" shows the places with the labels "Entered", "Not entered" and "Open places", and the start page has the open item "schedule not entered". Players on the places and the totals come next (story B).
- The sheet is still in use.
- The product is successful when the coordinator stops using the sheet.

## North star
Each week, the coordinator spends a few minutes in the tool and none in the sheet. Every
message that goes to the team, the parents or the referee committee comes from the tool,
correct the first time. No one questions whether the duties are shared fairly, because they can
see it.

## Where we are going
First the games come in from the club's game list, so the coordinator never types a schedule.
Then attendance at training. Then a start page that shows what comes next and what is still open, and a page with the whole season, so each later part has a place to show its open items (built). Then duties and the messages built on them, with their own open items on the start page. Each step must be good
enough to replace that part of the sheet.

## Longer horizon
Not for now:
- more coordinators in the club;
- a page that parents can open without asking the coordinator;
- past results and statistics from the game list.

These stay ideas until real use shows the need.

## User journeys
0. **Open the tool.** A few times a week, the coordinator opens the tool on the phone and sees the next events and the open items: what still needs the coordinator. One tap completes the record of the last training. The whole season is one tap further, for any game or training.
1. **Games come in.** The coordinator gets all games of the club from the federation's game list. This includes the games of other club teams, because the club's home games have table and referee places that the club teams fill. When a game moves, the tool shows the change.
2. **Attendance at training.** During training, the coordinator opens the tool on the phone and taps who is there and who is not, with the reason for each absence. Afterwards the coach sees how much effort each player gives to the team. Attendance at games follows.
3. **Set the duties of a game.** The coordinator copies the schedule of the referee committee: at each home game of the club, which club team has referee 1, referee 2, table 1 and table 2, with a pull-down list for each place, and then marks the game as ok. An away game of the team needs drivers.
4. **The message for the next game.** Around training, the coordinator copies one message for the team about the next game: time, place, and who drives or does table or referee duty.
5. **The message for the referee committee.** Before the weekend, the coordinator copies a message with each game, its start time, and which player (first and last name) does which duty.
6. **The overview for the parents.** Now and then, the coordinator shares an overview of the coming games and duties, and how many duties each player has done.

## Priorities
1. **Games from the club's game list** – everything else depends on games, and the coordinator does not want to type them.
2. **Attendance at training, then at games** – done every week, and the most direct reason to leave the sheet. For the coach it shows the effort each player gives to the team.
3. **The start page with the open items, and the page "Season"** – the weekly look at what comes and what is open; the place where later parts show their open items.
4. **Duties and the message for the next game** – the weekly message is the second thing done every week.
5. **The message for the referee committee** – built on the same duties.
6. **The overview for the parents, with duty totals** – makes the fairness visible.

## Principles
- **The coordinator carries the text to the chat.** The tool only produces text to copy. It never posts.
- **The schedule comes from the game list, not from typing.**
- **Duties are given to players.** The child gets the duty, even when a parent drives.
- **One user changes data.** Readers get text, not accounts.
- **Small and correct beats complete.** Add a feature only when it replaces work that is done now.

## Open questions
The first design answered the questions about logging in, the backend language, data storage
and the phone screens (see `docs/decisions/`). Open for a next `/vision`, from the spec of
`docs/changes/done/2026-10-10-games-from-game-list/`:
- The word list calls the game list "the federation's list", but the source appears to be a private site built on the federation's data. Must the meaning change?
- Is a word needed for the home games of other club teams, where players can get table and referee duties (the spec says "possible duty games" in plain words)?

## Change log
| Date | Change |
|------|--------|
| 2026-10-07 | Vision created at the restart: one coordinator, three readers, games from the club's game list first |
| 2026-10-10 | Open questions answered by the first design: Rust backend, Svelte front end, PostgreSQL, login through Pocket ID (decisions 0001–0004) |
| 2026-10-11 | Where we are: the base of the new product runs on the cluster (v0.1.0); games from the game list come next |
| 2026-10-11 | Where we are: choosing the club and team is built (story 2); the schedule comes next |
| 2026-10-12 | Where we are: reading the game list on demand is built (story 3a); the schedule screen comes next |
| 2026-10-12 | Where we are: the schedule is built (story 3b); the list of changes comes next |
| 2026-10-13 | Where we are: the list of changes is built (story 4); the nightly reading comes next |
| 2026-10-14 | Where we are: the nightly reading is built (story 5); the review of the whole of games from the game list comes next |
| 2026-10-14 | Where we are: games from the game list is built and reviewed (all 5 stories); a release comes next |
| 2026-10-14 | Where we are: games from the game list is released (v0.2.0) and runs on the cluster |
| 2026-10-14 | Attendance: the reason for each absence matters (it shows how invested a player is); attendance at games follows attendance at training; words player, attendance and reason |
| 2026-10-15 | Word player: a first name, and usually a last name (the last name is optional) |
| 2026-10-16 | Words for attendance at training: player is a person (in the team through a role); new words role, hidden, weekly training and training day; training exists only when opened or added |
| 2026-10-16 | Words team, player, club and coordinator: a team exists for one season and is the team chosen from the game list; the club has the players |
| 2026-10-17 | Where we are: the first part of attendance at training is built (players, the team with roles, weekly trainings, reasons); trainings and attendance come next |
| 2026-10-17 | Where we are: trainings and marking attendance are built (story B of attendance at training); the totals come next |
| 2026-10-17 | Where we are: the totals for each player are built (story C); attendance at training is complete, its review and a release come next |
| 2026-10-18 | Where we are: attendance at training is built and reviewed as a whole (all 3 stories); a release comes next |
| 2026-10-18 | Where we are: attendance at training is released (v0.3.0) and runs on the cluster |
| 2026-10-18 | Where we are: a nightly encrypted copy of the team database exists (built in `gitops-lab`); encrypted player names come next |
| 2026-10-18 | Where we are: the player names are stored encrypted (v0.3.1, v0.3.2) |
| 2026-10-18 | Word list: "duty place" added (one place at a game that a player of the team must fill), from `/vision duties` |
| 2026-10-25 | The start page and the page "Season" come before duties (new priority 3, journey 0 "Open the tool"), from the design of duty places; words event, checked, open item, Season |
| 2026-10-25 | Where we are: the page "Season" is built (story A of start page and season); checked trainings come next |
| 2026-10-25 | Where we are: checked trainings are built (story C); the start page with the open items comes next |
| 2026-10-25 | Where we are: the start page with the open items is built (story B); all three stories of start page and season are built, the review of the whole and a release come next |
| 2026-10-25 | Start page and season closed after its review of the whole (decision 0022); a release comes next, then duties again from Discover |
| 2026-10-25 | Where we are: start page and season is released (v0.4.0) and runs on the cluster; migration 0010 checked the 8 trainings before that day |
| 2026-10-25 | Duty places are given to club teams: the coordinator copies the schedule of the referee committee (4 places at each home game of the club: referee 1 and 2, table 1 and 2, each for a club team or empty), from the review of story A of duties; word "jury" becomes "table"; words duty place, referee committee changed |
| 2026-10-27 | Where we are: duty places are built and reviewed (story A of duties); players on the places and the totals come next (story B) |
