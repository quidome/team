# Technical spec: Duty places

**Status**: done
**Size**: story
**Change**: [2026-10-18-duty-places](./)
**Date**: 2026-10-26 (replaces the version approved 2026-10-25, which was built once on the branch `duty-places-v1` before the duty places were given to club teams)

## Objective
The coordinator copies the referee committee's schedule into the tool (at each home game of the
club: referee 1, referee 2, table 1, table 2, each a club team or empty, and "Schedule entered") and
keeps the drivers of the team's away games, on the page of a game; "Season", the start page and the
list of changes show the places of the team, so the committee's schedule and the drivers no longer
live in the sheet.

## Requirements summary
Story A of the epic [Duties](../2026-10-18-duties/) (`functional.md` approved 2026-10-25, second
change; epic `technical.md` approved 2026-10-25, second change):

- **The page of a game** opens with a tap on a game on "Season", in "Next" or "Last" on the start page, on a duty open item for one game, and on a change in the list of changes. It shows date, start time ("time not known yet"), home team, away team, hall and town, and the places. "Back" returns where it came from; to "Season" with the same labels and place in the list.
- **A home game of the club** (of any club team, also of the team; coming or past): 4 places, Referee 1, Referee 2, Table 1, Table 2, each a pull-down list with "—" and the club teams in the game list of the game's season, sorted by name; a choice saves at once. Below them the places of the team: "M16 1: Referee 1, Table 1, Table 2" or "No places for the team.". A button "Schedule entered": before "Schedule not entered", after "Schedule entered", a second tap undoes it; a later change of a place or a move of the game does not undo it.
- **An away game of the team** (at another club): drivers, proposed as one per 3 players with the role "plays games" on the date (rounded up; 10 players: 4; 0 players: 0), "proposed", following the team until the coordinator sets 0–10, which stays; "Use proposal"; a proposal below the filled driver places keeps them ("more than proposed"). No "Schedule entered".
- **Refused**: drivers below the filled driver places ("2 places are filled. Clear one first."); another team or "—" on a place of the team with a player ("Clear the player first.", from story B).
- **On "Season"**: a home game of the club shows the places of the team ("Referee 1: open · Table 1, 2: Anna, open") or "No places for the team", then "· Schedule not entered" until tapped (with no place chosen: only "Schedule not entered"); an away game its drivers ("Drivers (4, proposed): Anna, 3 open"); "all filled" when every place of the team has a player (story B). The label group "Duties": "Entered", "Not entered" (home games of the club), "Open places" (home games of the club and away games of the team); with a Duties label on, other games and trainings do not show.
- **On the start page**: the open item "Home game Sat 31 Oct: schedule not entered" / "5 home games: schedule not entered": the coming home games of the club in the game list from today to 4 weeks ahead without "Schedule entered"; a tap opens the game, or for more than one the list of those games below the item. After the changes not seen, before the trainings. "Next" also shows the home games of other club teams where the team has places. "Last" also shows the games of the last date before today with places of the team (table, referee or drivers).
- **Edge cases**: a moved game keeps its places, teams and "Schedule entered", and its change says "has duties" with the places of the team. A coming game that leaves the game list keeps its places, shows "no longer in the game list" on its page and its row, does not count and is no open item; it counts again when it comes back. A past game counts, also when gone. A game that stops being a home game of the club keeps its places with "This game is no longer a home game of the club." on its page and its row; they do not count, it is no open item, "—" removes them; its change says "has duties". An away game of the team that stops being one: "This game no longer needs drivers of the team.". A team that leaves the game list keeps its name on the place and in the pull-down list. Another team chosen: each place keeps its team; drivers of the old team as above. An earlier season can be changed. Past games: only the places of the team matter, never an open item for the schedule. Two screens: the last save wins, one place one team. A failed save: "Not saved. Try again." and the stored state.
- **Words**: "table", never "jury" (word list 2026-10-25). "Places of the team" = the table and referee places that name the team plus the driver places.
- **Not in this story**: players on places, the choice of a player, the totals, the page "Duties", the open item of open places (story B); the messages (C, D). In story A every place of the team is open.

