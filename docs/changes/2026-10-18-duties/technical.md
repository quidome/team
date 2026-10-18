# Technical spec: Duties

**Status**: approved
**Size**: epic
**Change**: [2026-10-18-duties](./)
**Date**: 2026-10-18

## Objective
Keep the team's duty places (drivers at its away games; jury and referee places at home games of
other club teams), the player on each place, the outcome after the game and the duty totals, and
make the two Dutch messages to copy (the next game; the referee committee), so the sheet is no
longer needed for duties.

## Requirements summary
From `functional.md` (approved 2026-10-18; spec review S1–S9 answered):

- **A – Duty places.** A page for each game (tap in the schedule). At a home game of another club team: referee places and jury places, each 0–2. At an away game of the team: drivers, proposed as one per 3 players with the role "plays games" on the date (rounded up), marked "proposed" and following the team until the coordinator sets a number (0–10), which then stays; "Use proposal" goes back. A proposal that drops below the filled driver places keeps them ("more than proposed"). A number below the filled places is refused ("2 places are filled. Clear one first."). A home game of the team has none. The schedule shows the mark "duties". The page "Duties": coming games with places (nearest first, "open" places, "all filled"), "Show past games". Start page link "Duties". Moved games keep places; the list of changes says "has duties" with the filled places. A coming game that is gone keeps its places, shows "no longer in the game list", does not count while gone. A past game counts, also when gone later. A game that no longer needs duties of the team (home and away swapped, the teams changed, another team chosen) keeps its places with "This game no longer needs duties of the team." and does not count (coming). An earlier season: read only ("Duties of an earlier season cannot be changed.").
- **B – Who does it, and the totals.** A tap on a place: the players with the role "plays games" on the date (hidden players too), by first name, each with that kind's totals in the current season ("2 done · 1 planned"); no proposal. A tap saves at once; "Clear". One place per player per game. Before today a filled place is done by its player; from today planned. On a past game: "Someone else" or "Nobody" (shows "not done (was Anna)"). The table "Totals" on "Duties": a row per player with the role "plays games" in the season (also left, also hidden), columns Drive, Jury, Referee, "done · planned"; "done" is the duty total. A player on a date without the role stays, with "Not in the team on this date". "Delete player" not offered with duties. Two screens: last save wins, never two players on a place. "Not saved. Try again."
- **C – The message for the next game** (start page button, shown when there is a coming game of the team or duties in the coming week): Dutch text with "Copy" ("Copied"; on failure "Select the text and copy it."). The next game of the team that is in the game list: date, time ("tijd nog niet bekend"), "Thuis"/"Uit tegen", hall and town; for an away game "Rijden:" first names, "(nog N open)"; "Jury en scheidsrechter deze week:" with the team's jury and referee duties from today until the next Sunday (today when Sunday), whether or not the team plays. Without a coming game: only the duty part. Same first names: first letter of the last name.
- **D – The message for the referee committee** (on "Duties"): Monday to Sunday of the week of the coming weekend (on Saturday or Sunday: this week), "Next week", "Previous week"; each game with the team's jury or referee places, in time order, with full names (first name only when no last name), "nog open"; left out: games gone or no longer needing duties; empty week: "Geen jury- of scheidsrechtersdiensten deze week."

Coordinator's answers in Design (2026-10-18): another team chosen mid-season keeps the old team's
duties; the rule "no longer needs duties of the team" applies (coming ones do not count; past
ones count; choosing the old team again brings them back).

## Current state
- Games: table `games`, key `(club_number, season_year, source_id)` (migration `0003`), never deleted; `in_list` false when gone. Core `Game` (`source_id`, `game_number`, `season`, `date`, `start_time: Option`, `home`/`away: Side { team, team_name, club }`, `place: Place { hall, town, … }`). Store `crates/server/src/games.rs`.
- The schedule rule `team_core::in_schedule(game, club, team) -> Option<ScheduleReason>` (`TeamGame` when the team plays home or away; `DutyGame` when the home club is the chosen club and the team does not play; `None` otherwise); `schedule`, `schedule_target`. `GET /api/schedule` (`api/schedule.rs`), start page `web/src/routes/+page.svelte` with `Schedule.svelte`.
- The team choice for each season: `followed_teams` (decision 0012); `current_season`, `today_in_netherlands`, `in_netherlands` in `clock.rs`.
- The list of changes: `team_core::comparison` (`change_between`, `update_open_changes`, `list_changes`), `changes.rs`, `api/changes.rs`, `/changes`. The source may give a game a new id: `same_number_pairs` logs it.
- Players and roles: `team_core::{Player, PlayerRole, Role, role_on, Player::sort_key}`; store `players.rs` with the row lock `lock` and the encrypted names (decision 0021); "Delete player" is not offered with absences (`trainings::player_has_absences`).
- Patterns: rules in the core; totals worked out at each request (decision 0020); a lock or a unique key for each save that two screens can make; logs carry ids only; Dutch dates.

