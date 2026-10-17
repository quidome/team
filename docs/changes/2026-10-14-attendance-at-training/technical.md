# Technical spec: Attendance at training

**Status**: approved (designed again 2026-10-16 after the change of the functional spec)
**Size**: epic
**Change**: docs/changes/2026-10-14-attendance-at-training
**Date**: 2026-10-15, changed 2026-10-16

## Objective
Bring the players into the tool and let the coordinator, who is also the coach, keep attendance at
training with a reason for each absence, in seconds on a phone, with season totals for each
player.

## Requirements summary
From `functional.md` (approved 2026-10-15; changed after the accept review of story A and approved
again 2026-10-16 after five spec reviews). Words from `docs/language.md` (player, role, hidden,
weekly training, training day, training: settled 2026-10-16).

1. **Players and team** (story 1): a player is the person: a first name (required), a last name (optional), and **hidden** (not offered as a choice; history and totals stay). No dates. A **role** in the team of a season, "plays games" or "only trains", has a start date and an end date (empty while it continues; at the latest 30 June). Roles of one player never overlap; a player's roles are the history. Two pages:
   - **Players**: all players (hidden ones behind a switch), each with the role today or "Not in the team"; the player: name, hidden, the history (all seasons, newest first) with "Correct" for a role of the current season (kind, start, end); "Delete player" when the player has no absences.
   - **Team** (current season): the players in the team today or later in the season, then "Left the team" (a role earlier in the season, none today or later). "Add player" (from the list, not hidden, last season's team first, or a new player) with a role and a date. "Change role" (new role and a date: the current role ends the day before; refused for the same role and before a role that starts later). "Remove from the team" ("Not in the team from": from that date no roles in the season; the role on the day before ends, later roles are removed after a confirmation that names them), with "Undo" while the page is open. "Last day today" (not on 30 June). "Add to the team" for a player under "Left the team". "Players of last season" with "Add all" (last season's team without hidden players, same role, a date) while no player of last season's team has a role in the season.
   - Proposed dates: "In the team from" of "Add player" and "Add all" is 1 September of the season, with "Today"; "Change role" and "Add to the team" today; "Not in the team from" today, with "Tomorrow". (The first training date is no longer used: changed 2026-10-16 after the second reviews of story A.)
   - Roles of an earlier season cannot be corrected. Warning, not error, for two players with the same name.
2. **Weekly trainings and trainings** (story 2): a **weekly training** is a day of the week, a start and an end time, and a window (from a date until a date) inside the season; more than one at a time; not two on the same day of the week with overlapping windows. Proposed window: that day on or after 1 September until that day on or before 31 May. "Change" (everything) and delete. A weekly training only suggests **training days**; a training exists only when the coordinator opens a training day (with its times at that moment) or adds one ("Add training", any date without a training). Once opened, a training is independent: a change of a weekly training never changes or deletes it. At most one training per date. "Change" of a training: its date and times, attendance kept. Any training can be deleted by hand (after a confirmation); nothing else deletes one. "Cancel training" / "Undo cancel" (absences kept, not counted). The start page card: today's training until midnight, else the next training day or training, also when cancelled; none without a coming training day or training. The list of the season: "Coming" above "Past", one row per date, "Not opened", "All there", "N absent", "Cancelled", "Show cancelled" (on by default). Trainings and attendance of an earlier season stay open. A new season proposes last season's weekly trainings with new windows.
3. **Attendance** (story 3): a training shows the players with a role on its date, all "there" at first; one tap per absence with a reason; each tap saved at once; a failed tap shows the old state with "Not saved. Try again."; "No player has a role on this date." when there is none.
4. **Reasons** (story 4): presets School, Ill, Injured (valid), Holiday, Work, No reason (not valid). Add (last), rename, valid / not valid (totals follow), hide. Names unique.
5. **Season totals** (story 5): for each player with a role in the season: there, absent valid, absent not valid, of N trainings; only trainings up to today, not cancelled, inside the player's roles; "Left the team"; "N training days not opened"; earlier seasons; the missed trainings of a player.
6. Dutch local dates and times; only the coordinator (login, decision 0007); phone first, no sideways scroll at 320 pixels.

Not in this epic: attendance at games, jersey number, birthday and association id, reading players from a sheet, trends, messages, late or early leave, the place of a training, more teams.

