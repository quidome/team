# Technical spec: Duties

**Status**: approved
**Size**: epic
**Change**: [2026-10-18-duties](./)
**Date**: 2026-10-25 (replaces the version approved on 2026-10-18: the data and the rules stay, the screens change)

## Objective
Keep the team's duty places (drivers at its away games; jury and referee places at home games of
other club teams), the player on each place, the outcome after the game and the duty totals, show
them where the coordinator works (the page of a game, "Season", the start page, a page "Duties"),
and make the two Dutch messages to copy, so the sheet is no longer needed for duties.

## Requirements summary
From `functional.md` (approved again 2026-10-25; spec review notes S1–S17 answered):

- **A – Duty places.** The page of a game (from "Season", "Next", "Last", a duty open item, the list of changes; "Back" to where it came from, to "Season" with the labels and the place kept). At a possible duty game: referee and jury places, each 0–2; a save makes the game **handled**, also with 0 and 0 ("No duty places for the team."); before a save "Not handled yet". At an away game of the team (at another club): drivers, proposed as one per 3 players with the role "plays games" on the date (rounded up), "proposed" and following the team until the coordinator sets a number (0–10), which stays; "Use proposal"; a proposal below the filled driver places keeps them ("more than proposed"); an away game is always handled. A home game of the team has none. A number below the filled places is refused ("2 places are filled. Clear one first."). On "Season" each game that can have duties shows its places ("Referee: open · Jury: Anna, open", "Drivers (4, proposed): Anna, 3 open", "No duty places", "Not handled yet", "all filled"); a new label group "Duties" ("Handled", "Not handled", "Open places"), only for possible duty games and away games. On the start page: the open item "Possible duty game Sat 31 Oct not handled" / "5 possible duty games not handled" (today to 4 weeks ahead; for more than one, a tap opens the list of those games); "Next" also shows the possible duty games with places; "Last" also shows the last game before today with places. Moved games keep places ("has duties" in the list of changes); a coming game that is gone keeps them, "no longer in the game list", does not count; a past game counts, also when gone later; a game that no longer needs duties of the team keeps them with "This game no longer needs duties of the team." and does not count (coming); another team chosen mid-season: the same rule. An earlier season can be changed and corrected, as trainings.
- **B – Who does it, and the totals.** A tap on a place: the players with "plays games" on the date (hidden too), by first name, each with that kind's totals in the current season ("2 done · 1 planned"), no proposal; a clash shows next to the name ("also Jury 13:00", "plays 14:30") without blocking; a player with a place at the same game is not offered. A tap saves at once; "Clear". Before today a filled place is done, from today planned. On a past game: "Someone else", "Nobody" ("not done (was Anna)"). The open item "Away game Sat 31 Oct: 2 drivers open" / "Possible duty game Sun 1 Nov: 1 jury place open" / "3 games with open places" (jury and referee: 4 weeks; drivers: this week and next; for more than one, a list). The page "Duties" (links "Season · Attendance · Duties · Team · Players · Settings"): "Totals", a row for each player with "plays games" in the season or a duty in it, Drive, Jury, Referee as "done · planned", a season choice; the same numbers as in the choice. "Not in the team on this date"; "Delete player" not offered with duties; two screens: last save wins, never two players on a place; "Not saved. Try again."; "Anna B." for two players with the same first name.
- **C – The message for the next game**: in a part "Messages" on the start page, below "Next"; Dutch text with "Copy" ("Copied"; "Select the text and copy it." on failure); the next game of the team in the game list (date, time or "tijd nog niet bekend", "Thuis" / "Uit tegen", hall and town; "Rijden:" with "(nog N open)" for an away game), and "Jury en scheidsrechter deze week:" (today until the next Sunday); without a coming game only the duty part; "Anna B.".
- **D – The message for the referee committee**: in "Messages"; Monday to Sunday of the week of the coming weekend, "Next week", "Previous week"; each game with the team's jury or referee places, full names, "nog open"; gone games and games that no longer need duties left out; "Geen jury- of scheidsrechtersdiensten deze week."

