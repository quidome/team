# Technical spec: Duties

**Status**: done
**Size**: epic
**Change**: [2026-10-18-duties](./)
**Date**: 2026-10-25 (second change of that day: the duty places belong to the game and are given to club teams; it replaces the version approved earlier that day); changed 2026-10-30 for the messages (stories C, D and E)

## Objective
Keep the duties that each game requires (at each home game of the club: referee 1, referee 2,
table 1, table 2, each given to a club team as the referee committee says; at each away game of the
team: its drivers), "Schedule entered" for each home game, the player on each place of the team,
the outcome after the game and the duty totals; show them where the coordinator works (the page of
a game, "Season", the start page, a page "Duties"), and make the two Dutch messages to copy, so the
sheet is no longer needed for duties.

## Requirements summary
From `functional.md` (approved 2026-10-25, second change; spec review notes S1–S23 answered):

- **The model (coordinator, 2026-10-25).** A game belongs to a season, and maybe to a club team (its home or away team). A game has **required duties**: a home game of the club (of any club team, also of the team) has 4 places, referee 1, referee 2, table 1, table 2, each given to a club team or empty; an away game of the team at another club has its drivers, always for the team. The **places of the team** are the table and referee places that name the team plus the driver places. A home game has two checks: (1) "Schedule entered", a tap; (2) players on the places of the team (story B, open places).
- **A – Duty places.** The page of a game (from "Season", "Next", "Last", a duty open item, the list of changes; "Back" to where it came from, to "Season" with the labels and the place kept). At a home game of the club: 4 pull-down lists ("—" and the club teams in the game list of the game's season, sorted by name), each saving at once; the places of the team listed apart ("M16 1: Referee 1, Table 1, Table 2" / "No places for the team."); the button "Schedule entered" (a second tap undoes it; a move or a later change does not). At an away game of the team: drivers as before (proposal one per 3 players with "plays games" on the date, rounded up, "proposed" until set 0–10, "Use proposal", "more than proposed"). "2 places are filled. Clear one first." for drivers; "Clear the player first." for a place of the team with a player. On "Season": the places of the team ("Referee 1: open · Table 1, 2: Anna, open", "No places for the team"), followed by "· Schedule not entered" until tapped (only "Schedule not entered" with no places); drivers ("Drivers (4, proposed): Anna, 3 open"); "all filled"; labels "Duties": "Entered", "Not entered" (home games of the club), "Open places" (home games and away games of the team). Start page: the open item "Home game Sat 31 Oct: schedule not entered" / "5 home games: schedule not entered" (coming, today to 4 weeks ahead; a list for more than one); "Next" also with home games of other club teams where the team has places; "Last" also with the games of the last date before today with places of the team. Edge cases: a moved game keeps everything ("has duties" with the places of the team); a coming game that is gone keeps them, "no longer in the game list", does not count; a past game counts; a swap: "This game is no longer a home game of the club.", does not count, "—" removes; an away game no longer of the team: "This game no longer needs drivers of the team."; a team that leaves the game list keeps its name on the place and in the list; another team chosen: each place keeps its team; past games need only the places of the team and are never an open item for the schedule; an earlier season can be changed; two screens: the last save wins, one place one team; "Not saved. Try again.".
- **B – Who does it, and the totals.** As before, on the places of the team only: the choice with "plays games" on the date (hidden too), by first name, totals of that kind ("2 done · 1 planned"), clashes "also Table 13:00" / "plays 14:30" (the game itself is never a clash), a tap saves at once, "Clear"; done before today, planned from today; "Someone else", "Nobody" ("not done (was Anna)"); the open item "Away game Sat 31 Oct: 2 drivers open" / "Home game Sun 1 Nov: 1 table place open" / "3 games with open places" (table and referee: 4 weeks; drivers: this week and next); the page "Duties" with Drive, Table, Referee; "Not in the team on this date"; "Delete player" not offered with duties; "Anna B.".
- **C – The message for the next game**: as before, with "Tafel en scheidsrechter deze week:", "Tafel:", and open places as "nog N open" for drivers, table and referee.
- **D – The message for the referee committee**: as before, with "Tafel", only the places of the team, "Geen tafel- of scheidsrechtersdiensten deze week."

## Current state
- **Games**: table `games` (key club, season, source id; `in_list`); core `Game`, `schedule::{in_schedule, schedule, ScheduledGame, ScheduleReason}`; `api/season.rs` with `games_of` (the games of a season for its own choice, with the open changes of the current season), `trainings_of`, `EventView` (with `ScheduleGame`, `TrainingRow`), `GET /api/season`.
- **Start page**: core `start_page::{ReadingState, OpenItem, open_item_seasons, open_items, last_training, next_events}` (cancelled trainings show in "Next" but do not count); `api/start.rs` `GET /api/start` (`open_items`, `last`, `next`); front end `+page.svelte`, `start.ts` (`openItemText`, `openItemLink`, `trainingLink`), `EventRow.svelte` (one row of a game or training), `SeasonEvents.svelte`, `Labels.svelte`, `season.ts` (labels, `keepForReturn` / `takeReturn` for a return from a training).
- **Decision 0022**: each open item comes from the function of its source; only the check of a training is stored; one route each for "Season" and the start page.
- **Players and roles**: `team_core::{Player, PlayerRole, Role, role_on, Player::sort_key, same_name}`; `players.rs` with the row lock and the encrypted names (decision 0021); "Delete player" not offered with absences (`trainings::player_has_absences`).
- **Changes**: `team_core::comparison`, `api/changes.rs` (`ChangeView`).
- **Patterns**: rules in the core; worked out at each request (0020); a lock or a unique key for each save that two screens can make, with its race test; logs with ids only; Dutch dates (`today_in_netherlands`); front-end "today" from the server.

## Architecture (shared by all stories)
- **Tables.** Everything hangs on the game key (club, season, source id).
  - `game_duties` (story A): the game key (primary key, foreign key to `games`), `schedule_entered_at` (`timestamptz`, `NULL` = not entered; check 1), `drivers` (`smallint` 0–10, `NULL` = the proposal), `saved_at`. A row is made at the first write to a game, under its row lock, and never deleted.
  - `duty_teams` (story A): the game key (foreign key to `game_duties`), `kind` (`referee`, `table`), `number` (1, 2), `team_number`, `team_name` (the name stays when the team leaves the game list). Primary key (game, kind, number): one place never holds two teams. An empty place has no row.
  - `duty_assignments` (story B): the game key, `kind` (`drive`, `table`, `referee`), `place` (`smallint` ≥ 1), `planned_player` (foreign key to `players`, `ON DELETE RESTRICT`), `outcome` (`as_planned`, `done_by`, `not_done`), `done_by`. Primary key (game, kind, place); unique (game, planned_player). A table or referee assignment is allowed only while its place names the team (checked under the row lock).
- **Core module `duties`** (no I/O, decision 0001): `PlaceKind` (`Referee`, `Table`), `DutyKind` (`Drive`, `Table`, `Referee`), `TeamRef { number, name }`, `required_duties(game, club, team) -> RequiredDuties { schedule, drivers }` (schedule: the home club is the club; drivers: the team is the away team and the home club is another club), `StoredDuties { schedule_entered, drivers, teams }`, `driver_proposal`, `players_playing_on`, `GamePlaces` and `game_places` (the only rule for the duties of one game: the 4 places with their teams, `entered`, the places of the team with open or filled, "proposed", "more than proposed", `no_longer` (no longer a home game of the club / no longer needs drivers of the team), `counts`), `open_places`, `check_team`, `check_drivers`; story B: `Assignment`, `Outcome`, `check_assignment`, `who_did`, `duty_totals` (the only source of the numbers), `display_names`, `clashes`.
- **Core `start_page`**: `OpenItem::ScheduleNotEntered { games }` (home games of the club, coming, in the game list, today to today + 27 days, not entered; story A) and `OpenItem::OpenPlaces { games }` (open table and referee places of the team in the same 4 weeks, open driver places until the Sunday of next week; story B), after the changes not seen and before the trainings; `next_events` also takes the home games of other club teams with places of the team; `last_games` (the games of the last date before today with places of the team), shown also without a past training.
- **Core module `messages`** (C, D): the Dutch texts with Dutch day and month names; exact-text tests.
- **Server.**
  - `duties.rs` (store): the stored duties (and in B the assignments) of a season; the row lock of one game (`INSERT … ON CONFLICT DO NOTHING`, then `SELECT … FOR UPDATE`) for every write: lock, read, core check, write, in one transaction; `set_team`, `set_entered`, `set_drivers`.
  - `club_teams(club, season)`: the home and away teams of the club in the stored games of that season, plus the teams that places of that season still name, sorted by name.
  - `duties_of(state, club, season)` in `api/season.rs`, next to `games_of` and `trainings_of`: the stored duties, assignments and roles a season needs; "Season", the start page, the page of a game and the list of changes use it, so each game carries its `GamePlaces` as a view.
  - Routes (under `/api`, with `utoipa`; front-end types generated): `GET /api/games/{season}/{id}` (with the club teams), `PUT /api/games/{season}/{id}/places/{kind}/{number}` (a team or none), `PUT …/schedule-entered`, `PUT …/drivers` (A); `PUT` / `DELETE /api/games/{season}/{id}/assignments/{kind}/{number}`, `PUT …/outcome`, `GET /api/duties?season=` (B); `GET /api/messages/next-game` (C); `GET /api/messages/committee?week=` (D).
  - `GET /api/start` gets the new open items with their games (date, start time, teams), `last_games` with their season, and places on the games of "Next".
- **Front end.** `/game/[season]/[id]` (A, B); `EventRow.svelte` shows the places of the team and a tap opens the page; `Labels.svelte` and `season.ts` get the group "Duties" ("Entered", "Not entered", "Open places"); the return to "Season" also covers a game; `start.ts` and the start page get the new open items (a list below an item for more than one game), "Last" with games, the part "Messages" (C, D); `/duties` (B); `/messages/next-game` (C) and `/messages/committee` (D); the links "Season · Attendance · Duties · Team · Players · Settings" (B).

| Option | For | Against |
|--------|-----|---------|
| One row for each place that names a team (chosen) | The database keeps one team per place; an empty place has no row; story B links to it | Two tables in story A |
| Four team columns on the game row | One table | One team per place only under the lock; story B links players to columns |
| "Schedule entered" as a time on the game row (chosen) | One stored fact, apart from the places (two checks) | – |
| A row with 0 and 0 means "handled" (the earlier design) | No extra column | Mixes check 1 with the places; dropped |
| Club teams from the stored games of the season (chosen) | No new source request; the teams of that season | A team with no game that season is missing (it has no home game to get places at either) |
| The places travel on the answers of "Season" and the start page (kept) | One request per page; the same `GamePlaces` everywhere | Larger answers |
| A row per filled place with a place number (story B, kept) | The database stops two players on one place | A place number the coordinator never sees |
| Totals worked out at each request (kept) | Corrections, moved and gone games always right (0020, 0022) | A little work per page |
| Messages in the core (kept) | The rules in one place, exact-text tests | Dutch words in the core |
| Store only a changed number of drivers (kept) | "Proposed, follows the team until changed" | `NULL` has a meaning |

## Stories
| # | Story | Type | Scope | Depends on | Closure | Release | Status |
|---|-------|------|-------|------------|---------|---------|--------|
| A | [Duty places](../2026-10-18-duty-places/) | feature | Migration 0011 rewritten: `game_duties` (`schedule_entered_at`, `drivers`) and `duty_teams`. Core `duties`: `required_duties` with all cases first (risk 2), `driver_proposal`, `game_places`, `open_places`, `check_team`, `check_drivers`; `start_page`: `ScheduleNotEntered` with its list, "Next" with home games of other club teams with places of the team, `last_games`. Store with the row lock; `club_teams`; `duties_of`. Routes: a game; a place; "Schedule entered"; drivers. The page of a game (4 pull-down lists, saved at once). The places on the rows of "Season" and of the start page; the labels "Duties"; "Back" keeps the labels. "has duties" in the list of changes (also after a swap). The findings of the first build that stay: R2, R3, R7, R8, A6. Rework of the first build (branch `duty-places-v1`, on disk). Measure the start page. | – | review build + accept | 1, with B | done |
| B | [Who does each duty place, and the totals](../2026-10-18-who-does-each-duty/) | feature | Migration `duty_assignments`. Core `check_assignment` (a table or referee place only while it names the team; "Clear the player first." for `check_team`), `who_did`, `duty_totals`, `display_names`, `clashes` (the game itself is no clash); the open item `OpenPlaces` with its two windows. Routes: fill, clear, correct; the choice with totals and clashes; the totals of a season. The page "Duties" (Drive, Table, Referee) and its link. "Delete player" with duties. Measure the start page again. Decision record (the rows, the two checks, what counts). | A | review build + accept | 1, with A | done |
| C | [The message for the next game](../2026-10-29-next-game-message/) | feature | Built and closed, not released. Replaced by C2 before a release. | B | review build + accept | – | done |
| C2 | [The message of the week](../2026-10-30-message-of-the-week/) | feature | Core `week_of`, `week_message` (replace `coming_week`, `next_game_message`), shared lines for table and referee, past places through `who_did`. `GET /api/messages/week`, the flag `messages.week`, the page `/messages/week`, the button. | C | review build + accept | when it suits the coordinator | done |
| D | [The committee message](../2026-10-31-committee-message/) | feature | Core message with full names, on the lines of C2, for this week. `GET /api/messages/committee`, the page `/messages/committee`, the button "Committee message" whenever a team is chosen. | C2 | review build + accept | when it suits the coordinator | done |
| E | [The duty overviews](../2026-10-31-duty-overviews/) | feature | Core driving overview and table and referee overview of all coming duties, on the lines of C2. Routes, the pages `/duties/driving` and `/duties/table-referee`, two buttons on "Duties" for the current season. | C2 | review build + accept | when it suits the coordinator | done |

Each story has its own change folder (type `feature`, stage `design`) and its own `/design`, which
writes its steps, routes, screens and exact tests from this spec and `functional.md`. Story A was
built once with the earlier design (7 steps, reviewed: R1–R9, A1–A7); that code was reworked on `main`, and
the branch `duty-places-v1` was deleted on 2026-10-30.

**Risks.** 1. (Closed: story A is on `main`.) 2. The
rules of the places change everywhere: step 1 of A makes the core rules complete, with all cases
tested, before the screens. 3. The start page gets more games (every home game of the club in 4
weeks): measure `GET /api/start` in A (limit about 200 ms locally). 4. The clipboard on a phone
(C): a spike first.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| The duties belong to the game: 4 places at each home game of the club, each given to a club team (number and name stored) or empty; drivers at an away game of the team, always for the team | The committee plans by team; copying its schedule as it is is faster and cannot go wrong (coordinator, 2026-10-25) | Yes, at the close of story B (with the next rows) |
| "Schedule entered" is a stored fact of a home game (a time), apart from the places; a move or a later change does not undo it | Two checks of a game (coordinator, 2026-10-25) | Same record |
| The places of the team are worked out from the team chosen for the game's season; another team chosen moves nothing | Each place keeps its team | Same record |
| The club teams of a pull-down list come from the stored games of that season, plus the teams that places still name | No new source request; a name never disappears | Same record |
| A row per filled place of the team with a place number; an open place has no row; one player per place, one place per player per game; a table or referee place gets a player only while it names the team | No lock for a fill; two screens never put two players on one place | Same record |
| After the game a filled place counts as done by its planned player, unless corrected to someone else or "not done" (the planned player kept) | Fast to keep, with a trace for fairness (S5) | Same record |
| What counts: done before today, planned from today (Dutch date); coming games that are gone or no longer require the duty do not count; past games count, also when gone | The totals must be right and checkable (S2) | Same record |
| `game_places` and `duty_totals` are the only rules; "Season", the page of a game, the start page, the list of changes and "Duties" all use them | One number everywhere (decision 0022) | Same record |
| The number of drivers stored only when the coordinator sets it; never shown below the filled driver places | "Proposed, follows the team until changed"; nobody removed silently (S1) | Same record |
| Duties hang on the game key (club, season, source id); a new id from the source leaves them on the gone game | The game list is the truth (0013) | Same record |
| Duties of an earlier season can be changed and corrected | As trainings (S13) | Same record |
| The open items of duties: schedule not entered and table and referee places in 4 weeks (coming games only), drivers until the Sunday of next week; each with its games in the answer | Coordinator, 2026-10-25 (S16, S19); the item and its list the same (S11) | Same record |
| The messages are made in the core, in Dutch | Rules in one place, exact-text tests | Yes, at the close of story C |

## Not in scope
- Reading the committee's sheet into the tool; naming the tasks at the table; a departure time; attendance at games; the report of what each player did; counts in the duty overviews; another week than this one in a message; proposing a player; message templates or other languages; duties of other kinds; totals over more than one season; a "wave" of places as a thing in the tool; a date "schedule entered until"; handling several games at once.

## Test plan
- **Existing tests that change or must still pass**: the tests of the first build of story A (`tests/games.rs`, `tests/start.rs`, `tests/season.rs`, `tests/changes.rs`, core `duties` and `start_page`, `web/e2e/duties.e2e.ts`): referee and jury numbers become places with teams and "Schedule entered"; `choose.e2e.ts` (the open item with its new text); `tests/players.rs` and `tests/player_logs.rs` (new routes need a login); every browser test that checks the links (with "Duties", story B).
- **New tests**, made exact by each story's `/design`. A: core `required_duties` for each case (home game of another club team, of the team, away game of the team at another club, in the club's own hall, away game of another club team), `game_places` (not entered, entered with empty places, places of the team, places of other teams only, a swap, a gone coming game, a gone past game, another team chosen), `ScheduleNotEntered` (4 weeks, past and gone left out), `last_games` (all games of one date), `check_team` for each refusal; server: a team on a place, "—", an unknown team refused, "Schedule entered" on and off, a move keeps teams and "Schedule entered", a team that leaves the game list keeps its name and stays in `club_teams`, the race test of the row lock, "has duties" after a swap (R3), "Last" without a past training (R2), an earlier season; browser: the walk through "Not entered" (4 choices, "Schedule entered", Back, the next game), the row with "· Schedule not entered" (S18), "Not saved. Try again.", an empty drivers field cannot be saved (R8), the open item and its list, a phone width without sideways scroll, accessibility. B: as before, plus a player only on a place of the team, "Clear the player first.", the game itself is no clash (S22). C and D: exact texts with "Tafel"; "nog N open" for table and referee places (S23).
- **Browser tests**: `duties.e2e.ts` in its own Playwright project after `start`; clipboard permission for "Copy"; the clock stays at 12 October 2026.
- **Checks**: while working only the tests of the changed part; `just check` once before a commit.

