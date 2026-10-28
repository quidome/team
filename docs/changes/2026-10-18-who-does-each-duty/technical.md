# Technical spec: Who does each duty place, and the duty totals

**Status**: in progress
**Size**: story
**Change**: [2026-10-18-who-does-each-duty](./)
**Date**: 2026-10-27

## Objective
The coordinator puts a player on each duty place of the team (on the page of a game), corrects it
after the game ("Someone else", "Nobody"), and sees the duty totals per player and kind (in the
choice and on a page "Duties"); the start page shows the games with open places, and the list of
changes keeps changes to past games with duties, so the work is shared fairly and the totals can be
trusted.

## Requirements summary
Story B of the epic [Duties](../2026-10-18-duties/) (`functional.md` approved 2026-10-25, second
change, and changed on 2026-10-27 at this design; epic `technical.md` approved 2026-10-25, second
change). Story A ([Duty places](../done/2026-10-18-duty-places/)) is built and closed on the branch
`duty-places`; both go to `main` together as release 1.

- **Places of the team**: the table and referee places that name the team, plus the driver places of an away game of the team. Only these get players.
- **The choice of a player**: a tap on a place of the team on the page of a game. Offered: the players with the role "plays games" on the game's date, hidden players too, sorted by first name. Next to each name the totals of that kind in **the game's season** ("2 done · 1 planned"; for a game of this season that is the current season, as the spec says). No player is marked as proposed. A tap saves at once; "Clear" makes the place open.
- **Not offered / blocked**: a player who already has a place at this game (also as "Someone else"). A **clash** is a duty place at another game ("also Table 13:00") or another game of the team ("plays 14:30"); the game itself is never a clash (S22). Rule (coordinator, 2026-10-27):

  | The two games are | start times < 1:30 apart | 1:30 to < 2:00 apart | ≥ 2:00 |
  |---|---|---|---|
  | same hall, same court | nothing | nothing | nothing |
  | different courts (same hall or another hall) | **blocked**: shown with the reason, cannot be chosen | **warning**: shown, can be chosen | nothing |

  The same court: the same place number (hall) and the same court, compared without case and outer spaces ("Veld 1" = "veld 1"); a game without a court is another court. A game without a known start time never blocks; it gives a warning for every game on another court on the same date. A player already on a place stays when a later change makes a clash (the place shows it as a warning). "Someone else" on a past game is never blocked (it shows the warning).
- **Done and planned**: a filled place of a game before today (Dutch date) counts as done by its player; from today as planned; an open place counts for nobody. On a past game: "Someone else" (the same choice) or "Nobody" ("not done (was Anna)"; counts for nobody). A correction replaces what counts. Another player on a corrected place resets it to as planned.
- **What counts** (A1, coordinator 2026-10-27): what the coordinator stored is never changed by a reading of the game list. A coming game counts its places while it is in the game list and still needs them. A **past game counts what is stored**, also after a later swap, a removal from the game list, or another team chosen. A game with players on driver places keeps those places, also when it no longer needs drivers.
- **A new open item**: "Away game Sat 31 Oct: 2 drivers open", "Home game Sun 1 Nov: 1 table place open", "3 games with open places": home games of the club from today to today + 27 days with open table or referee places of the team, and away games of the team from today to the Sunday of next week with open driver places, that count. A tap opens the game; for more than one, the list of those games below the item. After "schedule not entered", before the trainings.
- **The page "Duties"** (`/duties`): a table "Totals", one row per player with the role "plays games" in the season or with a duty in it, sorted by first name; columns Drive, Table, Referee, each "done · planned" ("3 · 1"); players who left and hidden players too; a season choice as on "Attendance"; "No duties yet this season.". The start page links become "Season · Attendance · Duties · Team · Players · Settings".
- **Names**: two players with the same first name show the first letter of the last name ("Anna B.", "Anna V.") in the choice, on "Season", on the start page (and later in the message for the next game); a player without a last name: the first name only. The set: the players with a role in that season, plus the players on its places.
- **Edge cases**: a player on a place without the role "plays games" on that date stays, with "Not in the team on this date", and still counts; the coordinator can clear or change it. "Delete player" is not offered for a player with duties. A driver proposal that rises opens new places (an open item without a save). Two screens: the last save wins; a place never holds two players. A failed save: "Not saved. Try again." and the stored state.
- **The list of changes** (facts of 2026-10-27): a change to a past game where something is stored for the team (a place that names the team, a number of drivers set, or a player on a place of the team) is kept with "has duties" until "Seen"; it does not drop off because its date has passed. Without duties of the team, a change to a past game is not kept, as now.
- **From the reviews of story A**: R3 (a past game with players stays on "Season" and in "Last"), R4 (a race test that fails without the row lock), A1 (above).