## Architecture (shared by all stories)
- **Tables.**
  - `game_duties` (story A): `club_number`, `season_year`, `source_id` (primary key, foreign key to `games`), `referee` and `jury` (`smallint`, `CHECK` 0–2), `drivers` (`smallint` 0–10, `NULL` = the proposal). A row exists while a game has places or a set number of drivers; setting everything to 0 (and drivers back to the proposal) deletes it.
  - `duty_assignments` (story B): the game key (foreign key to `game_duties`), `kind` (`drive`, `jury`, `referee`), `place` (`smallint` ≥ 1), `planned_player` (foreign key to `players`, `ON DELETE RESTRICT`), `outcome` (`as_planned`, `done_by`, `not_done`), `done_by` (player, only with `done_by`). Primary key `(game key, kind, place)`: one place never holds two players. Unique `(game key, planned_player)`: one place per player per game. An open place has no row.
- **Core module `duties`** (no I/O, decision 0001): `DutyKind`, `GameKey`, `DutyNeed` and `duty_need` (from `in_schedule`: `DutyGame` → jury and referee; `TeamGame` with `away.team == team` → drive; else none), `GameDuties`, `Assignment`, `Outcome`, `driver_proposal`, `places`, `check_places`, `check_assignment`, `who_did`, `duty_totals` (the only source of the numbers), `display_names`, and the weeks `coming_week`, `committee_week`.
- **Core module `messages`**: the Dutch texts of story C and D, with Dutch day and month names; exact-text unit tests.
- **Server.** A store for each table (`duties.rs`); every write to a game's duties takes the row lock of its `game_duties` row (`SELECT … FOR UPDATE`; inserted first when missing) in one transaction: lock, read, core check, write. Routes under `/api` (session, `Origin`), described with `utoipa`; front-end types generated.
- **Front end.** `/game/[season]/[id]` (story A, B), `/duties` (A: games; B: totals; D: the committee message), the start page (link; C: the message button), a tap on a game in the schedule and in the list of changes.
- **Changed existing parts.** The schedule answer gets the mark "duties" (A); a change in the list of changes of a game with duties says "has duties" (A); "Delete player" also checks duties (B); the player log test covers the new routes (each story).

| Option | For | Against |
|--------|-----|---------|
| A row per filled place with a place number (chosen) | The database stops two players on one place; no lock for a fill | A place number the coordinator never sees |
| A list of players per kind, checked against the number under a lock | No place numbers | Every fill needs the lock and a count |
| Totals worked out at each request (chosen) | Corrections, moved and gone games always right; same as attendance (0020) | A little work per page (about 20 players, 60 places a season) |
| Stored counters | Cheap to read | Every correction and game change must update them |
| Messages in the core (chosen) | The rules (which duties, which week, which names) in one place, exact-text tests, reused by later messages | Dutch words in the core |
| Messages in the front end | Fast to change | Rules split over two places; harder to test |
| Store only a changed number of drivers (chosen) | "Proposed, follows the team until changed" | `NULL` has a meaning |