## Compatibility and performance
- New tables and routes (migration 0011 is not released, so it is rewritten); the answers of `GET /api/season`, `GET /api/start` and `GET /api/changes` get new fields (the front-end types are generated).
- A tap on a game opens its page (before: nothing). The start page links become "Season · Attendance · Duties · Team · Players · Settings" (B).
- "Delete player" is also not offered for a player with duties (B).
- `GET /api/start` reads more: the duties of one or two seasons, every home game of the club in 4 weeks, and the roles for the proposal; measured in A and B (limit about 200 ms locally).
- Player names in the messages and on the pages come decrypted from `players.rs` (decision 0021); never in a log. Team names are not secret.

## Progress
- [x] Story A – duty places
- [x] Story B – who does each duty place, and the duty totals (release 1 with A; closed 2026-10-28, decision 0023)
- [x] Story C – the message for the next game (closed 2026-10-29, decision 0024)
- [x] Story C2 – the message of the week (closed 2026-10-31, decision 0025)
- [x] Story D – the committee message (closed 2026-10-31, decision 0026)
- [x] Story E – the duty overviews (closed 2026-10-31)
- [x] Epic review accept (2026-10-31, accepted with notes)

## Deviations
None from `functional.md`. Readings of the spec that the design makes exact:
- The window "4 weeks" is today to today + 27 days (Dutch date); the window of the drivers is the window of "Next".
- "Schedule entered" is a fact of a home game of the club only; an away game of the team needs none and is never in the open item "schedule not entered".
- "Last" shows the games of the last date before today with places of the team even when there is no past training (finding R2 of the first build of story A).