## Current state
- **Games**: table `games` (key club, season, source id; `in_list`); core `Game`, `schedule::{in_schedule, schedule, ScheduledGame, ScheduleReason}`; `api/season.rs` with `games_of` (the games of a season for its own choice, with the open changes of the current season), `trainings_of`, `EventView` (with `ScheduleGame`, `TrainingRow`), `GET /api/season`.
- **Start page**: core `start_page::{ReadingState, OpenItem, open_item_seasons, open_items, last_training, next_events}` (cancelled trainings show in "Next" but do not count); `api/start.rs` `GET /api/start` (`open_items`, `last`, `next`); front end `+page.svelte`, `start.ts` (`openItemText`, `openItemLink`, `trainingLink`), `EventRow.svelte` (one row of a game or training), `SeasonEvents.svelte`, `Labels.svelte`, `season.ts` (labels, `keepForReturn` / `takeReturn` for a return from a training).
- **Decision 0022**: each open item comes from the function of its source; only the check of a training is stored; one route each for "Season" and the start page.
- **Players and roles**: `team_core::{Player, PlayerRole, Role, role_on, Player::sort_key, same_name}`; `players.rs` with the row lock and the encrypted names (decision 0021); "Delete player" not offered with absences (`trainings::player_has_absences`).
- **Changes**: `team_core::comparison`, `api/changes.rs` (`ChangeView`).
- **Patterns**: rules in the core; worked out at each request (0020); a lock or a unique key for each save that two screens can make, with its race test; logs with ids only; Dutch dates (`today_in_netherlands`); front-end "today" from the server.

## Architecture (shared by all stories)
- **Tables.**
  - `game_duties` (story A): `club_number`, `season_year`, `source_id` (primary key, foreign key to `games`), `referee` and `jury` (`smallint`, `CHECK` 0–2), `drivers` (`smallint` 0–10, `NULL` = the proposal). A row exists from the first save; **a row with 0 and 0 stays and means "handled, no places"**. A possible duty game without a row is not handled; an away game needs no row.
  - `duty_assignments` (story B): the game key (foreign key to `game_duties`), `kind` (`drive`, `jury`, `referee`), `place` (`smallint` ≥ 1), `planned_player` (foreign key to `players`, `ON DELETE RESTRICT`), `outcome` (`as_planned`, `done_by`, `not_done`), `done_by` (player, only with `done_by`). Primary key (game, kind, place): one place never holds two players. Unique (game, planned_player): one place per player per game. An open place has no row.
- **Core module `duties`** (no I/O, decision 0001): `DutyKind`, `DutyNeed` and `duty_need` (from `in_schedule`; `DutyGame` → jury and referee; `TeamGame` with the team away and the home club another club → drive; else none), `StoredPlaces`, `driver_proposal`, `GamePlaces` and `game_places` (the only rule for the places of one game: state `NotHandled` / `NoPlaces` / `Places` / `NoLongerNeeded` / `Gone`, each place open or filled, "proposed", "more than proposed"), `handled`, `open_places`, `check_places`; story B: `Assignment`, `Outcome`, `check_assignment`, `who_did`, `duty_totals` (the only source of the numbers), `display_names`, `clashes`.
- **Core `start_page`**: `OpenItem::NotHandled { games }` (possible duty games, today to today + 27 days, not handled; story A) and `OpenItem::OpenPlaces { games }` (open jury and referee places in the same 4 weeks, open driver places until the Sunday of next week; story B), after the changes not seen and before the trainings; `next_events` also takes the possible duty games with places; `last_game` (the newest game before today with places).
- **Core module `messages`** (C, D): the Dutch texts with Dutch day and month names; exact-text tests.
- **Server.**
  - `duties.rs` (store): the places and assignments of a season; the row lock of one game (`SELECT … FOR UPDATE`, inserted first when missing) for every write: lock, read, core check, write, in one transaction.
  - `duties_of(state, season)` in `api/season.rs`, next to `games_of` and `trainings_of`: the stored places, assignments and roles a season needs; `GET /api/season` and `GET /api/start` call it, so each game event carries its `GamePlaces` as a view.
  - Routes (under `/api`, with `utoipa`; front-end types generated): `GET /api/games/{season}/{id}` and `PUT /api/games/{season}/{id}/places` (A); `PUT` / `DELETE /api/games/{season}/{id}/places/{kind}/{number}`, `PUT …/outcome`, `GET /api/duties?season=` (B); `GET /api/messages/next-game` (C); `GET /api/messages/committee?week=` (D).
  - `GET /api/start` gets the new open items with their games (date, start time, teams), `last_game` next to `last`, and places on the games of "Next".