Decisions that apply: 0001 (rules in a core without I/O), 0002 (static Svelte front end, generated API types), 0004 (forward-only migrations, compile-time checked queries), 0007 (every `/api` route needs a session), 0014 (seasons as start years).

## Current state
- Core, server, front end and tests on `main`: as in the first design (`Season`, `clock`, `db.rs`, `AppError`, OpenAPI with `utoipa`, migrations `0001`–`0004`, start page, `#[sqlx::test]`, router harness, Playwright with the server clock at 12 October 2026).
- **Story A is built in the first form on branch `feat/players-and-settings`, not merged, never on the cluster** (commits 905a767 to 7895c6f, reviewed twice): migration `0005` with `players`, `player_roles`, `reasons`, `training_seasons`, `training_days`; core `players` (`place`, `check_role`, `check_change`, `proposed_start`, `roles_to_carry`), `training` (sets valid from a date, `settings_change`, `replaced_from`, `shown_set`), `reasons`; the pages `/players`, `/players/[id]`, `/settings`; tests for all of it. Story A's own design decides what of it stays.
- Backups: none arranged for the team database. The coordinator accepted this for now (2026-10-15).

## Architecture (shared by all stories)

Stored data (each story adds its own migration; story A rewrites its `0005`, see Decisions):

| Table | Holds | Story |
|---|---|---|
| `players` | first name, last name (nullable), hidden (default false). No dates. | A |
| `player_roles` | player, season (start year), role (`plays_games` / `only_trains`), from date, until date (nullable). Inside its season; no overlap per player. | A |
| `reasons` | name (unique without case and outer spaces), valid, hidden, position. The 6 presets. | A |
| `weekly_trainings` | season, day of the week, start time, end time, from date, until date. Inside its season; no two on the same day of the week with overlapping windows. | A |
| `trainings` | date (unique), start time, end time, cancelled at (nullable). No link to a weekly training. | B |
| `absences` | training, player, reason. Primary key (training, player). | B |

- "There" is not stored: a player is there at a training when the training exists, the player has a role on its date, and there is no absence row.
- Training days are not stored: they are worked out at each request from the weekly trainings of the season. A training is "extra" when its date is not a training day; nothing stores it.
- The list of a season is one row per date: the stored training, or else the training day.
- Totals are worked out at each request from the stored rows (about 15 players, about 40 trainings a season).
- No overlap of roles, and of weekly trainings on the same day, is checked in the core under a lock (a row lock on the player; an advisory lock for the weekly trainings), not by a database constraint (an `EXCLUDE` constraint needs `btree_gist` on the cluster).

Core (`crates/core`):
- `season`: `Season` with `of`, `start_year`, `previous`, `first_day`, `last_day`.
- `players`: `Player` (with hidden), `Role`, `PlayerRole` (`last_day`: until, or the season's last day); `role_on`; `check_role`, `check_change` (only a role of the season); `change_role` (picks the role on the date; it ends the day before, the new one starts on the date; refused on the role's start, for the same role, and before a role that starts later); `remove_from_team` (ends the role on the day before, removes the roles from the date on); `undo_removal` (puts the roles back when nothing else changed); `team` (the groups of the team page, "Last day today"); `proposed_start(season)` (1 September). Errors `first_name_required`, `until_before_from`, `roles_overlap`, `outside_season`, `change_before_role_start`, `not_in_team`, `same_role`, `role_starts_later`, `changed_meanwhile`.
- `training`: `WeeklyTraining`; `check_weekly_training(season, weekly, others)` (window inside the season, until not before from, end after start, no overlap on the same day); `proposed_window(season, weekday)`; `training_days(season, weekly)` (each date with its times). Story B: `Training`; the list of a season (one row per date, status); `next_training` (the card); the proposal of last season's weekly trainings for a new season. Errors `end_before_start`, `last_before_first` (the window), `weekly_trainings_overlap`, `training_exists`, `not_a_training_day`.
- `reasons`: `clean_reason_name`. Story B/C `attendance`: `Absence`; `season_overview(…)`.