## Review findings
Epic review accept, 2026-10-31, by a fresh agent (`/review accept`), of the whole result of stories
A to E on `main` at `72e7575`. The arguments held only the mode and the change folder; nothing was
ignored.

**Verdict: accepted with notes.** The whole weekly walk works and matches the intent: after this
epic the sheet is no longer needed for duties and the weekly messages. No difference needs a
correction before the release; A1 is the one to look at.

Read: `idea.md`, `functional.md`, `change.md`, `docs/vision.md` (journeys 3 to 6),
`docs/product/duties.md` and `messages.md`, the core `duties`, `messages` and `start_page`, the
routes of the season, the start page and the messages, and the start page, "Duties" and `duties.ts`
of the front end. The view was formed before the Deviations were read. `docs/changes/done/` was not
read, so the findings of the reviews of each story are not repeated or checked here.

Checked by running:
- Tests: `cargo test -p team_core --lib` 298 passed; `cargo test -p team --test` for `duties`,
  `messages`, `start`, `season`, `games`, `changes` and `players` 99 passed; `npx playwright test`
  135 passed. `just check` and the other server test files were not run.
- The real server with the fake game list (real answers of the club) and a throw-away database,
  since dropped: club and team chosen, the game list read, 9 players (two named Anna, one without a
  last name), places entered, players put on them; clock on Wed 28 Oct, Sun 1 Nov and Mon 2 Nov, with
  "Someone else" and "Nobody" on the Sunday. All four texts came out as the spec says: "Thuis" and
  "Uit", "Rijden" only at away games, "Scheidsrechter:" before "Tafel:", "Anna B." and full names,
  "tijd nog niet bekend", "nog N open", "niemand", the week heading, no game of the team before today,
  places of other club teams left out, the "Geen …" texts. The totals followed the corrections and
  matched the choice on the page of a game.