## Current state
On the branch `duty-places` (story A, commit `e6147aa`; `main` has no duties code):

- **Tables**: `games` (key club, season, source id; `in_list`; `place_number`, `court`), `game_duties` (`schedule_entered_at`, `drivers`), `duty_teams` (one row per place that names a team). Migration `0011_game_duties` is the last.
- **Core `duties`** (`crates/core/src/duties.rs`): `PlaceKind`, `DutyKind`, `TeamRef`, `RequiredDuties`, `required_duties`, `StoredDuties`, `driver_proposal`, `players_playing_on`, `club_teams`, `kept_games` (a place names a team, or drivers above 0), `SchedulePlace`, `DutyPlace { kind, number, filled }`, `SchedulePart`, `DriverPart`, `GamePlaces`, `game_places(required, stored, team, filled, players_on_date, in_list, past)` with `counts = past || (in_list && required)` (already A1 as answered), `not_entered`, `open_places`, `has_team_places`, `DutyError`, `pick_team`, `check_team(…, place_filled)`, `check_entered`, `check_drivers(required, new, filled)`. The driver part exists only while drivers are required or a number above 0 is set (line 240).
- **Core `start_page`**: `OpenItem { ReadingHeldBack, OutOfDate, ChangesNotSeen, ScheduleNotEntered { games }, NotOpened, NotChecked }`, `schedule_not_entered`, `open_items`, `last_games`, `next_events`. **Core `comparison`**: `compare`, `change_between` (a past game gives no change, line 102), `update_open_changes` (drops a change whose game is past on both dates), `list_changes`.
- **Server**: `duties.rs` (`for_season`, `lock`: `INSERT … ON CONFLICT DO NOTHING` then `SELECT … FOR UPDATE`, `set_team`, `set_entered`, `set_drivers`); `api/games.rs` (`find`, `page`, `write` = lock, read, check, write in one transaction; `GET /api/games/{season}/{id}`, `PUT …/places/{kind}/{number}`, `PUT`/`DELETE …/schedule-entered`, `PUT …/drivers`); `api/season.rs` `SeasonDuties { stored, roles }` with `places_of` (passes an empty `filled` set), `duties_of`, `places_of_season`; `api/start.rs`; `api/changes.rs` (`ChangeView.places`); `changes.rs` `apply`, called from `reading.rs:185` in the reading transaction; `api/players.rs` (`has_absences`, "Delete player" refused with `player_has_absences`); `players.rs` (decrypted names, decision 0021).
- **Front end**: `/game/[season]/[id]`, `EventRow.svelte`, `start.ts`, the start page, `/attendance` (the season choice to copy), `duties.ts`.
- **Tests**: core `duties` 24, `comparison` 38, `start_page` 20; server `tests/games.rs` (with `a_save_waits_for_a_save_of_the_same_game_on_another_screen`, which cannot fail without the lock: R4), `tests/changes.rs`, `tests/start.rs`, `tests/season.rs`, `tests/readings.rs`, `tests/nightly.rs`, `tests/players.rs`; browser `web/e2e/duties.e2e.ts` (own Playwright project after `start`; clock 12 October 2026).
- **Real data** (`crates/server/tests/data/game-list/`): all home games of the club are in hall Bulwijk, on "Veld 1" or "Veld 2", 1 hour 30 minutes apart (3 October 2026: 13:00 Veld 1, 14:30 Veld 2, 16:00 Veld 1, …); one home game in two seasons has no court.

