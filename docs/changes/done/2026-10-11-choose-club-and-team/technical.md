# Technical spec: Choose club and team

**Status**: done
**Size**: story
**Change**: docs/changes/done/2026-10-11-choose-club-and-team
**Date**: 2026-10-11

## Objective
Let the coordinator choose the club (by name, from the source's club list) and the team (from the
club's own teams in the source's game list), change it later and choose again at a new season,
and keep the choice for each season, so that stories 3–5 of the epic
[Games from the game list](../../2026-10-10-games-from-game-list/technical.md) read the right games.

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
- [x] Step 1: core rules
- [x] Step 2: reader of the source
- [x] Step 3: storage and API
- [x] Step 4: front end
- [x] All checks pass

## Deviations
From the epic's technical spec (none from the functional spec):
- The strict reading of a full game, and its tests (real data, 00:00, field named in a format error), move from story 3 to story 2 (coordinator, 2026-10-11).
- The epic test "a change of team removes unseen changes and records none" moves to story 4; story 2 stores `chosen_at` for it (coordinator, 2026-10-11).
- The epic outline's `selection` table is `followed_teams`, one row for each season (coordinator, 2026-10-11). The API keeps the name `/api/selection`.

From Implement:
- 2026-10-11, step 2: `Source::new` and `Source::with_timeout` also take the tool's own address (`ORIGIN`), because the spec puts it in the User-Agent; the spec's signature left the parameter out. The base address must be the root of the source's site (paths are joined to it).
- 2026-10-11, step 3: `ChoiceState::Chosen` holds `{ season, club, team }` with a team that is always there, instead of a `FollowedTeam` whose team may be empty. The API then has no branch for a case that cannot happen.
- 2026-10-11, step 3: `AppError` does not log source errors; the reader already logs them with the club and season.
- 2026-10-11, step 4: the browser test setup migrates the fresh database with `sqlx migrate run` (as the GitHub Check stage does), not with `cargo run -- migrate`: Playwright starts the test servers before the setup, so they compile from `.sqlx/` (`SQLX_OFFLINE`) and connect on first use.
- 2026-10-11, step 4: the front-end type check found that the OpenAPI description of step 3 put `search` of `GET /api/clubs` in the path instead of the query; fixed (`#[into_params(parameter_in = Query)]`).
- 2026-10-11, step 4: the start page has no "Hello, <name>" heading any more; the name stays in the header. When the club is already saved without a team, the start page says so and offers no second "Save club" (seen on the phone screenshot; the browser test covers it).
- 2026-10-11, review fixes: `choice_state` takes the choice of the current season and the newest earlier choice instead of the latest row and the season; `followed_teams` has `for_season` and `newest_before` instead of `latest`; the team rule is the core function `choose_team`; `last` in the API also holds the club.

The design reads the functional spec more strictly than written, within its words:
- "Each result shows the club name and town": 29 clubs have no town in the source; they show the name only.
- "The coordinator can save the club" in summer: saving a club without a team is allowed only when the season has no own teams.
- "Nothing is saved" when the source fails: also when the source answers with data the tool does not understand.

## Review findings