## Current state
- **On `main`**: no duties code. Games: `games` (key club, season, source id; `in_list`); core `Game`, `schedule::{in_schedule, schedule(stored, club, team, today, open), ScheduledGame, ScheduleReason { TeamGame, DutyGame }}`; `team_choice::own_teams(club, games)` (the club's own teams with short names, sorted by name). `api/season.rs`: `games_of` → `SeasonGames { state, club, team, schedule, unseen_changes }`, `trainings_of`, `EventView`, `ScheduleGame`, `GET /api/season`. Start page: core `start_page::{ReadingState, OpenItem, open_item_seasons, open_items, last_training, next_events}`, `api/start.rs` `StartAnswer { open_items, last, next }`. Changes: `api/changes.rs` `ChangeView`, `/changes`. Roles: `team_core::{PlayerRole, Role, role_on}`. Front end: `EventRow.svelte`, `SeasonEvents.svelte`, `Labels.svelte`, `season.ts` (labels, `keepForReturn` / `takeReturn` for a training), `start.ts`, the start page, `/training/[date]`, `/changes`. The last migration is `0010_trainings_checked`.
- **The first build** (branch `origin/duty-places-v1`, commit `a98daca`, 42 files, not on `main`, not released): migration `0011_game_duties` (referee and jury numbers), core `duties` (`duty_need`, `StoredPlaces`, `driver_proposal`, `players_playing_on`, `GamePlaces`, `game_places`, `check_places`), `start_page` (`NotHandled`, `PlacesOf`, `last_game`, `next_events` with places), server `duties.rs` (`for_season`, `lock`, `save`), `api/games.rs` (`GET` a game, `PUT …/places`), `SeasonDuties`, `places_of_season`, `event_view_with`, `PlacesView`, `StartAnswer.last_game`, `ChangeView.places`, the page `/game/[season]/[id]`, `duties.ts`, the labels "Duties", the Playwright project `duties`. Its reviews (R1–R9, A1–A7) are in its own copy of this file (`git show origin/duty-places-v1:docs/changes/2026-10-18-duty-places/technical.md`).
- **What the new model settles from those reviews**: R1/A1 (the old team's places became the new team's): each place names its team. R4 (a save of one kind removed the other): one save per place. R5 (rows of 0 and 0 where "handled" was not meant): "Schedule entered" is a stored time. A5 (stale places stopped the drivers from counting): each part counts on its own. **To fix in this story**: R2, R3, R7, R8, A6 (see the steps); A4 (decided here, D4).
- **Patterns**: rules in the core; worked out at each request (0020, 0022); a lock or a unique key for each save that two screens can make, with its race test (`docs/architecture.md`); logs with ids only; Dutch dates; "today" for the front end from the server; links with a query: a resolved path plus the query, with a lint exception and its reason.

## Steps

### Step 0: The branch
- Branch `duty-places` from `main`. Copy the code of the first build: `git checkout origin/duty-places-v1 -- crates migrations web .sqlx` (not `docs/`). The tree need not compile; step 0 goes into the commit of step 1.

### Step 1: The tables and every core rule
- **Goal**: the stored shape and all rules of the places, tested before any route or screen uses them.
- **Changes**:
  - `migrations/0011_game_duties.sql` (rewritten; not released):
    ```sql
    -- The duties of a game (story A of duties). A row is made at the first save to the game, under
    -- its row lock, and never deleted.
    CREATE TABLE game_duties (
        club_number         bigint      NOT NULL,
        season_year         integer     NOT NULL,
        source_id           bigint      NOT NULL,
        schedule_entered_at timestamptz,                                -- NULL: not entered
        drivers             smallint    CHECK (drivers BETWEEN 0 AND 10), -- NULL: the proposal
        saved_at            timestamptz NOT NULL,
        PRIMARY KEY (club_number, season_year, source_id),
        FOREIGN KEY (club_number, season_year, source_id) REFERENCES games
    );
    -- One row for each committee place that names a club team; an empty place has no row.
    CREATE TABLE duty_teams (
        club_number  bigint   NOT NULL,
        season_year  integer  NOT NULL,
        source_id    bigint   NOT NULL,
        kind         text     NOT NULL CHECK (kind IN ('referee', 'table')),
        number       smallint NOT NULL CHECK (number BETWEEN 1 AND 2),
        team_number  bigint   NOT NULL,
        team_name    text     NOT NULL,   -- the short name ("M16 1"); stays when the team leaves the game list
        PRIMARY KEY (club_number, season_year, source_id, kind, number),
        FOREIGN KEY (club_number, season_year, source_id) REFERENCES game_duties
    );
    ```
    Apply locally with `sqlx migrate run`.
  - `crates/core/src/duties.rs` (rewritten), exported in `lib.rs`:
    ```rust
    pub enum PlaceKind { Referee, Table }
    pub enum DutyKind { Drive, Table, Referee }
    pub struct TeamRef { pub number: TeamNumber, pub name: String }
    pub struct RequiredDuties { pub schedule: bool, pub drivers: bool }
    /// schedule: the home club is the club. drivers: the team is the away team and the home club
    /// is another club.
    pub fn required_duties(game: &Game, club: ClubNumber, team: TeamNumber) -> RequiredDuties;
    pub struct StoredDuties {
        pub schedule_entered: bool,
        pub drivers: Option<u8>,                         // None: the proposal
        pub teams: BTreeMap<(PlaceKind, u8), TeamRef>,
    }
    pub fn driver_proposal(players: usize) -> u8;                              // kept
    pub fn players_playing_on(roles: &[PlayerRole], date: NaiveDate) -> usize; // kept
    /// own_teams over the stored games of the season (also gone ones), plus the teams that places
    /// of that season still name; sorted by name.
    pub fn club_teams(club: &Club, games: &[Game], stored: &BTreeMap<i64, StoredDuties>) -> Vec<TeamRef>;
    /// The games kept on "Season" for their duties: a place names a team, or drivers set above 0.
    pub fn kept_games(stored: &BTreeMap<i64, StoredDuties>) -> BTreeSet<i64>;

    pub struct GamePlaces { pub schedule: Option<SchedulePart>, pub drivers: Option<DriverPart> }
    pub struct SchedulePart {
        pub places: Vec<SchedulePlace>,   // always 4: Referee 1, Referee 2, Table 1, Table 2
        pub entered: bool,
        pub of_team: Vec<DutyPlace>,      // the places that name the team, in that order
        pub required: bool,
        pub counts: bool,
    }
    pub struct DriverPart {
        pub places: Vec<DutyPlace>, pub proposal: u8, pub proposed: bool,
        pub more_than_proposed: bool, pub required: bool, pub counts: bool,
    }
    pub struct SchedulePlace { pub kind: PlaceKind, pub number: u8, pub team: Option<TeamRef> }
    pub struct DutyPlace { pub kind: DutyKind, pub number: u8, pub filled: bool }
    pub fn game_places(required: RequiredDuties, stored: Option<&StoredDuties>, team: TeamNumber,
                       filled: &[(DutyKind, u8)], players_on_date: usize, in_list: bool, past: bool)
        -> Option<GamePlaces>;
    pub fn not_entered(places: &GamePlaces) -> bool;
    pub fn open_places(places: &GamePlaces) -> usize;
    pub fn has_team_places(places: &GamePlaces) -> bool;

    pub enum DutyError { NotAHomeGame, NoDriversNeeded, OutOfRange, UnknownTeam,
                         BelowFilled { filled: u8 }, PlayerOnPlace }
    pub fn pick_team(list: &[TeamRef], number: TeamNumber) -> Result<TeamRef, DutyError>;
    pub fn check_team(required: RequiredDuties, new: Option<&TeamRef>, place_filled: bool) -> Result<(), DutyError>;
    pub fn check_entered(required: RequiredDuties, on: bool) -> Result<(), DutyError>;
    pub fn check_drivers(required: RequiredDuties, new: Option<u8>, filled: u8) -> Result<(), DutyError>;
    ```
    `game_places`: the schedule part exists when `required.schedule` or a place names a team; the
    drivers part when `required.drivers` or drivers are set above 0; `None` when neither. A part
    counts when the game is past, or when it is in the game list and the game requires the part.
    Drivers: the set number, else the proposal, never below the filled driver places
    (`more_than_proposed` only for the proposal). `filled` is empty and `place_filled` false in
    story A. Removing is always allowed: "—", "Schedule entered" off, drivers 0, and drivers `null`
    on a game that needs no drivers.
  - `schedule::schedule(stored, club, team, today, open, kept)`: a game in `kept` stays when it left the game list and its change was seen, and a game outside `in_schedule` stays with the new reason `ScheduleReason::Kept`.
  - `start_page`: `PlacesOf = BTreeMap<i64, GamePlaces>`; `OpenItem::ScheduleNotEntered { games: Vec<i64> }` (after `ChangesNotSeen`, before `NotOpened`; `OpenItem` is no longer `Copy`); `schedule_not_entered(events, places, today)` (coming, today to today + 27 days, `not_entered`, time order); `open_items(reading, unseen, schedule_not_entered, lists)`; `next_events(events, places, today)` also takes the `DutyGame` games with `has_team_places`; `last_games(events, places)` (all games of the newest date before today with a place of the team in any part, also gone, in time order).
  - Every `jury` of the first build becomes `table`.
- **Tests** (core): `required_duties` for each case (home game of another club team: schedule; home game of the team: schedule; away game of the team at another club: drivers; away game of the team in the club's own hall: schedule, no drivers; away game of another club team: nothing). `driver_proposal` (0 → 0, 1 → 1, 3 → 1, 4 → 2, 10 → 4, 31 → 10); `players_playing_on`. `club_teams` (sorted, opponents left out, a gone game's team in, a team named only by a place in with its stored name). `kept_games` (a named place, drivers 3: kept; drivers 0, only entered: not). `game_places` (not entered and empty; entered and empty; places of other teams only; places of the team in order; a coming gone game does not count; a past gone game counts; a swap: schedule part not required, drivers part required and counting; another team chosen: the old team's places not in `of_team`; drivers proposed, set, more than proposed). The checks (`UnknownTeam`; `NotAHomeGame` for a team and for entered on an away game; "—" and off always allowed; `OutOfRange` for 11; `NoDriversNeeded`; 0 and `null` allowed; `BelowFilled`). `schedule` with `kept` (a seen gone game stays; a game outside the rule has `Kept`; an empty set gives the old result: all existing tests with an empty set). `schedule_not_entered` (inside and outside 27 days; past, gone, entered and swapped games left out; order); `next_events` with and without a home game of another club team with places of the team; `last_games` (all games of the newest date; an older date ignored; a gone past game in).

### Step 2: Store and routes of a game
- **Goal**: the page of a game, and each save, through the API.
- **Changes**:
  - `crates/server/src/duties.rs` (rewritten): `GameKey { club, season, source_id }`; `for_season(db, club, season) -> BTreeMap<i64, StoredDuties>` (two queries); `lock(tx, key, now) -> StoredDuties` (`INSERT … ON CONFLICT DO NOTHING`, `SELECT … FOR UPDATE`, the teams); `set_team(tx, key, kind, number, Option<&TeamRef>)` (an upsert `ON CONFLICT … DO UPDATE`, or a delete); `set_entered(tx, key, on, now)` (on: `COALESCE(schedule_entered_at, now)`; off: `NULL`); `set_drivers(tx, key, Option<u8>)`; each write sets `saved_at`. Never a delete of `game_duties`.
  - `api/season.rs`: `SeasonDuties { stored, roles }` with `places_of(game, in_list, club, team, today)`; `games_of` reads the stored duties first, passes `kept_games` to `schedule`, and gives `SeasonGames.duties`.
  - `api/games.rs` (rewritten). Every write: `find` (the game of the club chosen for its season, with a team; in the schedule or kept; else 404), then one transaction: `lock`, the core check, the write, commit; answer `GamePage`. Log: season and game id only.
    - `GET /api/games/{season}/{id}` → `GamePage { season, current_season, team: TeamView, game: ScheduleGame, club_teams: Vec<TeamView> }`.
    - `PUT /api/games/{season}/{id}/places/{kind}/{number}` `{ team: Option<i64> }` (`kind` `referee` or `table`, `number` 1 or 2).
    - `PUT` and `DELETE /api/games/{season}/{id}/schedule-entered`.
    - `PUT /api/games/{season}/{id}/drivers` `{ drivers: Option<u8> }` (`null`: the proposal).
    - Errors: 400 `invalid_season`, `invalid_game`, `invalid_place`, `unknown_team`, `not_a_home_game`, `no_drivers_needed`, `out_of_range`, `places_filled` (with `filled`), `player_on_place`; 401; 404.
  - Views: `ScheduleGame.places: Option<PlacesView>`; `PlacesView { schedule: Option<SchedulePartView>, drivers: Option<DriverPartView> }`, as the core types, with `TeamView { number, name }`, `PlaceView { kind, number, filled }`; `ScheduleReasonView::Kept`.
  - `app.rs`, `api/mod.rs`; `just api-types`; `just prepare`.
- **Tests** (`crates/server/tests/games.rs`, rewritten): without login 401 on each route. A home game of another club team: 4 empty places, not entered, `club_teams` sorted; a team on Referee 1, Table 1, Table 2 gives `of_team` with 3 places; "—" removes the row; entered on, off, and on twice keeps the first moment. A home game of the team: the same 4 places. An away game: the proposal from 4 players (2), a set number stays, `null` goes back. Refusals: `invalid_place` (`jury`, number 3), `unknown_team`, `not_a_home_game`, `no_drivers_needed`, `out_of_range`, 404 (a game of another club; an id that does not exist). An earlier season with its own choice. Another team chosen (`PUT /api/selection` with another Blue Drakes team): the old team's places stay on their places, are no places of the new team, and come back when the old team is chosen again. Race tests: (1) another connection holds an insert of the same place in an open transaction: the `PUT` waits 300 ms, answers after the commit, its team wins; must fail without `ON CONFLICT`. (2) another connection holds `FOR UPDATE` on the game's row: a drivers save waits 300 ms; in story A it cannot fail without the lock (no check reads filled places; as R6), story B adds the one that does. (3) a trigger in the test's own database refuses writes to `duty_teams`: the save fails and no `game_duties` row is left. `tests/players.rs` (login), `tests/player_logs.rs` (ids only).

### Step 3: The places on the answers
- **Goal**: "Season", the start page and the list of changes get the places from the one rule.
- **Changes**:
  - `api/season.rs`: `places_of_season` from `SeasonGames.duties`; `event_view_with` puts `places` on each game event of `GET /api/season`.
  - `api/start.rs`: `OpenItemView::ScheduleNotEntered { games: Vec<GameItemView> }` (`season`, `id`, `date`, `start_time`, `home`, `away`); "Next" with places; `last_games: Vec<EventView>` and `last_games_season` (the current season first; in July and August the season before when the current one has none). `last` (the last training) stays.
  - `api/changes.rs`: `ChangeView.places` set whenever the game has a part (R3: also a swap, also only stale places); `ChangesAnswer.season`.
  - Measure `GET /api/start` in a release build (limit about 200 ms locally); write the time under **Progress**.
- **Tests** (with the game list harness of `tests/common/game_list.rs`): a moved game keeps its teams and "Schedule entered", and its change has `places`. A swap (another club team now plays away): the game stays on "Season" with reason `kept` and `required: false`, its change has `places`, it is no open item. A coming gone game stays on "Season" while a place names a team, does not count, is no open item; a past gone game counts. A team that leaves the game list keeps its name on the place and in `club_teams`. `schedule_not_entered`: its games in time order, shrinking after "Schedule entered", a game 28 days ahead not in it. "Next" with a home game of another club team with a place of the team. `last_games`: all games of the last date; present without a past training (R2); in July from the season before. `tests/season.rs`: events with `places`. `tests/start.rs`: `a_held_back_reading_is_an_open_item` allows the new item. Existing core and server tests pass with empty places.

### Step 4: The page of a game
- **Goal**: `/game/[season]/[id]`.
- **Changes**: `web/src/routes/game/[season]/[id]/+page.ts` and `+page.svelte`; `web/src/lib/duties.ts` (`placesText(places)`, `teamPlacesText(team, part)` "M16 1: Referee 1, Table 1, Table 2" / "No places for the team.", the warnings "This game is no longer a home game of the club.", "This game no longer needs drivers of the team.", "no longer in the game list"); `client.ts` (`loadGame`, `setPlace`, `setScheduleEntered`, `setDrivers`, the messages of the new errors). A home game of the club: 4 pull-down lists ("—" and `club_teams`), each saving on change; the places of the team; the button "Schedule entered" with "Schedule not entered" / "Schedule entered". An away game: "Drivers: 4 (proposed)" (A6), a number field and "Save" (disabled while empty or not a whole number 0–10: R8; a number equal to the shown proposal goes out as `null`: D4), "Use proposal" when a number is set. A failed save: "Not saved. Try again." and a reload of the stored state. A game not found: "This game cannot be found." with a link to "Season". "Back" by `?from=start|changes`, else "Season" of its season. Removed from the first build: "Remove the places", "No duty places for a home game of the team.".
- **Tests** (`web/e2e/duties.e2e.ts`, rewritten, Playwright project `duties` after `start`): 4 choices and "Schedule entered" on a home game; an away game "Drivers: N (proposed)", a set number stays, saving the proposal unchanged keeps "proposed", "Use proposal"; an empty field cannot be saved; "Not saved. Try again." (`page.route`); the warnings of a swapped and a gone game (`page.route`); a game that cannot be found; accessibility; a phone width without sideways scroll.

### Step 5: "Season"
- **Goal**: the places on the rows, the label group "Duties", a tap on a game, "Back" with the labels.
- **Changes**: `EventRow.svelte` (the places line with `placesText`; "· Schedule not entered"; the warnings; a tap opens the page of the game); `Labels.svelte` and `season.ts` (`Labels { what, when, duties }`, `Duties = 'entered' | 'not_entered' | 'open_places'`; "Entered" / "Not entered" only for games with `schedule.required` (R7), "Open places" for `open_places > 0`; a `kept` game shows with "Possible duty games"); `keepForReturn` / `takeReturn` with the key of a training or `game:<id>`.
- **Tests** (browser): the walk with "Not entered" + "Coming": a tap on a game, 4 choices, "Schedule entered", "Back": the game left the list and the labels and the place are kept. The row "Referee 1: open · Table 1, 2: open". A game with places but not entered shows "… · Schedule not entered" (S18). "Entered", "Open places". `season.e2e.ts` and `choose.e2e.ts` adapted (the group "Duties"; a game row is a link).

### Step 6: The start page
- **Goal**: the open item and its list; "Next" and "Last" with games.
- **Changes**: `start.ts` (`openItemText` for `schedule_not_entered`: "Home game Sat 31 Oct: schedule not entered" / "5 home games: schedule not entered"; one game: a link; more: a button that opens the list below it, `aria-expanded`; `gameLink(season, id)` with `?from=start`); `+page.svelte` ("Last": the last training and `last_games` in date order, newest first (A6), also without a training (R2); "Next" with places).
- **Tests** (browser): the item for more than one game opens its list and a tap opens a game; "Next" shows a home game of another club team with places of the team (`page.route` with a real game); "Last" shows a game without a past training (`page.route`; R2) and the newest event first (A6). `start.e2e.ts` allows the new item.

### Step 7: The list of changes
- **Goal**: "has duties"; a tap on a change opens the game.
- **Changes**: `/changes` ("has duties" with `placesText` and the warnings, when the game has a place of the team or a part no longer required; the two team names are a link with `?from=changes`).
- **Tests**: browser (a change with places says "has duties"; a tap opens the game; `page.route` with one change). Search `jury|handled|not_handled|NotHandled` in `crates/`, `web/src/` and `web/e2e/`: no hits. `just check`.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| What a game requires comes from the game (`required_duties`): the 4 places at each home game of the club, drivers at an away game of the team at another club (D1) | The committee plans by game; each place names its team, so another team chosen moves nothing (fixes R1/A1) | In the record at the close of story B (epic) |
| A game has two parts, the committee places and the drivers; each counts on its own: past, or in the game list and still required (D2) | Old places of one part must not stop the other from counting (A5) | Same record |
| The pull-down list is `own_teams` over all stored games of the season plus the teams that places still name; the server takes the name from that list and stores the short name (D3) | No new source request; a name never disappears; the page cannot write a name | Same record |
| A drivers save equal to the proposal on the screen stays the proposal (D4) | "While the coordinator has not changed it, it is the proposal" (A4); a number equal to the proposal cannot be fixed, accepted | No |
| A game with a place that names a team, or drivers set above 0, stays on "Season" when gone or outside the schedule, with reason `Kept` (D5) | The spec shows the warning "on its row" (coordinator, 2026-10-26: option a) | In the epic record |
| "Schedule entered" is `schedule_entered_at`; the first moment stays; off is `NULL` | Two checks of a game; as "Checked" of a training | Same record |
| One route per place, one for "Schedule entered", one for drivers; each under the row lock of the game; one place one team by the primary key | Each choice saves at once; two screens: the last save wins (fixes R4) | No |
| The first build is the start: its code copied from `origin/duty-places-v1` to a new branch from `main`, reworked | Most of it stays (coordinator, 2026-10-26) | No |

## Not in scope
- Players on places, the choice of a player, the totals, the page "Duties", the open item of open places, "Delete player" with duties (story B); the messages (C, D).
- Reading the committee's sheet; saving more than one game at a time; copying the schedule of an earlier game.
- From the first build: the state "handled", the labels "Handled" / "Not handled", the save of 0 and 0, the button "Remove the places".

## Test plan
- **Existing tests that must still pass**: all core tests (`schedule` with an empty `kept`; `open_items` and `next_events` with empty places); `tests/season.rs` (events get `places`), `tests/start.rs` (the new item in the 4 weeks), `tests/changes.rs`, `tests/players.rs`, `tests/player_logs.rs`; browser `choose.e2e.ts` (the new item), `season.e2e.ts` (the group "Duties", a game row is a link), `start.e2e.ts`.
- **First-build tests removed**: "a save of zero and zero handles the game", "a home game of the team has no places", "an away game in the club's own hall needs no drivers".
- **New tests**: per step, above.
- **Not testable in the browser** (clock fixed at 12 October 2026, the fake source never changes): July, a new season, another team chosen (core and server); a swap and a gone game (rules on the server, words with `page.route`).
- **Checks while working**: `cargo test -p team_core`; `cargo test -p team --test games --test season --test start --test changes`; `just check-web`, then `npx playwright test duties` (and the changed files); `cargo clippy --workspace --all-targets -- -D warnings` at the end of each step; `just check` once before the commit.

## Compatibility and performance
- New tables (migration 0011 is not released, so it is rewritten) and routes; `ScheduleGame`, `ChangeView`, `ChangesAnswer` and `StartAnswer` get new fields; the front-end types are generated.
- "Season" can show a gone game, or a game outside the schedule, while it keeps places of a team (D5).
- A tap on a game (on "Season", in "Next", "Last", the list of changes) opens its page; before it did nothing.
- `GET /api/start` reads the duties of one or two seasons and every home game of the club in 4 weeks; measured in step 3 (limit about 200 ms locally; the first build measured 2–4 ms).
- Released with story B (release 1 of the epic).

## Progress
- [x] Step 1 – the tables and every core rule (2026-10-26: 233 core tests pass; clippy clean with `SQLX_OFFLINE=true`; `svelte-check` and lint clean. After Rancher Desktop forwarded port 5432 again: migrations 0010 and 0011 applied locally, 264 server tests pass)
- [x] Step 2 – store and routes of a game (2026-10-26: 11 tests in `tests/games.rs`; 275 server tests pass; the race test of a place fails without `ON CONFLICT` (checked once: 500); clippy, `svelte-check` and lint clean)
- [x] Step 3 – the places on the answers (2026-10-26: 8 new tests in `tests/games.rs`; 283 server and 233 core tests pass; 95 browser tests pass (see Deviations for the local accessibility check). `GET /api/start`: median 8.6 ms, min 7.8, max 21.5 over 20 requests, release build on the browser-test database (real game list, players, trainings) with "Schedule entered" and a place of the team on 10 coming home games of the club)
- [x] Step 4 – the page of a game (2026-10-26: 7 browser tests in `web/e2e/duties.e2e.ts`, Playwright project `duties` after `start`; all 100 browser tests pass; `just check-web` passes. The drivers test fails when a save of the shown proposal sends the number instead of `null` (D4))
- [x] Step 5 – "Season" (2026-10-27: 3 browser tests added to `web/e2e/duties.e2e.ts` (10 in all); all 103 browser tests pass; `just check-web` passes)
- [x] Step 6 – the start page (2026-10-27: 3 browser tests added to `web/e2e/duties.e2e.ts` (13 in all); all 106 browser tests pass; `just check-web` passes; `start.e2e.ts` needed no change after Step 3)
- [x] Step 7 – the list of changes (2026-10-27: 1 browser test added to `web/e2e/duties.e2e.ts` (14 in all); the search for `jury|handled|not_handled|NotHandled` in `crates/`, `web/src/` and `web/e2e/` has no hits; `just check` passes except `api-types-check`, see Deviations: 233 core and 283 server tests, all 109 browser tests, `check-sqlx`)
- [ ] All checks pass

## Deviations
None from `functional.md`. Readings of the spec that the design makes exact:
- The window "4 weeks" is today to today + 27 days (Dutch date), as in the epic.
- An away game of the team in the club's own hall is a home game of the club: it gets the 4 places and no drivers.
- The driver proposal is at most 10, the most that can be set (31 players or more).
- Saving the drivers number that equals the shown proposal keeps the proposal (D4); a number equal to today's proposal cannot be fixed.
- A game kept on "Season" for its places (gone, or outside the schedule) shows with the label "Possible duty games".
- "Last" shows the last training and the last games in date order, newest first, also without a training (R2, A6).
- A game that cannot be found: "This game cannot be found." with a link to "Season" (first build, accepted as A7).

Found while building (2026-10-26):
- Step 0: only the core and the migration of the first build were copied. The copied server and front-end code use the old core types, so with all of it the workspace would not compile until step 3. Each later step takes its own files from `origin/duty-places-v1`, so each step stays mergeable alone.
- Step 1: `filled` of `game_places` is the set of filled places by kind and number (`BTreeSet<(DutyKind, u8)>`), not a count for each kind: story B puts a player on one exact place. `check_team` also takes the current team of the place, so choosing the same team again is never refused.
- Step 1: the server and the front end had to follow the new core rules to compile: `ScheduleReasonView::Kept` (the front end shows it with "Possible duty games"), and `OpenItemView::ScheduleNotEntered { games }` with `GameItemView` and its words ("Home game Sat 31 Oct: schedule not entered" / "5 home games: schedule not entered"). Until step 3 the start page passes no games and no places, and until step 4 a tap on the item goes to "Season". This is a small part of steps 3 and 6, done early.
- Step 1: `SCHEDULE_PLACES` (the 4 places in their order) is public in the core, for the store and the views.
- Step 2: `games_of` reads the stored duties only for `kept_games`; `SeasonGames.duties` comes in step 3, where "Season" and the start page read it (until then the field would be dead code, which clippy refuses).
- Step 2: `find` also reads the stored duties and makes `club_teams` (all stored games of the season, also gone ones), so a `PUT` of a place takes the team and its name from that list; the page is read again after each write. The writes share one helper (`write`: begin, `lock`, the check and the write, commit, one log line with season and game id).
- Step 2: `PlacesView` also carries `not_entered` and `open_places` from the core rules (decision 0022), so the labels and the rows need not work them out in the front end.
- Step 3: the tests of the answers are in `tests/games.rs`, not in `tests/season.rs`, `tests/start.rs` and `tests/changes.rs`: they need its helpers that find a home game of another club team, an away game and a home game of the team. Changed in the other files: `tests/start.rs` (`a_held_back_reading_is_an_open_item` expects the item "schedule not entered" next to the held-back reading) and `tests/changes.rs` (the answer without a choice has `season: null`).
- Step 3: `web/e2e/choose.e2e.ts` expects the new open item where it expected "Nothing open." (the real games have home games of the club in the 4 weeks).
- Step 3: on this computer the browser test "the games on Season › is accessible, …" fails on `main` too (2026-10-26, checked in a worktree of `main`): axe measures the colour contrast of a label button while its colours fade (`transition-all` in `toggle.svelte`; foreground #b3afac on #d47d55). It is not caused by this change. With a wait for running animations in `expectAccessible` (only for that run, not kept) all 95 browser tests pass. Fixed apart, on `main` (commit `abe7009`, 2026-10-26).
- Step 4: the page also shows "Proposal: N (one for every 3 players)" while a number is set, so the coordinator sees the proposal that a save turns back into "proposed" (D4); `driversNumber` in `duties.ts` gives "Drivers: 4 (proposed)" / "(more than proposed)". "Use proposal" and the proposal rule of a save are only for drivers the game still needs; on a game that no longer needs them, 0 removes them. "Schedule entered" shows only while the game needs the schedule, or while it is on (so it can be undone). Messages for the new refusals: `places_filled` "2 places are filled. Clear one first.", `player_on_place` "Clear the player first.", `not_a_home_game`, `no_drivers_needed` (the warning texts), `out_of_range` "Enter a number from 0 to 10.", `unknown_team`, `invalid_place`, `invalid_game`.
- Step 5: `season.e2e.ts` and `choose.e2e.ts` needed no change: their tests do not count the label buttons, and a game row is a link inside the row (`data-testid="game-link"`), so the row itself stays. The row gets `data-id`. The row builds the link to the game itself from a `gameSeason` (the lint rule wants `resolve()` there); without it the row is no link (the start page until Step 6). With a label of "Duties" on, an empty part says "No coming games with these labels." "Not entered" uses `not_entered` of the server (the rule of the open item, decision 0022); "Entered" is a game that needs the schedule and has it entered. The warnings show on the row without "no longer in the game list", which the row already shows as a badge.
- Step 6: "Last" shows its events under a heading per date, as "Next" does (before: one date line above the one training), since it can now hold a training and games of other dates. A game of an open item reads "Sat 31 Oct · 14:30 Blue Drakes X14 1 – BV Alphia X14 1" (`gameItemText`). The game links of "Next" use the season of the choice; those of "Last" use `last_games_season`.
- Step 7: `loadChanges` gives the whole answer (with `season`), for the links to the games. A change links to its game except a change "out of the schedule" of a game without places (that page does not exist: 404). "has duties" shows for a game with places of the team, drivers, or a part it no longer needs; the warnings show without "no longer in the game list" (a gone game says so in its line of what happened).
- Step 7: `tests/games.rs` sent the old place name `jury` to check that an unknown place kind is refused; it now sends `drive` (a duty kind with no place in the schedule), so the search has no hits and the test keeps its point.
- Step 7: `api-types-check` (`git diff --exit-code` on `web/openapi.json` and `schema.d.ts`) fails while the change is not committed, since it compares with the last commit. Checked instead (2026-10-27): `just api-types` generates the same two files as in the working tree. It passes once the files are committed.
- Step 2: as planned, the race test of the row lock (`a_save_waits_for_a_save_of_the_same_game_on_another_screen`) cannot fail without the lock in story A (as R6); story B adds the one that does.

## Review findings
Build review, 2026-10-27 (fresh agent; input: the diff `main..duty-places` at `e6147aa`, clean tree;
this change folder, the epic's `functional.md` for intent, `docs/architecture.md`, the valid decision
records, `CLAUDE.md`). No other text came with the arguments.

Overall: the rules sit in the core (`duties`, `schedule`, `start_page`), the server only moves data,
each write is one transaction under the row lock of the game, logs carry season and game id only, no
`unwrap`/`expect` outside tests. No finding blocks the close of story A.

- **R1 – Two rules live in the front end** (low). `dutiesOf` in `web/src/lib/changes.ts` decides when
  a change "has duties" (`of_team` not empty or a part no longer required; drivers likewise), and
  `hasPage` guesses whether the page of the game exists (`kind !== 'out_of_schedule' || places`),
  which mirrors `find` in `api/games.rs` (`in_schedule` or `kept_games`). By trace the two agree
  today (a game outside `in_schedule` has `places` only when it is kept), but they are copies of
  server rules, the kind the step 2 deviation avoided by putting `not_entered` and `open_places` on
  `PlacesView` ("New rules go in `crates/core` first", decision 0022). Suggestion: `has_duties` and
  `has_page` on `ChangeView` from the core, or accept and record. ("Entered" in `season.ts` is a
  plain read of two fields; no finding.) Checked: reading and a trace of `in_schedule`,
  `required_duties`, `game_places` and `kept_games`.
  - 2026-10-27: Not fixed, accepted by the coordinator: the two checks stay in `web/src/lib/changes.ts`; they agree with the server today (trace of the review). A later change that moves them to the core puts them on `ChangeView`.
- **R2 – The pull-down list of an answer is from before the write** (low). `set_place`,
  `set_entered` and `set_drivers` answer `page(&state, found)`, and `found.club_teams` was made in
  `find` before the write. A team that only a place still names (it left the game list) stays in the
  list after that place is set to "—"; choosing it on another place from the same screen gives
  `unknown_team`, after which the page reloads without it. Fix: build `club_teams` in `page` from the
  duties it already reads. Checked: trace of `find`, `write` and `page`; not run.
  - 2026-10-27: Not fixed, no defect: games are never deleted (a game that leaves the game list stays stored with `in_list` false), and the pull-down list is `own_teams` over all stored games of the season (D3), so a team stays in the list for the whole season, named by a place or not. Checked: a fix that built the list again in `page`, with a server test that sets the place of a leaving team to "—"; the team was still offered, with and without the fix. Fix and test reverted (coordinator).
- **R3 – A past gone game keeps only named places and set drivers, not the proposal** (note for story
  B). `kept_games` keeps a game whose place names a team or whose drivers are set above 0. A past
  away game of the team that left the game list, with the drivers still proposed, is dropped by
  `schedule` once its change is seen, so it leaves "Season", "Open places" and "Last", although
  "a past game counts, also when gone" (epic, Edge cases) and `game_places` would count it. In story
  A no place has a player, so nothing is lost yet; in story B a filled driver place on a proposed
  number must also keep the game (or `kept_games` must keep every past game with a counting part).
  Checked: trace of `schedule`, `kept_games`, `game_places`; not run. The same rule decides the
  earlier season in the "Last" fallback of `api/start.rs`.
  - 2026-10-27: Not fixed, for story B: in story A no place has a player. Story B keeps a past game whose filled driver places are on a proposed number (or every past game with a counting part); in `docs/backlog.md`.
- **R4 – Claims about checks** (verified, one known gap). Run on 2026-10-27: `cargo test -p team
  --test games` 19 passed; `cargo test -p team_core` 233 passed; `cargo clippy --workspace
  --all-targets -- -D warnings` clean; `npm run build` then `npx playwright test` 109 passed;
  `just api-types` gives no diff on the committed files, so the `api-types-check` deviation of step 7
  is settled by the commit. Search `jury|handled|not_handled|NotHandled` in `crates`, `web/src`,
  `web/e2e`, `migrations`: no hits. By trace, `a_place_waits_for_an_insert_of_the_same_place_on_another_screen`
  fails without `ON CONFLICT` (the `PUT` waits on the uncommitted `game_duties` key in `lock`, then
  its plain insert of `duty_teams` hits the primary key: 500, as the progress says), and
  `a_failed_save_writes_nothing` fails without the transaction (the `game_duties` row of `lock` would
  stay). `a_save_waits_for_a_save_of_the_same_game_on_another_screen` cannot fail without the lock,
  as written; the architecture asks that each such test can fail, so story B must add its test
  (a check that reads filled places).
  - 2026-10-27: Not fixed, for story B: its first check that reads filled places gets the race test that fails without the row lock; in `docs/backlog.md`.
- **R5 – Text the diff made false or left incomplete** (for `/close`). The module doc of
  `crates/server/src/api/start.rs` lists the sources of the open items without the page of a game.
  `CLAUDE.md` (Architecture) names neither the core `duties` nor the server `duties.rs` and
  `api/games.rs`. `docs/architecture.md`: the core row (`schedule` "gone games only while their
  change shows", `open_items` order, `next_events` "team games"), the server row (`games.rs` "places"
  is the source's places, no `duties.rs` or `api/games` route), migrations up to `0010`, the start
  page ("About 10 queries"), "Season" (the return kept "for that one date"; now also a game).
  `docs/product/schedule.md` still says "jury". Checked: search (`duties` in `CLAUDE.md`: 0 hits;
  `jury` in `docs/product`: 1 hit) and reading.
  - 2026-10-27: Not fixed here, for `/close`: the module doc of `api/start.rs`, `CLAUDE.md`, `docs/architecture.md` and `docs/product/schedule.md`.

Accept review, 2026-10-27 (fresh agent; input: the mode `accept` and this change folder, nothing
else in the arguments). Read: the epic's `idea.md` and `functional.md` (story A, the facts, the
edge cases, the interface), `change.md` of both, the objective and requirements of this file, and
the result: the diff `main..duty-places` at `e6147aa` and the program. Run: `npm run build`, then
`npx playwright test` (109 passed); then the server on the browser-test database (clock 12 October
2026, logged in as Developer, port 4110) at phone width: the start page, the open item and its
list, the page of a home game of the team and of an away game, a choice of two places, "Season"
with the rows and the labels. The view below was formed before reading **Deviations**.

**Verdict: accepted with notes.** Story A does what was meant: the coordinator can walk "Season"
with "Not entered" + "Coming", copy the committee's schedule with four pull-down lists that save
at once, tap "Schedule entered", and come back with the labels kept; the drivers are proposed and
can be set; the rows, the open item, "Next", "Last" and the list of changes show the places of
the team. One rule reading (A1) must be settled before story B counts totals.

- **A1 – A past game that is no longer a home game of the club counts its places** (concerning
  deviation, not blocking story A; settle before story B). The spec says of a swapped game "its
  places do not count and it is no open item", and of a game no longer needing drivers "the same
  way"; it lets past duties count only for a game that left the game list and for drivers of an
  old team after another team was chosen. `game_places` uses one rule for both parts, `counts =
  past || (in_list && required)`, so once its date passes a swapped game counts again: the team's
  table or referee places of a home game that was played at another club, and drivers set for an
  away game that became a home game. In story A it shows in "Last" (`has_team_places`) with the
  warning "This game is no longer a home game of the club."; in story B those places would count
  as **done** in the totals, which is the thing the totals must not do ("the totals and open
  items must never count a duty of a game that is gone or no longer needs duties", warnings of the
  spec). The core cannot tell a swap from another team chosen, which the spec treats differently
  for drivers. Not listed under **Deviations** ("None from `functional.md`"). Ask the coordinator
  which wins for a past swapped game, then fix the rule or record the reading. Checked: trace of
  `game_places`, `has_team_places`, `last_games`; the core tests
  `drivers_no_longer_needed_stay_until_cleared_and_do_not_count` (past: counts) and
  `a_swapped_game_keeps_its_places_and_its_drivers_count` (coming only); no test of a past swap.
  - 2026-10-27: Not fixed, the coordinator decides in the design of story B, which adds the totals: in story A no totals count the places (the game only shows under "Last" with its warning). In `docs/backlog.md`.
- **A2 – The readings under Deviations** (faithful adaptation). The window of 27 days after today;
  a game of the team in the club's own hall gets the 4 places and no drivers (the answer of Phase
  1, "drivers only when the home club is another club"); the proposal capped at 10; a kept game
  under "Possible duty games"; "Last" newest first with a heading per date and without a training;
  "This game cannot be found."; "Proposal: N" shown while a number is set; "Schedule entered"
  shown only while needed or on; the warnings not repeated next to the badge on a row. Each keeps
  the intent of the criterion it touches. Checked: reading against story A and the facts; the
  start page and the two game pages seen at 390 px.
  - 2026-10-27: No action.
- **A3 – D4: a drivers number equal to the proposal stays "proposed"** (neutral drift). The spec
  says a set number "stays, without 'proposed'". With D4, typing the number the screen proposes
  keeps following the team, so a coordinator who means "4, fixed" gets 4 that may later move.
  "Proposal: N" and the label "(proposed)" make it visible, and it was decided on purpose (A4 of
  the first build). Acceptable; worth a line in the product doc at `/close`. Checked: reading of
  `driversInput` and step 4 of the progress (its test fails when the number is sent).
  - 2026-10-27: Not fixed, as decided (D4); `/close` adds a line to the product doc.
- **A4 – Past home games say "Schedule not entered" for ever on "Season"** (neutral drift,
  accepted in S19). `not_entered` holds for past games (`counts` is true for a past game), so the
  catch-up of this season's past places leaves every past home game of the club with "Schedule not
  entered" on its row and under the label "Not entered" (without "Coming"). The open item stays
  free of past games, as the spec asks. Seen on the server: the past rows; checked by trace of
  `not_entered` and `schedule_not_entered`.
  - 2026-10-27: No action (accepted in S19).
- **A5 – The one button "Schedule entered" in both states** (neutral drift, small). The page shows
  the state as text ("Schedule not entered" / "Schedule entered") next to a button that is always
  named "Schedule entered": filled before the tap, outlined (`aria-pressed`) after. This is what the
  spec wrote; on a phone, "undo" is not obvious from the button. A note for real use, no change
  needed now. Checked: seen on the page of a home game; reading of the page.
  - 2026-10-27: No action; a note for real use.
- **A6 – Things the spec left open, as built** (faithful adaptation). The team's own home games
  are in the open item and the walk (S20, accepted); the pull-down lists hold the club teams of the
  season's stored games plus the teams places still name (D3), sorted as text ("M16 1", "M16 2",
  …, "X14 1" on the server: fine for this club); a moved coming game with only places of other
  teams does not say "has duties" (the line is about the team's places); a coming gone game with
  only "Schedule entered" or proposed drivers leaves "Season" once its change is seen (nothing of
  the team is lost in story A; for story B see R3). Checked: the server answer of
  `/api/games/2026-2027/1073152`, reading of `dutiesOf` and `kept_games`.
  - 2026-10-27: No action.

Not checked: July and August ("Last" from the season before), another team chosen, a real swap or
gone game (the browser clock and the fake source cannot make them; covered by server tests, read
only). The places this review chose on the browser-test database are reset by the next browser test
run.
