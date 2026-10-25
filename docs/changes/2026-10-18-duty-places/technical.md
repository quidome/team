# Technical spec: Duty places

**Status**: approved
**Size**: story
**Change**: [2026-10-18-duty-places](./)
**Date**: 2026-10-25

## Objective
The coordinator records the duty places of the team at each game (referee and jury places at home
games of other club teams, drivers at away games of the team), on the page of a game, and sees
them on "Season" and the start page, with "not handled" as an open item, so the committee's list
and the drivers no longer live in the sheet.

## Requirements summary
Story A of the epic [Duties](../2026-10-18-duties/) (`functional.md` approved again 2026-10-25;
epic `technical.md` approved again 2026-10-25):

- **The page of a game** opens from a tap on a game on "Season", in "Next" or "Last" on the start page, on a duty open item, and on a change in the list of changes. It shows date, start time ("time not known yet"), home team, away team, hall and town, and the places. "Back" returns where it came from; to "Season" with the labels and the place kept.
- **A possible duty game** (a home game of another club team): referee and jury places, each 0–2, "Save". A save makes it **handled**, also with 0 and 0 ("No duty places for the team."); before a save "Not handled yet". Past games too.
- **An away game of the team** (at another club): drivers, proposed as one per 3 players with the role "plays games" on the date (rounded up; 10 players: 4; 0 players: 0), "proposed", following the team until the coordinator sets 0–10, which stays; "Use proposal"; a proposal below the filled driver places keeps them ("more than proposed"). Always handled.
- **A home game of the team**: no places; the page says so.
- **Refused**: a number below the filled places ("2 places are filled. Clear one first.").
- **On "Season"**: each game that can have duties shows its places ("Referee: open · Jury: Anna, open", "Drivers (4, proposed): Anna, 3 open", "No duty places", "Not handled yet", "all filled"); the label group "Duties" ("Handled", "Not handled", "Open places") for possible duty games and away games only; a tap on a game opens its page.
- **On the start page**: the open item "Possible duty game Sat 31 Oct not handled" / "5 possible duty games not handled" (today to 4 weeks ahead; after the changes not seen, before the trainings; for more than one, a tap opens the list of those games below it); "Next" also with the possible duty games where the team has places; "Last" also with the last game before today with places.
- **Changes**: a game that moves keeps its places and players; its change says "has duties" with the places. A coming game that is gone keeps them, shows "no longer in the game list", does not count and is no open item; it counts again when it comes back. A past game counts, also when gone later. A game that no longer needs duties of the team (home and away swapped, the team changed, another team chosen) keeps its places with "This game no longer needs duties of the team." and they do not count (coming); setting them to 0 removes them. An earlier season can be changed (S13).
- **Not in this story**: players on places, the choice, the totals, "Duties", the open item of open places (story B); the messages (C, D). In story A every place is open.

## Current state
- **Games**: `games` (key club, season, source id; `in_list`); core `Game { source_id, date, start_time, home, away: Side { team, team_name, club }, place, … }`, `schedule::{in_schedule, schedule, ScheduledGame { game, reason, in_list }, ScheduleReason }`.
- **Season**: `api/season.rs`: `games_of(state, season, current, today) -> SeasonGames { state, club, team, schedule, unseen_changes }`, `trainings_of`, `SeasonAnswer`, `EventView { date, week, week_first_day, week_last_day, game: Option<ScheduleGame>, training: Option<TrainingRow> }`, `ScheduleGame { id, date, start_time, home, away, hall, town, reason, in_list }`; `GET /api/season`.
- **Start page**: core `start_page::{ReadingState, OpenItem, open_item_seasons, open_items(reading, unseen, lists), last_training(lists), next_events(events, today)}` (cancelled trainings do not count in "Next"); `api/start.rs` `StartAnswer { open_items, last, next }`, `OpenItemView` (tagged by `kind`).
- **Changes**: `api/changes.rs` `ChangeView { id, version, kind, differences, reason, before, after }`, `/changes` page.
- **Roles**: `team_core::{PlayerRole, Role, role_on}`; `players::roles_in(db, seasons)`.
- **Front end**: `EventRow.svelte` (a game row: start time, teams, hall and town, badge "Team"; a training row with a tap), `SeasonEvents.svelte`, `Labels.svelte`, `season.ts` (`Labels { what, when }`, `visible`, `parts`, `emptyLine`, `groupByDate`, `groupByWeek`, `keepForReturn(season, date, labels, scrollY)` / `takeReturn` for a training), `start.ts` (`openItemText`, `openItemLink`, `trainingLink`), the start page `+page.svelte`, `/training/[date]` ("Back" by `?from=start|attendance`), `/changes`.
- **Patterns**: rules in the core; worked out at each request (0020, 0022); a lock for each save that two screens can make, with its race test (`docs/architecture.md`); logs with ids only; Dutch dates; "today" for the front end from the server; links with a query: a resolved path plus the query, with a lint exception and its reason. The last migration is `0010_trainings_checked`.

