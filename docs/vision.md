# Product vision: team

## Purpose
The tool shows what each player gives to the team: the effort at training and games, and a fair
share of the duties.

## The user's problem space
One coordinator runs one youth team in a club. Each week the same work comes back: which games
are there, who is at training, who drives to away games, and which players cover the table and
referee places that the referee committee gives to the team. The team talks in a WhatsApp group; the coordinator
keeps the record. Until this tool, the record was a Google Sheet, because the first version of the
tool was not good enough. Keeping it up to date by hand, and writing the same kinds of messages
again and again, cost time.

Three groups read what the coordinator produces:
- **The team and the parents**: when their child plays, when they must drive or do table or referee duty, and whether the work is shared equally.
- **The club's referee committee**: which of our players do which duty at the games of this week.
- **The coordinator**: who makes an effort for the team, what happened, and what must still be arranged.

## What this product is
A small tool for one coordinator. It keeps the games, attendance and duties of one team correct,
and turns them into text that the coordinator copies into the chat.

It is not:
- a tool for the whole club, or for several coordinators;
- a place where parents or players log in;
- connected to WhatsApp or any other messenger;
- a replacement for the federation's schedule. It reads that schedule.

## Core values
1. **See who makes an effort.** Attendance, with the reason for each absence, shows the coordinator who puts time into the team. It is for the coordinator; the coordinator may copy parts of it into the team chat.
2. **Share the duties fairly and easily.** Giving the duties takes little work, and parents and players can see who did what, so they trust the division.

Both values need fast capture and correct data. For example, attendance at training takes seconds, on a phone, with one hand. The stored data is the truth and must be right.

## Where we are
- **Fair duties: reached.** The games come in from the game list, the coordinator gives the duty places to players, the duty totals show the split, and the messages and duty overviews come from the tool.
- **Effort: reached.** Attendance at training and at games is recorded with the reason for each absence, and the report shows the attendance and the duties of each player side by side, with the details behind the numbers.
- **The sheet is no longer in use** (since about 31 October 2026). The product is successful when the coordinator sees the effort of each player and shares the duties fairly from the tool, and no longer needs the sheet for it: that is reached.

What each release brought is in `docs/releases.md`; what each feature does is in `docs/product/`.

## North star
Each week, the coordinator spends a few minutes in the tool and none in the sheet. The coordinator
knows who makes an effort for the team. Every
message that goes to the team, the parents or the referee committee comes from the tool,
correct the first time. No one questions whether the duties are shared fairly, because they can
see it.

## Where we are going
The parts that replace the sheet are built: the games from the game list, attendance at training
and at games, the start page and the page "Season", duties with their messages and duty overviews,
and the report of what each player did. Now the tool must take less of the coordinator's time:
it asks for attention only when there is something to do, and each page shows only what that work
needs. Real use will show many small improvements, and maybe new features; each one must serve
the purpose.

## Longer horizon
Not for now:
- more coordinators in the club;
- a page that parents can open without asking the coordinator;
- past results and statistics from the game list.

These stay ideas until real use shows the need.

## User journeys
0. **Open the tool.** A few times a week, the coordinator opens the tool on the phone and sees the next events and the open items: what still needs the coordinator. One tap completes the record of the last training. The whole season is one tap further, for any game or training.
1. **Games come in.** The coordinator gets all games of the club from the federation's game list. This includes the games of other club teams, because the club's home games have table and referee places that the club teams fill. When a game moves, the tool shows the change.
2. **Attendance at training.** During training, the coordinator opens the tool on the phone and taps who is there and who is not, with the reason for each absence. Afterwards the coach sees how much effort each player gives to the team.
3. **Set the duties of a game.** The coordinator copies the schedule of the referee committee: at each home game of the club, which club team has referee 1, referee 2, table 1 and table 2, with a pull-down list for each place, and then marks the game as ok. An away game of the team needs drivers.
4. **The message of the week.** Before the weekend, the coordinator taps a button on the start page and copies one message for the team and the parents about this week (Monday to Sunday): each game of the team (time, place, home or away), the drivers of an away game, and which players have the team's table and referee places this week. The coordinator puts a short poll under it in the chat ("who comes?"), one for each game. Trainings are not in it: the team knows them. A week with nothing to announce has no message.
5. **The committee message.** Before the weekend, the coordinator copies a message for the referee committee with each game of this week where the team has table or referee places, its start time, and which player (first and last name) does which duty.
6. **The duty overviews.** After a new or changed schedule, or whenever the coordinator wants, the coordinator shares an overview of all coming duties: one for driving, one for table and referee. The overviews show who does what, not how much each player has done.
7. **Attendance at games.** After the coordinator announces the week, players say in the chat that they come or not. During the week the coordinator opens the tool and updates who comes to the coming game, who does not, and who has not answered. From 3 days before the game the start page warns when fewer than 7 players come, so extra players can be found from other teams; when a player who must drive does not come, it warns at once. From the start of the game the coordinator records who was there, and confirms it with "Checked".
8. **The report of what each player did.** Now and then, to prepare or steer a conversation, the coordinator opens one page for the current season with a row for each player: attendance at trainings and games as a percentage, and driving, table and referee, done and planned. A tap on a player shows the details: how many absences were valid and not valid, and how many duties someone else took over. It is for the coordinator; to show an imbalance, the coordinator may copy a part of it into the team chat.

