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
- [ ] Review build

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

## Review findings
