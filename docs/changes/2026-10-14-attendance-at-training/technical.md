# Technical spec: Attendance at training

**Status**: approved
**Size**: epic
**Change**: docs/changes/2026-10-14-attendance-at-training
**Date**: 2026-10-15

## Objective
Bring the players into the tool and let the coordinator, who is also the coach, keep attendance at
training with a reason for each absence, in seconds on a phone, with season totals for each
player.

## Requirements summary
From `functional.md` (approved 2026-10-15 after two spec reviews, changed after approval for
player roles):

1. **Players** (story 1): a player is the person: a first name (required) and a last name (optional), no dates. A **role** in the team of a season, "plays games" or "only trains", has a start date and an end date (empty while it continues; at the latest 30 June, the end of its season). A player who leaves and comes back gets a new role; roles of one player never overlap. A new season starts with no roles; "Add to this season" and "Add all" give last season's players the same role again, from the first training date (or today). A player with no absences can be deleted. A change of role within a season must be possible in the stored data (a screen comes later). Warning, not error, for two players with the same name.
2. **Training days and trainings** (story 2): for each season, one or more days of the week with a start and an end time (for example Tuesday 19:45–21:00), and the first and the last training date (proposed: the first training day of September, the last of May). A change of training days applies from the day it is made; earlier dates keep the days set then. Opening a training makes it exist, with every player there. "Cancel training" marks it as cancelled (shown, not counted, absences kept); "Undo cancel". Only an extra training ("Add training", any date, own times) can be deleted. The start page card shows today's training until midnight, else the next one, also when cancelled ("Training · Tue 20 Oct · 19:45", "Cancelled"), only between the first and the last training date. A list of the whole season: "Coming" above "Past", each day "Not opened", "All there", "N absent" or "Cancelled", with "Cancel training" on a day that is not opened and a switch "Show cancelled" (on by default).
3. **Attendance** (story 3): a training shows the players with a role on its date, sorted by first name, all "there" at first. A tap on a player opens the reasons ("There" on top for an absent player); each tap is saved at once; a tap that was not saved shows the old state again with "Not saved. Try again." Past trainings can be opened and corrected. A cancelled training can still be changed.
4. **Reasons** (story 4): presets School, Ill, Injured (valid), Holiday, Work, No reason (not valid), in this order. Add (last in the list), rename, change valid / not valid (the totals follow), hide (not offered for new absences, still shown on old ones). Names are unique.
5. **Season totals** (story 5): for each player with a role in the season: there, absent with a valid reason, absent without one, of N trainings. Only trainings up to today, not cancelled, inside the player's roles. "Left the team" group. "N training days not opened" above the totals. Earlier seasons. A tap on a player shows the trainings the player missed. The season runs from 1 July to 30 June (Dutch time).
6. Dutch local dates and times; only the coordinator (login, decision 0007); phone first, no sideways scroll at 320 pixels.

Not in this epic: attendance at games, jersey number, birthday and association id, reading players from a sheet, trends, messages, late or early leave, the place of a training, more teams.

Decisions that apply: 0001 (rules in a core without I/O), 0002 (static Svelte front end, generated API types), 0004 (forward-only migrations, compile-time checked queries), 0007 (every `/api` route needs a session), 0014 (seasons as start years).

## Current state
- Core (`crates/core`): `Season` with `Season::of(date)` (1 July) and `start_year()`; `chrono` without a clock; validation errors as `thiserror` enums; rules tested with fixed dates.
- Server (`crates/server`): `clock::today_in_netherlands`, `clock::current_season`; `db.rs` (pool, migrations); `AppError` turns errors into `{"error": "<code>"}` (400 with a short code, 404, 500); routes on the protected `/api` router in `app.rs`; OpenAPI with `utoipa` (`api/mod.rs`), front-end types by `just api-types`.
- Migrations `0001`–`0004` (sessions, followed teams, games/places/readings, changes). No table for players or trainings.
- Front end (`web/`): SvelteKit static app; start page `/` (team header, "Game list" card, schedule), `/changes`, `/choose`; shadcn-svelte parts `button`, `badge`, `separator`, `toggle`, `toggle-group`; helpers in `src/lib/`.
- Tests: core unit tests; `#[sqlx::test]` (a new database per test); router harness `crates/server/tests/common/` (`Harness`, test clock, login); browser tests in `web/e2e/` (Playwright, phone-sized screen, server clock 12 October 2026, fresh database `team_e2e`).
- Backups: none arranged for the team database (`gitops-lab/applications/productivity/team/README.md`: "Configure and verify TrueNAS snapshots … before production use"). The coordinator accepted this for now (2026-10-15).

## Architecture (shared by all stories)

Stored data (each story adds its own migration):

| Table | Holds | Story |
|---|---|---|
| `players` | first name, last name (nullable). No dates. | A |
| `player_roles` | player, season (start year), role (`plays_games` / `only_trains`), `from` date, `until` date (nullable). Inside its season; no overlap per player. | A |
| `reasons` | name (unique, compared without case and outer spaces), valid, hidden, position. The migration adds the 6 presets. | A |
| `training_seasons` | season, first training date, last training date | A |
| `training_days` | season, valid from (date), day of the week, start time, end time. A change adds a new set with valid from = the day of the change; for a date the set with the newest valid from on or before it applies. | A |
| `trainings` | date, start time, end time, extra (bool), cancelled at (nullable). Unique on date and start time. | B |
| `absences` | training, player, reason. Primary key (training, player). | B |

