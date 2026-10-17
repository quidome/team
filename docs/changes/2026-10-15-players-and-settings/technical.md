# Technical spec: Players and settings

**Status**: approved (designed again 2026-10-16, steps 1–4 built; steps 5–9 designed and approved 2026-10-17 after the second build and accept review)
**Size**: story
**Change**: docs/changes/2026-10-15-players-and-settings
**Date**: 2026-10-15, designed again 2026-10-16, steps 5–9 on 2026-10-17

## Objective
Let the coordinator keep the club's players, the team of the current season with each player's
role, the weekly trainings, and the reasons for an absence, so that story B can mark attendance.

## Requirements summary
Story A of the epic [Attendance at training](../2026-10-14-attendance-at-training/technical.md),
from functional stories 1, 2 (the weekly trainings) and 4, as approved again on 2026-10-16 (story 1 changed after the second reviews of this story, approved after the sixth spec review). Words
from `docs/language.md` (player, role, hidden, weekly training, training day).

The model (coordinator, 2026-10-16): the **club** has the players; a **team** exists for one
season and is the team chosen from the game list for that season; a player is in the team of a
season through a **role**. One club for the whole tool, and one team per season. Roles and weekly
trainings are stored by season, so they belong to the team of their season without a separate
link, and can be entered before the team is chosen.

