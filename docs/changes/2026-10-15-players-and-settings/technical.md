# Technical spec: Players and settings

**Status**: in progress
**Size**: story
**Change**: docs/changes/2026-10-15-players-and-settings
**Date**: 2026-10-15

## Objective
Let the coordinator keep the players with their roles in the season, the training days with their
times and dates, and the reasons for an absence, so that story B can mark attendance.

## Requirements summary
Story A of the epic [Attendance at training](../2026-10-14-attendance-at-training/technical.md),
from functional stories 1, 2 (settings part) and 4:

1. **Players.** A player is the person: a first name (required) and a last name (optional), with no dates. Names are trimmed; an empty last name means no last name. Two players with the same first and last name are allowed, with a warning "There is already a player with this name." A player can be deleted (story B adds: only without absences).
2. **Roles.** A role is "plays games" or "only trains" in the team of one season (1 July – 30 June), from a date and until a date (empty while it continues). The proposed start is the first training date of the season, or today when trainings have started (or today when the season has no settings). Roles of one player never overlap. "Left the team" sets the end (proposed: today); "Undo" clears it. A player who comes back gets a new role. A change of role within a season (one role ends, another starts the same day) is allowed by the data; no screen for it now.
3. **Players page.** For the current season: the players with a role today (sorted by first name, with "Plays games" or "Only trains"), "Left the team" (a role earlier in the season, none today), and in a new season "Players of last season" with "Add to this season" for each and "Add all" (same role as at the end of last season, from the first training date, or today when later).
4. **Training settings** (current season): one or more days of the week, each with a start and an end time (for example Tuesday 19:45–21:00); the first and the last training date, proposed as the first chosen training day in September and the last in May. A change of days applies from the day it is made; earlier dates keep the days set then.
5. **Reasons.** Presets in this order: School, Ill, Injured (valid), Holiday, Work, No reason (not valid). Add (last), rename, valid / not valid, hide (still listed with "hidden"). Names unique without case and outer spaces.
6. **Navigation.** The start page shows "Players · Settings" below the team header, also before a team is chosen.
7. Only the coordinator (login, decision 0007). Logs never contain player names.

Decisions that apply: 0001, 0002, 0004, 0007, 0014; the epic's architecture (players are not linked to the game list choice; attendance stored as absences only, from story B).

## Current state
- Core: `Season` (`of`, `start_year`, `from_start_year`, `Display` "2026-2027"); validation errors as `thiserror` enums; no players or trainings.
- Server: `clock::today_in_netherlands`, `clock::current_season`; `db.rs` (`MIGRATOR`, `StoreError`); `AppError::BadRequest(&'static str)` gives `{"error": "<code>"}`; protected `/api` router in `app.rs`; OpenAPI paths listed in `api/mod.rs`. The schema name `Reason` is taken (`api/selection.rs`).
- Migrations `0001`–`0004`.
- Front end: start page `web/src/routes/+page.svelte` (team header, "Game list" card, schedule); typed client and `messageFor` in `web/src/lib/api/client.ts`; shadcn-svelte parts `button`, `badge`, `separator`, `toggle`, `toggle-group`.
- Tests: core unit tests; `#[sqlx::test(migrator = "team::db::MIGRATOR")]`; `crates/server/tests/common/` (`Harness` with `clock.set`, `log_in`, `send`; `LogBuffer` for log tests); browser tests in `web/e2e/` (projects `choose` then `logged-in`, server clock `DEV_NOW=2026-10-12T10:00:00Z`, fresh database each run).

## Steps