"There" is not stored: a player is there at a training when the training exists, the player has a
role on its date, and there is no absence row. Totals are worked out at each request from the
stored rows (about 15 players, about 40 trainings a season).

Core (`crates/core`, new modules):
- `players`: `Player`, `Role` (`PlaysGames`, `OnlyTrains`), `PlayerRole`; `role_on(roles, player, date)`; `roles_to_carry(last_season_roles, training_season, today)`; validation (`first_name_required`, `until_before_from`, `roles_overlap`, `outside_season`).
- `training`: `TrainingDay`, `TrainingDaySet`, `TrainingSeason`, `Training`; `training_dates(season)` (each date with the set valid on it); the proposed first and last training date; `next_training(season, trainings, today)` (the card); validation (`end_before_start`, `last_before_first`).
- `attendance`: `Reason`, `Absence`; the status of each training day; `season_overview(season, trainings, absences, players, roles, reasons, today)` (totals, "Left the team", number not opened); validation of reasons (`reason_name_required`, `reason_name_taken`).

Server (`crates/server`): storage modules `players.rs`, `reasons.rs`, `training_days.rs`, `trainings.rs`; API files under `api/` for each group. API outline (details in each story's design):
- A: `GET/POST /api/players`, `PUT/DELETE /api/players/{id}`, role routes under `/api/players/{id}/roles`, `POST /api/players/carry` ("Add all" / "Add to this season"); `GET/POST /api/reasons`, `PUT /api/reasons/{id}`; `GET/PUT /api/training-settings/{season}`.
- B: `GET /api/trainings?season=` (the list of training days with status), `PUT /api/trainings/{date}` (open; `INSERT … ON CONFLICT DO NOTHING`, then the stored training; safe to send twice), `POST /api/trainings` (extra training), `POST /api/trainings/{id}/cancel` and `/uncancel`, `DELETE /api/trainings/{id}` (extra only), `PUT`/`DELETE /api/trainings/{id}/absences/{player}`, `GET /api/trainings/next` (the card).
- C: `GET /api/attendance?season=` (totals, "Left the team", number not opened; the missed trainings of each player).

Error codes (400, `{"error": "<code>"}`): `first_name_required`, `until_before_from`, `roles_overlap`, `outside_season`, `end_before_start`, `last_before_first`, `reason_name_required`, `reason_name_taken`, `reason_hidden`, `player_has_absences`, `not_extra`, `not_a_training_day`, `outside_training_dates`, `not_in_team`. Unknown ids give 404.

Front end: new pages `/players`, `/settings` (training days and reasons), `/training/[date]`, `/attendance`; the card on the start page. The start page links to "Attendance"; "Attendance" links to "Players" and "Settings". Taps for one player are sent one after the other, so a late answer never overwrites a newer tap; a failed request shows the stored state again with "Not saved. Try again."

Rules that stay: every route under the protected `/api` router; `Origin` on every changing request; generated API types; compile-time checked queries (`sqlx migrate run`, then `just prepare`); no `unwrap`/`expect` outside tests. New: **logs never contain player names** (ids only); added to `CLAUDE.md` at the close of story A.

Options considered:
| Option | For | Against |
|---|---|---|
| Absences only; "there" follows from the data (chosen) | A new player or a changed role date needs no rewrite; deleting a player without absences loses nothing; totals always match the data | Totals are worked out at each request (cheap at this size) |
| Store "there" for each player at each training | Simple counting | Many rows to add or remove on every change of a role; deleting a player loses data |
| Dates on the player (the first design) | Simplest | No return in the same season, no change of role, no season boundary (coordinator, 2026-10-15) |
| Players linked to the game list team of the season | One link for games and attendance | The team gets a new name each season; attendance would need a game list choice first |
| One fixed training schedule per season | Simpler | A change in January rewrites the past (spec review S10) |

## Stories
Each story gets its own change folder (stage `design`) and its own `/design`.

| # | Story | Type | Scope | Depends on | Expected closure | Status |
|---|---|---|---|---|---|---|
| A | [Players and settings](../2026-10-15-players-and-settings/) | feature | Migration `players`, `player_roles`, `reasons` (6 presets), `training_seasons`, `training_days`. Core `players`, `training` (dates, valid from, proposals), validation. API and screens `/players` (add, change, delete, roles, "Left the team", "Add all" / "Add to this season") and `/settings` (training days with times, first and last training date, reasons: add, rename, valid, hide). A link from the start page. | – | review build | implement |
| B | Trainings and attendance | feature | Migration `trainings`, `absences`. Core `next_training`, the players at a training, the status of each training day. API and screens `/training/[date]` (open, absences, cancel and undo, delete an extra training, "Add training"), the start page card, and the list of training days of the season ("Coming" above "Past", status, "Cancel training" on a day not opened). | A | review build + accept | open |
| C | The overview | feature | Core `season_overview`. API and screen `/attendance`: totals, "Left the team", "N training days not opened", "Show cancelled", earlier seasons, the missed trainings of a player. | B | review build + accept | open |

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| Attendance is stored as absences only; "there" follows from the training, the player's role and the absence | No rewrite on role changes; deleting a player without absences loses nothing; totals follow the data (spec note S7, S8) | yes, 0018 at the close of story B |
| A player is the person; a role (plays games / only trains) in a season has a start and an end | Players join, leave and come back during a season; a role can change; next season they are added again (coordinator, 2026-10-15) | yes, 0019 at the close of story A |
| Training days have a "valid from" date | A change in the season does not rewrite the past (spec note S10) | no (design detail; in 0019 or the story A design) |
| Opening a training stores it with everyone there; safe to send twice | Coordinator's rule (2026-10-15); two screens make one training | no |
| Cancel instead of delete; only an extra training can be deleted | Coordinator's rule (2026-10-15); nothing disappears without a trace | no |
| Players are not linked to the game list choice | The team gets a new name each season; attendance must work without a game list choice | in 0019 |
| No backup before story A goes live | Coordinator's choice (2026-10-15): later | no; a backlog line and a debt line at the close of story A |

## Not in scope
- Everything under "Not in scope" in `functional.md`.
- A backup of the team database (accepted risk; backlog).
- A screen to change a role within a season (the data allows it).

## Test plan
- **Existing tests that must still pass**: all of `just check`.
- **Test data**: made in each test (players, roles, training days for the season 2026-2027: Tuesdays 19:45–21:00 from Tue 1 Sep 2026 to Tue 25 May 2027, 39 dates). No personal data from the coordinator's sheet in the repository.
- **New tests per story**:
  - Story A: core `role_on` (from 1 Sep with no end; first day; a role ended on 15 Nov gives none on 16 Nov; two roles with a gap); overlap refused (`roles_overlap`); end on 30 June in its season, 1 July refused (`outside_season`); `roles_to_carry` (14 roles, 1 ended in March: 13 carried, same roles, from the first training date, or today after it); training dates (39 Tuesdays; proposals first Tuesday of September, last Tuesday of May; Thursday valid from 5 Jan 2027 keeps the Tuesdays before); validation codes. Database and API: add, change, delete a player; the 6 presets in order with School, Ill, Injured valid; rename and hide a reason; settings saved per season; "Add all"; every new route 401 without login. Browser: add a player at 390 pixels; the same-name warning; "Left the team"; no sideways scroll at 320 pixels.
  - Story B: core `next_training` (Tue 20 Oct 19:00 and 23:59 give Tue 20 Oct; Wednesday gives Tue 27 Oct; a cancelled Tue 27 Oct shows as cancelled; an extra training on Thu 22 Oct comes first; none before the first or after the last training date). Database and API: open twice gives one training; two opens at the same time give one; an absence and "there" again; `reason_hidden`; `not_in_team`; cancel and undo keep the absences; `not_extra`; deleting an extra training; `not_a_training_day`; `outside_training_dates`; player delete with an absence gives `player_has_absences`. Browser: open today's training from the card (everyone there); two taps give "2 absent"; a failed save (`page.route` 500) shows the old state and "Not saved. Try again."; "Cancelled" on the card; the list "Coming" above "Past".
  - Story C: core `season_overview` (a role from 3 Nov has no missed trainings before it; a cancelled training does not count and counts again after undo with its absences; a training next week with a noted absence does not count; a reason changed from valid to not valid moves the count; an absence outside the role is kept but not counted; "not opened" counts only past training days; last season's totals unchanged in a new season). Database and API: `GET /api/attendance` for the current and an earlier season. Browser: enter four past Tuesdays from the list, the totals change; "Show cancelled".
- **Checks**: `just check` for each story; `sqlx migrate run` and `just prepare` after each migration; `just api-types` after each API change.
- **By hand**: the real phone in the hall after story B (the coordinator).

## Compatibility and performance
- No change for the games, the schedule or the list of changes. The start page gets the training card (story B) and a link (story A).
- Each story adds a forward-only migration; old app code runs on the new tables (decision 0004).
- The data of this epic exists only in the tool: no source to read it again, and no backup yet (accepted risk).
- Load: one coordinator, about 15 players, about 40 trainings and a few hundred absences a season; totals worked out at each request.

## Progress
- [ ] Story A: players and settings
- [ ] Story B: trainings and attendance
- [ ] Story C: the overview
- [ ] Epic review accept

## Deviations
- 2026-10-15, during /design, agreed by the coordinator: the functional spec was changed after approval, without a third spec review: a player has no dates; a role in the team of a season has a start and an end (see "Change after approval" in `functional.md`).

The design reads the spec more strictly within its words:
- A role ends at the latest on 30 June; "Add all" gives each player the role it had at the end of last season.
- "There is already a player with this name" is a warning in the answer of a save, not an error.
- Reason names are compared without case and without outer spaces.

## Review findings