## Steps

### Step 1: Migration and core `duties`
- **Goal**: the table, and the only rule for the places of a game.
- **Changes**:
  - `migrations/0011_game_duties.sql`:
    ```sql
    -- The duty places of the team at a game (story A of duties). A row exists from the first save;
    -- a row with 0 and 0 means "handled, no places for the team" and is never deleted.
    CREATE TABLE game_duties (
        club_number  bigint   NOT NULL,
        season_year  integer  NOT NULL,
        source_id    bigint   NOT NULL,
        referee      smallint NOT NULL CHECK (referee BETWEEN 0 AND 2),
        jury         smallint NOT NULL CHECK (jury BETWEEN 0 AND 2),
        drivers      smallint CHECK (drivers BETWEEN 0 AND 10),     -- NULL: the proposal
        saved_at     timestamptz NOT NULL,
        PRIMARY KEY (club_number, season_year, source_id),
        FOREIGN KEY (club_number, season_year, source_id) REFERENCES games
    );
    ```
    Apply locally with `sqlx migrate run` / `just migrate`.
  - `crates/core/src/duties.rs` (new), exported in `lib.rs`:
    ```rust
    pub enum DutyKind { Drive, Jury, Referee }
    pub enum DutyNeed { Drivers, JuryAndReferee }
    /// Jury and referee at a home game of another club team; drivers at an away game of the team
    /// at another club (not in the club's own hall); else None.
    pub fn duty_need(game: &Game, club: ClubNumber, team: TeamNumber) -> Option<DutyNeed>;
    pub struct StoredPlaces { pub referee: u8, pub jury: u8, pub drivers: Option<u8> }
    /// One per 3 players, rounded up; at most 10.
    pub fn driver_proposal(players: usize) -> u8;
    pub enum PlacesState { NotHandled, NoPlaces, Places }
    pub struct Place { pub kind: DutyKind, pub number: u8, pub filled: bool }
    pub struct GamePlaces {
        pub need: Option<DutyNeed>, pub state: PlacesState,
        pub places: Vec<Place>, pub no_longer_needed: Vec<Place>,
        pub drivers_proposed: bool, pub more_than_proposed: bool,
        /// False: coming and gone from the game list, or no longer needed; does not count.
        pub counts: bool,
    }
    pub fn game_places(need: Option<DutyNeed>, stored: Option<&StoredPlaces>, filled: &[(DutyKind, u8)],
                       players_on_date: usize, in_list: bool, past: bool) -> Option<GamePlaces>;
    pub fn handled(places: &GamePlaces) -> bool;
    pub fn open_places(places: &GamePlaces) -> usize;
    pub enum PlacesError { NotNeeded, OutOfRange, BelowFilled { filled: u8 } }
    pub fn check_places(need: Option<DutyNeed>, new: &StoredPlaces, filled: &[(DutyKind, u8)]) -> Result<(), PlacesError>;
    ```
    `game_places`: `None` for a game with no need and nothing stored. Jury and referee: no row → `NotHandled`; 0 and 0 → `NoPlaces`; else `Places`. Drivers: the stored number, else the proposal, never below the filled driver places (`more_than_proposed`); 0 → `NoPlaces`; always handled. Stored places of a kind the game does not need go to `no_longer_needed`, and then `counts` is false while the game is coming. A coming game not in the game list: `counts` false. A past game counts. `filled` is empty in story A.