### Step 1: Core rules
- **Goal**: the rules for names, roles, training days and reasons, without I/O.
- **Changes**: new modules in `crates/core`, exported from `lib.rs`:

  ```rust
  // players.rs
  pub struct PlayerId(pub i64);
  pub struct Player { pub id: PlayerId, pub first_name: String, pub last_name: Option<String> }
  pub enum Role { PlaysGames, OnlyTrains }
  pub struct RoleId(pub i64);
  pub struct PlayerRole { pub id: RoleId, pub player: PlayerId, pub season: Season, pub role: Role,
                          pub from: NaiveDate, pub until: Option<NaiveDate> }
  pub fn clean_name(first: &str, last: Option<&str>) -> Result<(String, Option<String>), PlayerError>;
  pub fn role_on(roles: &[PlayerRole], player: PlayerId, date: NaiveDate) -> Option<&PlayerRole>;
  /// until ≥ from; from and until inside the season; no overlap with `others` (same player).
  pub fn check_role(season: Season, role: Role, from: NaiveDate, until: Option<NaiveDate>,
                    others: &[PlayerRole]) -> Result<(), PlayerError>;
  pub enum PlayerError { FirstNameRequired, UntilBeforeFrom, OutsideSeason, RolesOverlap }

  // training.rs
  pub struct TrainingDay { pub weekday: Weekday, pub start: NaiveTime, pub end: NaiveTime }
  pub struct TrainingDaySet { pub valid_from: NaiveDate, pub days: Vec<TrainingDay> }
  pub struct TrainingSeason { pub season: Season, pub first: NaiveDate, pub last: NaiveDate,
                              pub sets: Vec<TrainingDaySet> }
  pub fn proposed_dates(season: Season, weekdays: &[Weekday]) -> Option<(NaiveDate, NaiveDate)>;
  pub fn training_dates(season: &TrainingSeason) -> Vec<(NaiveDate, TrainingDay)>;
  /// Checks a whole settings save (dates in the season, last ≥ first, days valid) and gives the
  /// set to store: valid from today, or from `first` when today is before it; a set with the
  /// same valid-from date is replaced.
  pub fn settings_change(season: Season, first: NaiveDate, last: NaiveDate, days: Vec<TrainingDay>,
                         today: NaiveDate) -> Result<TrainingDaySet, TrainingError>;
  pub enum TrainingError { EndBeforeStart, LastBeforeFirst, NoTrainingDays, DuplicateWeekday, OutsideSeason }

  // reasons.rs
  pub fn clean_reason_name(name: &str, others: &[&str]) -> Result<String, ReasonError>;
  pub enum ReasonError { NameRequired, NameTaken }
  ```

  `proposed_dates`: the first chosen weekday on or after 1 September of the season's start year, and the last chosen weekday on or before 31 May of the next year. `training_dates`: every date from `first` to `last` whose weekday is in the set valid on that date (the set with the newest `valid_from` on or before the date; dates before the first set use the first set).
- **Tests** (unit tests, fixed dates):
  - `clean_name`: `"  Avery "` → `"Avery"`; empty or spaces → `FirstNameRequired`; last name `""` or `"  "` → `None`.
  - `role_on`, role from 1 Sep 2026 with no end: 31 Aug → none, 1 Sep and 12 Oct → the role. Role ended 15 Nov: 15 Nov → the role, 16 Nov → none. Roles 1 Sep–15 Nov and from 10 Jan 2027: 1 Dec → none, 10 Jan → the second.
  - `check_role`: until before from → `UntilBeforeFrom`; until 30 Jun 2027 → ok; until 1 Jul 2027 or from 30 Jun 2026 → `OutsideSeason`; second role from 15 Nov when the first ends 15 Nov → `RolesOverlap`; from 16 Nov → ok; an open role (no end) and a later role → `RolesOverlap`.
  - `proposed_dates`: Tuesday in 2026-2027 → Tue 1 Sep 2026, Tue 25 May 2027; Tuesday and Thursday → Tue 1 Sep 2026, Thu 27 May 2027.
  - `training_dates`: Tuesdays 1 Sep 2026 – 25 May 2027 → 39 dates, each 19:45–21:00. With a Thursday set valid from 5 Jan 2027: Tuesdays up to 29 Dec 2026, Thursdays from 7 Jan 2027.
  - `settings_change`: on 5 Jan 2027 → valid from 5 Jan; on 15 Aug 2026 (before the first date 1 Sep) → valid from 1 Sep; errors `EndBeforeStart`, `LastBeforeFirst`, `NoTrainingDays`, `DuplicateWeekday` (Tuesday twice), `OutsideSeason` (first date in 2025-2026).
  - `clean_reason_name`: `" Family "` → `"Family"`; `"school"` with "School" in `others` → `NameTaken`; empty → `NameRequired`.