- A walk in a Chromium browser at phone width over the same data: the start page, the page of a game
  (past, away, with the choice), "Duties", the overviews, "Copy" with the text read back from the
  clipboard, "Back", "Season"; no console errors. This matters because the browser tests of the
  messages use fixed answers of the server (`web/e2e/messages.e2e.ts`). `GET /api/start`: about 7 ms
  locally on 71 games (limit about 200 ms).

| Story | Judgement | Notes |
|---|---|---|
| A Duty places | matches | A1, A4 |
| B Who does it, the totals | matches | A1, A3 |
| C The message of the week | matches | A5 |
| D The committee message | matches | – |
| E The duty overviews | matches | A2 |

- **A1 – Neutral drift, closest to concerning: "all filled" hides the names and a correction.** The
  - 2026-10-31: Not fixed: the coordinator keeps the behaviour; a line in `docs/ideas.md`.
  start-page example in `functional.md` shows names on full rows ("Last": "Table 1, 2: Eva, Fleur"),
  while story A says places with all players chosen show "all filled". The build follows the
  criterion. So after the weekend "Last" says "all filled" without who did it, and a row where a place
  was corrected to "Nobody" also says "all filled", because a place counts as filled while a player is
  on it, whatever the outcome (`scheduleText` and `driversText` in `web/src/lib/duties.ts`). Story B
  wants the planned player to stay visible; on the page of the game they do, and the totals are right.
  The harm is small: "Last" is where the coordinator looks for what went otherwise. Checked: a run for
  the rows with an open place ("Referee 1: Iris (was Eva) · Table 1, 2: not done (was Anna B.), open");
  a trace of `duties.ts` for a full row with "Nobody", not run. For the coordinator: keep it, or a line
  in `docs/ideas.md`.
