# Product vision: team

## The user's problem space
One coordinator runs one youth team in a club. Each week the same work comes back: which games
are there, who is at training, who drives to away games, and which players cover the jury and
referee places that the team must fill. The team talks in a WhatsApp group; the coordinator
keeps the record. Today the record is a Google Sheet, because the first version of this tool was
not good enough. Keeping it up to date by hand, and writing the same kinds of messages again and
again, costs time.

Three groups read what the coordinator produces:
- **The team and the parents**: when their child plays, when they must drive or do jury duty, and whether the work is shared equally.
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
- Choosing the club and team is built and reviewed (story 2 of games from the game list), not yet released: the club by name from the source's club list, the team from the club's own teams, again at each new season; the choice of each season is kept. The schedule from the game list comes next. The sheet is still in use.
- The product is successful when the coordinator stops using the sheet.

## North star
Each week, the coordinator spends a few minutes in the tool and none in the sheet. Every
message that goes to the team, the parents or the referee committee comes from the tool,
correct the first time. No one questions whether the duties are shared fairly, because they can
see it.

## Where we are going
First the games come in from the club's game list, so the coordinator never types a schedule.
Then attendance at training. Then duties and the messages built on them. Each step must be good
enough to replace that part of the sheet.

## Longer horizon
Not for now:
- more coordinators in the club;
- a page that parents can open without asking the coordinator;
- past results and statistics from the game list.

These stay ideas until real use shows the need.

## User journeys
1. **Games come in.** The coordinator gets all games of the club from the federation's game list. This includes the games of other club teams, because our players do jury duty at those games when we play at home. When a game moves, the tool shows the change.
2. **Attendance at training.** During training, the coordinator opens the tool on the phone and taps who is there and who is not.
3. **Set the duties of a game.** The coordinator sets how many duties a game needs. An away game needs drivers. A home game means jury or referee places at other club games.
4. **The message for the next game.** Around training, the coordinator copies one message for the team about the next game: time, place, and who drives or does jury duty.
5. **The message for the referee committee.** Before the weekend, the coordinator copies a message with each game, its start time, and which player (first and last name) does which duty.
6. **The overview for the parents.** Now and then, the coordinator shares an overview of the coming games and duties, and how many duties each player has done.

## Priorities
1. **Games from the club's game list** – everything else depends on games, and the coordinator does not want to type them.
2. **Attendance at training** – done every week, and the most direct reason to leave the sheet.
3. **Duties and the message for the next game** – the weekly message is the second thing done every week.
4. **The message for the referee committee** – built on the same duties.
5. **The overview for the parents, with duty totals** – makes the fairness visible.

## Principles
- **The coordinator carries the text to the chat.** The tool only produces text to copy. It never posts.
- **The schedule comes from the game list, not from typing.**
- **Duties are given to players.** The child gets the duty, even when a parent drives.
- **One user changes data.** Readers get text, not accounts.
- **Small and correct beats complete.** Add a feature only when it replaces work that is done now.

## Open questions
The first design answered the questions about logging in, the backend language, data storage
and the phone screens (see `docs/decisions/`). Open for a next `/vision`, from the spec of
`docs/changes/2026-10-10-games-from-game-list/`:
- The word list calls the game list "the federation's list", but the source appears to be a private site built on the federation's data. Must the meaning change?
- Is a word needed for the home games of other club teams, where players can get jury and referee duties (the spec says "possible duty games" in plain words)?

## Change log
| Date | Change |
|------|--------|
| 2026-10-07 | Vision created at the restart: one coordinator, three readers, games from the club's game list first |
| 2026-10-10 | Open questions answered by the first design: Rust backend, Svelte front end, PostgreSQL, login through Pocket ID (decisions 0001–0004) |
| 2026-10-11 | Where we are: the base of the new product runs on the cluster (v0.1.0); games from the game list come next |
| 2026-10-11 | Where we are: choosing the club and team is built (story 2); the schedule comes next |