### Step 2: Migration and storage
- **Goal**: the tables and the functions that read and write them.
- **Changes**:
  - `migrations/0005_players_and_settings.sql`:
    - `players (id bigint identity PK, first_name text NOT NULL CHECK (btrim(first_name) <> ''), last_name text, created_at timestamptz NOT NULL)`;
    - `player_roles (id bigint identity PK, player_id bigint NOT NULL REFERENCES players ON DELETE CASCADE, season_year integer NOT NULL, role text NOT NULL CHECK (role IN ('plays_games','only_trains')), from_date date NOT NULL, until_date date, CHECK (until_date IS NULL OR until_date >= from_date))`, index on `(player_id)` and `(season_year)`;
    - `reasons (id bigint identity PK, name text NOT NULL, valid boolean NOT NULL, hidden boolean NOT NULL DEFAULT false, position integer NOT NULL)`, unique index on `lower(btrim(name))`; the 6 presets inserted with positions 1–6;
    - `training_seasons (season_year integer PK, first_date date NOT NULL, last_date date NOT NULL, CHECK (last_date >= first_date))`;
    - `training_days (season_year integer NOT NULL REFERENCES training_seasons, valid_from date NOT NULL, weekday smallint NOT NULL CHECK (weekday BETWEEN 1 AND 7), start_time time NOT NULL, end_time time NOT NULL, CHECK (end_time > start_time), PRIMARY KEY (season_year, valid_from, weekday))`.
  - Storage modules in `crates/server/src/`: `players.rs` (list players with roles of two seasons; insert, update, delete a player; insert, update a role; `lock_player` with `SELECT … FOR UPDATE`; `same_name_exists`), `reasons.rs` (list, insert at the end, update), `training_settings.rs` (read a season with its sets; save first/last and replace or add a set). Unique violations on the reason name map to a store error the API turns into `reason_name_taken`.
  - `sqlx migrate run`, then `just prepare`; commit `.sqlx/`.
- **Tests** (`#[sqlx::test]`): the 6 reasons in order with School, Ill, Injured valid; a player with two roles is read back with both; a second set for a season is stored next to the first, a set with the same valid-from replaces it; deleting a player deletes its roles.

### Step 3: API
- **Goal**: the routes, behind the login, with the error codes.
- **Changes**: new files `api/players.rs`, `api/reasons.rs`, `api/training_settings.rs`; routes on the protected router in `app.rs`; paths in `api/mod.rs`; `just api-types`.
  - `GET /api/players` → `PlayersView { season, players: [PlayerView], left: [PlayerView], last_season: [PlayerView], others: [PlayerView] }` (`others`: players with no role in this or the previous season, for example added without a role or from an older season); `PlayerView { id, first_name, last_name, role (today's, or the last in the season for `left`, or last season's for `last_season`), roles: [RoleView { id, role, from, until }] }`. `last_season` stays empty until step 5.
  - `POST /api/players` `{ first_name, last_name, role: { role, from } | null }` → `{ player: PlayerView, same_name: bool }`.
  - `PUT /api/players/{id}` `{ first_name, last_name }` → `{ player, same_name }`.
  - `DELETE /api/players/{id}` → 204.
  - `POST /api/players/{id}/roles` `{ role, from, until }` and `PUT /api/players/{id}/roles/{role_id}` `{ role, from, until }` → `PlayerView`. The season is the season of `from`. In one transaction: `lock_player`, read the player's other roles, `check_role`, write.
  - `GET /api/reasons` → `[AbsenceReason { id, name, valid, hidden }]` by position; `POST /api/reasons` `{ name, valid }` and `PUT /api/reasons/{id}` `{ name, valid, hidden }` → the list.
  - `GET /api/training-settings` → `TrainingSettings { season, saved, first, last, days: [TrainingDayView { weekday, start, end }] }` (the set valid today, or the first set before the first date); `saved: false` with empty days and `null` dates when nothing is saved. `GET /api/training-settings/proposal?weekdays=2,4` → `{ first, last }`. `PUT /api/training-settings` `{ first, last, days }` → `TrainingSettings`.
  - Errors (`AppError::BadRequest`): `first_name_required`, `until_before_from`, `outside_season`, `roles_overlap`, `end_before_start`, `last_before_first`, `no_training_days`, `duplicate_weekday`, `reason_name_required`, `reason_name_taken`. Unknown player, role or reason → 404. The weekday in JSON is 1 (Monday) to 7 (Sunday); times as `"19:45"`.
  - Logs: ids only.