- **A2 – Faithful adaptation: the overview buttons without a chosen team.** Story E does not say what
  - 2026-10-31: Not fixed: faithful to the spec.
  happens without a team. The buttons show and the page says "No overview: no team chosen.", as the
  committee message does. Checked: search for `no team chosen` in `web/src`.
- **A3 – Faithful adaptation: players who cannot be chosen stay in the list, with the reason.** Story B
  - 2026-10-31: Not fixed: faithful to the spec.
  says such a player "is not offered"; the choice shows them, not tappable, with for example "already
  at this game". That answers "why is Bram not there" without a question. Also, a number of drivers
  below a filled place with a gap says "Driver place 3 has a player. Clear it first.", because "2
  places are filled" would not be true of the cause. Checked: a run of the choice; search for `Clear
  one first` in `web/src`.
- **A4 – Neutral drift: "Last" can show a past away game where nobody was ever chosen.** Places of the
  - 2026-10-31: Not fixed: in the same line in `docs/ideas.md` as A1.
  team include the proposed drivers, so on Wed 28 Oct "Last" held the away game of 10 Oct, with only
  the proposal behind it. This is the letter of the spec and costs one row; it also shows a game that
  was forgotten. Checked: a run (`last_games` of `GET /api/start`); the text of the row by a trace.
- **A5 – Neutral drift: `docs/product/messages.md` says too little about "Messages".** Its first
  - 2026-10-31: For Close.
  bullet says the part is not shown when the week has nothing to announce; with a chosen team the
  part always shows, because of "Committee message" (as the spec says, and as a later bullet there
  says). For `/close`. Also for `/close`: the top of `change.md` still says four stories, "jury", the
  message for the next game and "after story D". Checked: read, and a run on Mon 2 Nov
  (`messages.committee` true in a week without places).
- **Not differences, for the record.** On a Sunday evening the message of the week can hold only days
  that have passed, with "Tafel: niemand, niemand" in a text for the parents. That is what the spec
  says (S24, S25, R1 of story C2), so it is no finding, but it sits oddly with "messages announce what
  comes"; real use decides (the line of 2026-10-29 in `docs/ideas.md`). The other known limits are in
  `docs/ideas.md` too (past places of an old team, 2026-10-28; a player who left and stays named on a
  coming duty, 2026-10-31), and neither stands in the way of the weekly walk.
- **Not checked.** "Copy" on the coordinator's own phone (in `docs/backlog.md`; only Chromium was run).
  Real use with the committee's real schedule: the time of a walk through the home games (S20).
  Moved, gone and swapped games were not run again: read in `game_places` and `week_message`, and
  covered by the tests above. Deviations: "None", with three readings; all three are faithful and
  match what ran. A1 is the only place where the build and an example of the spec differ.