- **Tests** (core): `duty_need` for each case (home game of another club team; away game of the team at another club; away game of the team in the club's own hall: none; home game of the team: none; away game of another club team: none); `driver_proposal` (0 → 0, 1 → 1, 3 → 1, 4 → 2, 10 → 4, 31 → 10); `game_places` (not handled; no places; places; drivers proposed; drivers set; more than proposed only for filled places above a lower proposal; coming and gone: `counts` false; past and gone: `counts` true; home and away swapped: `no_longer_needed`; home game of the team: `None`); `check_places` (3 referees: `OutOfRange`; jury at an away game: `NotNeeded`; a kind no longer needed down to 0: allowed; below the filled places: `BelowFilled`); `handled`, `open_places`.

### Step 2: Store, `duties_of`, and the routes of a game
- **Goal**: the page of a game and saving its places through the API.
- **Changes**:
  - `crates/server/src/duties.rs` (store): `for_season(db, club, season) -> BTreeMap<i64, StoredPlaces>`; `lock(tx, club, season, source_id)` (in the transaction of the save: `INSERT … (0, 0, NULL) ON CONFLICT DO NOTHING`, then `SELECT … FOR UPDATE` on the row; a save that fails rolls back, so a failed save never makes a game handled, and the lock never waits for a reading of the game list); `save(tx, key, places, now)` (an upsert). Never a delete.
  - `api/season.rs`: `duties_of(state, season) -> SeasonDuties { stored, players_on: fn(date) -> usize }` (the stored places, and from the roles of the season the number of players with "plays games" on a date); `game_places_of(game, need, &SeasonDuties, today)`.
  - `api/games.rs` (new): `GET /api/games/{season}/{id}` → `GamePage { season, current_season, game: ScheduleGame, need: Option<DutyNeedView>, proposal: Option<u8> }`; `PUT /api/games/{season}/{id}/places` `PlacesInput { referee, jury, drivers: Option<u8> }` → `GamePage`: one transaction: lock, read, `check_places`, save. The game must be a game of the club chosen for that season with a team (else 404). Errors: 400 `invalid_season`, `invalid_game`, `not_needed`, `out_of_range`, `places_filled` (with `filled`). `ScheduleGame` gets `places: Option<PlacesView>`: `PlacesView { state, referee, jury, drivers: Vec<PlaceView>, no_longer_needed, drivers_proposed, more_than_proposed, counts }`, `PlaceView { kind, number }`.
  - `app.rs`, `api/mod.rs`; `just api-types`; `just prepare`.
- **Tests** (`crates/server/tests/games.rs`, new): without login 401; a possible duty game is "not handled"; save 1 and 2 → places, also on `GET /api/season`; save 0 and 0 → handled, no places, the row stays; an away game: the proposal from 4 players (2), a set number stays, `drivers: null` goes back; errors `not_needed`, `out_of_range`, 404 for a game of another club, `invalid_game`; an earlier season with its own choice can be saved; the race test of the lock (another connection holds the lock in an open transaction: the save waits 300 ms and answers after the commit; must fail without the lock); a moved game keeps its places after a reading; a coming gone game has `counts: false`. `tests/players.rs` (login), `tests/player_logs.rs`.

### Step 3: The places on the answers
- **Goal**: "Season", the start page and the page of a game get the places from one rule.
- **Changes**:
  - `start_page.rs`: `OpenItem::NotHandled { games: Vec<GameRef> }` (after `ChangesNotSeen`, before `NotOpened`); `open_items(reading, unseen, lists, events, places, today)`; `next_events(events, places, today)` also takes the possible duty games whose places count and are `Places`; `last_game(events, places)` (the newest game before today with `Places`, also when gone).
  - `api/season.rs`: `GET /api/season` fills `places` on each game event with `game_places_of`.
  - `api/start.rs`: `StartAnswer` gets `last_game: Option<EventView>`; `OpenItemView::NotHandled { games: Vec<GameItemView> }` (`season`, `id`, `date`, `start_time`, `home`, `away`); "Next" with places.
  - Measure `GET /api/start` again (about 200 ms locally at most); write the time under **Progress**.
- **Tests**: core `open_items` with `NotHandled` (inside and outside the 4 weeks; a gone game left out; the order); `next_events` with and without a duty game with places; `last_game` (the newest past game with places, also gone). `tests/start.rs`: the open item `not_handled` with its games, shrinking after a save; "Next" and `last_game` with places. `tests/season.rs`: events with `places`. The existing core and server tests pass with empty places.

### Step 4: The page of a game
- **Goal**: `/game/[season]/[id]`.
- **Changes**: `web/src/routes/game/[season]/[id]/+page.ts` and `+page.svelte`; `web/src/lib/duties.ts` (new: `placesText(places)`, the warnings); `client.ts` (`loadGame`, `savePlaces`, the messages of the new errors). A possible duty game: Referee (0, 1, 2) and Jury (0, 1, 2) and "Save"; an away game: Drivers (0–10), "Save", "Use proposal" when a number is set; a home game of the team: "No duty places for a home game of the team."; "Not saved. Try again." with the stored state; "This game no longer needs duties of the team.", "no longer in the game list"; "Back" by `?from=start|changes`, else "Season" of its season.
- **Tests** (`web/e2e/duties.e2e.ts`, new, Playwright project `duties` after `start`): a possible duty game "Not handled yet", "Referee 1, Jury 2" saved; an away game "Drivers: N (proposed)", a set number, "Use proposal"; a home game of the team; "Not saved. Try again." (`page.route`); accessibility, small phone.

### Step 5: "Season"
- **Goal**: the places on the rows, the label group "Duties", a tap on a game, "Back" with the labels.
- **Changes**: `EventRow.svelte` (the places line with `placesText`; a tap on a game opens its page; `from` for the return); `Labels.svelte` and `season.ts` (`Labels { what, when, duties }`, `Duties = 'handled' | 'not_handled' | 'open_places'`; `visible`: with a Duties label on, only games with `places`); `keepForReturn` / `takeReturn` with a key of a training or a game.
- **Tests** (browser): the rows show places; "Possible duty games" + "Not handled": after a save and "Back" the game left the list and the labels are still on; "Handled", "Open places"; a tap on a game opens its page. `season.e2e.ts` and `choose.e2e.ts` adapted (the group "Duties"; a game row is a link).

### Step 6: The start page
- **Goal**: the open item "not handled" with its list; "Next" and "Last" with games.
- **Changes**: `start.ts` (`openItemText` for `not_handled`; a list below the item for more than one game; `gameLink(season, id)` with `?from=start`); `+page.svelte` ("Last" with the training and the game; "Next" with places; the item list).
- **Tests** (browser): "N possible duty games not handled" opens its list, a tap opens a game; "Next" shows a duty game with places; "Last" shows a past game with places, a tap opens it. `start.e2e.ts`: the tests that read the open items with real data allow the new item.

### Step 7: The list of changes
- **Goal**: "has duties"; a tap on a change opens the game.
- **Changes**: `api/changes.rs` (`ChangeView.places: Option<PlacesView>`, from the same `game_places_of`); `/changes` ("has duties" with `placesText`; a tap opens the game with `?from=changes`).
- **Tests**: `tests/changes.rs` (a moved game with places has `places`; a game without has none); browser (the change shows "has duties" with its places; a tap opens the game). `just check`.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| One table `game_duties`; a row with 0 and 0 stays and means "handled"; the store never deletes a row | "Handled" is one stored fact | In the record at the close of story B |
| `game_places` is the only rule for the places; every page and answer uses it | One number everywhere (0022) | In the same record |
| The duties a game needs come from the team chosen for its own season; drivers only at another club | As "Season" (0022); the club's own hall needs no car | In the same record |
| The number of drivers stored only when the coordinator sets it; the proposal at most 10 | "Proposed, follows the team"; 10 is the most that can be set | In the same record |
| One save of all numbers (`PUT …/places`) under the row lock of the game | One form, one check; two screens wait for each other | No |
| The places travel on `ScheduleGame` and `ChangeView` | One request per page; "has duties" as on "Season" | No |
| One story of 7 steps, not A1 and A2 | A1 alone gives no use; A and B go out together (coordinator, 2026-10-25) | No |

## Not in scope
- Players on places, the choice, the totals, "Duties", the open item of open places, "Delete player" with duties (story B); the messages (C, D).

## Test plan
- **Existing tests that must still pass**: all core tests (the calls of `open_items` and `next_events` get empty places); `tests/season.rs` (events get `places`), `tests/start.rs`, `tests/changes.rs`, `tests/players.rs`, `tests/player_logs.rs`; browser `season.e2e.ts` (the group "Duties"), `start.e2e.ts` (the new open item in the 4 weeks), `choose.e2e.ts` (a game row is a link).
- **New tests**: per step, above.
- **Not testable in the browser** (clock fixed at 12 October 2026): July and August, a new season (core and server only).
- **Checks while working**: `cargo test -p team_core`; `cargo test -p team --test games --test season --test start --test changes`; `just check-web`, then the changed Playwright projects; `cargo clippy --workspace --all-targets -- -D warnings` at the end of each step; `just check` once before the commit.

## Compatibility and performance
- New table and routes; `ScheduleGame`, `ChangeView` and `StartAnswer` get new fields (the front-end types are generated).
- A tap on a game (on "Season", in "Next", "Last", the list of changes) opens its page; before it did nothing.
- `GET /api/start` reads the stored places of one or two seasons and the roles (already read for the trainings); measured in step 3.
- Released with story B (release 1 of the epic).

## Progress
- [ ] Step 1 – migration and core `duties`
- [ ] Step 2 – store, `duties_of`, and the routes of a game
- [ ] Step 3 – the places on the answers
- [ ] Step 4 – the page of a game
- [ ] Step 5 – "Season"
- [ ] Step 6 – the start page
- [ ] Step 7 – the list of changes
- [ ] All checks pass

## Deviations
None from `functional.md`. Readings of the spec that the design makes exact:
- The driver proposal is at most 10, the most that can be set (with 31 players or more).
- An away game of the team in the club's own hall (the home team is a club team) needs no drivers (coordinator, 2026-10-18, first design of this story).
- "Last game" is the newest past game with places, also when it left the game list later.
- One story of 7 steps instead of the split into A1 and A2 that the epic proposed for a story above 6 steps (coordinator, 2026-10-25).

## Review findings
