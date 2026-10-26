# Technical spec: Duty places

**Status**: approved
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
- [ ] Step 1 – the tables and every core rule
- [ ] Step 2 – store and routes of a game
- [ ] Step 3 – the places on the answers
- [ ] Step 4 – the page of a game
- [ ] Step 5 – "Season"
- [ ] Step 6 – the start page
- [ ] Step 7 – the list of changes
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

## Review findings