### Build review (2026-10-11)
Fresh agent; input: mode `build` and this change folder. Read: the diff against `main` (committed
and not), this folder, `docs/architecture.md`, the valid decision records, `CLAUDE.md`.
Ran: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --workspace` (all pass, 1 ignored
live test), `just check-web`, `just e2e` (15 passed), `just api-types-check`, `just check-sqlx`,
`just lint-ci`: all pass, so "All checks pass" is true. No test was found that cannot fail.

- **R1 – a product rule in the server (pattern).** `PUT /api/selection` decides in
  `api/selection.rs` that the team must be an own team (`not_an_own_team`) and that a team may be
  left out only when the season has no own teams (`team_required`). That is the rule of story
  item 8 as Design read it strictly, and `architecture.md` says new rules go in `crates/core`
  first. It is testable there without I/O (for example `fn choose(club, teams, team: Option<TeamNumber>) -> Result<Option<Team>, ChoiceError>`);
  now it is covered only by the router test. Checked: trace of `selection::save`.
  - 2026-10-11: fixed: the rule is the core function `choose_team(own_teams, team) -> Result<Option<Team>, TeamChoiceError>` with three unit tests; `api/selection.rs` only maps its errors to the 400 codes.
- **R2 – the new season in summer drops last season (bug against the intent).** On 1 August the
  new season usually has no games yet (item 8), so the new-season path is mostly this path:
  the start page shows "A new season started" and `ChooseTeam` with `clubSaved = false`
  (`+page.svelte` sets it only for `no_games_yet`), so the coordinator sees "Save club" for a
  club that is already kept. Pressing it saves a new-season row with no team; `latest` then
  returns that row and `choice_state` gives `NoGamesYet` with `last: None`. The message "a new
  season started" and last season's team disappear from the API, while story 3 needs exactly
  that link "until then" (item 6). The old row stays in the table, so no data is lost, but
  nothing reads it. Options: offer no "Save club" for `new_season` (the club is kept), or let
  the state also look at the newest row with a team. Checked: trace (`+page.svelte`,
  `ChooseTeam.svelte`, `followed_teams::latest`, `choice_state`); not run, because the browser
  test cannot set the server clock and no router test saves a club without a team in a new season.
  - 2026-10-11: fixed in two places. Core: `choice_state(this_season, earlier)` takes the row of the current season and the newest earlier row, and gives the earlier choice as `last` also in `no_games_yet`, so a save for an empty new season keeps last season's team (core test, and the router test `saving_the_club_for_an_empty_new_season_keeps_last_season`). Screen: in every `team_needed` state the club is kept, so the start page and `/choose` offer no "Save club" for it (`clubKept`).
- **R3 – sibling of the step-4 fix on `/choose`.** `choose/+page.svelte` passes `club` but not
  `clubSaved`, so in state `team_needed`/`no_games_yet` the page `/choose` offers "Save club"
  again – the screen problem the step-4 deviation fixed on `/`. Reachable with the browser's Back
  button right after "Save club" (it went `/choose` → `/`), or by typing the address. Harmless
  (the save is an upsert of the same row) but confusing. The two `+page.ts` loaders are also the
  same code twice. Checked: trace.
  - 2026-10-11: fixed: `/choose` passes `clubKept` in the state `team_needed`; the browser test opens `/choose` after the save and checks that there is no "Save club". The two `+page.ts` loaders call one function `loadPageSelection` in `client.ts`.
- **R4 – "choose the team again" when no team was chosen.** A club saved without a team in
  season N gives `NewSeason` in season N+1 with `last.team = null`; the page then says "choose
  the team again" with no last team. Small wording issue. Checked: trace of `choice_state` and
  `+page.svelte`.
  - 2026-10-11: fixed: with no last team the start page says "A new season started: choose the team."; "again" only when a last team exists. The last team shows with its own club (`last.club` added to the API).
- **R5 – a newest row later than the current season.** `choice_state` handles only
  `latest.season < current`. When the newest row is of a later season (the clock went back, or a
  wrong clock), it gives `Chosen` and the API labels it with the current season; a `PUT` then
  writes the current-season row, but `GET` keeps returning the later row, so the save seems
  ignored. Low risk (needs a wrong clock); no core test for it. Checked: trace.
  - 2026-10-11: fixed: storage reads the row of the current season (`for_season`) and the newest row before it (`newest_before`), never a later row; router test `a_choice_of_a_later_season_does_not_hide_the_current_season`.
- **R6 – browser tests share one database in parallel.** `logged-in.e2e.ts` and `choose.e2e.ts`
  are in the same project and run in parallel workers on `team_e2e` (seen in the run: 3 workers,
  interleaved). Nothing in `logged-in.e2e.ts` depends on the saved choice now (its comment says
  so), but a later start-page assertion there would be flaky. Consider `workers: 1` for that
  project or a note in `architecture.md`. Checked: run of `just e2e`.
  - 2026-10-11: fixed: the choose tests are their own Playwright project `choose`; the project `logged-in` depends on it, so it runs after them. The logged-out tests do not touch the choice.
- **R7 – new files are not in git yet.** The new sources, `migrations/0002_followed_teams.sql`,
  the two `.sqlx/query-*.json` files and the test data are untracked; `check-sqlx` and the tests
  pass locally only because they are on disk. Commit them all. Checked: `git status`.
  - 2026-10-11: not fixed yet: the files are committed when the coordinator asks for a commit (`just check` passes with them on disk).
- **R8 – documents that are now false or incomplete (for Close).** `architecture.md`: core
  "Now: the access rule `may_access`"; the server module list (no `source/`, `followed_teams.rs`,
  `current_season` in `clock.rs`); "AppError turns them into 401, 403, 404 or 500" (now also 400
  and 502); browser tests now use a fresh `team_e2e` database and a fake source. `CLAUDE.md`
  "Architecture" lists the server modules without `source/` and `followed_teams.rs`.
  `docs/product/` has no page for choosing the club and team. Checked: reading `architecture.md`,
  `CLAUDE.md` and `docs/product/`; search for `team_v2|team_e2e` in the repository.
  - 2026-10-11: not fixed here: the living documents are updated by `/close`.

Checked with no finding: decision 0001 (core uses `chrono` without the clock feature; no clock is
read in core), 0004 (new forward-only migration), 0007 (all four routes on the protected router;
401 test), `Origin` on `PUT` (403 test); no `unwrap`/`expect` outside tests (clippy); logs hold
club number, season and error kind only, the search text is not logged; the source is read
before the database is touched on `PUT`; names come from the source, not the browser.

### Accept review (2026-10-11)
Fresh agent; input: mode `accept` and this change folder (no other arguments were given). This is
a technical spec for story 2 of an epic, so the yardstick is story 1 of the epic's
`functional.md` and its `idea.md`, plus the objective here. Read: those, this folder, the diff
against `main` (committed and not). Note: this file was read in one piece, so **Deviations** and
the build review were seen before the own view was complete; each point below was checked
against the code or a run, not taken from them.
Ran: `just e2e` after `just web-build` (15 passed); the real server (debug, login bypass) on a
scratch database `team_accept_review` (dropped afterwards) with `web/e2e/fake-source.mjs`,
rows put in by hand to act as last season, and `curl` on the API (runs (1)–(3) below).

| # | Criterion or open point | Built | Sort |
|---|---|---|---|
| A1 | Before a choice, only this step | Start page shows only the search; no sample link. `/sample` still opens by address (temporary page, story 3 removes it). Checked: e2e "first visit", reading `+page.svelte`. | faithful adaptation |
| A2 | Search by part of the name; result shows name and town | Yes; the 29 clubs with no town show the name only. Checked: e2e, run (search "giants"). | faithful adaptation |
| A3 | The club's own teams, not opponents; the example | Exactly M16 1, M16 2, M18 1, MSE 1, X12 1, X14 1. Buttons show the short name, as the spec's own example does (not the "short names" kept out of scope, which are for drives). Checked: e2e. | faithful adaptation |
| A4 | No club found, search again | Yes. Checked: e2e. | matches |
| A5 | Change club or team later | "Change" on the start page; `/choose` opens at the club's teams, "Other club" goes back to the search. The current team is not marked in the list. Checked: e2e "changes the team later", reading `ChooseTeam.svelte`. | neutral drift (the unmarked current team) |
| A6 | After a change: new games with no changes, old unseen changes removed | Not in this story (moved to story 4 by the coordinator); `chosen_at` is stored for it. Checked: reading the migration and `followed_teams.rs`. | faithful adaptation |
| A7 | New season: keep the club, ask the team again, show last season's games and say a new choice is needed until then | **When the new season already has games**: works. Run (1): last-season row Blue Drakes M16 1 gives `team_needed`, `new_season`, `last` 2025-2026 M16 1. **When the new season is still empty, which is the normal case on 1 August** (item 8 of the spec says so): the start page passes `clubSaved` only for `no_games_yet`, so it shows "has no games yet. Save the club now" and a "Save club" button for a club that is already kept. Run (2), club 1067 with a 2025-2026 row: `GET` gives `new_season` with last season's team; the `PUT` that "Save club" sends returns `no_games_yet`, `last: null`, and every later `GET` the same. "A new season started" and last season's team are gone from the screen and the API, so story 3 has no way to "show the games of the last season until then" from the selection. The 2025-2026 row stays in the table (no data lost). Same cause as R2. | **concerning deviation** |
| A8 | Source cannot be reached: say so, try again, nothing saved | Message "The game list cannot be reached. Please try again." with "Try again", which repeats the last action. Run (3): source stopped, `PUT` gives 502 `source_unavailable`, `GET` unchanged. Also for data the tool does not understand. | matches (format case: faithful adaptation) |
| A9 | Summer: club found, no games; save the club, team later | Message, "Save club", then the start page says the club is saved and offers the teams when games appear. Checked: e2e. On `/choose` the same state still offers "Save club" again (R3); harmless upsert. | matches; `/choose` sibling: neutral drift |
| A10 | Seasons kept (coordinator, 2026-10-11) | Run (2): after the save the table holds 2025-2026 and 2026-2027 rows for the club, the old one unchanged. | matches |
| A11 | Open point: saving without a team | Allowed only when the season has no own teams. Within the spec's words. | faithful adaptation |
| A12 | Open point: reading politely | While a team is needed, each opening of the start page reads the club list and the club's season from the source (the team list loads on mount). With weekly use this is a few requests; worth knowing for story 5. Checked: reading `ChooseTeam.svelte` `onMount`. | neutral drift |
| A13 | Wording when no team was ever chosen | A club saved without a team in season N says "choose the team again" in season N+1 with no last team (R4). Checked: reading `+page.svelte`. | neutral drift |

**Verdict: needs correction**, for A7 only. The spec is clear here ("keeps the club", "until the
coordinator chooses … says that a new choice is needed"), so the correction is in the build, not
in Discover: in the `new_season` state with an empty season, do not offer "Save club" (the club
is already kept; say it has no games for the new season yet and that the team is chosen when
games appear), or otherwise keep `new_season` and last season's team until a team is chosen.
A test should cover the empty-new-season case (no router or browser test does now). After that
fix the result is **accepted with notes** (A5, A9, A12, A13).

Answers to the accept review (2026-10-11):
- A7: fixed with R2 (core keeps `last` after a save for an empty new season; no "Save club" for a kept club; router test for the empty new season).
- A9: fixed with R3.
- A13: fixed with R4.
- A5: not fixed: marking the current team in the list is not asked for by the spec.
- A12: not fixed: while a team is needed, each visit reads the source twice; with weekly use these are a few requests. Noted for story 5, which adds the nightly reading.