## Steps
Work on the branch `duty-places`. Each step ends with the tests of the part it changed
(`cargo test -p team_core`, `cargo test -p team --test <file>`, `cd web && npx playwright test duties`),
then `cargo clippy --workspace --all-targets -- -D warnings`. `just check` once, before the commit.

### Step 1: The table and the rules of one place
- **Goal**: players can be stored on places of the team, and every check of one place is a core rule.
- **Changes**:
  - Migration `0012_duty_assignments.sql` (new file; 0011 stays as applied):
    ```sql
    CREATE TABLE duty_assignments (
        club_number    bigint   NOT NULL,
        season_year    integer  NOT NULL,
        source_id      bigint   NOT NULL,
        kind           text     NOT NULL CHECK (kind IN ('drive', 'table', 'referee')),
        number         smallint NOT NULL CHECK (number BETWEEN 1 AND 10),
        planned_player bigint   NOT NULL REFERENCES players ON DELETE RESTRICT,
        outcome        text     NOT NULL DEFAULT 'as_planned'
                                CHECK (outcome IN ('as_planned', 'done_by', 'not_done')),
        done_by        bigint   REFERENCES players ON DELETE RESTRICT,
        CHECK ((outcome = 'done_by') = (done_by IS NOT NULL)),
        PRIMARY KEY (club_number, season_year, source_id, kind, number),
        UNIQUE (club_number, season_year, source_id, planned_player),
        FOREIGN KEY (club_number, season_year, source_id) REFERENCES game_duties
    );
    CREATE INDEX duty_assignments_planned ON duty_assignments (planned_player);
    CREATE INDEX duty_assignments_done_by ON duty_assignments (done_by);
    ```
  - Core `duties`: `Outcome { AsPlanned, DoneBy(PlayerId), NotDone }`; `Assignment { kind, number, planned, outcome }`; `Counted { Done(PlayerId), Planned(PlayerId), Nobody }`; `who_did(&Assignment, past) -> Counted`; `check_assignment(places: &GamePlaces, kind, number, player, at_game: &[Assignment], plays_on_date: bool, clash: Option<ClashLevel>) -> Result<(), DutyError>` (the clash comes from step 2; until then `None`); `check_outcome(past, assignment: Option<&Assignment>, new: &Outcome, at_game, plays_on_date) -> Result<(), DutyError>` (never blocks on a clash). New `DutyError` variants: `NotAPlaceOfTheTeam` (the place does not name the team, or the driver place is beyond the number), `PlayerAtGame`, `NotInTeam`, `Clash`, `NotPast`, `NoPlayerOnPlace`.
  - `game_places`: the driver part also exists when a driver place has a player; `number` is at least the highest filled driver place (already so through `full`). `kept_games` takes the assignments: a game with a player on a place is kept (R3).
  - `check_drivers` gets the highest filled driver place (`full`), not the count.
  - A1 needs no rule change: a test of a past swapped game.