- **Tests** (`crates/server/tests/players.rs`, `tests/settings.rs`, clock 12 Oct 2026 unless said):
  - a player with a role from 1 Sep is under `players` with `plays_games`; the same name again → `same_name: true`;
  - "Left the team" with `until` 15 Oct, clock 20 Oct → under `left`; `until: null` → back under `players`;
  - a player added without a role → under `others`;
  - a new role from 1 Nov, clock 2 Nov → under `players` with two roles;
  - an overlapping role → 400 `roles_overlap`, nothing stored; two role saves for one player at the same time (two tasks) → one stored, the other `roles_overlap`;
  - `DELETE` removes the player and its roles;
  - reasons: add "Family" (not valid) comes last; rename "Work" to "work" ok; rename to "school" → `reason_name_taken`; hide "Holiday" → listed with `hidden: true`;
  - settings: `GET` with nothing saved → `saved: false`; proposal for Tuesday → 1 Sep 2026 and 25 May 2027; `PUT` Tuesday 19:45–21:00 saved; with the clock at 5 Jan 2027 a `PUT` with Thursday adds a set valid from 5 Jan and keeps the first;
  - every new route without login → 401; every changing route without `Origin` → 403;
  - the log after adding and renaming a player has no player name (`LogBuffer`, own test file `tests/player_logs.rs`).

### Step 4: Screens
- **Goal**: the coordinator can type in players and settings.
- **Changes** (`web/`):
  - `src/routes/players/+page.svelte` (+ `+page.ts`): the players with a role (name, badge "Plays games" / "Only trains"), "Left the team", "Other players" (folded; a tap opens the player, where a role can be given), "Add player" (first name, last name, "Plays games" / "Only trains", from with the proposed date). The warning "There is already a player with this name." after a save with `same_name`.
  - `src/routes/players/[id]/+page.svelte`: the name (change and save), the roles of the season (role, from, until), "Left the team" (date, proposed today) / "Undo", "Back in the team" (a new role from a date), "Delete player" (with a confirmation).
  - `src/routes/settings/+page.svelte`: the training days (day of the week, start and end time; add and remove a day), the first and the last training date (filled from the proposal when nothing is saved), "Save"; the reasons (name, valid / not valid, hide / show, rename; "Add reason").
  - Start page: the line "Players · Settings" below the team header, shown with a login also before a team is chosen.
  - `client.ts`: load and save functions; `messageFor` sentences: "A first name is needed.", "The end must be after the start.", "This date is outside the season.", "This player already has a role on that date.", "The end time must be after the start time.", "The last training date must be after the first.", "Choose at least one training day.", "This day is already a training day.", "A name is needed.", "There is already a reason with this name."
  - Phone first: large rows and fields, no sideways scroll at 320 pixels; date and time fields use the browser's own inputs.
- **Tests** (`web/e2e/players.e2e.ts`, project `logged-in`, after the existing tests of that project): add a player with a first name only, "Only trains", from the proposed date, shown in the list; the same name again shows the warning; open the player, "Left the team", then "Undo"; `/settings`: Tuesday 19:45–21:00 with the proposed dates (Tue 1 Sep 2026 – Tue 25 May 2027), save; add "Family"; hide "Holiday"; the start page shows "Players · Settings"; no sideways scroll at 320 pixels and `expectAccessible` on both pages.

### Step 5: "Add all"
- **Goal**: last season's players come back in a new season with one tap.
- **Changes**:
  - Core `players::roles_to_carry(last_season_roles: &[PlayerRole], current: &[PlayerRole], season: Season, start: NaiveDate) -> Vec<(PlayerId, Role)>`: for each player with a role in the previous season whose newest role there has no end before 30 June, and with no role in `season`, the same role kind. `start` is the first training date of the season, or today when later, or today without settings.
  - `GET /api/players` fills `last_season`.
  - `POST /api/players/carry` `{ players: [ids] }` → `PlayersView`: starts the carried roles from `start` for the listed players (all of `last_season` for "Add all"); players that already have a role in the season are skipped.
  - `/players`: in a new season, "Players of last season" with "Add to this season" for each and "Add all".