Server (`crates/server`): storage modules `players.rs`, `reasons.rs`, `weekly_trainings.rs` (A), `trainings.rs` (B); API files under `api/`. API outline (details in each story's design):
- A: `GET/POST /api/players`, `GET/PUT /api/players/{id}` (the player with the history; name, hidden), `DELETE /api/players/{id}`; `POST /api/players/{id}/roles` ("Add player" to the team, "Add to the team"), `PUT /api/players/{id}/roles/{role}` ("Correct"); `POST /api/players/{id}/role-change` ("Change role"); `POST /api/players/{id}/team-leave` ("Remove from the team", with the confirmed roles); `POST /api/players/{id}/team-undo` ("Undo"); `GET /api/team` (the team page); `POST /api/players/carry` ("Add all"); `GET/POST /api/reasons`, `PUT /api/reasons/{id}`; `GET/POST /api/weekly-trainings`, `PUT/DELETE /api/weekly-trainings/{id}`, `GET /api/weekly-trainings/proposal?weekday=`.
- B: `GET /api/trainings?season=` (the list), `PUT /api/trainings/open/{date}` (open a training day; `INSERT … ON CONFLICT (date) DO NOTHING`, then the stored training; safe to send twice), `POST /api/trainings` ("Add training"), `PUT /api/trainings/{id}` ("Change": date and times), `POST /api/trainings/{id}/cancel` and `/uncancel`, `DELETE /api/trainings/{id}` (any training), `PUT`/`DELETE /api/trainings/{id}/absences/{player}`, `GET /api/trainings/next` (the card), `GET /api/weekly-trainings/proposal-for-season` (last season's weekly trainings with new windows).
- C: `GET /api/attendance?season=`.

Error codes (400, `{"error": "<code>"}`): `first_name_required`, `until_before_from`, `roles_overlap`, `outside_season`, `change_before_role_start`, `end_before_start`, `last_before_first`, `weekly_trainings_overlap`, `reason_name_required`, `reason_name_taken`, `reason_hidden`, `player_has_absences`, `training_exists`, `not_a_training_day`, `not_in_team`, `same_role`, `role_starts_later` (with `date`). Conflicts (409): `confirm_removal` (with the `roles` a removal would remove, and whether it `ends` a role), `changed_meanwhile` ("Undo" after another change). Unknown ids give 404 (`not_found`). Gone from the first design: `not_extra`, `outside_training_dates`, `no_training_days`, `duplicate_weekday`.

Front end: pages `/players` and `/players/[id]` (all players, the player with the history), `/team` (the team of the current season), `/settings` (weekly trainings and reasons), `/training/[date]`, `/attendance`; the card on the start page. The start page links to "Attendance"; "Attendance" links to "Team", "Players" and "Settings" (until story C, the start page links to "Team", "Players" and "Settings"). Taps for one player are sent one after the other, so a late answer never overwrites a newer tap; a failed request shows the stored state again with "Not saved. Try again."

Rules that stay: every route under the protected `/api` router; `Origin` on every changing request; generated API types; compile-time checked queries (`sqlx migrate run`, then `just prepare`); no `unwrap`/`expect` outside tests. New: **logs never contain player names** (ids only); added to `CLAUDE.md` at the close of story A.

Options considered:
| Option | For | Against |
|---|---|---|
| Absences only; "there" follows from the data (chosen) | No rewrite on role changes; deleting a player without absences loses nothing; totals always match the data | Totals are worked out at each request (cheap at this size) |
| Store "there" for each player at each training | Simple counting | Many rows on every change of a role; deleting a player loses data |
| Weekly trainings with a window that only suggest training days (chosen, coordinator 2026-10-16) | A mistake is fixed by a change for the whole window; more than one at a time; an opened training is never touched; same pattern as roles | Training days are worked out at each request (cheap: about 40 a season) |
| Sets of training days valid from a date, with a first and last date of the season (the first design, built in story A) | One form for the whole season | A mistake corrected later stays in force for the past and is hidden (accept review A1, R3, R10) |
| Store every training day of the season up front | Simple list | Every Tuesday would exist and count with everyone there, also forgotten holidays; against "a training exists only when opened" |
| Rewrite migration `0005` (chosen) | One clean migration for story A; the first form was never released | Local databases with the old `0005` must be made again |
| A new migration `0006` on top | Follows decision 0004 word for word | The history keeps a form that was never released |

## Stories
Each story has its own change folder and its own `/design`.

| # | Story | Type | Scope | Depends on | Expected closure | Status |
|---|---|---|---|---|---|---|
| A | [Players and settings](../done/2026-10-15-players-and-settings/) | feature | Rewrite migration `0005`: `players` (with hidden), `player_roles`, `reasons` (6 presets), `weekly_trainings`. Core `players` (`change_role`, `remove_from_team`, `undo_removal`, `proposed_start` = 1 September, hidden in "Add all") and `training` (weekly trainings, check, proposed window, training days). API and pages `/players` and `/players/[id]` (all players, hidden, history with "Correct", delete), `/team` ("Add player", "Change role", "Remove from the team" with "Undo", "Add to the team", "Add all" with a date), `/settings` (weekly trainings: add, change, delete; reasons). Links from the start page. Built first in the old form on its branch; its design says what stays. | – | review build + accept | done |
| B | [Trainings and attendance](../done/2026-10-17-trainings-and-attendance/) | feature | Migration `trainings` (date unique), `absences`. Core: the list of a season (one row per date), `next_training`, the players at a training, the proposal of last season's weekly trainings. API and pages `/training/[date]` (open, absences, "Change" of date and times, cancel and undo, delete any training, "No player has a role on this date."), "Add training", the start page card, the list of the season. Trainings of an earlier season stay open. | A | review build + accept | done |
| C | [The overview](../2026-10-17-the-overview/) | feature | Core `season_overview`. API and page `/attendance`: totals, "Left the team", "N training days not opened", "Show cancelled", earlier seasons, the missed trainings of a player. | B | review build + accept | open |

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| Attendance is stored as absences only; "there" follows from the training, the player's role and the absence | No rewrite on role changes; deleting a player without absences loses nothing; totals follow the data (spec note S7, S8) | yes, 0020 (written at the close of story B; 0019 is not used) |
| A player is the person; a role (plays games / only trains) in a season has a start and an end; a real change of role ends one and starts the next | Players join, change role, leave and come back; the history stays right (coordinator, 2026-10-15 and 2026-10-16) | yes, 0018 (written at the close of story A) |
| Weekly trainings with a window only suggest training days; an opened training is independent; training days are not stored | A mistake is corrected for the whole window; the past of opened trainings never changes by itself (coordinator, 2026-10-16; accept review A1) | in 0018 |
| One training per date; any training can be deleted by hand; "extra" is worked out, not stored | Coordinator's rules (2026-10-16, spec review S13); one row per date in the list | no |
| Opening a training stores it with everyone there; safe to send twice | Coordinator's rule (2026-10-15); two screens make one training | no |
| Cancel for a holiday; delete for a training opened or added on a wrong date | A deleted training day counts as not opened (spec review S22) | no |
| The club has the players; a team exists for one season and is the team chosen from the game list for that season; roles and weekly trainings are stored by season, so they belong to it without a separate link, and can be entered before the team is chosen. One club for the whole tool. | The coordinator's model (2026-10-16, during /design of story A; replaces "players are not linked to the game list choice"); one team per season; one club while the coordinator is the only user | in 0018 |
| Story A rewrites its migration `0005`; "an applied file is never edited" (decision 0004) means applied on the cluster | The first form of story A was never merged or released (coordinator, 2026-10-16) | no; one line in 0018 |
| No backup before story A goes live | Coordinator's choice (2026-10-15): later | no; a backlog line and a debt line at the close of story A |

## Not in scope
- Everything under "Not in scope" in `functional.md`.
- A backup of the team database (accepted risk; backlog).
- A season page for a player; the history on the player page shows all seasons.

## Test plan
- **Existing tests that must still pass**: all of `just check`. Story A's design names which of its first-form tests stay, change or go.
- **Test data**: made in each test (players, roles, weekly trainings for 2026-2027: Tuesday 19:45–21:00 from Tue 1 Sep 2026 to Tue 25 May 2027, 39 training days). No personal data from the coordinator's sheet in the repository.
- **New tests per story**:
  - Story A, core: `role_on` and `check_role` (as built: an open role ends on 30 June; overlap; seasons); `change_role` (a change on Sun 1 Nov ends the old role on Sat 31 Oct; on or before the start → `change_before_role_start`); `proposed_start` (1 September of the season); `remove_from_team` (a later role is removed; nothing to end or remove → `not_in_team`); `undo_removal` (a change in the meantime → `changed_meanwhile`); "Add all" leaves out hidden players; `check_weekly_training` (Tuesday 1 Sep–25 May and Tuesday from Tue 5 Jan → `weekly_trainings_overlap`; Tuesday and Thursday together → ok; window outside the season → `outside_season`; until before from → `last_before_first`; end before start → `end_before_start`); `proposed_window` (Tuesday → Tue 1 Sep 2026 – Tue 25 May 2027; Thursday → Thu 3 Sep 2026 – Thu 27 May 2027); `training_days` (Tuesday and Thursday together, sorted by date; 39 Tuesdays).
  - Story A, database and API: "Change role" all or nothing (an overlap refused → both roles unchanged); two weekly-training saves with the lock held (the method of fix 3 of story A); a hidden player not offered to add, still in the team; "Correct" refused for a role of last season; 401 without login and 403 without `Origin` on every new route; no player names in the logs.
  - Story A, browser: "Add player" on `/team` with the proposed 1 Sep (server clock 12 Oct); "Change role" with a date, the history shows both roles; "Correct"; "Remove from the team"; a weekly training with the proposed window; `/players` with "Show hidden players"; no sideways scroll at 320 pixels and `expectAccessible`.
  - Story B, core: `next_training` (Tue 20 Oct 19:00 and 23:59 give Tue 20 Oct; Wednesday gives Tue 27 Oct; a cancelled Tue 27 Oct shows as cancelled; an extra training on Thu 22 Oct comes first; none without a coming training day or training); the list of a season (a training on a date that is no longer a training day stays, as extra; an extra training on a date that becomes a training day is one row); the proposal for a new season. Database and API: open twice gives one training; two opens at the same time give one; "Change" keeps the absences; "Change" to a date with a training → `training_exists`; delete any training; an absence and "there" again; `reason_hidden`; `not_in_team`; cancel and undo keep the absences; a training of last season changed in a new season; player delete with an absence → `player_has_absences`. Browser: open today's training from the card; two taps give "2 absent"; a failed save (`page.route` 500) shows the old state and "Not saved. Try again."; "Cancelled" on the card; the list "Coming" above "Past".
  - Story C: core `season_overview` (a role from 3 Nov has no missed trainings before it; a cancelled training does not count and counts again after undo; a training next week with a noted absence does not count; a reason changed from valid to not valid moves the count; an absence outside the role is kept but not counted; "not opened" counts only past training days; a player after "Change role" is counted once, not under "Left the team"). Database and API: `GET /api/attendance` for the current and an earlier season. Browser: enter four past Tuesdays from the list, the totals change; "Show cancelled".
- **Checks**: `just check` for each story; `sqlx migrate run` and `just prepare` after each migration; `just api-types` after each API change.
- **By hand**: the real phone in the hall after story B (the coordinator).

## Compatibility and performance
- No change for the games, the schedule or the list of changes. The start page gets the training card (story B) and links (story A).
- Each story adds a forward-only migration; old app code runs on the new tables (decision 0004). Story A's `0005` is rewritten before it is merged; local databases that ran the old `0005` are made again (`just db-down`, `just db-up`, `just migrate`).
- The data of this epic exists only in the tool: no source to read it again, and no backup yet (accepted risk).
- Load: one coordinator, about 15 players, a few weekly trainings, about 40 trainings and a few hundred absences a season; training days and totals worked out at each request.

## Progress
- [x] Story A: players and settings (built once in the first form and reviewed; designed again 2026-10-16; steps 5–9 on 2026-10-17; closed 2026-10-17, decision 0018)
- [x] Story B: trainings and attendance (closed 2026-10-17, decision 0020)
- [ ] Story C: the overview
- [ ] Epic review accept

## Deviations
- 2026-10-15, during /design, agreed by the coordinator: the functional spec was changed after approval, without a third spec review: a player has no dates; a role in the team of a season has a start and an end.
- 2026-10-16: the functional spec was changed after the accept review of story A (pages Players and Team, weekly trainings, one training per date) and approved again after the third to fifth spec review; this technical spec is designed again to match.
- 2026-10-17, design of story A steps 5–9: story 1 was changed after the second reviews of story A ("Remove from the team" from a date with "Undo", proposed dates 1 September and today) and approved again after the sixth spec review. The rule "the first training date until a training is opened" is dropped here (requirements, core `players` and `training`, story B's scope and tests), as Steve's warning in the functional spec asks.

The design reads the spec more strictly within its words:
- A role ends at the latest on 30 June; "Add all" gives each player the role it had at the end of last season.
- "There is already a player with this name" is a warning in the answer of a save, not an error.
- Reason names are compared without case and without outer spaces.
- A weekly training's window is inside one season; the season of a weekly training is the season of its window.

## Review findings