1. **Players page** (`/players`): all players, sorted by first name, each with "Plays games" / "Only trains" when in the team today, else "Not in the team"; "Add player" (first name required, last name optional; trimmed; an empty last name is none); the switch "Show hidden players" (off: hidden players are not listed). The warning "There is already a player with this name." after a save with the same first and last name.
2. **Player page** (`/players/[id]`): the name (change and save), the switch "Hidden", the history (all roles of all seasons, newest first, for example "Plays games · from Sun 1 Nov" and "Only trains · Tue 1 Sep – Sat 31 Oct"), "Correct" on a role of the current season (kind, start, end; the same checks as a new role), "Delete player" (after a confirmation; story B adds: only without absences). Roles of an earlier season are shown, not corrected.
3. **Hidden**: only keeps lists short; it is not a status (coordinator, 2026-10-16). A hidden player is left out of the list on "Players" (unless "Show hidden players" is on), the choices of "Add player" on the team page, and "Players of last season" (so "Add all" does not add it). Nothing else changes: a hidden player in the team stays in the team, and a hidden player under "Left the team" keeps "Add to the team".
4. **Team page** (`/team`, current season): the heading with the chosen team's name and the season ("U14-1 · 2026-2027"), or "Team not chosen yet" with a link to choose it (coordinator, 2026-10-16). The players in the team today or later in the season (sorted by first name, with the role), then "Left the team" (a role earlier in the season, none today or later). "Add player": a player from the list (not hidden, not in the team, last season's team first) or a new player, with a role and "In the team from".
   - **"Change role"** (the new role and "From", the first day of the new role, filled with today) acts on the role that covers the date: it ends on the day before; the new role starts on the date and keeps the planned end of the old role. A later planned role stays. Refused: a date on the start of the role ("The change must be after the start of the role. To change the role from its start, use "Correct" in the history."); the same role ("The player already has this role."); no role on the date but one later in the season ("This player's role starts on Mon 2 Nov. To change it, use "Correct" in the history."); no role on the date and none later ("This player is not in the team on that date.").
   - **"Remove from the team"** asks "Not in the team from" (the first day without the team), filled with today, with a button "Tomorrow". From that date the player has no roles in the season: the role that covers the day before and the date ends on the day before; every role that starts on the date or later is removed. Nothing to end or remove: refused ("This player is not in the team on that date."). When the removal removes a role, the tool first asks for a confirmation that names it ("This also removes "Plays games · from Sun 1 Nov"."); a removal that only ends a role is saved at once. A player with a role earlier in the season is under "Left the team" at once; a player whose roles of the season all start on the date or later has nothing left in the season (in no group, back in the choices of "Add player").
   - **"Undo"** after a removal, until the coordinator leaves the team page: the player's roles of the season come back exactly as they were (kinds and dates). Refused when another screen changed the player's roles in the meantime ("This player was changed in the meantime. Use "Correct" in the history."). Later, a mistake is put right with "Correct" and "Add to the team"; there is no "Delete" for a role in the history.
   - **"Last day today"** in the row of a player in the team whose role today ends today, with no later role in the season; not on 30 June.
   - Under "Left the team": **"Add to the team"** (a role and "In the team from", filled with today).
   - **"Players of last season"** (last season's team, not hidden) with **"Add all"** shows while no player of last season's team has a role in the season; a newcomer added first does not make it go. "Add all" asks "In the team from"; each player gets the same role as at the end of last season, from that date.
5. **Proposed dates**: "In the team from" of "Add player" and "Add all" is filled with 1 September of the season, with a button "Today"; "Add to the team" and "Change role" with today; "Not in the team from" with today, with "Tomorrow". (The first training date is no longer used: changed after the sixth spec review, 2026-10-16.)
6. **Weekly trainings** (`/settings`, current season): add (a day of the week, a start and an end time, a window from a date until a date inside the season; the proposed window is that day on or after 1 September until that day on or before 31 May), "Change" (everything), "Delete", the list sorted by the start of the window ("Tuesday 19:45–21:00 · Tue 1 Sep – Tue 25 May"). More than one at a time; not two on the same day of the week with overlapping windows ("There is already a weekly training on Tuesday in these dates.").
7. **Reasons**: presets in this order: School, Ill, Injured (valid), Holiday, Work, No reason (not valid). Add (last), rename, valid / not valid, hide (still listed with "hidden"). Names unique without case and outer spaces. As built.
8. **Navigation**: the start page shows "Team · Players · Settings" below the team header, also before a team is chosen.
9. Only the coordinator (login, decision 0007). Logs never contain player names.

Decisions that apply: 0001, 0002, 0004 (read as "applied on the cluster", see Decisions), 0007, 0012 (the choice kept for each season), 0014; the epic's architecture (absences only, from story B; weekly trainings only suggest training days).

## Current state
On branch `feat/players-and-settings` (merged with `main` up to 9f9efe6), the first form of this
story is built and reviewed (commits 905a767, ff065be, 7895c6f):
- Migration `0005_players_and_settings.sql`: `players` (no `hidden`), `player_roles`, `reasons` (6 presets), `training_seasons`, `training_days`.
- Core: `season.rs` (`previous`, `first_day`, `last_day`); `players.rs` (`Player`, `Role`, `PlayerRole::last_day`, `role_on`, `check_role`, `check_change`, `proposed_start(first, today)`, `Group`, `Placement`, `place`, `roles_to_carry`); `training.rs` (sets valid from a date, `proposed_dates`, `set_on`, `shown_set`, `replaced_from`, `training_dates`, `settings_change`); `reasons.rs` (`clean_reason_name`).
- Server: `players.rs` (storage, `lock` with `SELECT … FOR UPDATE`), `reasons.rs`, `training_settings.rs` (with an advisory lock); `api/players.rs` (`GET /api/players` with the groups `players`, `left`, `last_season`, `others`; add, rename, delete, roles, carry), `api/reasons.rs`, `api/training_settings.rs`; `followed_teams::for_season` gives the team choice of a season (with the team name).
- Front end: `/players`, `/players/[id]`, `/settings`; the start page line "Players · Settings"; `client.ts` with the sentences; `lib/players.ts`.
- Tests: core unit tests (135 in the crate), `tests/players.rs` (15), `tests/settings.rs` (9), `tests/players_storage.rs`, `tests/player_logs.rs`, `web/e2e/players.e2e.ts` (39 browser tests in all).
- The full first-form spec (steps, deviations) is in git at 7895c6f; its review findings are kept below.

Steps 1–4 of this design are built (2baa9c9 to df54748) and reviewed a second time (064ef52: A6–A10,
R13–R17). What steps 5–9 change:
- Core `players.rs`: `change_role(season, current, role, date, others)` (the server picks `current`); `proposed_start(first_training, training_opened, today)`; `TeamPage` with `last_season` = every player of last season's team without a role in the season; `PlayerError::NotInTeam` is never made; the comments on `Group`, `Placement` and `place` still speak of the players page. `training.rs`: `first_training_date`, used only by the proposed start.
- Server `api/team.rs`: `covering` (the role of the season on a date, else `not_in_team`), `role_change`, `team_leave` (`{ until }`: ends the covering role on that date, nothing else); `api/players.rs`: `proposed_from` (from the weekly trainings), `carry` (`{ players }`, from `proposed_from`, players locked in the order sent). `AppError::BadRequest(&'static str)` gives `{ "error": code }` only. Storage has no delete of a role.
- `/team`: "Last day in the team" (proposed today, or the role's start when later), `currentRole` (a copy of `place`), "Add all" without a date. `/settings`: `propose()` returns at once when a window is in the form and `windowProposed` is false, so "Change" of the day keeps the old day's window (R15). `client.ts`: `ApiError` holds only the message.

## Steps

### Step 1: Weekly trainings, end to end
- **Goal**: weekly trainings replace the season's training days in the stored data, the core, the API and the settings page.
- **First**: `just db-down`, `just db-up`, `just migrate` (local databases ran the old `0005`).
- **Changes**:
  - Rewrite `migrations/0005_players_and_settings.sql` with its final tables: `players` gets `hidden boolean NOT NULL DEFAULT false`; `player_roles` and `reasons` as now; `training_seasons` and `training_days` go; new `weekly_trainings (id bigint identity PK, season_year integer NOT NULL, weekday smallint NOT NULL CHECK (weekday BETWEEN 1 AND 7), start_time time NOT NULL, end_time time NOT NULL, from_date date NOT NULL, until_date date NOT NULL, CHECK (end_time > start_time), CHECK (until_date >= from_date))`, index on `(season_year)`.
  - Core `training.rs`, rewritten:

    ```rust
    pub struct WeeklyTrainingId(pub i64);
    pub struct WeeklyTraining { pub id: WeeklyTrainingId, pub season: Season, pub weekday: Weekday,
                                pub start: NaiveTime, pub end: NaiveTime,
                                pub from: NaiveDate, pub until: NaiveDate }
    /// The window inside `season`, until ≥ from, end > start, no overlap with `others` (the other
    /// weekly trainings of the season, without the one that is changed) on the same day of the week.
    pub fn check_weekly_training(season: Season, weekday: Weekday, start: NaiveTime, end: NaiveTime,
                                 from: NaiveDate, until: NaiveDate, others: &[WeeklyTraining])
                                 -> Result<(), TrainingError>;
    /// That day on or after 1 September until that day on or before 31 May.
    pub fn proposed_window(season: Season, weekday: Weekday) -> Option<(NaiveDate, NaiveDate)>;
    /// Every date in a window on its day of the week, sorted by date.
    pub fn training_days(weekly: &[WeeklyTraining]) -> Vec<(NaiveDate, &WeeklyTraining)>;
    pub fn first_training_date(weekly: &[WeeklyTraining]) -> Option<NaiveDate>;
    pub enum TrainingError { EndBeforeStart, LastBeforeFirst, OutsideSeason, WeeklyTrainingsOverlap }
    ```

    The sets valid from a date, `proposed_dates`, `set_on`, `shown_set`, `replaced_from`, `training_dates` and `settings_change` go. A window without its day of the week in it is allowed (no training days).
  - Server `weekly_trainings.rs` replaces `training_settings.rs`: `for_season`, `insert`, `update` (false when the id is not of the season), `delete`, `lock` (an advisory lock, key "team" "wktr" in ASCII). `api/weekly_trainings.rs` replaces `api/training_settings.rs`:
    - `GET /api/weekly-trainings` → `WeeklyTrainings { season, season_first_day, season_last_day, weekly_trainings: [WeeklyTrainingView { id, weekday, start, end, from, until }] }`, sorted by `from`, then by day of the week.
    - `GET /api/weekly-trainings/proposal?weekday=2` → `{ from, until }`.
    - `POST /api/weekly-trainings` `{ weekday, start, end, from, until }`, `PUT /api/weekly-trainings/{id}` (same body), `DELETE /api/weekly-trainings/{id}` → `WeeklyTrainings`. The season is the current season; each save in one transaction: `lock`, read the others, `check_weekly_training`, write.
    - Errors: `end_before_start`, `last_before_first`, `outside_season`, `weekly_trainings_overlap`, `invalid_weekday`, `invalid_time`; unknown id → 404.
  - `/api/training-settings` and its proposal route go. `api/players.rs` `proposed_from` uses `first_training_date` of the season's weekly trainings (`proposed_start(first, today)` stays until step 2).
  - `/settings`: the weekly trainings (list, "Add weekly training" with the proposed window filled when a day is chosen, "Change", "Delete" after a confirmation); the reasons stay. `client.ts`: load and save functions; the sentence "There is already a weekly training on Tuesday in these dates." (the day from the request), "The last training date must be after the first.", "The end time must be after the start time.", "This date is outside the season."; the sentences of `no_training_days` and `duplicate_weekday` go. The date fields have the season's first and last day as limits.
  - `sqlx migrate run`, `just prepare`, `just api-types`.
- **Tests**:
  - Core: `check_weekly_training` (Tuesday Tue 1 Sep 2026 – Tue 25 May 2027 with a Tuesday from Tue 5 Jan 2027 in `others` → `WeeklyTrainingsOverlap`; Tuesday and Thursday with the same window → ok; the weekly training itself left out of `others` → ok; from 30 Jun 2026 or until 1 Jul 2027 → `OutsideSeason`; until before from → `LastBeforeFirst`; end 19:45 and start 21:00 → `EndBeforeStart`); `proposed_window` (Tuesday → Tue 1 Sep 2026 – Tue 25 May 2027; Thursday → Thu 3 Sep 2026 – Thu 27 May 2027); `training_days` (Tuesday in the proposed window → 39 dates; Tuesday and Thursday → sorted by date, Tue 1 Sep before Thu 3 Sep); `first_training_date` (none without weekly trainings; Thursday from Thu 3 Sep and Tuesday from Tue 8 Sep → Thu 3 Sep).
  - Storage (`tests/players_storage.rs`): the 3 tests of training-day sets go; a weekly training is read back; `update` with an id of another season → false.
  - API (`tests/weekly_trainings.rs`, replaces `tests/settings.rs`): nothing saved → an empty list; the proposal for Tuesday; add Tuesday 19:45–21:00 → listed; an overlapping Tuesday → 400 `weekly_trainings_overlap`, nothing stored; "Change" to Thursday → listed as Thursday; delete → empty; unknown id → 404; a save waits while another connection holds the lock (the method of the first form's settings lock test); the proposed start in `GET /api/players` is Tue 1 Sep on 12 Oct with a Tuesday weekly training, and today without one.
  - Routes: the new routes in the login (401) and `Origin` (403) test; the old training-settings routes leave it.
  - Browser (`players.e2e.ts`): the settings part rewritten: add Tuesday 19:45–21:00 with the proposed window (Tue 1 Sep 2026 – Tue 25 May 2027), change it, "Delete"; the reasons part stays; no sideways scroll at 320 pixels, `expectAccessible`.

### Step 2: Core rules for players
- **Goal**: the rules of the team page, "Change role", hidden and the proposed start, without I/O.
- **Changes** (`crates/core/src/players.rs`):

  ```rust
  pub struct Player { pub id: PlayerId, pub first_name: String,
                      pub last_name: Option<String>, pub hidden: bool }
  pub struct Member<'a> { pub player: &'a Player, pub role: Role, pub roles: Vec<&'a PlayerRole> }
  pub struct Team<'a> {
      pub members: Vec<Member<'a>>,     // a role today or later in the season
      pub left: Vec<Member<'a>>,        // a role earlier in the season, none today or later
      pub last_season: Vec<Member<'a>>, // stayed until 30 June last season, no role now, not hidden
      pub candidates: Vec<&'a Player>,  // not hidden, not in members or left; last season's team first
  }
  pub fn team<'a>(players: &'a [Player], roles: &'a [PlayerRole], season: Season,
                  today: NaiveDate) -> Team<'a>;
  pub struct RoleChange { pub ended_until: NaiveDate, pub role: Role,
                          pub from: NaiveDate, pub until: Option<NaiveDate> }
  /// `current` ends on the day before `date`; the new role starts on `date` and keeps the planned
  /// end of `current`.
  pub fn change_role(season: Season, current: &PlayerRole, role: Role, date: NaiveDate,
                     others: &[PlayerRole]) -> Result<RoleChange, PlayerError>;
  pub fn proposed_start(first_training: Option<NaiveDate>, training_opened: bool,
                        today: NaiveDate) -> NaiveDate;
  pub enum PlayerError { FirstNameRequired, UntilBeforeFrom, OutsideSeason, RolesOverlap,
                         ChangeBeforeRoleStart, NotInTeam }
  ```

  - `team` uses `place` for each player (`Group::Team` → members, `Left` → left, `LastSeason` → last season unless hidden), each group sorted by first name (without case); `candidates`: not hidden and in `LastSeason` or `Other`, the `LastSeason` ones first. A hidden player in the team stays in `members`.
  - `change_role` refuses: `current` of another season or `date` outside the season → `OutsideSeason`; `date` ≤ `current.from` → `ChangeBeforeRoleStart`; `date` after `current.until` → `UntilBeforeFrom`; the new role overlapping `others` → `RolesOverlap`.
  - `proposed_start`: `first_training` when it is some and `training_opened` is false, else `today`.
  - `roles_to_carry` is replaced by `team(…).last_season` (the kind is `Member::role`).
  - Server: `players.rs` storage reads and writes `hidden` (new players: false); `api/players.rs` passes `false` as `training_opened` and uses `team` for its groups (the routes keep their shape until steps 3 and 4).
- **Tests** (core unit tests, fixed dates; `place`, `check_role`, `check_change` tests stay):
  - `change_role`: a role from Tue 1 Sep 2026, change on Sun 1 Nov 2026 → ended until Sat 31 Oct, new from Sun 1 Nov, until none; a role until Wed 31 Mar 2027 → the new role until Wed 31 Mar 2027; on Tue 1 Sep → `ChangeBeforeRoleStart`; on Thu 1 Apr 2027 (after the end) → `UntilBeforeFrom`; a role of 2025-2026 → `OutsideSeason`; an overlap with another role from Sun 15 Nov → `RolesOverlap`.
  - `team`: a player after `change_role` (two roles, the newest from Sun 1 Nov; today 12 Oct and 2 Nov) is in `members`, not in `left`; a hidden player with a role today is in `members`; a hidden player who stayed last season is in neither `last_season` nor `candidates`; candidates: last season's team before others, each part by first name; a player under "Left the team" is not a candidate.
  - `proposed_start`: (Tue 1 Sep, false, Mon 12 Oct) → Tue 1 Sep; (Tue 1 Sep, true, Mon 12 Oct) → Mon 12 Oct; (none, false, Mon 12 Oct) → Mon 12 Oct.
  - The first-form tests of `roles_to_carry` move to `team().last_season` (14 roles, one ended 10 Mar 2026 → 13; the newest role's kind).

### Step 3: The team page
- **Goal**: the coordinator works with the team of the current season on one page.
- **Changes**:
  - `GET /api/team` → `TeamView { season, team_name: Option<String>, today, season_first_day, season_last_day, proposed_from, members: [MemberView], left: [MemberView], last_season: [MemberView], candidates: [PlayerSummary] }`; `MemberView { id, first_name, last_name, role, roles: [RoleView] }` (the roles of the shown season); `PlayerSummary { id, first_name, last_name }`. `team_name` from `followed_teams::for_season` (none without a choice or without a team).
  - `POST /api/players/{id}/roles` `{ role, from, until }` ("Add player" for a player from the list, "Add to the team") → `TeamView` (was `PlayerView`); `POST /api/players` keeps its body and answers `{ player, same_name }` as now.
  - `POST /api/players/{id}/role-change` `{ role, from }` → `TeamView`. In one transaction: lock the player, read its roles, the role that covers `from` (none → `not_in_team`), `change_role`, update that role's end, insert the new role. Both or neither.
  - `POST /api/players/{id}/team-leave` `{ until }` → `TeamView`: under the lock, the role that covers `until` (none → `not_in_team`), `check_change` with the new end, update.
  - `POST /api/players/carry` `{ players: [ids] }` → `TeamView`: only players in `last_season` (so not hidden); others are skipped.
  - Errors: `change_before_role_start` ("The change must be after the start of the role. To change the role from its start, use "Correct" in the history."), `not_in_team` ("This player is not in the team on that date."); unknown player → 404.
  - `/team`: the heading ("U14-1 · 2026-2027", or "Team not chosen yet" with a link to `/choose`); the members (name, badge), "Left the team"; "Add player" (a choice of candidates or "New player" with first and last name; a role; "In the team from" with `proposed_from`); a tap on a member opens "Change role" (role, "From", proposed today) and "Remove from the team" ("Last day in the team", proposed today, or the role's start when later); under "Left the team" "Add to the team" (role, date proposed the day after the end, or today when later); in a new season "Players of last season" with "Add all". Date fields with the season's first and last day as limits.
  - Start page: the line "Team · Players · Settings".
  - `just api-types`.
- **Tests**:
  - API (`tests/team.rs`, clock 12 Oct 2026 unless said): a player added with a role from Tue 1 Sep is in `members`; `team_name` is the chosen team's name, and none without a choice; "Change role" on Sun 1 Nov (clock 2 Nov): `members` with two roles, the first until Sat 31 Oct; "Change role" refused by an overlap (a later role from Sun 15 Nov exists) → 400 `roles_overlap`, both roles unchanged; "Change role" while another connection holds the player's lock waits, then answers (the method of fix 3); "Change role" on the role's start → `change_before_role_start`; "Remove from the team" on Thu 15 Oct (clock 20 Oct) → under `left`; "Remove from the team" for a player without a role on that date → `not_in_team`; "Add to the team" after "Left the team" → a new role, in `members`; "Add all" in summer (clock 15 Aug 2026, last season's roles) leaves out a hidden player; a hidden player in the team stays in `members`; the first form's R1 tests (after "Add all" in summer the players stay in the team; last season's players join during the season) on `GET /api/team`.
  - Routes: the new routes in the 401 and 403 test; the log test: no player name after "Change role".
  - Browser: `/team`: "Add player" (new player, "Only trains", the proposed Tue 1 Sep on the server clock 12 Oct) listed; the same name again shows the warning; "Change role" from a date; "Remove from the team", the player is under "Left the team"; "Add to the team"; "Add all" with a replaced answer (`page.route`, as in the first form); the heading "Team not chosen yet" with a replaced answer; the start page links; no sideways scroll at 320 pixels, `expectAccessible`.

### Step 4: The players page and the player page
- **Goal**: all players, hidden, and the history with "Correct".
- **Changes**:
  - `GET /api/players` → `PlayersList { season, players: [PlayerListItem { id, first_name, last_name, hidden, role_today: RoleKind | null }] }`, sorted by first name. The groups of the first form (`players`, `left`, `last_season`, `others`, `proposed_from`, `today`) go from this route.
  - `GET /api/players/{id}` → `PlayerDetail { id, first_name, last_name, hidden, season, season_first_day, season_last_day, history: [HistoryRole { id, season, role, from, until, correctable }] }` (newest first; `correctable` for roles of the current season).
  - `PUT /api/players/{id}` `{ first_name, last_name, hidden }` → `{ player: PlayerDetail, same_name }`.
  - `PUT /api/players/{id}/roles/{role}` ("Correct") → `PlayerDetail`; `check_change` as now (only a role of the current season).
  - `DELETE /api/players/{id}` as now.
  - `/players`: the list (name, badge or "Not in the team", "hidden" for a hidden player), "Show hidden players", "Add player" (name only; the warning after a save with `same_name`); a tap opens the player.
  - `/players/[id]`: the name (save), "Hidden", the history with "Correct" (kind, from, until; date limits of the season), "Delete player" (confirmation).
  - The first-form pages and their code go (the groups, `last_season` on the players page, the detail page's "Left the team" and "Back in the team" now on `/team`).
  - `just api-types`.
- **Tests**:
  - API (`tests/players.rs`, adapted): `GET /api/players` lists all players with `hidden` and `role_today`; `GET /api/players/{id}` gives the history of two seasons, newest first, `correctable` only for this season; "Correct" of a role of last season → `outside_season`; "Correct" with an overlap → `roles_overlap`; `PUT` with `hidden: true` → listed with `hidden`, still in `members` of `GET /api/team`; delete removes the player and its roles; the first-form tests of names, errors, overlap and two saves at once stay, on the new routes.
  - Browser: `/players`: add a player with a first name only; a hidden player is shown only with "Show hidden players"; `/players/[id]`: "Hidden", "Correct" of the start date (the history shows the new date), "Delete player"; no sideways scroll at 320 pixels, `expectAccessible`.

### Step 5: Core rules and "Change role"
- **Goal**: every rule of the changed story 1 is in the core (decision 0001, R14); "Change role" uses it end to end; the proposed start is 1 September.
- **Changes**:
  - Core `crates/core/src/players.rs`:

    ```rust
    pub enum PlayerError { FirstNameRequired, UntilBeforeFrom, OutsideSeason, RolesOverlap,
                           ChangeBeforeRoleStart, NotInTeam,
                           SameRole,                   // "The player already has this role."
                           RoleStartsLater(NaiveDate), // the start of the later role
                           ChangedMeanwhile }          // "Undo" after another change
    /// A role without its id: what "Undo" sends back and puts back.
    pub struct RoleDates { pub role: Role, pub from: NaiveDate, pub until: Option<NaiveDate> }
    pub struct RoleChange { pub ended: RoleId, pub ended_until: NaiveDate,
                            pub role: Role, pub from: NaiveDate, pub until: Option<NaiveDate> }
    /// `roles`: all roles of the player; picks the role of `season` that covers `date`.
    pub fn change_role(season: Season, roles: &[PlayerRole], role: Role, date: NaiveDate)
        -> Result<RoleChange, PlayerError>;
    pub struct Removal<'a> { pub ended: Option<(RoleId, NaiveDate)>, pub removed: Vec<&'a PlayerRole> }
    pub fn remove_from_team(season: Season, roles: &[PlayerRole], from: NaiveDate)
        -> Result<Removal<'_>, PlayerError>;
    pub fn undo_removal(season: Season, before: &[RoleDates], from: NaiveDate,
                        stored: &[PlayerRole]) -> Result<Vec<RoleDates>, PlayerError>;
    /// 1 September of the season.
    pub fn proposed_start(season: Season) -> NaiveDate;
    ```

    - `change_role`, in this order: `date` outside the season → `OutsideSeason`; no role of the season covers `date` → `RoleStartsLater(start)` when a role of the season starts after `date`, else `NotInTeam`; the covering role has kind `role` → `SameRole`; `date` is its start → `ChangeBeforeRoleStart`; the new role (from `date` until the old planned end) overlaps another role → `RolesOverlap`. The old "date after the end" (`UntilBeforeFrom`) cannot happen and goes.
    - `remove_from_team`: `from` outside the season → `OutsideSeason`; `ended` is the role of the season with `role.from < from ≤ role.last_day()`, its new end `from − 1 day`; `removed` are the roles of the season with `role.from ≥ from`, in order of start; both empty → `NotInTeam`. So a role that already ends on the day before is refused, and a removal from 1 July removes all roles of the season.
    - `undo_removal`: applies `remove_from_team` to `before` (as roles of the season) and compares the result with `stored`'s roles of the season (kinds and dates, sorted by start); a difference → `ChangedMeanwhile`; then each role of `before` gets `check_role` against the others; gives `before`.
    - `team`: `last_season` is empty when a player who stayed until the end of last season (also a hidden one) has a role in the season; `candidates` as now (last season's team first). `Member` gets `last_day_today: bool`: the role today ends today, no role of the season starts later, and today is not the season's last day.
    - R17: the comments on `Group`, `Placement` and `place` speak of the team page.
    - `training::first_training_date` and the old `proposed_start` go (with their tests).
  - Server:
    - `error.rs`: `AppError::Refused { status: StatusCode, code: &'static str, details: serde_json::Value }`, answered as `{ "error": code, …details }`; the OpenAPI schema `Refusal { error, date?, roles? }`.
    - `From<PlayerError>`: `SameRole` → 400 `same_role`; `RoleStartsLater(d)` → 400 `role_starts_later` with `"date": "YYYY-MM-DD"`; `ChangedMeanwhile` → 409 `changed_meanwhile`; the others as now.
    - `api/team.rs`: `role_change` calls the new `change_role` with all roles of the player, under the lock as now; `covering` goes (`team_leave` keeps a local search until step 6).
    - `api/players.rs`: `proposed_from` becomes `proposed_start(season)` (no read of the weekly trainings). `TeamPageView::proposed_from` documents 1 September; `MemberView` gets `last_day_today`.
  - `client.ts`: `ApiError` gets `code` and `body`; the messages "The player already has this role." (`same_role`), "This player's role starts on Mon 2 Nov. To change it, use "Correct" in the history." (`role_starts_later`, the date with `formatDay`), "This player was changed in the meantime. Use "Correct" in the history." (`changed_meanwhile`).
  - `just api-types`.
- **Tests**:
  - Core (season 2026-2027, fixed dates):
    - `remove_from_team`: one role from Tue 1 Sep, from Thu 15 Oct → ended Wed 14 Oct, nothing removed; with a planned role from Sun 1 Nov → that role removed too; one role from Tue 1 Sep, from Sat 15 Aug → nothing ended, the role removed; a role that starts on the date → removed; a role until Wed 14 Oct, from Thu 15 Oct, no later role → `NotInTeam`; from Wed 1 Jul 2026 → all roles of the season removed, a role of 2025-2026 untouched; from Thu 1 Jul 2027 → `OutsideSeason`.
    - `change_role`: only trains from Tue 1 Sep to plays games on Sun 1 Nov → ended Sat 31 Oct, new from Sun 1 Nov with the old planned end; the same kind → `SameRole`; a role from Mon 2 Nov, change on Mon 12 Oct → `RoleStartsLater(Mon 2 Nov)`; no role in the season → `NotInTeam`; left on Wed 14 Oct and back from Mon 2 Nov, change on Tue 20 Oct → `RoleStartsLater(Mon 2 Nov)`; on the role's start → `ChangeBeforeRoleStart`; a date in 2027-2028 → `OutsideSeason`; a role of 2025-2026 is never picked.
    - `undo_removal`: stored as the removal left it → `before`; "Add to the team" from Mon 2 Nov after the removal → `ChangedMeanwhile`; `before` stored again (a second "Undo") → `ChangedMeanwhile`; `before` with two overlapping roles → `RolesOverlap`.
    - `team`: last season's team Avery and Blake and a newcomer with a role → `last_season` still both; Avery with a role → `last_season` empty, Blake still the first candidate; a hidden player of last season's team with a role → `last_season` empty; `last_day_today`: a role until Thu 15 Oct on Thu 15 Oct → true; the same with a role from Fri 16 Oct → false; an open role on Wed 30 Jun 2027 → false; a role until Wed 30 Jun 2027 on that day → false.
    - `proposed_start(2026-2027)` → Tue 1 Sep 2026.
  - Server (`tests/team.rs`, clock 12 Oct 2026): `change_role_errors_have_their_codes` with `same_role`, `role_starts_later` (`"date": "2026-11-02"`) and `not_in_team`; `the_proposed_start_is_the_first_of_september` (also without weekly trainings); `last_seasons_team_stays_until_one_of_them_has_a_role`; `change_role_saves_both_or_neither`: a trigger made in the test database refuses an insert into `player_roles`; "Change role" answers 500 and the old role keeps its end. The weekly-trainings test of the proposed start goes.
- **Checks**: `just check`; `just api-types` (types checked against the code before the commit).

### Step 6: "Remove from the team" from a date
- **Goal**: "Not in the team from", with "Tomorrow", the confirmation that names the removed roles, and "Last day today".
- **Changes**:
  - Storage `players.rs`: `delete_role(tx, player, role) -> Result<bool, StoreError>`.
  - `POST /api/players/{id}/team-leave`: body `LeaveTeam { from, removes: Vec<i64> }` (the first day without the team; the ids of the confirmed roles); answer `LeftTeam { team: TeamPageView, undo: UndoRemoval { from, roles: Vec<RoleDatesView { role, from, until }> } }` (`roles`: the player's roles of the season before the removal). In one transaction: lock the player (404 when missing), `roles_of`, `remove_from_team`; a removed role whose id is not in `removes` → 409 `confirm_removal` with `"roles": [RoleView]` (all removed roles), nothing written; else update the ended role, delete the removed roles, commit. Ids in `removes` that are no longer removed are ignored. Log "player left the team" (id only). The local search of step 5 goes.
  - `client.ts`: `leaveTeam(id, from, removes)` → `LeftTeam`.
  - `/team`: "Remove from the team" has the field "Not in the team from" (today) with a button "Tomorrow" (`dayAfter(today)`). The save sends `removes: []`; on `confirm_removal` the page asks `This also removes "Plays games · from Sun 1 Nov".` (all roles named, joined with "and") and on OK saves again with their ids; on cancel nothing happens. "Last day today" next to the badge when `last_day_today`. `currentRole` and `later` go.
- **Tests**:
  - Server (`tests/team.rs`): `remove_from_today_puts_the_player_under_left_at_once` (stored until Sun 11 Oct, under `left`); `remove_with_a_later_role_asks_first` (R13: 409 with the role from Sun 1 Nov and nothing changed; with its id: ended Sun 11 Oct, the later role gone, under `left`); `remove_after_add_all_leaves_nothing_in_the_season` (clock 15 Aug, roles from Tue 1 Sep, removal from today with the id: no roles in the season, in no group, a candidate again); `the_last_day_today_is_shown` (removal from Tue 13 Oct: in `members` with `last_day_today`); `remove_from_the_team_needs_a_role_on_that_date` (adapted: `not_in_team`); `remove_from_the_team_waits_for_the_players_lock` (the other connection ends the role on Thu 15 Oct and holds the lock; the removal from Sun 1 Nov waits 300 ms, then answers `not_in_team`; fails without the lock); `remove_saves_all_or_nothing` (a trigger refuses the delete; 500, the ended role keeps its end). `remove_from_the_team_and_add_again` adapted to the new body.
  - Browser: "Remove from the team" proposes today; "Tomorrow" sets Tue 13 Oct; after the save the row shows "Last day today"; a removal from today puts the player under "Left the team" at once; a player with a planned "Change role": the dialog names "Plays games · from Sun 1 Nov", after OK the player is under "Left the team"; no sideways scroll at 320 pixels, `expectAccessible`.

### Step 7: "Undo"
- **Goal**: a removal is undone with one tap while the team page is open.
- **Changes**:
  - `POST /api/players/{id}/team-undo` (operation `undoRemoval`): body `UndoRemoval { from, roles }`; answer `TeamPageView`. In one transaction: lock the player (404), `roles_of`, `undo_removal`; then delete the player's roles of the season and insert `before` (new ids; nothing refers to a role id); commit. Log "removal undone". Errors: 409 `changed_meanwhile`, 400 `roles_overlap`, `outside_season`.
  - `client.ts`: `undoRemoval(id, undo)`.
  - `/team`: a map in the page from player id to the `undo` of the last removal. "Undo" on the player's row (in the team or under "Left the team"); a player in no group gets a line below the team: "Blake · not in the team this season · Undo". An entry goes after its "Undo" (saved or refused) and after another action on that player; all go when the coordinator leaves the page (page state only).
- **Tests**:
  - Server: `undo_puts_the_roles_back` (a removal with a later role, then "Undo" with the `undo` of the answer: the same kinds and dates, in `members`); `undo_after_another_change_is_refused` (removal, "Add to the team" from Mon 2 Nov, "Undo" → 409, roles as they are); `a_second_undo_is_refused`; `undo_waits_for_the_players_lock` (the other connection adds a role and holds the lock; "Undo" waits, then 409); `undo_saves_all_or_nothing` (a trigger refuses the insert; 500, the roles as the removal left them); `team-undo` in the 401 and 403 test; the log test sees "removal undone" and no name.
  - Browser: "Undo" on a row after a removal brings both roles back (the player with the planned "Change role"); "Undo" without a row: "Add all" (replaced answers), removal of all roles, the line "· not in the team this season · Undo"; "Change role" to the same kind shows "The player already has this role."; no sideways scroll at 320 pixels, `expectAccessible`.

### Step 8: "Add all" with a date and the "Today" buttons
- **Goal**: "In the team from" of "Add player" and "Add all" is 1 September with "Today"; "Add all" asks the date.
- **Changes**:
  - `POST /api/players/carry`: body `CarryRequest { players, from }`. The ids are sorted before the locks are taken (two "Add all" at once cannot block each other). For each player in `last_season`: `check_role(season, from, None, roles)`, insert. One refusal answers its code and undoes the whole save (one transaction).
  - `client.ts`: `carryPlayers(ids, from)`.
  - `/team`: "Add all" opens "In the team from" (`proposed_from`, 1 September) with "Today" and a save button; "Add player" gets "Today" next to "In the team from".
- **Tests**:
  - Server: `add_all_uses_the_given_date` (from Mon 12 Oct: the roles start Mon 12 Oct; a date in 2025-2026 → 400 `outside_season`, no player added); `add_all_waits_for_the_players_lock` (the other connection gives the player a role in the season and holds the lock; "Add all" waits 300 ms, then skips that player: one role stored; fails without the lock); `add_all_brings_last_seasons_team_back` and the tests from summer with a date.
  - Browser: "Add player" proposes Tue 1 Sep and "Today" sets Mon 12 Oct; "Add all" (replaced answers) sends the date of the field.

### Step 9: Settings: a change of day moves a proposed window (R15)
- **Goal**: "Change" of a weekly training to another day keeps no window of the old day by accident.
- **Changes**: `/settings` keeps the day before the change. When the day changes and the form has a window, the page asks the proposal of the old day; when the window equals it, the window is "proposed" and the page asks and fills the new day's proposal. A typed window stays.
- **Tests**: Browser: add Wednesday with the proposed window (Wed 2 Sep – Wed 26 May 2027), "Change" only the day to Tuesday, save: listed as "Tuesday … · Tue 1 Sept – Tue 25 May"; a typed window (Wed 9 Sep – Wed 26 May 2027) stays when the day changes.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| The club has the players; a team exists for one season and is the team chosen from the game list for that season; a player is in the team through a role | The coordinator's model (2026-10-16) | yes, 0019 at Close |
| One club for the whole tool; players have no stored link to a club | One coordinator, the only user (coordinator, 2026-10-16: "as long as I'm the only user") | in 0019 |
| Roles and weekly trainings are stored by season, not linked to the team choice; they can be entered before the team is chosen | One team per season, so the season finds the team; attendance does not wait for a choice | in 0019 |
| Weekly trainings with a window only suggest training days | A mistake is corrected for the whole window (accept review A1) | in 0019 |
| "Change role" acts on the role that covers the given date; the new role keeps the planned end of the old one | The screen cannot change an old role by mistake; a planned end stays | no |
| No overlap of weekly trainings on one day is checked in the core under an advisory lock | An `EXCLUDE` constraint needs `btree_gist` on the cluster; the same as for roles | in 0019 |
| Migration `0005` is rewritten; "an applied file is never edited" (decision 0004) means applied on the cluster | The first form was never merged or released (coordinator, 2026-10-16) | one line in 0019 |
| `PUT /api/players/{id}` with the whole player (name and hidden), not `PATCH` | Like the first form's rename; the epic left the detail to the story | no |
| The schema name `AbsenceReason` | `Reason` is taken in the OpenAPI description | no |
| Deleting a player is always allowed in story A | Absences exist only from story B, which adds `player_has_absences` | no |
| "Remove from the team" means: from a date, no roles in the season; the role on the day before ends, later roles are removed | The coordinator's rule (2026-10-16, second reviews A6, A7, R13) | in 0019 |
| "Undo" sends the roles from before back; the core applies the removal to them again and compares with the stored roles under the player's lock | No stored undo state or version column; one rule in the core; a change from another screen is always seen (Phase 1, option A1, coordinator 2026-10-17) | in 0019 |
| The confirmation of a removal comes from a 409 of the server with the roles it would remove | The rule stays in the core (R14), and the question names what is really removed, also from an old page (option B1, coordinator 2026-10-17) | no |
| `AppError::Refused` with details (`date`, `roles`) next to `BadRequest` | Two messages of the spec need a date or a list of roles | no |
| "Undo" puts the roles back with new ids | Nothing refers to a role id; absences (story B) refer to the player and the training | no |
| The proposed "In the team from" of "Add player" and "Add all" is 1 September; the first training date is not used | Changed spec (sixth spec review, S27) | no |

## Not in scope
- Trainings, absences, the start page card, the list of training days, "Change" of a training, the proposal of last season's weekly trainings in a new season (story B); totals (story C).
- Weekly trainings or roles of a season other than the current one (shown in the history, not changed).
- Reordering reasons; a backup of the database (backlog).
- "Delete" for a role in the history (coordinator, 2026-10-16); stored undo data; "Undo" after leaving the team page.
- A test of two "Add all" saves that could block each other (the sorted locks are checked by reading the code).

## Test plan
- **Existing tests that must still pass**: all of `just check`; the browser tests of the start page (the link line must not move what they check).
- **First-form tests**: what stays, changes or goes is named in each step; each step's progress note lists the tests it removed and why.
- **New tests**: per step, above.
- **Checks**: `just check` after each step; `sqlx migrate run` and `just prepare` after step 1 and after each changed query (step 6: `delete_role`); `just api-types` after steps 1, 3, 4, 5, 6, 7 and 8.
- **Mutation checks** (steps 5–8): each new lock test fails without its lock; each "both or neither" test fails when the writes are not in one transaction.

## Compatibility and performance
- The migration adds tables only; the running app works on it (decision 0004). Local databases that ran the first `0005` are made again before step 1.
- The start page gets one line of links; nothing else changes for the games.
- The coordinator stores names of players (children) on the cluster: no names in logs (test), no backup yet (accepted, backlog).
- Load: about 15 players, a few roles each season, one or two weekly trainings (about 40 training days).
- From step 6, "Remove from the team" deletes stored roles (those that start on the date or later). "Undo" while the page is open puts them back; later only "Correct" and "Add to the team". No absences exist yet (story B), so nothing else is lost.

## Progress
- [x] First form: steps 1–5, fixes 1–5, build review twice, accept review (2026-10-15/16; replaced by this design)
- [x] Step 1: weekly trainings, end to end (2026-10-16; `just check` passes apart from `api-types-check` before the commit, the types checked against the code; 39 browser tests)
- [x] Step 2: core rules for players (2026-10-16; `just check` passes, 39 browser tests)
- [x] Step 3: the team page (2026-10-16; `just check` passes apart from `api-types-check` before the commit, the types checked against the code; 42 browser tests)
- [x] Step 4: the players page and the player page (2026-10-16; `just check` passes apart from `api-types-check` before the commit, the types checked against the code; 40 browser tests)
- [ ] Step 5: core rules and "Change role"
- [ ] Step 6: "Remove from the team" from a date
- [ ] Step 7: "Undo"
- [ ] Step 8: "Add all" with a date and the "Today" buttons
- [ ] Step 9: settings, a change of day moves a proposed window (R15)
- [ ] Review build
- [ ] Review accept

## Deviations
None from `functional.md` (as approved again on 2026-10-16, with the two additions of this design
written into it: the team page heading with the team name, and "Team not chosen yet").

The design reads the spec more strictly within its words:
- A weekly training's window is inside one season, and every save is for the current season.
- A window without its day of the week in it is allowed; it suggests no training days.
- "Change role" acts on the role that covers the given date; the new role keeps the planned end of the old role. (Steps 3–4 also had "Remove from the team" act on one role; steps 5–9 follow the changed spec.)
- "Add player" from the list offers players in "Players of last season" and "Other players" (not hidden); a player under "Left the team" is added again with "Add to the team".
- (Steps 5–9, coordinator 2026-10-17) Every role that "Remove from the team" deletes needs the confirmation, also a role that starts on the date itself; the spec names "a role that starts after the date". Only a removal that only ends a role is saved at once.
- (Steps 5–9, coordinator 2026-10-17) A player whose roles of the season were all removed has no row; its "Undo" is on a line below the team: "Blake · not in the team this season · Undo".
- (Steps 5–9) A removal whose date is the day after the role's last day ends nothing and removes nothing, so it is refused as "not in the team on that date".
- (Steps 5–9) "Players of last season" goes also when a hidden player of last season's team has a role in the season.

Implement:
- 2026-10-16, step 1:
  - `just db-down` keeps the database volume, so it resets nothing; the local database was made again with `sqlx database drop --force`, `sqlx database create` and `sqlx migrate run` (it held no data; `--force` ended the editor's idle connection).
  - The step 1 test "the proposed start is Tue 1 Sep on 12 Oct with a Tuesday weekly training" contradicts the step itself (the rule with "a training is opened" comes in step 2; until then the proposal is the later of the first training date and today). Step 1 tests on 15 Aug instead (1 Sep with a Tuesday weekly training, today without); the case on 12 Oct moves to step 2.
  - Removed first-form tests: `tests/settings.rs` (9 tests of the training-day sets and their API; replaced by `tests/weekly_trainings.rs`, 7 tests), the 2 set tests in `tests/players_storage.rs` (replaced by one test of a weekly training), the core tests of the sets, `proposed_dates` and `settings_change` (replaced by 7 tests of weekly trainings), and the browser test of the training days (replaced by the test of weekly trainings).
  - Dates on the settings page use the tool's format (`formatDay`, as on the schedule: "Tue 1 Sept").
  - The browser test of weekly trainings expects the one console message of the refused second Thursday (a 400 answer).
- 2026-10-16, step 2:
  - The struct of the team page is `TeamPage`, not `Team`: the core already has `team_choice::Team` (the game list team).
  - Storage writes `hidden` with its own function `players::set_hidden`; step 4 uses it for `PUT /api/players/{id}`.
  - Until step 4, `GET /api/players` keeps its first-form groups, built from `team`: "others" are the candidates that are not in last season's team, so a hidden player without a role is no longer listed there (nothing can hide a player on the screens yet).
  - Removed first-form tests: the core test of `proposed_start` with two inputs (replaced by the test with three) and the two tests of `roles_to_carry` (moved to `team(…).last_season`, same cases).
  - Added beyond the step's list: the server tests `add_all_leaves_out_a_hidden_player` and `a_player_is_not_hidden_until_it_is_set`, and the case of 12 Oct moved from step 1. The hidden tests fail without the rule.
- 2026-10-16, step 3:
  - Schema names: the team page is `TeamPageView` (the club API already has `TeamView`, a game list team); the bodies are `ChangeRole` and `LeaveTeam`; the operation of "Change role" is `changeRoleFrom` (`changeRole` is "Correct", `PUT /api/players/{id}/roles/{role}`, until step 4).
  - The handlers are in `api/team.rs`; `POST /api/players/{id}/roles` and `POST /api/players/carry` stay in `api/players.rs` and answer `TeamPageView`. The first-form pages `/players` and `/players/[id]` ignore these answers and load again, so they keep working until step 4.
  - The test of a refused "Change role" (`roles_overlap`) writes the later role from Sun 15 Nov through storage, past the checks: the new role lies inside the old one and stored roles never overlap, so the screens cannot make this case. It shows that nothing is stored when the check refuses.
  - The lock test: the other connection ends the role on Thu 15 Oct and holds the lock; "Change role" on Sun 1 Nov waits, then answers `not_in_team`. Without the lock the test fails.
  - Moved from `tests/players.rs` to `tests/team.rs`, on `GET /api/team`: the "Add all" test (with the hidden player of step 2 in it), the two R1 tests, and "last season's players join during the season". The test helper `Coordinator::add_player` replaces the local `add`.
  - The team page has no "Add to this season" for one player of last season (the first form had it): "Add player" offers last season's team first. "Players of last season" shows each player's role.
  - The browser tests of the team page are in `web/e2e/players.e2e.ts` (one database, in order); the test adds a Tuesday weekly training again, because the settings test deletes its own.
- 2026-10-16, step 4:
  - Operation names: with the first-form "Correct" route renamed, "Change role" (`POST /api/players/{id}/role-change`) is now `changeRole` (step 3 had `changeRoleFrom`); "Correct" is `correctRole`; `PUT /api/players/{id}` is `changePlayer`; `GET /api/players/{id}` is `player`. The body of `PUT /api/players/{id}` is `PlayerChange`.
  - `POST /api/players` keeps its body; its answer `{ player, same_name }` now carries `PlayerDetail` (the first-form `PlayerView` is gone).
  - `PUT /api/players/{id}` saves the name and "Hidden" in one transaction with `players::update_name` and `players::set_hidden` (the function of step 2).
  - `role_today` is the role on today only; a player whose role starts later shows "Not in the team", as the spec words it.
  - The history shows the season after a role of an earlier season ("· 2025-2026"); in "Correct", an empty "Until" means the role continues. The players page shows the warning of the same name on the page itself.
  - Removed first-form tests: in `tests/players.rs` the tests of the groups (`a_player_without_a_role_is_under_others`, `left_the_team_and_undo`, `back_in_the_team_is_a_new_role`; the team page tests of step 3 cover them); in the browser the tests of the first-form player page ("Left the team" and "Undo", its proposal, the "Add to the team" of a player of last season) and "Add all" on `/players` (now on `/team`). The weekly trainings test of the proposed start reads `GET /api/team`.
  - The browser test of "Correct" uses the player of the team page test (the tests run in order on one database).

## Review findings of the first form
Kept as history; the dated lines say what was fixed in the first form. The new form is reviewed
again under **Review findings** at the end.

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

### Build review again, 2026-10-16
By a fresh agent (`/review build`). The arguments held only the mode and the change folder.
Read: the diff `main...feat/players-and-settings` (905a767 and the fix commit ff065be; nothing
uncommitted; `main` is one docs commit ahead, a9aec5d, which this branch does not touch), this
folder, the epic's `functional.md`, `docs/architecture.md`, the decision records, `CLAUDE.md`,
`docs/language.md`. Ran in the repository: `cargo fmt --check`, `cargo clippy --workspace
--all-targets -D warnings`, `cargo test --workspace`, `cargo sqlx prepare --workspace --check`,
`npm run check && npm run lint && npm run build`, `npx playwright test` (39 passed), and
`team openapi` compared with `web/openapi.json` (same); all pass. Probes and mutations ran in a
copy of the tree (`git archive HEAD`) in the scratchpad; nothing in the repository changed. (The
copy shared the repository's `target/`; a mutated `team_core` was left in it, so the stale
artifacts of `team_core` and `team` were removed with `cargo clean -p` and everything above was
run again on a clean build.)

**The fixes of R1–R9: checked.**
- R1: run. With the old meaning of an open role (`last_day` = `NaiveDate::MAX`) the two new API
  tests fail; with the fix they pass. `last_day` is now the one meaning in `covers`, `check_role`
  and `stayed_until_end`.
- R2, R4: trace. `one()` and the list both go through core `place`; `proposed_start` is in the
  core; `api/players.rs` loads and maps.
- R3: run. With the old query (`valid_from = $2`) `before_the_season_the_last_save_wins` and
  `a_save_removes_the_sets_from_its_date_on` fail. But see R10: the fix covers only sets on or
  after the new date.
- R5: run. Without `players::lock` in `save_role` the test failed 3 of 3 runs.
- R9 (two saves at once): run. Without `training_settings::lock` `a_save_waits_for_another_save`
  fails.
- R6, R7, R8, R9 (other points): trace through the screens and `client.ts`; the browser tests for
  them pass. The new 404 sentence also replaces the club sentence for `confirm-gone`, which is
  better there.

**R10 – a save of the settings that moves the first training date later keeps an older set in
force on the screen, and a save of the screen as shown brings it back (bug, medium; sibling of
R3). Run.** The save removes only the sets valid from the new set's date or later. When the new
date is later than an older set's date and later than today, that older set stays. Before the
season (clock 15 Aug 2026): save Tuesday with first 1 Sep, then Thursday with first 8 Sep. The
answer of the second save, and `GET`, give **Tuesday**: `set_on(today)` finds no set on or before
15 Aug and takes the first set (1 Sep, Tuesday, which `training_dates` never uses any more, since
the season now starts on 8 Sep). The settings page puts the answer in its fields, so right after
saving Thursday it shows Tuesday. Saving the page as shown (for example only the last date
changed) stores Tuesday from 8 Sep: on 10 Sep `GET` gives Tuesday; the Thursday change is lost
without a sign. The same in the season: on 12 Oct with Tuesday from 1 Sep, a save of Thursday
with first 2 Nov answers Tuesday. The spec: a change applies from the day it is made, earlier
dates keep the days set then; here a set that applies to no date is shown and wins. The test of
R3 checks `GET` only on 10 Sep, after the new date, so it cannot see this. Suggest: one rule in the
core for which stored sets a save replaces (all sets that start after today, as well as those
from the new date on, since no date before `first` is a training date), and let `GET` show the set
of `max(today, first)`; a test of the probe above that checks the answer of the save and `GET`
before the first date.
- 2026-10-16: fixed. Core `training::replaced_from` (a save replaces the stored sets from its own date on and every set that starts after today) and `training::shown_set` (the set of today, or of the first date when that is later); `training_settings::save` takes the date from which it replaces. Core tests of both; API tests `a_first_date_moved_later_shows_and_keeps_the_last_save` (the probe, with the answer, `GET` before the first date, and a save of the page as shown) and `in_the_season_a_first_date_moved_later_keeps_the_past`. Both API tests fail with the old rule.

**R11 – "Add to the team" on the player page proposes today, not the proposed start (bug,
low). Trace.** `players/[id]/+page.svelte` `startReturning` sets the date to `today` for a player
with no roles in the season ("Add to the team"), while "Add player" and "Add to this season" use
`proposed_from` (the first training date, or today when trainings have started). The spec
(story 1): "the tool proposes the first training date of the season, or today when the season's
trainings have started". In August a player from "Other players", or a player of last season
opened from a link, gets a role from today and not from 1 Sep. For "Back in the team" (the
player had a role earlier in the season) today is the right proposal. Suggest: `proposed_from`
when the player has no roles in the season, `today` otherwise.
- 2026-10-16: fixed. "Add to the team" (no roles in the season) proposes `proposed_from`; "Back in the team" still proposes today. The browser test of a player of last season checks the proposed date (replaced answer, 3 Nov).

**R12 – small rules still in the server (decision 0001, low). Trace.** "A role of another season
cannot be changed" (`stored.season != season` in `save_role`) is a product rule outside
`check_role`; and "the roles of this season and the season before" is filtered twice in
`api/players.rs` (`roles_around` and `carry`). `carry` inserts a role without `check_role`; that is
safe today (`roles_to_carry` asks for no role in the season, and an open role of last season ends
on 30 June), but it is a second path that writes a role without the one check. Suggest: move the
season rule into `check_role` (for example with the season of the changed role) and let `carry`
call `check_role` too.
- 2026-10-16: fixed. Core `players::check_change` (only a role of the season is changed, then `check_role`), with a unit test; `carry` calls `check_role` before it writes; one function `roles_around` gives the roles of this season and the season before, for `one()` and `carry`.

**Checked and found right.** Every new route still needs a login (tests for 401 and 403 pass);
logs carry ids only (`player_logs` passes); no `unwrap`/`expect` outside tests (clippy);
`.sqlx/` matches the queries (`cargo sqlx prepare --check`); the migration only adds tables;
`web/openapi.json` matches the code. The claims in **Progress** and the fix notes under **Review
findings** (each new test fails without its fix) are true where they were run (R1, R3, R5, R9).

**Gap in the plan, not in this build.** Story 2 of the epic: "A new season asks for its training
days and dates again, with the days of last season and new proposed dates filled in." Neither
this story nor the epic's split names a step for it; the settings page of a new season starts
empty. For `/review accept` or the design of story B.

**Docs made false.** None by the fixes. At Close, as before: `docs/architecture.md` (core
`players::place`, `proposed_start`, `Season::previous`/`first_day`/`last_day`; the server
modules, migration `0005`, the routes, the pages), the `CLAUDE.md` module list, and decision 0019.

**Verdict:** not ready to close. R1–R9 are fixed and their tests guard them. R10 is the same bug
as R3 on the other side of the new date, and loses a change of training days without a sign; fix
it before Close. R11 and R12 are small and can go with it.

### Accept review, 2026-10-16
By a fresh agent (`/review accept`). The arguments held only the mode and the change folder.
Read: the epic's `functional.md` (stories 1, 2 settings part, 4; the Interface and the warnings)
and `idea.md`, this folder's `change.md` and the objective and requirements here, the diff
`main...HEAD` (905a767, ff065be) and the screens. The view below was formed before reading
**Deviations** and the earlier findings. Ran on HEAD: `cargo test -p team_core` (135 passed) and
the four new server test files (31 passed); one probe as an extra test in a copy of the tree
(`git archive HEAD`, own `CARGO_TARGET_DIR`, in the scratchpad), against the local PostgreSQL.
Nothing in the repository was changed. The browser tests were not run (the second build review ran
them on the same commit). During this review the working tree held uncommitted fixes of R10–R12;
they were read where they touch a finding (A1), not judged.

**A1 – a mistake in the training days, made during the season, can never be corrected; the
screen hides it (concerning deviation). Run.** The tool starts now, in October, and the
coordinator enters the trainings since 1 September (functional, Context). The first save applies
from the first training date (deviation of step 3), but every later save applies from today. So a
mistake in the first save, corrected a minute later, stays in force for all earlier dates. Probe,
clock 12 Oct 2026: save Wednesday 19:00–21:00 from 1 Sep, then Tuesday 19:45–21:00 on the same
day. Stored sets `[(1 Sep, Wed), (12 Oct, Tue)]`; `training_dates` gives Wed 2 Sep … Wed 7 Oct at
19:00, then Tue 13 Oct at 19:45. The answer of the save, and the page, show Tuesday 19:45, so it
looks corrected. No screen shows the earlier set, and no save can change it: the first date is the
only lever, and moving it changes the season, not the days. Story B builds the list of past
training days and "N training days not opened" (story 5) on these sets, so the list the
coordinator works through from the sheet would show six Wednesdays. The uncommitted R10 fix
(`replaced_from` = the earlier of tomorrow and the set's date) gives the same result in the
season. The spec's words are met ("the change applies from that day on"), but they are about a
real change of schedule; the spec leaves open how a *mistake* is corrected, and its own warning
says "every mistake must be correctable in the app" (Steve). That is a product choice, not a
build fix: for example "a correction applies to the whole season" next to "a change from today",
or saves on the day of the first save replace it.

**A2 – the start and the kind of a role cannot be corrected on the screens (concerning
deviation). Search.** `changeRole(` in `web/src/routes` is called only by "Left the team" and
"Undo", both with the stored `from` and `role`; `addRole(` only by "Back in the team" / "Add to the
team". The API can change both (`PUT /api/players/{id}/roles/{role}`). Story 1 expects a start date
to move ("A role starts (or its start is moved earlier) …", and the answer to S8: "Moving 'in the
team since' or the date a player left keeps the absences"). A player added from the proposed date
who joined on 3 Nov, or added as "Plays games" by mistake, can now only be deleted and added again.
That works in story A, but from story B on a player with absences cannot be deleted, so the mistake
stays and counts in the totals. No story of the epic names this screen. Needs a place: this story,
or a line in story B's scope (with B's message "This player now counts as there at N earlier
trainings").

**A3 – a new season does not fill in last season's training days (neutral drift, unassigned).
Trace.** Story 2: "A new season … asks for its training days and dates again, with the days of
last season and new proposed dates filled in." In a new season `GET /api/training-settings` gives
`saved: false` and no days, so the page starts empty (the dates follow the proposal once a day is
chosen). The build review already noted that no story has a step for it. Harmless once a year for
one Tuesday, but it must be given to story B or the backlog, or it is lost at Close.

**A4 – navigation from the start page (neutral drift). Trace.** The spec: the start page links to
"Attendance", and "Attendance" links to "Players" and the settings. Story A puts "Players ·
Settings" on the start page, since "Attendance" does not exist yet. Right for now; story C should
decide whether the line stays or moves.

**A5 – the settings page says "A change applies from today; earlier trainings keep their days."
(neutral drift, small). Trace.** Before the first training date, and for the first save, a change
applies from the first training date. The sentence is wrong in exactly the case the coordinator
meets first (setting up in October, entering the past). Follows the answer to A1.

**Faithful adaptations** (trace): one training time for each day of the week; players and
settings only for the current season (the spec says "the chosen season"; earlier seasons come with
story C's overview); "Other players" (folded) for players with no role in this or last season;
"Players of last season" lists only players who stayed until 30 June, the ones "Add all" brings
back; a role's end is its last day, so a change of role "on the same day" is an end on the 14th and
a start on the 15th, which the data allows; a player whose role starts later in the season is shown
with the team; the same-name warning after the save; "Left the team" proposes today, or the role's
start when later; the 404 sentence. R11 (the proposed start of "Add to the team") is a real
difference from story 1 and is already a build finding.

**Checked and found as meant** (trace through the screens, the API and the tests that ran): first
name required, last name optional, names trimmed, shown by name only and sorted by first name
(`ORDER BY lower(first_name)`); a role from a proposed date (first training date, or today when
later), "Left the team" with a date and "Undo", a new role on return, no overlap ("This player
already has a role on that date."), at the latest 30 June; "Add all" and "Add to this season" with
the same kind of role from the first training date or today; "Delete player" after a confirmation;
training days with start and end time, more than one day, the proposal of the first Tuesday of
September and the last of May, the errors for times and dates; the six reasons in order and with
their weight, add (last), rename, valid / not valid, hide and show again, unique names without case.

**Verdict: needs correction** (next: `/discover review:`). A1 needs a product rule for correcting
the training days during the season before story B builds the list of past trainings on them, and
the tool starts in the season now. A2 needs a place in this story or in story B's scope. A3 needs an
owner. The rest of story A is what was meant.

## Review findings

### Accept review, 2026-10-16 (second form)
By a fresh agent (`/review accept`). The arguments held only the mode and the change folder.
Read: the epic's `functional.md` (all of it; stories 1, 2 for the weekly trainings, 4, the
Interface and the warnings) and `idea.md`, this folder's `change.md`, the objective and
requirements here, the diff `main...HEAD` (up to df54748) and the screens' code. The view below was
formed before reading **Deviations** and the earlier findings. Ran on HEAD: `cargo test -p team
--test team --test players --test weekly_trainings` (30 passed), and the server itself
(`DEV_AUTH_BYPASS`, `DEV_NOW` 15 Aug 2026, own port) against a scratch database made for this
review and dropped after it, driven with `curl` on the API. Nothing in the repository was changed.
The browser tests were not run. The numbers go on from the first form's A1–A5, which `change.md`
and `functional.md` cite.

The first form's findings are answered: A1 by weekly trainings (a change corrects the whole window;
run: the proposal for Tuesday is Tue 1 Sep – Tue 25 May 2027), A2 by "Correct" in the history, A3
moved to story B (functional story 2), A5 by the new sentence on the settings page (trace).

**A6 – "Remove from the team" on the proposed date leaves the player in the team (concerning
deviation). Run.** "Remove from the team" proposes today and ends the role on that date, so the
role still covers today. The team page puts a player with a role today in the team, so after the
save the row stays where it was, with the same badge; "Left the team" shows the player only from
tomorrow. Run: Casey in the team from 1 Aug, `team-leave` with `until` 15 Aug (today, the proposal)
→ Casey under `members`, not `left`. The browser test of "Remove from the team" checks the proposal
(today) and then types 1 Oct before it saves, so it never sees this. Each rule is as the spec words
it ("ends the role on that date"; "Left the team" is "none today or later"), but the coordinator who
taps "Remove from the team" and sees nothing change will think it was not saved, and may tap again.
A product choice for `/discover`: for example the row shows "Last day today" (or "until Thu 15
Oct"), or a role that ends today counts as left on the team page, or the proposal is yesterday.
- 2026-10-17: designed (story 1 changed, approved 2026-10-16): "Not in the team from" (the first day without the team) with "Tomorrow", and "Last day today"; steps 5 and 6.

**A7 – a role added by mistake cannot be taken away; "Add all" followed by "remove those who did
not come back" keeps them in the team on the first training date (concerning deviation). Run and
search.** No route or screen deletes a role (search `delete(` in `app.rs`: only players and
weekly trainings; `DELETE FROM player_roles` in `crates/server/src`: none; "Correct" refuses an end before the start: run, `until_before_from`). "Remove
from the team" needs a role that covers the date (`covering`), so its earliest end is the role's
start. Run, clock 15 Aug 2026, Tuesday weekly training from 1 Sep: "Add all" gives Avery and Blake
a role from Tue 1 Sep; "Remove from the team" for Blake on 31 Aug → `not_in_team`; on 1 Sep (the
proposal: the start of the role, which is later than today) → a role Tue 1 Sep – Tue 1 Sep, and
Blake stays under `members` (a role later in the season). So the spec's own behaviour ("Next
season, the coordinator taps 'Add all' … and removes those who did not come back") leaves every
player who did not come back in the team on the first training: from story B they show at that
training and count as "there", and later under "Left the team" in the totals. The same holds for a
wrong player picked in "Add player" (the choice opens on the first player of the list, not on "New
player": trace, `startAdding`). The only full undo is "Delete player", which also deletes the
player's roles of every earlier season, and from story B is refused once the player has an
absence. The spec covers the reverse mistake ("A mistake is undone in the history ('Correct', end
date empty)") but not this one, against Steve's warning "every mistake must be correctable in the
app". Needs a rule, for example: "Remove from the team" on or before the start of a role removes
the role, or "Delete" in the history for a role of the current season.
- 2026-10-17: designed: a removal deletes the roles from its date on, with a confirmation, and "Undo" while the page is open; steps 5, 6 and 7.

**A8 – "Change role" for a player whose role starts later proposes a date that fails (neutral
drift). Run and trace.** After "Add all" in summer each role starts on the first training date.
"Change role" proposes today; `role-change` looks for the role that covers that date and answers
"This player is not in the team on that date." The coordinator must type a date in the role or use
"Correct" (the right tool here); the message names neither. Small; a proposal of the later of
today and the role's start (as "Remove from the team" does) would fit.
- 2026-10-17: designed: "Change role" on a date before a later role answers `role_starts_later` with that date ("… use "Correct" in the history."); step 5.

**A9 – a hidden player under "Left the team" is offered "Add to the team" (neutral drift).
Trace.** The core puts every player with an earlier role of the season under "Left the team",
hidden or not, and the page shows "Add to the team" for each. Story 1: a hidden player "is not
offered when a player is added to the team". Defensible (the coordinator taps that player on
purpose), but the spec's words say otherwise; `/discover` or Close can settle which.
- 2026-10-17: settled in the spec (hidden only keeps lists short; the player keeps "Add to the team"); built so; no change.

**A10 – "Players of last season" with "Add all" stays for the whole season (neutral drift).
Trace.** The group holds last season's players with no role in this season, so after the
coordinator adds players one by one, the players who did not come back stay listed above the team
with "Add all" until each is added. The spec shows it "in a new season". Harmless alone; with A7 a
stray tap on "Add all" in January cannot be undone.
- 2026-10-17: designed: the group shows while no player of last season's team has a role in the season, and "Add all" asks the date; steps 5 and 8.

**Faithful adaptations** (trace): the line "Team · Players · Settings" on the start page, also
before a team is chosen, while "Attendance" does not exist (first form A4); "Add player" on the
players page takes a name only, the team page gives the role; the players page shows a role only
when it covers today, so a player whose role starts later is "Not in the team" there and in the
team on the team page (as the spec words both); the history names the season after a role of an
earlier season; "Delete" of a weekly training and of a player asks first; dates in the tool's
format ("Tue 1 Sept", as on the schedule).

**Checked and found as meant**: the team heading "U14-1 · 2026-2027", or "Team not chosen yet"
with a link to choose it, and players can be added before (trace; API test); players in the team
today or later, then "Left the team", and a player after "Change role" stays in the team (core and
API tests ran); "Change role" ends the old role the day before and keeps both, in one transaction
(trace, test ran); the proposed start is the first training date until a training is opened, today
without a weekly training (run: 1 Sep on 15 Aug; core test); "Add all" gives the same role and
leaves out hidden players (test ran); "Add to the team" proposes the day after the end or today
(trace); first name required, last name optional, sorted by first name, shown by full name, the
same-name warning (trace, tests); "Hidden" and "Show hidden players"; "Correct" only for a role of
the current season, with the checks of a new role (tests ran); weekly trainings: add, the proposed
window, "Change", "Delete", inside the season, no overlap on one day ("There is already a weekly
training on Tuesday in these dates."), the list sorted by the window's start as "Tuesday
19:45–21:00 · Tue 1 Sept – Tue 25 May" (run, trace); the reasons as in the first form; the error
sentences of story 1 and 2 word for word (search in `client.ts`).

**Verdict: needs correction** (next: `/discover review:`). A6 and A7 need product rules: what the
team page shows on the last day of a role, and how a role added by mistake is taken away (this
must be settled before story B, where it starts to count). A8 can go with them; A9 and A10 can be
settled at the same time or at Close.

### Build review, 2026-10-16 (second form)
By a fresh agent (`/review build`). The arguments held only the mode and the change folder.
Read: the diff `main...HEAD` (steps 1–4, 2baa9c9 to df54748; uncommitted only the accept review
above, which was read after the findings below were formed, so as not to repeat them), this
folder, the epic's `functional.md`, `docs/architecture.md`, the decision records, `CLAUDE.md`,
`docs/language.md`. Ran in a copy of the tree (`git archive HEAD`, own `CARGO_TARGET_DIR`, in the
scratchpad) against the local PostgreSQL: `cargo fmt --check`, `cargo clippy --workspace
--all-targets -D warnings`, `cargo test --workspace` (139 core tests, all server test files pass),
`team openapi` and `npm run api-types` compared with the committed files (same), `cargo sqlx
prepare --workspace --check` (passes); probes and mutations as extra tests there. In the
repository: `npm run check`, `npm run lint`, `npm run build` and `npx playwright test` (40 passed).
The local database has the rewritten `0005` (checksum of the file). Nothing in the repository was
changed.

**R13 – "Remove from the team" leaves a planned later role, so the player stays in the team (bug,
medium). Run.** `team_leave` ends only the role that covers `until`; a later role of the season is
kept. A planned "Change role" makes exactly that: on 12 Oct, Avery (only trains from Tue 1 Sep)
gets "Change role" to plays games from Sun 1 Nov, then "Remove from the team" on the proposed date
(today: `startLeave` takes the role that covers today). Stored: only trains 1 Sep – 12 Oct, plays
games from 1 Nov, still open. The answer, and `GET /api/team` on 20 Oct, list Avery under
`members`, not under "Left the team"; the players page shows "Not in the team". From 1 Nov Avery is
back in the team with no tap, and from story B shows and counts at every training. The spec: "A
player under 'Left the team' … none today or later in the season" and "Remove from the team …
ends the role on that date". The tests of "Remove from the team" use players with one role, so
they cannot see it. What happens to the later role (removed, or the save refused with a message)
is a product choice; it belongs with the rule A7 asks for (the same function, the same "a role
that should not exist").
- 2026-10-17: designed: the removal deletes the later roles after a confirmation that names them; steps 5 and 6.

**R14 – the rules of "Change role" and "Remove from the team" are in the server and the screen
(decision 0001, "New rules go in `crates/core` first"; low, the root of R13). Search and trace.**
Which role a date acts on (`covering` in `api/team.rs`: the role of the season that covers the
date, else `not_in_team`) and the whole of "Remove from the team" (end that role, check, nothing
about other roles) are written in the server; the core's `PlayerError::NotInTeam` is never made
(search `NotInTeam` in `crates/`: only the enum and its mapping to a code). The proposed dates of
"Remove from the team" and "Add to the team", and "the role today or the coming one"
(`currentRole`, a copy of `place`), are worked out in `team/+page.svelte`. This is the pattern of
the first form's R4 and R12 again. Suggest: a core function for "Remove from the team" (and one
that picks the role for "Change role") with unit tests, including a player with a later role; the
proposals in `TeamPageView` or `MemberView`, from the core.
- 2026-10-17: designed: `change_role`, `remove_from_team` and `undo_removal` in the core; `last_day_today` from the core; `currentRole` goes from the page; steps 5 and 6.

**R15 – "Change" of a weekly training to another day keeps the window of the old day, so the first
training day is lost without a sign (bug, medium). Trace.** In "Change" the form holds the stored
window and `windowProposed` is false, so `propose()` returns at once when the day is changed. The
spec's own case ("A Wednesday typed by mistake is corrected with 'Change'"): Wednesday with the
proposed window Wed 2 Sep – Wed 26 May, changed to Tuesday, is saved as Tuesday Wed 2 Sep – Wed 26
May; its training days start on Tue 8 Sep. `first_training_date`, and so the proposed start of
every new role and of "Add all", become 8 Sep, and story B's list will have no Tue 1 Sep. The list
line shows "Tuesday … · Wed 2 Sept – Wed 26 May", which is easy to miss on a phone. The browser
test of "Change" types both dates by hand, so it cannot see it. Suggest: when the day changes, move
a window that was the old day's proposal to the new day's proposal (or move each end to the new day
in the same week), with a browser test that changes only the day.
- 2026-10-17: designed: step 9.

**R16 – two locks are not guarded by a test; one test checks a state the rules exclude (test,
low). Run.** With `players::lock` removed from `team_leave` and from `carry` (in the copy), every
test passes except the 404 of an unknown player (the lock also finds the player). "Two taps at the
same time give one role" (`carry`) and the lock of "Remove from the team" have no test like
`change_role_waits_for_the_players_lock`, which does fail without its lock (run: the second request
then answers 200 and overwrites the other connection's end, 15 Oct, with 31 Oct). The claim of step
3, "Both or neither" for "Change role": `a_refused_change_of_role_changes_nothing` refuses in
`change_role`, before any write, on overlapping roles written past the checks (a state stored
roles never have, as the deviation says); the two writes in one transaction are right by trace,
not by a test.
- 2026-10-17: designed: lock tests for "Remove from the team", "Undo" and "Add all"; "both or neither" tests with a trigger in the test database; steps 5–8.

**R17 – smaller points (low).**
- "Change role" to the same kind is accepted (run): Avery only trains from 1 Sep, "Change role" to
  only trains on 1 Oct gives two equal roles 1 Sep – 30 Sep and from 1 Oct. The form proposes the
  other kind, so only by choice; the history then shows a change that is none.
- Sibling of A8: "Change role" proposes today also for a role that starts later; the API answers
  `not_in_team` (run, a role from 2 Nov, change on 12 Oct). Its twin "Remove from the team" has the
  later-start rule; this one does not (and needs the day after the start, since a change on the
  start is `change_before_role_start`).
- Comments made false by step 4: `Group` ("Where the players page lists a player"), `Placement`
  and `place` in `crates/core/src/players.rs` speak of the players page, which no longer has groups;
  they now serve the team page.
- 2026-10-17: designed: the same role is refused (`same_role`), the later start answers `role_starts_later`, the comments are fixed; step 5.

**Checked and found right.** Every new route is on the protected router and in the 401 and 403
test (test ran); logs carry ids only, and the log test covers "Change role" and `PUT` with a new
name (test ran); no `unwrap`/`expect` outside tests (clippy); the migration only adds tables, and
`weekly_trainings` has the checks of the design; the advisory lock key spells "team" "wktr" and
differs from the reading lock; `change_role_waits_for_the_players_lock` and the hidden tests
(`hiding_only_leaves_a_player_out_of_the_choices`, `add_all_brings_last_seasons_team_back`) fail
without their code (run, mutations); a weekly training of another season gives 404 on change and
delete (trace, test ran); `.sqlx/` and the API types match the code; the claims in **Progress**
(the checks, 40 browser tests) are true on HEAD.

**Docs made false.** None of the living documents by this diff; the stale comments are in R17. At
Close: `docs/architecture.md` (core `players`, `training`, `reasons`, `Season::previous` /
`first_day` / `last_day`; server `players.rs`, `reasons.rs`, `weekly_trainings.rs`, `api/players`,
`api/team`, `api/reasons`, `api/weekly_trainings`; migration `0005`; the pages `/team`, `/players`,
`/players/[id]`, `/settings` and the start page line), the module list in `CLAUDE.md`, a product
document for players and settings, and decision 0019 with the reading of decision 0004.

**Verdict:** not ready to close. R13 and R15 are bugs against the spec's intent that the tests do
not see: R13 keeps a removed player in the team (and from story B at its trainings), R15 drops the
first training day after the spec's own correction of a wrong day. R14 belongs with the fix of R13
(and with the rule A7 asks for); R16 and R17 are small and can go with them.