- **Tests**: core: 14 roles in 2025-2026, one ended 10 Mar 2026 → 13 carried with the same kinds; `start` 1 Sep 2026 when today is 15 Aug 2026 with settings, today (12 Oct) when later, today without settings. API (clock 15 Aug 2026, roles in 2025-2026): `last_season` lists them; "Add all" gives roles, `last_season` becomes empty; a second "Add all" adds nothing.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| A player is the person; a role in a season has a start and an end; not linked to the game list choice | Players join, leave and come back; roles change; the team's game list name changes each season (coordinator, 2026-10-15) | yes, 0019 at Close |
| Training days have a valid-from date; a change applies from today (or from the first date before the season starts); a change on the same day replaces | The past does not change (spec note S10); no pile of sets | in 0019 |
| No overlap of roles is checked in the core under a row lock, not by a database constraint | An `EXCLUDE` constraint needs the extension `btree_gist` on the cluster | in 0019 |
| One training time for each day of the week | The spec has one training a day | no |
| The proposal of the training dates is a server route | One rule (in the core), not again in TypeScript | no |
| The schema name `AbsenceReason` | `Reason` is taken in the OpenAPI description | no |
| Deleting a player is always allowed in story A | Absences exist only from story B, which adds `player_has_absences` | no |

## Not in scope
- Trainings, absences, the start page card and the list of training days (story B); totals (story C).
- A screen to change a role within a season (the data allows it).
- Settings for a season other than the current one.
- Reordering reasons.
- A backup of the database (backlog).

## Test plan
- **Existing tests that must still pass**: all of `just check`; the browser tests of the start page (the new link line must not move what they check).
- **New tests**: per step, above.
- **Checks**: `just check`; `sqlx migrate run` and `just prepare` after step 2; `just api-types` after step 3.

## Compatibility and performance
- The migration adds tables only; the running app works on it (decision 0004).
- The start page gets one line of links; nothing else changes for the games.
- From step 4 the coordinator stores names of players (children) on the cluster: no names in logs (test), no backup yet (accepted, backlog).
- Load: about 15 players and a few roles each season.

## Progress
- [x] Step 1: core rules
- [x] Step 2: migration and storage
- [x] Step 3: API
- [x] Step 4: screens
- [x] Step 5: "Add all"
- [x] All checks pass (`just check`, 2026-10-15; 36 browser tests)
- [x] Review build (2026-10-15): not ready, findings R1–R9 below
- [x] Fix 1: R1, R2, R4 – an open role ends on the last day of its season; the grouping of the players page and the proposed start of a role in the core
- [x] Fix 2: R3 and R9 (two saves at once) – a settings save removes the stored sets from its valid-from date on, under a lock
- [x] Fix 3: R5 – the test of two role saves at once always catches a missing lock
- [x] Fix 4: R6, R7, R8, R9 (the other two points) – roles only in the current season, the detail page, the 404 sentence, the proposed end, the settings page after the first save
- [ ] Review build again

## Deviations
None from `functional.md` (as changed on 2026-10-15). The design reads the spec more strictly within its words:
- One training time for each day of the week.
- Settings only for the current season.
- Without saved settings the proposed start of a role is today.
- A player with no role in this or the previous season is listed under "Other players" (folded), where it can get a role or be deleted.

Implement:
- 2026-10-15, step 1: `check_role` takes no `role` argument (the kind of role is not part of any check); `training::set_on` (the set that applies on a date) is public, because story B needs it for the start page card.
- 2026-10-15, step 2: the reason name also has `CHECK (btrim(name) <> '')` in the table; `StoreError::is_unique_violation` (in `db.rs`) tells the API that a reason name is taken. The storage of reasons uses its own type `StoredReason` (the core has only the name rule).
- 2026-10-15, step 3:
  - The first save of a season's settings applies from the first training date, also when it is made during the season (`settings_change` gets `first_save`). Found by a test: the coordinator sets up the tool in October and enters the trainings since September; with "valid from today" the stored set would start on the day of the save.
  - Two extra error codes for input the screen should never send: `invalid_weekday` (not 1–7) and `invalid_time` (not "HH:MM").
  - `GET /api/players` already fills `last_season` (the grouping is one rule); the route "Add all" stays in step 5.
  - Test helper `tests/common/api.rs` (`Coordinator`: a logged-in harness that sends JSON).
- 2026-10-15, step 4:
  - `GET /api/players` also gives `today` and `proposed_from` (the proposed start of a role), so the screens use the server's clock, not the phone's; the browser tests run with the server's clock at 12 October 2026.
  - Until step 5, last season's players are listed under "Other players".
  - Before the first save of the settings, a change of the chosen days proposes the dates again (unless the coordinator typed dates).
  - The proposal route marks `weekdays` as a query parameter in the OpenAPI description (`#[into_params(parameter_in = Query)]`).
  - A hidden reason has a dashed border, not a lower opacity (the accessibility test failed on contrast); date and time fields can shrink, and the two training dates stand below each other on a phone (no sideways scroll at 320 pixels).