## Priorities
1. **Games from the club's game list** – everything else depends on games, and the coordinator does not want to type them.
2. **Attendance at training** – done every week, and the most direct reason to leave the sheet. For the coach it shows the effort each player gives to the team.
3. **The start page with the open items, and the page "Season"** – the weekly look at what comes and what is open; the place where later parts show their open items.
4. **Duties and the message of the week** – the weekly message is the second thing done every week.
5. **The committee message** – built on the same duties.
6. **The duty overviews** – show the team and the parents who does which coming duty.
7. **The report of what each player did** (built) – the place where the effort and the duties of each player come together; the main goal of the tool. With trainings, games and duties.
8. **Attendance at games** (built) – shows during the week if there are enough players; adds the games to the report.

## Principles
- **The coordinator carries the text to the chat.** The tool only produces text to copy. It never posts.
- **The schedule comes from the game list, not from typing.**
- **Duties are given to players.** The child gets the duty, even when a parent drives.
- **One user changes data.** Readers get text, not accounts.
- **Small and correct beats complete.** Add a feature only when it replaces work that is done now.

## Open questions
None now. The first design answered the questions about logging in, the backend language, data
storage and the phone screens (see `docs/decisions/`).

## Change log
| Date | Change |
|------|--------|
| 2026-10-07 | Vision created at the restart: one coordinator, three readers, games from the club's game list first |
| 2026-10-10 | Open questions answered by the first design: Rust backend, Svelte front end, PostgreSQL, login through Pocket ID (decisions 0001–0004) |
| 2026-10-14 | Attendance: the reason for each absence matters (it shows how invested a player is); attendance at games follows attendance at training; words player, attendance and reason |
| 2026-10-15 | Word player: a first name, and usually a last name (the last name is optional) |
| 2026-10-16 | Words for attendance at training: player is a person (in the team through a role); new words role, hidden, weekly training and training day; training exists only when opened or added |
| 2026-10-16 | Words team, player, club and coordinator: a team exists for one season and is the team chosen from the game list; the club has the players |
| 2026-10-18 | Word list: "duty place" added (one place at a game that a player of the team must fill), from `/vision duties` |
| 2026-10-25 | The start page and the page "Season" come before duties (new priority 3, journey 0 "Open the tool"), from the design of duty places; words event, checked, open item, Season |
| 2026-10-25 | Duty places are given to club teams: the coordinator copies the schedule of the referee committee (4 places at each home game of the club: referee 1 and 2, table 1 and 2, each for a club team or empty), from the review of story A of duties; word "jury" becomes "table"; words duty place, referee committee changed |
| 2026-10-30 | Messages announce this week (Monday to Sunday): the message for the next game becomes the message of the week (journey 4, a button on the start page); the committee message covers the same week (journey 5); the overview for the parents becomes the duty overviews without totals (journey 6); new journeys 7 (attendance at games) and 8 (the report of what each player did, for the coordinator for now); words message of the week, committee message, duty overview, report |
| 2026-10-31 | Attendance at games is for the week before the game: who comes, who does not, who has not answered, with a warning for too few players and for a driver who does not come (journey 7, priority 7); words attendance and checked also cover games |
| 2026-10-31 | Words extra player and not played (a game that did not or will not take place stops counting for duties and attendance), from `/discover` of attendance at games |
| 2026-11-03 | Purpose made explicit: the tool shows what each player gives to the team. Core values: 1 see who makes an effort, 2 share the duties fairly and easily; fast capture and correct data are a condition for both. Success tied to these goals. The report comes before attendance at games (priorities 7 and 8 change places). Word contribution |
| 2026-11-03 | "Where we are" describes the product compared to its purpose (fair duties reached, effort halfway, the sheet still in use); the releases move to `docs/releases.md`; the change log keeps only changes to the vision |
| 2026-11-08 | "Where we are": attendance at games counts in the totals of a season; effort stays halfway until the report |
| 2026-11-08 | Attendance at games is built: "Where we are going", journeys 2, 7 and 8 and the priorities say so; the report of what each player did is the next step, with trainings, games and duties. Word checked: a game can be checked from its start time |
| 2026-11-09 | Journey 8: the report is one page with a table (attendance as percentages, duties done and planned) and the details of a player on a tap (valid and not valid absences, duties taken over), for the current season, used now and then to steer a conversation; a part can be copied to show an imbalance. Word report: one or more tables |
| 2026-11-09 | The report is built: "Where we are" says effort is reached in the tool (the sheet still to be replaced in real use); "Where we are going" and priority 7 say built |
| 2026-11-11 | Words team game (a game the team plays; its only duty is drive, at an away game) and duty game (a home game of another club team where the team has table or referee places); "checked" means an event is done in the administration and no longer asks for attention, for every event, also a duty game (was: only attendance is checked). Answers the open question about a word for these games. From `/vision` of "every past event that is not done is an open item" |
| 2026-11-14 | The sheet is no longer in use (since about 31 October): "Where we are" says the purpose is reached, and the problem space speaks of the sheet in the past. "Where we are going": less of the coordinator's time, attention only when there is something to do, and small improvements from real use. The open question about the game list closes: the word stays, its meaning says the list comes from the federation's data. New words season, half season and competition |