## Stories
| # | Story | Type | Scope | Depends on | Closure | Release | Status |
|---|-------|------|-------|------------|---------|---------|--------|
| A | [Duty places](../2026-10-18-duty-places/) | feature | Migration `game_duties`. Core `duty_need` (all cases first, risk 1), `driver_proposal`, `places`, `check_places`. Store with the row lock. Routes: a game with its places; save places and drivers. Pages `/game/[season]/[id]` and `/duties` (games, no totals); the mark "duties" in the schedule; a tap on a game in the schedule and in the list of changes; the link "Duties" on the start page; "has duties" in the list of changes. | – | review build + accept | with B | not started |
| B | Who does each duty place, and the duty totals | feature | Migration `duty_assignments`. Core `check_assignment`, `who_did`, `duty_totals`, `display_names`. Routes: fill, clear, correct; the choice with totals; the totals table. "Delete player" with duties. Decision record (the outcome and what counts). | A | review build + accept | release 1 (A + B) | not started |
| C | The message for the next game | feature | Core `coming_week` (tests around 25 October 2026, the clock change; risk 3) and the message. Route, page with "Copy" (clipboard spike in the browser tests; risk 4), button on the start page. | B | review build + accept | release 2 | not started |
| D | The message for the referee committee | feature | Core `committee_week` and the message. Route with a week offset, page with "Copy", "Next week", "Previous week" on "Duties". | B | review build + accept | release 3, or with C | not started |

Each story gets its own change folder (type `feature`, stage `design`) and its own `/design`,
which writes its steps, routes, screens and exact tests from this spec and `functional.md`.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| A row per filled duty place with a place number; an open place has no row; the database keeps one player per place and one place per player per game | No lock for a fill; two screens never put two players on one place | Yes, at the close of story B (with the next row) |
| After the game a filled place counts as done by its planned player, unless corrected to someone else or "not done" (the planned player kept) | Fast to keep (as "there" at training), with a trace for fairness (coordinator, 2026-10-18; S5) | Yes, same record |
| What counts: done before today, planned from today (Dutch date); coming games that are gone or no longer need duties do not count; past games count, also when gone | The totals must be right and checkable; a past game was played (S2) | Yes, same record |
| Totals worked out at each request, one core function for every number | As decision 0020 | In the same record |
| The number of drivers stored only when the coordinator sets it; never shown below the filled driver places | "Proposed, follows the team until changed" (coordinator); nobody removed silently (S1) | In the same record |
| Duties hang on the game key (club, season, source id); a new id from the source leaves them on the gone game | The game list is the truth; `same_number_pairs` logs it | In the same record |
| The messages are made in the core, in Dutch | Rules in one place, exact-text tests | Yes, at the close of story C |
| Another team chosen mid-season keeps the old duties under the rule "no longer needs duties of the team" | Coordinator, 2026-10-18 | In the record of story B |

## Not in scope
- A departure time; attendance at games; the overview for the parents; proposing a player; message templates or other languages; duties at the team's own home games or of other kinds; totals over more than one season; the club's own division of places over its teams; a "wave" of places as a thing in the tool (places are entered game by game).

## Test plan
- **Existing tests that change or must still pass**: `tests/schedule.rs` and the schedule core (the mark "duties"), `tests/changes.rs` and `team_core::comparison` (a moved game with duties says "has duties"; existing kinds unchanged), `tests/players.rs` and the browser test of "Delete player" (one case with duties), the browser tests that click the start page links (new link order), `tests/player_logs.rs` (new routes, also the messages).
- **New tests**: per story, as in the stories table; each story's `/design` makes them exact. In short: A: `duty_need` for every case, the proposal, `places`, `check_places`, the lock test, a moved and a gone game; B: `check_assignment` errors, `who_did`, `duty_totals` (gone coming, gone past, "Nobody", "Someone else", no longer needed, hidden), `display_names`, a race on one place, the totals equal in the choice and the table; C: exact texts of every case in `functional.md`, weeks around 25 October 2026, "Copy" in the browser; D: exact texts, "Next week" and "Previous week", "Copy".
- **Browser tests**: a new file `duties.e2e.ts` in its own Playwright project after `trainings`, on the shared database `team_e2e` and the clock `DEV_NOW` (12 October 2026); clipboard permission for "Copy".
- **Checks**: `just check`.

## Compatibility and performance
- New tables and routes; the schedule and change answers get one more field each (the front-end types are generated).
- "Delete player" is also not offered for a player with duties.
- The start page links become "Attendance · Duties · Team · Players · Settings".
- Each page with totals reads the season's games with duties, the assignments, the roles and the players: about 60 places and 20 players a season; no concern.
- Player names in the messages come decrypted from `players.rs` (decision 0021); never in a log.

## Progress
- [ ] Story A – duty places
- [ ] Story B – who does each duty place, and the duty totals (release 1 with A)
- [ ] Story C – the message for the next game
- [ ] Story D – the message for the referee committee
- [ ] Epic review accept

## Deviations

## Review findings