- **Tests** (core): `who_did` as planned / done by / not done / today = planned; each refusal of `check_assignment` (another team's place, driver place 4 of 3, the player already at the game, no role on the date) and a hidden player allowed; `check_outcome` on a game of today refused, "Someone else" already at the game refused, a correction resets on another player; `check_team` with a player on the place: `PlayerOnPlace`; `check_drivers` with place 3 filled and 1, 2 open: below 3 refused; `game_places` keeps the driver places with players when drivers are no longer required; `kept_games` keeps a past game with only a player on a proposed driver place; a past swapped game: its places of the team count (A1). Migration test (`tests/migrations.rs`): the table exists, two players on one place and one player twice at one game are refused by the keys.

### Step 2: Totals, names and clashes in the core
- **Goal**: the numbers, the names and the clash notes have one rule each, tested without a database.
- **Changes**:
  - `GameDuties<'a> { game: &'a Game, places: &'a GamePlaces, assignments: &'a [Assignment], past: bool }`; `KindTotals { drive: Total, table: Total, referee: Total }`, `Total { done: u32, planned: u32 }`; `duty_totals(&[GameDuties]) -> BTreeMap<PlayerId, KindTotals>`: only places of the team in a part that `counts`; done by `who_did`.
  - `display_names(&[Player]) -> BTreeMap<PlayerId, String>`: the first name, plus " " + the first letter of the last name + "." when another player of the set has the same first name (with `same_name`); without a last name the first name only.
  - `ClashLevel { Blocked, Warning }`; `Clash { level, what: ClashWhat { Duty { kind, start }, Plays { start } } }`; `clashes(player, game: &Game, season: &[GameDuties], team_games: &[&Game]) -> Vec<Clash>` with the table of the requirements: same `place_number` and the same court (trimmed, without case; `None` or empty = another court) → nothing; otherwise the distance between the start times: < 90 minutes `Blocked`, < 120 minutes `Warning`; a start time not known → `Warning` on the same date (not on the same court); the game itself is skipped.
- **Tests** (core): totals per kind; a coming game out of the list: 0; a coming swapped game: 0; a past swapped game: counts (A1); a past game out of the list: counts; past drivers after another team was chosen: count; an open place: nobody; "Nobody": nobody. Names: "Anna B." / "Anna V.", without a last name. Clashes: same hall same court 13:00/13:00: nothing; same hall Veld 1 13:00 / Veld 2 14:30: warning (the real day 3 October 2026); Veld 1 13:00 / Veld 2 14:00: blocked; "Veld 1" / "veld 1": same court; no court: another court; other hall 13:00/14:00: blocked; 13:00/14:30: warning; 13:00/15:00: nothing; time not known, other court, same date: warning, never blocked; the game itself: nothing.

### Step 3: Store and routes
- **Goal**: the coordinator can put, clear and correct a player through the API, safely from two screens.
- **Changes**:
  - `duties.rs`: `assignments_for_season(db, club, season) -> BTreeMap<i64, Vec<Assignment>>`; `set_assignment`, `clear_assignment`, `set_outcome`, each inside `write` (lock the game's row, read the stored duties and the assignments of the game, core check, write, one transaction); `has_duties(db, player)`.
  - `SeasonDuties` gets `assignments`; `places_of` passes the filled places from them (no longer an empty set); `kept_games` with the assignments.
  - `set_place` passes `place_filled` from the assignments to `check_team`; `set_drivers` passes the highest filled driver place.
  - Routes (with `utoipa`; `just api-types`):

    | Route | Body | Refusals (400) |
    |---|---|---|
    | `PUT /api/games/{season}/{id}/assignments/{kind}/{number}` | `{ "player": 12 }` | `not_a_place_of_the_team`, `player_at_game`, `not_in_team`, `clash`, `unknown_player` |
    | `DELETE` the same | – | – |
    | `PUT /api/games/{season}/{id}/assignments/{kind}/{number}/outcome` | `{ "outcome": "as_planned" \| "done_by" \| "not_done", "player": 12 }` | `not_past`, `no_player_on_place`, `player_at_game`, `not_in_team`, `unknown_player` |

    Each answers the page of the game as stored now (as story A). `kind` is `drive`, `table` or `referee`.
  - `PlaceView` gets `player: Option<PlayerOnPlace { id, name, not_in_team, outcome, done_by }>`; the rows of "Season", "Next", "Last" and the list of changes show the names (`display_names` of the season).
  - `api/players.rs`: `has_duties` on the player; "Delete player" refused with `player_has_duties`.
  - Logs: the season, the game id and the player id only, never a name.
- **Tests** (server, `tests/games.rs`): put, clear, correct, each answer the stored page; each refusal with its code writes nothing; a place with a player cannot get another team (`player_on_place`); **race test (R4)**: one transaction puts Anna on table 1 and holds; a second request gives table 1 to another team; with the lock the second waits and is refused with `player_on_place`; checked once by removing `FOR UPDATE` that the test fails. `tests/players.rs`: "Delete player" refused with a duty; `tests/season.rs`: a past game with only a player on a proposed driver place stays (R3).

### Step 4: The choice on the page of a game
- **Goal**: the coordinator fills and corrects places on the page of a game.
- **Changes**:
  - `GamePage` gets `players: Vec<Choice { id, name, hidden, totals: KindTotals, clashes: Vec<ClashView>, at_game: bool, plays_on_date: bool }>`: the players with "plays games" on the date and the players already on places of the game; totals from `duty_totals` of the game's season.
  - Front end `/game/[season]/[id]`: a tap on a place of the team opens the choice (sorted by first name; "2 done · 1 planned" of that kind; a warning as a note; a blocked player and a player already at the game shown without a tap, with the reason); a tap saves at once; "Clear". On a past game: "Someone else" (the same choice, never blocked), "Nobody", and the place shows "not done (was Anna)" or "Bram (was Anna)". "Not in the team on this date" next to such a player. "Not saved. Try again." and the stored state after a failed save.
- **Tests** (browser, `duties.e2e.ts`): a tap on a place and a player: the name shows and the totals in the choice change; "Clear"; "Someone else" and "Nobody" on a past game; a blocked player cannot be chosen; a failed save; a phone width without sideways scroll; the accessibility check.

### Step 5: The open item "games with open places"
- **Goal**: the start page shows the games that still need players.
- **Changes**: core `start_page`: `OpenItem::OpenPlaces { games: Vec<i64> }`, `open_places_games(…, today)`: home games of the club from today to today + 27 days with `open_places` of the schedule part > 0, and away games of the team from today to the Sunday of next week with open driver places, both where the part counts; after `ScheduleNotEntered`, before the trainings. `GET /api/start`: the item with its games (season, id, date, start time, home, away) and the kinds open per game for the text of one game. Front end `start.ts` and the start page: "Away game Sat 31 Oct: 2 drivers open", "Home game Sun 1 Nov: 1 table place open", "3 games with open places" with the list below.
- **Tests**: core: a table place on day 27 is in it, on day 28 not; a driver place on the Sunday of next week is in it, the Monday after not; a coming game out of the list: not; a rising proposal makes a game an item without a save. Server `tests/start.rs`: the item and its games. Browser: the item and its list.

### Step 6: The page "Duties"
- **Goal**: the totals of a season in one table.
- **Changes**: `GET /api/duties?season=` → `{ season, seasons, rows: [{ player, name, hidden, drive, table, referee }] }` from `duty_totals` (`invalid_season`); the rows: players with "plays games" in the season or with a duty in it, by first name. Front end `/duties` with the season choice as on "Attendance", "done · planned", "No duties yet this season."; the start page link "Duties".
- **Tests**: server `tests/duties.rs` (new): the same numbers as the totals in the choice of a game of that season; a player who left with duties has a row; an empty season. Browser: the page, the season choice, the empty text; `start.e2e.ts` and `players.e2e.ts` for the links.

### Step 7: Changes to past games with duties
- **Goal**: the coordinator sees when the game list changes a past game that has duties of the team.
- **Changes**: core `comparison`: `change_between`, `compare`, `update_open_changes` and `list_changes` take `with_duties: &BTreeSet<i64>` (the games with duties of the team: a place that names the team, drivers above 0, or a player on a place); a past game in that set gives a change as a coming game does and is not dropped because its date passed. Server: `duties::with_duties(db, club, season, team)`, read in the reading transaction before `changes::apply` (`reading.rs:185`) and in `api/changes.rs`. The change says "has duties" with the places of the team (as now).
- **Tests**: core: a moved past game with a player: a change with duties; the same without duties: nothing (the existing `a_change_to_a_past_game_gives_nothing` keeps its meaning); a past change with duties is not dropped by `update_open_changes`. Server `tests/changes.rs`: a reading that moves a past game with a player on a place gives the change in `GET /api/changes`. `tests/readings.rs` and `tests/nightly.rs` must still pass.

### Step 8: Measure and finish
- **Goal**: release 1 is fast enough and checked.
- **Changes**: measure `GET /api/start` with one season of duties and players (limit about 200 ms locally); the progress below; `just check`.
- **Tests**: `just check` (all recipes, as CI).

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| A new migration file `0012`, not a change of `0011` | 0011 is applied on the local databases; sqlx stops on a changed file | No |
| One row per filled place of the team; keys: one player per place, one place per player per game; players `ON DELETE RESTRICT` | The database refuses what two screens could do | Yes, at the close (the epic's record) |
| Every write of a player under the row lock of the game; a table or referee place gets a player only while it names the team | The check reads the team on the place; the lock makes it hold until the write | Same record |
| A reading of the game list never changes what the coordinator stored; a past game counts what is stored | The record of what happened is the coordinator's (coordinator, A1, 2026-10-27) | Same record |
| Totals worked out at each request by one rule (`duty_totals`) | Corrections, moves and gone games always right (0020, 0022) | Same record |
| The choice comes with the page of the game | One request; a tap on a phone does not wait | No |
| Clashes: same court never; other court < 1:30 blocks, < 2:00 warns; a player already on a place and "Someone else" never blocked | One court runs games one after the other; games often run over 1:30 (coordinator, 2026-10-27) | Same record |
| The totals in the choice are of the game's season | For a game of an earlier season the current season says nothing; the same for this season (coordinator, 2026-10-27) | No |
| The comparison of games gets the games with duties of the team from the server | The core has no database (0001); a past game with duties needs a check | Same record |

## Not in scope
- The messages (stories C and D), proposing a player, totals over more than one season, attendance at games, a travel time between halls beyond the 2 hours of the warning.

## Test plan
- **Existing tests that change or must still pass**: core `duties`, `comparison` (`a_change_to_a_past_game_gives_nothing`, `a_change_whose_game_is_now_past_on_both_dates_is_removed` for games without duties), `start_page`; server `tests/games.rs`, `tests/season.rs`, `tests/start.rs`, `tests/changes.rs`, `tests/readings.rs`, `tests/nightly.rs`, `tests/players.rs`, `tests/player_logs.rs` (new routes need a login); browser `duties.e2e.ts`, `start.e2e.ts`, `players.e2e.ts`, `changes.e2e.ts`.
- **New tests**: per step, above. The race test of step 3 must fail without the lock (checked once by hand, R4).

## Compatibility and performance
- New table and routes; the answers of `GET /api/games/…`, `GET /api/season`, `GET /api/start`, `GET /api/changes` and the player get new fields (front-end types generated).
- "Delete player" is also not offered for a player with duties.
- The reading of the game list reads the duties of the season once more.
- `GET /api/start` reads the assignments too; measured in step 8 (about 200 ms locally).
- Player names come decrypted from `players.rs` (decision 0021) and never go into a log.

## Progress
- [x] Step 1 – the table and the rules of one place (2026-10-27: core 242 tests, 9 new; migration test of the keys; `tests/games.rs` 19 and `tests/season.rs` 8 pass; clippy clean)
- [x] Step 2 – totals, names and clashes in the core (2026-10-27: core 251 tests, 9 new; clippy clean)
- [x] Step 3 – store and routes (2026-10-27: `tests/games.rs` 28, 9 new; the race test fails without `FOR UPDATE`, checked by hand; all server tests pass; `just prepare`, `just api-types`; clippy and svelte-check clean)
- [x] Step 4 – the choice on the page of a game (2026-10-28: `tests/games.rs` 29, 1 new; `duties.e2e.ts` 4 new, all 111 browser tests pass; `just api-types`; clippy, svelte-check and lint clean)
- [x] Step 5 – the open item "games with open places" (2026-10-28: core 254 tests, 3 new; `tests/games.rs` 30, 1 new; `duties.e2e.ts` 1 new, all 112 browser tests pass; `just api-types`; clippy, svelte-check and lint clean)
- [x] Step 6 – the page "Duties" (2026-10-28: `tests/duties.rs` new, 4 tests; the route in the login and log tests; `duties.e2e.ts` 1 new, links in `start`, `season` and `players` e2e; all 115 browser tests pass; `just api-types`; clippy, svelte-check and lint clean)
- [x] Step 7 – changes to past games with duties (2026-10-28: core 256 tests, 2 new; `tests/changes.rs` 25, 2 new; all server tests pass, also `readings` and `nightly`; `just prepare`; all 115 browser tests pass; clippy clean)
- [x] Step 8 – measure and finish (2026-10-28: one season with 15 players, 93 places of the team and 123 players on places, debug build: `GET /api/start` median 27–31 ms, at most 75 ms; release build median 11 ms; the page of a game median 35 ms and `GET /api/duties` median 28 ms; limit about 200 ms)
- [x] All checks pass (2026-10-28: `lint-ci`, `check-rust`, `check-web`, `e2e` 115 tests, `check-sqlx`; `api-types-check` passes after the commit of the generated types, as before)

## Deviations
None from `functional.md` (changed with the coordinator on 2026-10-27 for A1, the changes to past games and the clashes). Readings of the spec that the design makes exact:
- "Overlaps" for a clash: the table of the requirements (start times; same court; 1:30 and 2:00).
- "The totals ... in the current season" in the choice: the game's season (the same for a game of this season).
- "Two players with the same first name": within the players with a role in that season plus the players on its places.
- A player on a place without the role "plays games" on that date still counts.

Found during the build:
- 2026-10-27, step 1: a number of drivers is checked against the **highest** filled driver place, not the count (the design). The refusal `places_filled` carries `filled` (the count) and `highest`. When they differ (driver place 3 has a player, 1 and 2 are open), the page says "Driver place 3 has a player. Clear it first." instead of "1 place is filled. Clear one first.", which would be false. Otherwise the text of the spec. (coordinator)
- 2026-10-27, step 3: the clash and the role of the player are read before the lock of the game, from the stored duties of the season. The lock covers this game only; a duty saved at the same moment at another game is not seen by the check. One coordinator, so accepted; the database keys still refuse two players on one place and one player twice at a game.
- 2026-10-27, step 3: the names on the places come with each place (`player.name`), worked out once per season from the players with a role in it or on its places (`display_names`); the step-4 choice reuses them.
- 2026-10-28, step 4: the players are in their own block "Players on the places" below the pull-down lists of the teams, one line per place of the team ("Table 2 · Yara"). On a past game a place with a player first offers "Someone else", "Nobody" and "Clear"; the choice opens with "Someone else". In "Someone else" the planned player can be chosen again: that undoes the correction (as planned). A player that cannot be chosen shows why: "on this place", "already at this game", "Not in the team on this date", "at another game at almost the same time". A clash without a known start time shows "plays (time not known)" or "also Table (time not known)"; a driver place at another game shows "also Driver 13:00".
- 2026-10-28, step 5: the item carries the open places of each kind per game (`OpenGame { id, open }`, not only the ids), so the text of one game comes from the same rule (`open_by_kind`, which `open_places` now sums). Each kind keeps its own window: table and referee places to today + 27 days, drivers to the Sunday of next week. A home game with both kinds open says "1 referee place, 2 table places open"; "Away game" when drivers are open, else "Home game".
- 2026-10-28, step 6: the totals of the choice and of "Duties" come from one server function (`SeasonPlaces`, moved from `api/games.rs` to `api/season.rs`) over all stored games of the season, not over the games of "Season" (which hides the gone games of an earlier season; a past game counts what is stored, A1). The season choice offers the seasons with a chosen team, the current season and the chosen season; a season without a team has no rows. "No duties yet this season." shows when no player has a duty done or planned. The answer has `rows: [{ player, name, hidden, totals: { drive, table, referee } }]`, each total `{ done, planned }` as in the choice.
- 2026-10-28, step 7: `change_between` takes `has_duties: bool` (one game); `compare`, `update_open_changes` and `list_changes` take the set. A gone past game with duties keeps its open change too ("kept while gone"), so it is not dropped because its date passed. The result line of a reading counts such a change ("1 moved"). The set includes players on places of an earlier team of the season (`duty_assignments` of the season, without a team filter), as the totals do.

## Review findings