- **Front end.** `/game/[season]/[id]` (A, B); `EventRow.svelte` shows the places of a game and a tap opens its page; `Labels.svelte` and `season.ts` get the group "Duties"; the return to "Season" (`keepForReturn`) also covers a game; `start.ts` and the start page get the new open items (a list below an item for more than one game), "Last" with a game, the part "Messages" (C, D); `/duties` (B); `/messages/next-game` (C) and `/messages/committee` (D); the links "Season · Attendance · Duties · Team · Players · Settings" (B).

| Option | For | Against |
|--------|-----|---------|
| A row per filled place with a place number (chosen) | The database stops two players on one place; no lock for a fill | A place number the coordinator never sees |
| A list of players per kind, checked under a lock | No place numbers | Every fill needs the lock and a count |
| A row with 0 and 0 means "handled" (chosen) | No extra column; one fact | The store must never delete such a row |
| A column `handled` | Explicit | Two facts that must agree |
| The places travel on the answers of "Season" and the start page (chosen) | One request per page; the same `GamePlaces` everywhere | Larger answers |
| The page asks the places of each game | Small answers | Many requests on a phone |
| The games of an open item are in the answer (chosen) | The item and its list are the same games (S11) | A few more lines |
| `last_game` next to `last` (chosen) | Clear for the page | Two fields |
| Totals worked out at each request (chosen) | Corrections, moved and gone games always right (0020, 0022) | A little work per page (about 20 players, 60 places a season) |
| Messages in the core (chosen) | The rules in one place, exact-text tests | Dutch words in the core |
| Store only a changed number of drivers (chosen) | "Proposed, follows the team until changed" | `NULL` has a meaning |

## Stories
| # | Story | Type | Scope | Depends on | Closure | Release | Status |
|---|-------|------|-------|------------|---------|---------|--------|
| A | [Duty places](../2026-10-18-duty-places/) | feature | Migration `game_duties`. Core `duties`: `duty_need` with all cases first (risk 1), `driver_proposal`, `game_places`, `handled`, `open_places`, `check_places`. Store with the row lock. `duties_of`. Routes: a game; save its places. The page of a game. The places on the rows of "Season" and of the start page; the label group "Duties"; "Back" from a game keeps the labels; a tap on a game opens its page. The open item `NotHandled` with its list; "Next" with duty games; `last_game`. "has duties" in the list of changes. Measure the start page. If its `/design` finds more than about 6 steps: split into A1 (the places and the page of a game) and A2 (the places on "Season" and the start page), after asking the coordinator. | – | review build + accept | 1, with B | design |
| B | Who does each duty place, and the totals | feature | Migration `duty_assignments`. Core `check_assignment`, `who_did`, `duty_totals`, `display_names`, `clashes`; the open item `OpenPlaces` with its two windows. Routes: fill, clear, correct; the choice with totals and clashes; the totals of a season. The page "Duties" and its link. "Delete player" with duties. Measure the start page again. Decision record (the rows, "handled", what counts). | A | review build + accept | 1, with A | not started |
| C | The message for the next game | feature | Core `coming_week` (tests around 25 October 2026, the clock change) and the message. Route, the message page with "Copy" (clipboard spike first; risk 4), the part "Messages" on the start page. | B | review build + accept | 2 | not started |
| D | The message for the referee committee | feature | Core `committee_week` and the message. Route with a week, the page with "Copy", "Next week", "Previous week"; the button in "Messages". | B | review build + accept | 2 or 3 | not started |