- 2026-10-15, step 5:
  - `roles_to_carry(roles, season)` gives the players and their kinds of role; the start date is the same as `proposed_from` (the first training date, or today when later, or today without settings), worked out in the API, not in the core.
  - "Players of last season" lists only the players who stayed until the end of last season (the ones "Add all" brings back); a player who left early last season is under "Other players", where "Add to the team" works. Otherwise "Add to this season" would do nothing for such a player.
  - A player whose role starts later in the season (for example after "Add all" in August, from 1 September) is listed with the current players, not under "Left the team".
  - A browser test with replaced answers checks the part "Players of last season" and the request of "Add all" (the browser server's clock has no last season).
- 2026-10-16, fix 1: the browser test of "Add all" failed 2 of 6 runs ("Response has been disposed": a load of the list was still under way when the test ended). It now removes its replaced answers at the end (`page.unrouteAll({ behavior: 'ignoreErrors' })`); 6 runs passed. `Season::previous` replaces the three copies of "the season before".
- 2026-10-16, fix 2: the lock of the settings is an advisory lock, not `SELECT … FOR UPDATE` on `training_seasons`: before the first save there is no row to lock.
- 2026-10-16, fix 4: a role of another season cannot be changed (not only not added), so "Undo" on a role of last season gives `outside_season`. `Season::first_day` is new. The new sentence for a 404 is "This no longer exists. Load the page again." (not in the spec's list of sentences).

## Review findings
Build review, 2026-10-15, by a fresh agent (`/review build`). The arguments held only the mode
and the change folder. Read: the diff `main...feat/players-and-settings` (one commit, 905a767;
nothing uncommitted), this folder, the epic's `functional.md` and `technical.md`,
`docs/architecture.md`, decisions 0001, 0004, 0007, 0011, `CLAUDE.md`, `docs/language.md`.
Ran: `cargo test -p team_core`, the four new server test files, `cargo clippy --all-targets -D
warnings`, `cargo fmt --check` (all pass). Probes ran as extra tests in a copy of the tree in the
scratchpad (`git archive HEAD`), against the local PostgreSQL; nothing in the repository was
changed. The browser tests and `just check` as a whole were not run.

**R1 – an open role of last season never ends (bug, high). Run.** `until = NULL` means "until the
end of its season" in `roles_to_carry` (it counts as "stayed until 30 June"), but `covers` /
`role_on` and the overlap test in `check_role` treat it as open for ever, and `save_role` checks
against `roles_of` (all seasons). Every player who stayed until the end of last season has such a
role. Reproduced (Avery and Blake with a role from 2 Sep 2025, no end; clock 12 Oct 2026):
- after "Add to this season" for Avery, `GET /api/players` lists Avery under **"Left the team"**: `role_on` finds the 2025 role first, the season filter drops it, and the new role (from today) is not "later in the season";
- "Left the team" on Avery's new role: `400 roles_overlap`;
- "Add to the team" (`POST /roles`) for Blake from 12 Oct 2026: `400 roles_overlap`;
- "Add all" in summer (clock 15 Aug 2026, first training 1 Sep): the players are listed with the team until 1 Sep; on 2 Sep the carried player is under "Left the team".
- 2026-10-16: fixed. `PlayerRole::last_day` (its end, or 30 June of its season while it continues; `Season::last_day`) is the one meaning of an open role in `role_on`, the overlap of `check_role` and "Add all". Core tests with an open role of 2025-2026 against dates and roles in 2026-2027; API tests `after_add_all_in_summer_the_players_stay_in_the_team_and_can_leave` (clock 2 Sep and 2 Oct) and `last_seasons_players_join_during_the_season`. Both API tests fail with the old meaning.

So "Add all" and every role change in the second season of use do not work, and story B
(who shows at a training, `role_on`) would put last season's players at this season's trainings
with last season's role. The tests did not see it: the core tests of `role_on` use one season, and
the API test of "Add all" stops before the first training date (the "role starts later" branch
hides it). Fix in the core: one meaning for an open role, its end is `until` or the season's last
day (30 June), in `covers`, `role_on` and the overlap of `check_role`; a core test with an open role
of 2025-2026 against a date and a role in 2026-2027; an API test of "Left the team" after "Add
all", with the clock after the first training date.

**R2 – `one()` gives a role from another season (bug, low; same root as R1). Run.** The answer
of `POST /api/players` with a role from 2025-09-01 (clock 12 Oct 2026) has `"role": "plays_games"`
and `"roles": []`; `one()` uses `role_on` without the season filter that `group` has. The answer
of a role that starts later (clock 15 Aug, from 1 Sep) has `"role": null`, while the list shows
"only_trains". The two views of one player follow two rules; the screens use only the list now.
Fix with R1, and give `one()` the same rule as `group` (better: one core function, see R4).
- 2026-10-16: fixed. `one()` and the list both use the core function `place`; the API test checks that the answer of a new role equals the player's row in the list.

**R3 – a change of training days before the season leaves an older set in force (bug, medium).
Run.** `training_settings::save` replaces only a set with the same `valid_from`. Before the first
training date `valid_from` is `first`, so a set can lie in the future. Probe, clock 15 Aug 2026:
save Tuesdays with first 8 Sep, then Thursdays 19:00–20:30 with first 1 Sep. Stored sets:
`[(1 Sep, Thu), (8 Sep, Tue)]`; `training_dates` gives Thu 3 Sep, then Tue 8, 15, 22 Sep …; the
settings page shows Thursday until 7 Sep and **Tuesday 19:45 from 8 Sep** (`GET` on 10 Sep), though
the last save said Thursday. The same happens in the season when `first` is moved later than
today. The spec says a change applies from the day it is made; a later save must win from its
`valid_from` on. Fix: a save removes the sets with `valid_from` on or after the new one (rule in
`settings_change` or the store, with a test of the case above).
- 2026-10-16: fixed. `training_settings::save` removes the stored sets valid from the new set's date or later, so the newest save applies from its valid-from date on. Storage test `a_save_removes_the_sets_from_its_date_on`; API test `before_the_season_the_last_save_wins` (the probe above: one set from 1 Sep, Thursday on 10 Sep). Both fail with the old query.

**R4 – product rules in the server (decision 0001, "New rules go in `crates/core` first").
Trace.** `api/players.rs` holds the grouping of the players page (`group`: with the team, "Left
the team", last season, others; a role that starts later counts as current) and the proposed start
of a role (`proposed_from`: first training date, or today when later, or today without settings),
which the technical spec put in the core (`roles_to_carry(…, start)`). The deviation of step 5
names the move but not why. Both are product rules that story B and C need again (who is in the
team on a date; the proposed start), and `group` is tested only through the API. R1 and R2 show
the cost: two rules for "the role now" in one file. Suggest: a core function for the grouping and
one for the proposed start, with unit tests; the server only loads and maps.
- 2026-10-16: fixed. Core `players::place` (the group, the role shown, the roles listed; `Group` is with the team, left, last season or other) and `players::proposed_start`, with unit tests; `api/players.rs` only loads and maps.

**R5 – the concurrency test catches a missing lock only sometimes (test, medium). Run.** With the
`players::lock` call in `save_role` removed (in the scratch copy), `two_role_saves_at_the_same_time_never_overlap`
failed 1 of 9 runs. `tokio::join!` on one task interleaves the two requests only by chance. The
claim "two role saves … one stored, the other `roles_overlap`" is true for the code, but the test
does not guard it. Suggest: hold the first request after its read (for example a second connection
that takes the row lock first and lets go after the second request has started), or run the
saves many times in a loop.
- 2026-10-16: fixed. `two_role_saves_at_the_same_time_never_overlap` now holds the player's lock on a second connection and writes an overlapping role there; the save through the API must wait (still pending after 300 ms), and after the commit it answers `400 roles_overlap`, with one role stored. Without `players::lock` in `save_role` it failed 10 of 10 runs; with it, it passed 10 of 10.

**R6 – roles can be saved in any season, though the spec and screens speak only of the current
one (gap, medium). Run.** `POST /api/players` with a role from 2025-09-01 (clock 12 Oct 2026) and
from 2027-09-01: both `200`. The first player lands under "Players of last season" with an open
role, the second under "Other players" with a role that no screen shows (the detail page lists
only the current season's roles). The date fields have no `min`/`max`, so a wrong year on the phone
gives this without a sign. `outside_season` ("This date is outside the season.") can never come
from `add`, and from the role routes only when `until` is in another season than `from`. Suggest:
the role routes check `Season::of(from) == current season` (the spec: "a role in the current
season"), and the date fields get the season's bounds.
- 2026-10-16: fixed (the coordinator chose: current season only). `POST /api/players` and the role routes check against the current season, so a start date in another season gives `400 outside_season`; a role of another season cannot be changed either. `PlayersView` gives `season_first_day` and `season_last_day`; the date fields of a role use them as `min` and `max`. API test `roles_are_saved_only_in_the_current_season`; the browser test of "Add player" checks the limits.

**R7 – the player detail page shows last season's roles under this season's heading (bug, low).
Trace.** `players/[id]/+page.ts` takes the player from `last_season` too, where `roles` are last
season's. The page then shows them under "In the team · 2026-2027" and offers "Left the team" on a
role of 2025-2026; a save with today as end gives `outside_season`. The rows of "Players of last
season" are not links, so the page is reached only by URL or a stale link. Suggest: on the detail
page, take `roles` only for the current season (or show "Last season" apart).
- 2026-10-16: fixed. The detail page gives a player of last season no roles, so it shows "Not in the team this season." and "Add to the team", with last season's kind of role proposed. Browser test `a player of last season has no roles of this season yet` (replaced answer).

**R8 – a 404 on a player, role or reason shows "This club is not in the game list." (bug, low).
Trace.** `messageFor` maps every 404 without a known code to that sentence (`client.ts`). A player
deleted in another tab, then renamed, or a role id that no longer exists, shows it. Suggest: the
club message only for the club route, or a code (`not_found`) checked before it with a general
sentence.
- 2026-10-16: fixed. `loadTeams` (the club route) gives "This club is not in the game list." for its 404; every other 404 gives "This no longer exists. Load the page again." Browser test `a player that no longer exists gives a general message` (replaced answer). This test needs a player from the test before it, so it was not run alone against the old screens.

**R9 – smaller points (low). Trace.**
- "Left the team" proposes today also for a role that starts later (after "Add all" in summer): the save gives `until_before_from`. Propose `max(today, from)`.
- `/settings`: after the first save `data.settings.saved` stays `false` until the page is loaded again, so a change of days then proposes the dates again over the saved ones (when they came from the proposal).
- Two first saves of the settings at the same time can both delete and insert the same set; the second fails on the primary key with `500 internal`. One coordinator, so unlikely; `SELECT … FOR UPDATE` on `training_seasons` or an advisory lock would close it.
- 2026-10-16, third point (two first saves at once): fixed. A settings save takes its own advisory lock (`training_settings::lock`, key "team" "trng") before it reads what is stored. API test `a_save_waits_for_another_save`: with the lock held on another connection the save waits; it answers 200 after the release. It fails without the lock.
- 2026-10-16, first point: fixed. "Left the team" proposes today, or the role's start when that is later; the date field has the role's start as `min`. Browser test `"Left the team" proposes the start of a role that starts later`.
- 2026-10-16, second point: fixed. After a save the settings page treats the dates as saved, so a change of days no longer asks for a proposal. The browser test of the settings adds a day after the save and checks that no proposal is asked and the last date stays.

**Checked and found right.** Every new route is on the protected router and tested for 401 and
403 (run). Logs carry ids only; `AppError::Internal` logs `StoreError` by `Display`, which has no
row data; the log test sees "player added" (so it is live) and no name (run). No `unwrap`/`expect`
outside tests (clippy, run). The migration only adds tables (decision 0004). The `CHECK
(btrim(...) <> '')` cannot refuse a name the core let through (Rust `trim` removes at least what
`btrim` removes). Whether `.sqlx/` matches the queries was not checked (`just check-sqlx` not
run; the builds used the live database).

**Docs made false.** None by this diff. At Close: `docs/architecture.md` (core and server modules,
migration `0005`, the routes, the new pages), the `CLAUDE.md` module list and the rule "logs never
contain player names", and decision 0019, as the specs plan.

**Verdict:** not ready to close. R1 and R3 are bugs against the spec's intent that the tests do not
see; R1 breaks "Add all" and every role change from the second season on, and is the base story B
builds on. R4 and R5 belong with the fix of R1.
