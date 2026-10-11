# Technical spec: Choose club and team

**Status**: approved
**Size**: story
**Change**: docs/changes/2026-10-11-choose-club-and-team
**Date**: 2026-10-11

## Objective
Let the coordinator choose the club (by name, from the source's club list) and the team (from the
club's own teams in the source's game list), change it later and choose again at a new season,
and keep the choice for each season, so that stories 3–5 of the epic
[Games from the game list](../2026-10-10-games-from-game-list/technical.md) read the right games.

## Requirements summary
From story 1 of the epic's `functional.md` (approved after two spec reviews):

1. Before a club and team are chosen, the tool shows only the choose step.
2. The coordinator searches the club by part of its name and chooses it from the results. Each result shows the club name and town.
3. The tool reads the club's games of the current season and shows the club's own teams (not the opponents). The coordinator chooses one. Example: "drakes" finds "Blue Drakes, Woerden"; its teams are M16 1, M16 2, M18 1, MSE 1, X12 1, X14 1.
4. No club found: the tool says so, and the coordinator can search again.
5. The coordinator can change the club or the team later.
6. At the start of a new season, the tool keeps the club and asks for the team again (a youth team moves up an age group). Until then it says that a new choice is needed. (Showing last season's games in that time is story 3, which has the games.)
7. The source cannot be reached: the tool says so, the coordinator can try again, nothing is saved.
8. In summer the new season can be empty: the tool says it found the club but no games for this season yet. The coordinator can save the club without a team, and chooses the team when games appear.

Also from the functional spec, for later stories: after a change of club or team, the games of
the new choice come in with no changes recorded, and unseen changes of the old choice are removed.
Story 2 stores no games or changes; it stores the time of the choice, which stories 3 and 4 use.

From the coordinator (2026-10-11): **games and choices of more than one season must be kept**,
for example 2025-2026 and 2026-2027. A coordinator may later manage more than one team; that is
not built now (see **Not in scope**), but the data must not block it.

Decisions that apply: 0001 (pure core), 0002 (Svelte static front end, generated API types), 0004
(PostgreSQL, sqlx, forward-only migrations), 0007 (access by Pocket ID group; every `/api` route
needs a session).

## Current state
- `crates/core` holds only `access.rs` (`may_access`). No dependencies.
- `crates/server`: `config.rs` (settings through `Config::from_lookup`), `app.rs` (`AppState { pool, config, oidc, clock, web }`, the protected `/api` router), `api/` (`health`, `me`, `ApiDoc` for OpenAPI), `error.rs` (`AppError`: `Unauthenticated`, `Internal`), `clock.rs` (`Clock` trait; tests use `TestClock` in `tests/common/mod.rs`). `reqwest` (rustls, json) and `chrono` are already dependencies.
- `migrations/0001_sessions.sql` is the only migration.
- `web/`: the start page `src/routes/+page.svelte` greets the user and links to the sample schedule `/sample` (temporary, replaced in story 3). Browser tests (`web/e2e/`) run the real server with the login bypass against the shared database `team_v2`.
- The deployment does not change in this story.

### The source (checked on 2026-10-11, one request each)
- **Club list**: `GET https://db.basketball.nl/db/json/club.pl` gives `{ clubs: [...], last_change, version }`. 316 clubs. Per club: `id` (number), `naam` (text, never empty), `plaats` (town: text, empty or `null`; 29 clubs have no town). Blue Drakes is `id` 37, `plaats` Woerden.
- **Club games**: `GET https://db.basketball.nl/db/json/wedstrijd.pl?seizoen=2026-2027&clb_ID=37` gives `{ wedstrijden: [...], aantal_wedstrijden, seizoen, ... }`. 71 games this season, 118 last season (2025-2026).
- Per game, the fields the tool uses (all present in all 189 games):

  | Field | Type | Example | Meaning |
  |---|---|---|---|
  | `id` | number | 1073597 | the source's own game id |
  | `nr` | text | "499858" | the game number (probably the federation's) |
  | `seizoen` | text | "2026-2027" | season |
  | `datum` | text | "2026-09-12 11:15:00.000" | date and time, Dutch local time |
  | `tijd` | text | "11:15" | start time; "00:00" means not known (2 games this season) |
  | `thuis_ploeg_id`, `uit_ploeg_id` | number | 184 | home / away team number |
  | `thuis_ploeg`, `uit_ploeg` | text | "Blue Drakes M16 1" | home / away team name, with the club name in front |
  | `thuis_club_id`, `uit_club_id` | number | 37 | home / away club number |
  | `loc_id` | number | 43 | place number |
  | `loc_naam`, `loc_plaats` | text | "Korf De", "Leusden" | hall name, town |
  | `veld` | text | "Veld 3" or "" | court; empty in 2 games |
  | `cmp_naam` | text | "Mannen U16 Regionaal …" | competition |

- Not reliable: `thuis_club` / `uit_club` (for club 24 they hold team names such as "Utrecht Basketball X12-1"). The tool takes club names from the club list only.
- `aantal_wedstrijden` is the number `118` for a full season and the text `"0"` for an empty one. The tool does not read it.
- An unknown club number gives HTTP 200 with an empty `wedstrijden`, the same as an empty season. The tool checks the club in the club list first.
- Team numbers stay the same between seasons (M16 1 is 184 in both seasons), so a team is always stored with its season.

## Steps

### Step 1: Core rules
- **Goal**: the rules of this story, testable without I/O.
- **Changes**: `crates/core` gets `chrono` (`default-features = false`; it reads no clock, so decision 0001 holds). New modules, exported from `lib.rs`:

  ```rust
  // season.rs
  pub struct Season { start_year: i32 }                // Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug
  impl Season {
      pub fn of(date: NaiveDate) -> Season;            // on or after 1 August: start_year = year, else year - 1
      pub fn start_year(self) -> i32;
  }
  // Display: "2026-2027". FromStr: only "YYYY-YYYY" with the second year one more; else SeasonError (thiserror).

  // club.rs
  pub struct ClubNumber(pub i64);
  pub struct Club { pub number: ClubNumber, pub name: String, pub town: Option<String> }
  pub fn search_clubs(clubs: &[Club], query: &str) -> Vec<Club>;
  // The query is trimmed; empty gives []. Match: the name contains the query, upper/lower case not
  // important. Sorted by name.

  // game.rs
  pub struct TeamNumber(pub i64);
  pub struct PlaceNumber(pub i64);
  pub struct Side  { pub team: TeamNumber, pub team_name: String, pub club: ClubNumber }
  pub struct Place { pub number: PlaceNumber, pub hall: String, pub town: String, pub court: Option<String> }
  pub struct Game {
      pub source_id: i64, pub game_number: String, pub season: Season,
      pub date: NaiveDate, pub start_time: Option<NaiveTime>,   // Dutch local time; None = not known
      pub home: Side, pub away: Side, pub place: Place, pub competition: String,
  }
  pub fn start_time_from_source(time: NaiveTime) -> Option<NaiveTime>;   // 00:00 gives None

  // team_choice.rs
  pub struct Team { pub number: TeamNumber, pub name: String }           // short name, e.g. "M16 1"
  pub fn own_teams(club: &Club, games: &[Game]) -> Vec<Team>;
  pub struct FollowedTeam { pub season: Season, pub club: Club, pub team: Option<Team> }
  pub enum ChoiceState {
      NotChosen,
      TeamNeeded { club: Club, reason: TeamNeededReason, last: Option<FollowedTeam> },
      Chosen(FollowedTeam),
  }
  pub enum TeamNeededReason { NoGamesYet, NewSeason }
  pub fn choice_state(latest: Option<FollowedTeam>, current: Season) -> ChoiceState;
  ```

  Rules:
  - **Own team**: a side of a game whose club number is the chosen club. Each team number once, sorted by name.
  - **Short name**: remove the club name (from the club list) from the front of the team name, then a following " - " or space. If nothing is left, keep the full name. If one team number has two names in the season, take the name of its latest game (date, then start time).
  - **State**: no row gives `NotChosen`. Latest row of an older season than `current` gives `TeamNeeded { reason: NewSeason, club: <its club>, last: Some(<that row>) }`. Latest row of the current season with no team gives `TeamNeeded { reason: NoGamesYet, last: None }`. Otherwise `Chosen`.
- **Tests** (unit tests in core):
  - `Season::of`: 31 July 2027 gives 2026-2027; 1 August 2027 gives 2027-2028. "2026-2027" shows and reads back; "2026-2028" and "2026" are refused.
  - `search_clubs`: "drakes", "DRAKES" and " drakes " find Blue Drakes; "zzz" gives []; "" gives []; results sorted by name.
  - `start_time_from_source`: 00:00 gives None; 11:15 stays.
  - `own_teams` on small hand-made games: opponents left out; a team number once; "Blue Drakes M16 1" gives "M16 1"; "Utrecht Basketball - MSE-3" gives "MSE-3"; a name that is only the club name stays full; two names give the name of the latest game.
  - `choice_state`: the four cases above (not chosen; chosen; no games yet; new season keeps the club and gives last season's team).

### Step 2: Reader of the source
- **Goal**: read the club list and the club's games strictly into the core types.
- **Changes**:
  - New module `crates/server/src/source/` (exported from `lib.rs`):

    ```rust
    pub struct Source { client: reqwest::Client, base: Url }
    impl Source {
        pub fn new(base: Url) -> Result<Self, SourceError>;                    // timeout 10 s
        pub fn with_timeout(base: Url, timeout: Duration) -> Result<Self, SourceError>;   // for tests
        pub async fn clubs(&self) -> Result<Vec<Club>, SourceError>;            // {base}/db/json/club.pl
        pub async fn club_games(&self, club: ClubNumber, season: Season)
            -> Result<Vec<Game>, SourceError>;                                  // {base}/db/json/wedstrijd.pl?seizoen=<season>&clb_ID=<club>
    }
    #[derive(Debug, thiserror::Error)]
    pub enum SourceError {
        #[error("game list cannot be reached")]          Unreachable(#[source] reqwest::Error),   // connection, timeout
        #[error("game list answered {0}")]               Status(reqwest::StatusCode),
        #[error("game list format: {place}: {reason}")]  Format { place: String, reason: String },
        #[error("game list client: {0}")]                Client(#[source] reqwest::Error),        // building the client
    }
    ```
  - **Strict parsing, field by field**: parse the body into `serde_json::Value`, then take each field with small helpers (`number(obj, "loc_id")`, `text(obj, "tijd")`, ...) that return `Format { place, reason }`. `place` names the game and the field, for example `game 1073597, field tijd`, or `club 37, field naam`. Fields the tool does not use (scores, `uitslag`, logos, `lat`/`lon`, `aantal_wedstrijden`) are not read, so they cannot make a reading fail and "0 - 0" can never become a result.
  - Game: the fields in the table under **Current state**. `datum`: the date part (`YYYY-MM-DD`) of the text. `tijd`: `HH:MM`, through `start_time_from_source`. `seizoen` must equal the season that was asked for, else `Format`. `veld` "" gives None. `wedstrijden` must be a list, else `Format`.
  - Club: `id` number, `naam` non-empty text, `plaats` text, or `null` or "" for None. `clubs` must be a list.
  - Header `User-Agent: team-coordinator/<package version> (+<ORIGIN>)`.
  - Logs: club number, season and error kind only.
  - Setting `GAME_LIST_URL` in `Config` (optional, default `https://db.basketball.nl`, must be a URL). Add it to `.env.example` and to the settings list in `README.md` (not a cluster setting: the cluster uses the default).
  - Test data in `crates/server/tests/data/game-list/`, all public data read from the source on 2026-10-11, with a `README.md` that says where each file comes from and when: `clubs.json` (316 clubs), `games-37-2026-2027.json` (71 games), `games-37-2025-2026.json` (118 games), `games-empty.json` (empty season with `"aantal_wedstrijden":"0"`).
- **Tests** (integration tests with wiremock and the real files, `crates/server/tests/source.rs`):
  - The club list gives 316 clubs; Blue Drakes is 37 in Woerden; a club with `plaats` null or "" has no town.
  - 2026-2027 of club 37 gives 71 games; `own_teams` gives M16 1, M16 2, M18 1, MSE 1, X12 1, X14 1; the games on 16 January and 13 February 2027 have no start time; an empty `veld` gives no court.
  - 2025-2026 gives 118 games; its own teams have VSE 1 and no M16 2.
  - The empty season gives no games and no error.
  - Each request sends the User-Agent, `seizoen` and `clb_ID`.
  - Format errors, each naming game and field: `tijd` missing; `tijd` "abc"; `thuis_ploeg_id` a text; `seizoen` another season; `wedstrijden` not a list; a club `naam` empty.
  - HTTP 500 gives `Status`; an answer slower than the timeout (`with_timeout`) gives `Unreachable`.
  - Config: `GAME_LIST_URL` default, and a value that is not a URL is refused.
  - `#[ignore]`: the live source gives the club list and this season of club 37 (run by hand: `cargo test -- --ignored`).

### Step 3: Storage and API
- **Goal**: save the choice for each season and offer it through the API.
- **Changes**:
  - Migration `migrations/0002_followed_teams.sql`:

    ```sql
    -- The chosen club and team, one row for each season. Rows of earlier seasons stay.
    CREATE TABLE followed_teams (
        id           bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
        season       text NOT NULL UNIQUE,          -- '2026-2027'; one team for each season for now
        club_number  bigint NOT NULL,
        club_name    text NOT NULL,
        club_town    text,
        team_number  bigint,
        team_name    text,
        chosen_at    timestamptz NOT NULL,
        CHECK ((team_number IS NULL) = (team_name IS NULL))
    );
    ```
    The current choice is the row with the newest season. For more teams later, `UNIQUE (season)` is removed and a link to the coordinator is added; existing rows stay valid.
  - New module `crates/server/src/followed_teams.rs`: `latest(pool) -> Result<Option<FollowedTeam>, sqlx::Error>` (newest season) and `save(pool, &FollowedTeam, chosen_at)`: `INSERT … ON CONFLICT (season) DO UPDATE` of club, team and `chosen_at`. One statement on one row: two saves at the same time need no lock; the last one wins. A row of another season is never changed.
  - Season of today: `today_in_netherlands(&dyn Clock) -> NaiveDate` with `chrono-tz` (`Europe/Amsterdam`, new dependency of the server), then `Season::of`.
  - `AppState` gets `source: Arc<Source>`; `main.rs` builds it from `GAME_LIST_URL`; the test harness points it at wiremock.
  - `AppError` gets `BadRequest(&'static str)` (400, `{"error": "<reason>"}`), `NotFound` (404), `SourceUnavailable` (502, `{"error":"source_unavailable"}`, for `Unreachable` and `Status`) and `SourceFormat` (502, `{"error":"source_format"}`). The details go to the log only.
  - Routes on the protected `/api` router, described with `utoipa` and added to `ApiDoc` (new module `api/selection.rs`, `api/clubs.rs`):

    | Route | Answer | Errors |
    |---|---|---|
    | `GET /api/selection` | `Selection` (below) | – |
    | `GET /api/clubs?search=<text>` | `[{ number, name, town? }]` | 502 |
    | `GET /api/clubs/{number}/teams` | `{ season, teams: [{ number, name }] }`; empty `teams` means "club found, no games yet" | 404 when the club is not in the club list; 502 |
    | `PUT /api/selection` with `{ club, team? }` | `Selection` after the save | 400 `unknown_club`, `not_an_own_team`, `team_required` (no team while own teams exist); 502; nothing is saved on an error |

    `Selection` (a tagged union on `state`):
    - `{ state: "not_chosen", season }`
    - `{ state: "team_needed", season, club, reason: "no_games_yet" | "new_season", last?: { season, team } }`
    - `{ state: "chosen", season, club, team }`

    `season` is the current season ("2026-2027"). On `PUT`, the server reads the club list (and the club's games when a team is given) again and takes all names from the source, never from the browser. The source is read before the database is touched, so a slow source holds no database connection.
  - `just api-types` (OpenAPI and `web/src/lib/api/schema.d.ts`), `just prepare` (`.sqlx/`).
- **Tests** (router tests with `#[sqlx::test]`, the fake source on wiremock and `TestClock`, `crates/server/tests/team_choice.rs`):
  - Without login, all four routes give 401.
  - No row: `GET /api/selection` gives `not_chosen` with the season of the test clock.
  - Search "drakes" gives `[{ number: 37, name: "Blue Drakes", town: "Woerden" }]`; "zzz" gives `[]`.
  - Teams of club 37: six teams; club 999999: 404; a club with the empty season: `teams: []`.
  - Save 37 + 184 gives `chosen`, Blue Drakes M16 1; a later `GET` gives the same. Team 1688 (an opponent): 400 `not_an_own_team`. No team while teams exist: 400 `team_required`. A club with no games and no team: `team_needed`, `no_games_yet`. No `Origin` header: 403.
  - Source down: save gives 502 `source_unavailable`, and `GET` still gives `not_chosen`. A format error: 502 `source_format`, nothing saved.
  - A change in the same season (clock moved on): still one row, with the new team and the new `chosen_at`.
  - **New season, seasons kept**: save M16 1 at 31 July 2027 21:59 UTC (23:59 Dutch time, season 2026-2027); move the clock to 22:00 UTC (1 August 00:00 Dutch time); `GET` gives `team_needed`, `new_season`, club Blue Drakes, `last` 2026-2027 M16 1; save M18 1 gives `chosen` 2027-2028; the table has two rows and the 2026-2027 row is unchanged.

### Step 4: Front end
- **Goal**: the screens for choosing, changing and choosing again.
- **Changes**:
  - **Browser test setup first** (it also changes how the tests run on GitHub):
    - `web/e2e/fake-source.mjs`: a small server on `node:http` with no new libraries. It serves the club list and club 37's games from `crates/server/tests/data/game-list/`, with `seizoen` set to the season that was asked for, so the tests do not depend on the date. It gives the empty season for one other club. `GAME_LIST_URL` of both test servers points to it.
    - A Playwright `globalSetup` makes a fresh database `team_e2e` (`sqlx database drop -y`, `sqlx database create`, then `cargo run -- migrate`), so a choice saved in an earlier run cannot change the next run. sqlx-cli is in the Nix shell and in the GitHub Check stage.
  - Start page `/` (logged in): `not_chosen` shows only the choose step. `team_needed` shows the club, the message ("A new season started: choose the team again." / "Blue Drakes has no games for 2026-2027 yet.") and the team choice. `chosen` shows club and team (for example "Blue Drakes M16 1") and "Change". The link to the sample schedule shows only when a team is chosen (the sample page goes away in story 3).
  - New page `/choose`: a search field with a search button (the search runs on submit, not at each key); results as a list of buttons with name and town (name only when there is no town); "No club found" with the field still open; then the club's teams as buttons; for a club with no games, the message and "Save club". Errors: "The game list cannot be reached. Please try again." and "The game list has data the tool does not understand." with a "Try again" action.
  - The existing test "after login, the start page shows the name and a logout action" changes: the name is in the header and the choose step is shown.
- **Tests** (Playwright, phone-sized screen, one file that runs in order on the fresh database):
  - First visit: only the choose step, no sample link; accessibility check passes; no horizontal scrolling at 320 px.
  - "zzz": "No club found", and a second search works.
  - Search "drakes", choose "Blue Drakes, Woerden", six teams shown, choose M16 1: the start page shows "Blue Drakes M16 1" and "Change".
  - Change to M16 2: the start page shows M16 2.
  - The club with no games: the message, and "Save club" saves it; the start page shows that a team is needed.
  - The new season and source errors are covered by step 3 (the browser's clock and failures cannot be set in the same way).

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| One row for each season in `followed_teams`; the current choice is the newest season | The coordinator wants seasons kept; more teams later needs no rewrite | no (design detail; Close may add it to `architecture.md`) |
| Strict reading of the full game in story 2, not only the team fields | The coordinator wants date, time, both teams and place known; one request already gives them | no (epic deviation) |
| Club names from the club list, never from a game | The game's club name field holds team names for some clubs | no |
| Club check in the club list before reading games | An unknown club number gives an empty answer, the same as an empty season | no |
| Search on the server, at each submit, with no store in memory | The epic's API outline; a few searches per season are polite enough | no |
| Source address as setting `GAME_LIST_URL`, default `https://db.basketball.nl` | The epic chose this address; the setting allows `www.basketballstats.nl` if needed and a fake source in tests | no |
| Fresh database for the browser tests | Saved choices would make the tests depend on earlier runs | no |
| Closure `review build` instead of the expected `checks` | The strict reader moved here from story 3, and the browser test setup changes | no |

## Not in scope
- Storing games, and places as their own table (story 3; see the epic's input for story 3).
- Showing last season's games until a new team is chosen (story 3).
- Removing unseen changes after a change of choice, and recording nothing on the first reading (story 4, using `chosen_at`).
- More than one team, or more than one coordinator (`docs/ideas.md`, for `/vision`).
- Reading an old season again on request.
- A store in memory for the club list.
- A check that the chosen team is still in the source during the season.
- Any change to the cluster.

## Test plan
- **Existing tests that must still pass**: all Rust tests of story 1 (login, sessions, health, web, logs, the migration twice), the browser tests for logging out and the sample page, and the changed start page test.
- **New tests**: per step, above. Order of value: core rules, reader with real data, the new-season test that keeps both seasons, router errors, browser flow.
- **By hand**: `cargo test -- --ignored` reads the live source.

## Compatibility and performance
- New table only; no existing data changes. The migration runs as an init container as before.
- No new cluster setting; `GAME_LIST_URL` uses its default.
- Load: one search reads about 90 KB from the source; a team list or a save reads the club list and one season (about 75–125 KB). A few times per season.

## Progress
- [ ] Step 1: core rules
- [ ] Step 2: reader of the source
- [ ] Step 3: storage and API
- [ ] Step 4: front end
- [ ] All checks pass

## Deviations
From the epic's technical spec (none from the functional spec):
- The strict reading of a full game, and its tests (real data, 00:00, field named in a format error), move from story 3 to story 2 (coordinator, 2026-10-11).
- The epic test "a change of team removes unseen changes and records none" moves to story 4; story 2 stores `chosen_at` for it (coordinator, 2026-10-11).
- The epic outline's `selection` table is `followed_teams`, one row for each season (coordinator, 2026-10-11). The API keeps the name `/api/selection`.

The design reads the functional spec more strictly than written, within its words:
- "Each result shows the club name and town": 29 clubs have no town in the source; they show the name only.
- "The coordinator can save the club" in summer: saving a club without a team is allowed only when the season has no own teams.
- "Nothing is saved" when the source fails: also when the source answers with data the tool does not understand.

## Review findings