Each story has its own change folder (type `feature`, stage `design`) and its own `/design`, which
writes its steps, routes, screens and exact tests from this spec and `functional.md`. The folder of
story A exists (`2026-10-18-duty-places`); its earlier Phase 1 (2026-10-18) is replaced by this spec.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| A row per filled duty place with a place number; an open place has no row; the database keeps one player per place and one place per player per game | No lock for a fill; two screens never put two players on one place | Yes, at the close of story B (with the next rows) |
| A row of `game_duties` with 0 and 0 stays and means "handled"; an away game is always handled | "Handled" is one stored fact (coordinator, 2026-10-25) | Same record |
| After the game a filled place counts as done by its planned player, unless corrected to someone else or "not done" (the planned player kept) | Fast to keep, with a trace for fairness (S5) | Same record |
| What counts: done before today, planned from today (Dutch date); coming games that are gone or no longer need duties do not count; past games count, also when gone | The totals must be right and checkable (S2) | Same record |
| `game_places` and `duty_totals` are the only rules; "Season", the page of a game, the start page and "Duties" all use them | One number everywhere (decision 0022) | In the same record |
| The number of drivers stored only when the coordinator sets it; never shown below the filled driver places | "Proposed, follows the team until changed"; nobody removed silently (S1) | In the same record |
| Duties hang on the game key (club, season, source id); a new id from the source leaves them on the gone game | The game list is the truth (0013) | In the same record |
| Duties of an earlier season can be changed and corrected | As trainings (S13) | In the same record |
| The open items of duties: not handled and jury and referee places in 4 weeks, drivers until the Sunday of next week; each with its games in the answer | Coordinator, 2026-10-25 (S16); the item and its list the same (S11) | In the same record |
| The messages are made in the core, in Dutch | Rules in one place, exact-text tests | Yes, at the close of story C |
| Another team chosen mid-season keeps the old duties under the rule "no longer needs duties of the team" | Coordinator, 2026-10-18 | In the record of story B |

## Not in scope
- A departure time; attendance at games; the overview for the parents; proposing a player; message templates or other languages; duties at the team's own home games or of other kinds; totals over more than one season; the club's own division of places over its teams; a "wave" of places as a thing in the tool; a date "committee list entered until"; handling several games at once.

## Test plan
- **Existing tests that change or must still pass**: `tests/season.rs` and `tests/start.rs` (new fields: places on games, the new open items, `last_game`); core `start_page` tests (new cases); `tests/changes.rs` and `team_core::comparison` ("has duties"); `tests/players.rs` ("Delete player" with duties, new routes need a login); `tests/player_logs.rs` (new routes, also the messages); browser `start.e2e.ts`, `season.e2e.ts`, `choose.e2e.ts` (a tap on a game opens its page) and every test that checks the links (with "Duties").
- **New tests**: per story, made exact by its `/design`. In short: A: `duty_need` for every case (also an away game in the club's own hall), the proposal (0, 1, 3, 10 players), `game_places` in each state, `handled`, `open_places`, `check_places`, `NotHandled` with its 4 weeks, "Next" and `last_game`; the page of a game, a save (also 0 and 0), "Use proposal", "no longer needs duties", a gone and a moved game, the race test of the row lock, "has duties"; the walk through the committee's list on "Season" in the browser. B: `check_assignment`, `who_did`, `duty_totals` (gone coming, gone past, "Nobody", "Someone else", no longer needed, hidden, joining later), `display_names`, `clashes`, `OpenPlaces` with its two windows; fill, clear, correct, the race on one place, the totals equal in the choice and on "Duties", "Delete player"; the choice and "Duties" in the browser. C: exact texts of every case of the spec, the week around 25 October 2026, "Copy" in the browser. D: exact texts, "Next week" and "Previous week", "Copy".
- **Browser tests**: a new file `duties.e2e.ts` in its own Playwright project after `start` (it changes data that `start.e2e.ts` checks); clipboard permission for "Copy"; the clock stays at 12 October 2026 (July and August, a new season and the clock change in the core and on the server only).
- **Checks**: `just check`.

## Compatibility and performance
- New tables and routes; the answers of `GET /api/season` and `GET /api/start` get new fields (the front-end types are generated).
- A tap on a game opens its page (before: nothing). The start page links become "Season · Attendance · Duties · Team · Players · Settings".
- "Delete player" is also not offered for a player with duties.
- `GET /api/start` reads more: the places and assignments of one or two seasons, and the roles for the proposal; measured in A and B (limit about 200 ms locally, as for the earlier epic).
- Player names in the messages and on the pages come decrypted from `players.rs` (decision 0021); never in a log.

## Progress
- [ ] Story A – duty places
- [ ] Story B – who does each duty place, and the duty totals (release 1 with A)
- [ ] Story C – the message for the next game
- [ ] Story D – the message for the referee committee
- [ ] Epic review accept

## Deviations
None from `functional.md`. Readings of the spec that the design makes exact:
- "Handled" is a fact of a possible duty game only; an away game counts as handled (its proposal gives the number), so it never gives the open item "not handled".
- The window "4 weeks" is today to today + 27 days (Dutch date); the window of the drivers is the window of "Next".

## Review findings
