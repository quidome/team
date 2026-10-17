# Technical spec: Trainings and attendance

**Status**: done
**Size**: story
**Change**: docs/changes/2026-10-17-trainings-and-attendance
**Date**: 2026-10-17

## Objective
Let the coordinator open or add a training, mark who is absent and why with one tap each, keep
trainings right (change, cancel, delete), see the season's trainings and the next one, so that
attendance at training leaves the sheet; story C adds the totals.

## Requirements summary
Story B of the epic [Attendance at training](../2026-10-14-attendance-at-training/technical.md),
from the epic's `functional.md` (approved again 2026-10-16): story 2 (trainings, the list, the card,
a new season), story 3 (marking), and the parts of stories 1 and 4 that need trainings and
absences. Words from `docs/language.md`: training, training day, weekly training, attendance,
reason, role, player.

1. **A training** exists only when the coordinator opens a training day or adds one. Opening stores it at once, with the times of its weekly training at that moment and every player "there"; from then on it counts. Opening twice, or from two screens, gives one training. At most one training per date. Once opened, a training is independent: a change or delete of a weekly training never changes or deletes it.
2. **"Add training"**: a date of the current season without a training (an extra training, also in the past), with the times of a weekly training on that day of the week, or typed times.
3. **"Change"** of a training: its date and times; the attendance stays. Refused: a date with a training ("There is already a training on this date."), a date outside the training's season ("This date is outside the season."), an end before the start.
4. **"Cancel training"** marks a training as cancelled; a training day can be cancelled from the list without opening it. A cancelled training stays in the list with "Cancelled", keeps its absences (not counted), and can still be marked. **"Undo cancel"** makes it a normal training again; its absences count again.
5. **"Delete training"** (after a confirmation: "Delete the training of Thu 22 Oct? Its attendance is removed.") deletes any training and its attendance. Nothing else deletes a training. On a training day the date shows "Not opened" again.
6. **The list** (page "Attendance", `/attendance`, current season): every training day and every training of the season, one row per date (a training on a training day is that day's row). "Coming" (today and later, nearest first) above "Past" (newest first). Each row: the date and time, and "Not opened", "All there", "N absent" or "Cancelled". "Cancel training" on a day that is not opened. The switch "Show cancelled", on by default; off hides cancelled rows. "Add training". A tap on a row opens the training (and so makes it exist).
7. **The start page card**: the first coming row of the current season: today's training until midnight, else the next training day or training, also when cancelled ("Training · Tue 20 Oct · 19:45" with "Not opened yet", "All there", "N absent" or "Cancelled"; "Training · Tue 20 Oct · Cancelled"). None without a coming row. A tap opens the training. One line, below the team header. The start page also links to "Attendance".
8. **Marking** (`/training/[date]`): the date and times at the top with "N absent", "All there" or "Cancelled"; the players with a role on the date (also those who only train), sorted by first name, large rows, each "there" at first. A tap on a player who is there opens the reasons; a tap on a reason marks the player absent with it, and the row shows the reason. A tap on an absent player shows the reasons with "There" on top. Each tap is saved at once, no save button. When a tap cannot be saved, the row shows the old state again and the tool says "Not saved. Try again."; it never shows a mark as saved when it was not. A late answer of an earlier tap never overwrites a newer tap. When the page is opened or shown again, it shows the stored marks. The last tap wins between two screens.
9. **Empty states**: no player with a role on the date: "No player has a role on this date."; all reasons hidden: only "There", and "No reasons. Add one in Reasons.". A cancelled training opens with "Cancelled" and "Undo cancel".
10. **Earlier seasons**: trainings and attendance of an earlier season can still be opened, marked, changed and cancelled. In story B the list shows the current season; a training of an earlier season opens by its address (`/training/2027-05-25`); the choice of season on the screen comes with story C.
11. **Story 4**: a hidden reason is not offered for new marks; earlier absences keep it and show it.
12. **Story 1**: "Delete player" only for a player with no absences ("This player has absences and cannot be deleted. Remove the player from the team, or hide the player."). When a role starts (or its start is moved earlier, also with "Correct") while trainings on or after that date exist, the tool says "This player now counts as there at N earlier trainings." and lists them.
13. **A new season** (story 2): the settings page proposes the weekly trainings of last season with new proposed windows.
14. Dutch local dates and "today", as the games; only the coordinator (decision 0007); phone first, no sideways scroll at 320 pixels; logs never contain player names.

Decisions that apply: 0001, 0002, 0004, 0007, 0014, 0018 (players, roles, weekly trainings; the checks in the core under locks).

## Current state
On `main` (b558232 and later), story A is built and on the cluster:
- Migrations up to `0005_players_and_settings` (`players`, `player_roles`, `reasons` with 6 presets, `weekly_trainings`).
- Core `players` (`PlayerRole::last_day`, `role_on`, `check_role`, `check_change`, `change_role`, `remove_from_team`, `undo_removal`, `team`, `proposed_start`), `training` (`WeeklyTraining`, `TrainingError { EndBeforeStart, LastBeforeFirst, OutsideSeason, WeeklyTrainingsOverlap }`, `check_weekly_training`, `proposed_window`, `training_days`), `reasons` (`clean_reason_name`), `season`.
- Server: storage `players.rs` (with `lock`), `reasons.rs` (`StoredReason`), `weekly_trainings.rs`; routes `api/players.rs`, `api/team.rs`, `api/reasons.rs`, `api/weekly_trainings.rs`; `AppError::Refused` for refusals with details; `clock::today_in_netherlands`, `current_season`.
- Front end: `/team`, `/players`, `/players/[id]`, `/settings`; the start page line "Team · Players · Settings" (`+page.svelte`, snippet `links`); `client.ts` (`ApiError` with `code` and `body`, `messageFor`); `lib/players.ts`, `lib/schedule.ts` (`formatDay`).
- Tests: `tests/team.rs`, `tests/players.rs`, `tests/weekly_trainings.rs`, `tests/player_logs.rs`, `web/e2e/players.e2e.ts`; the lock-wait pattern and `refuse_role_writes` in `tests/team.rs`.

## Steps

### Step 1: The data and the rules, read only
- **Goal**: trainings and absences exist in the data, the core and the read routes.
- **Changes**:
  - Migration `migrations/0006_trainings.sql`:

    ```sql
    CREATE TABLE trainings (
        id           bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
        date         date NOT NULL UNIQUE,
        start_time   time NOT NULL,
        end_time     time NOT NULL,
        cancelled_at timestamptz,
        created_at   timestamptz NOT NULL,
        CHECK (end_time > start_time)
    );
    CREATE TABLE absences (
        training_id bigint NOT NULL REFERENCES trainings ON DELETE CASCADE,
        player_id   bigint NOT NULL REFERENCES players ON DELETE RESTRICT,
        reason_id   bigint NOT NULL REFERENCES reasons,
        PRIMARY KEY (training_id, player_id)
    );
    CREATE INDEX absences_player ON absences (player_id);
    ```

    No link to a weekly training; the season follows from the date.
  - Core `training.rs` (added):

    ```rust
    pub struct TrainingId(pub i64);
    pub struct Training { pub id: TrainingId, pub date: NaiveDate, pub start: NaiveTime,
                          pub end: NaiveTime, pub cancelled: bool }
    // TrainingError gets TrainingExists, NotATrainingDay
    pub enum DayStatus { NotOpened, AllThere, Absent(usize), Cancelled }
    pub struct SeasonRow<'a> { pub date: NaiveDate, pub start: NaiveTime, pub end: NaiveTime,
                               pub training: Option<&'a Training>, pub extra: bool, pub status: DayStatus }
    pub struct SeasonList<'a> { pub coming: Vec<SeasonRow<'a>>, pub past: Vec<SeasonRow<'a>> }
    pub fn season_list<'a>(season: Season, weekly: &'a [WeeklyTraining], trainings: &'a [Training],
                           absences: &[Absence], roles: &[PlayerRole], today: NaiveDate) -> SeasonList<'a>;
    pub fn check_training(season: Season, date: NaiveDate, start: NaiveTime, end: NaiveTime,
                          trainings: &[Training], except: Option<TrainingId>) -> Result<(), TrainingError>;
    pub fn day_times(weekly: &[WeeklyTraining], date: NaiveDate) -> Result<(NaiveTime, NaiveTime), TrainingError>;
    pub fn proposed_times(weekly: &[WeeklyTraining], date: NaiveDate) -> Option<(NaiveTime, NaiveTime)>;
    pub struct WeeklyProposal { pub weekday: Weekday, pub start: NaiveTime, pub end: NaiveTime,
                                pub from: NaiveDate, pub until: NaiveDate }
    pub fn proposals_from(last: &[WeeklyTraining], season: Season) -> Vec<WeeklyProposal>;
    ```

    - `season_list`: the training days of the season (`training_days`) and the trainings of the season, one row per date; a training wins over a training day of its date (`extra` when its date is not a training day). Status: `Cancelled` for a cancelled training; else `Absent(n)` with n the absences of players with a role on the date (`role_on`), `AllThere` when n = 0; `NotOpened` for a training day without a training. `coming`: date ≥ today, nearest first; `past`: newest first.
    - `check_training`: `Season::of(date) != season` → `OutsideSeason`; another training (not `except`) on `date` → `TrainingExists`; `end <= start` → `EndBeforeStart`.
    - `day_times`: the times of the weekly training whose training day is `date`; else `NotATrainingDay`.
    - `proposed_times`: `day_times`, else the newest weekly training of `Season::of(date)` on the same day of the week, else `None`.
    - `proposals_from`: for each day of the week the weekly training of `last` with the latest `from`, with `proposed_window(season, weekday)`; sorted by `from`.
  - Core `reasons.rs`: `pub struct ReasonId(pub i64)`.
  - Core `attendance.rs` (new, exported from `lib.rs`):

    ```rust
    pub struct Absence { pub training: TrainingId, pub player: PlayerId, pub reason: ReasonId }
    pub struct AtTraining<'a> { pub player: &'a Player, pub role: Role, pub absence: Option<ReasonId> }
    pub fn players_at<'a>(players: &'a [Player], roles: &[PlayerRole], absences: &[Absence],
                          training: &Training) -> Vec<AtTraining<'a>>;
    pub fn now_there<'a>(player: PlayerId, before: &[PlayerRole], after: &[PlayerRole],
                         trainings: &'a [Training], absences: &[Absence]) -> Vec<&'a Training>;
    pub enum AttendanceError { NotInTeam, ReasonHidden }
    pub fn check_mark(player: PlayerId, roles: &[PlayerRole], training: &Training,
                      reason: ReasonId, reason_hidden: bool, current: Option<ReasonId>)
                      -> Result<(), AttendanceError>;
    ```

    - `players_at`: players with a role on `training.date` (also hidden players), sorted by first name (without case), then last name, then id; `role` is the role on the date.
    - `now_there`: the trainings covered by `after` (a role of the player on the date) and not by `before`, not cancelled, without an absence of the player; sorted by date.
    - `check_mark`: no role on the date → `NotInTeam`; `reason_hidden` and `current != Some(reason)` → `ReasonHidden`.
  - Server `trainings.rs` (new storage): `for_season(db, season) -> Vec<Training>` (dates between the season's first and last day), `on(db, date) -> Option<Training>`, `absences_of(db, &[TrainingId]) -> Vec<Absence>`; written in step 2: `open`, `insert`, `update`, `set_cancelled`, `delete`, `mark`, `unmark`.
  - Server `api/trainings.rs` (new):
    - `GET /api/trainings?season=2026-2027` (default the current season; a season that cannot be read → 400 `invalid_season`) → `TrainingList { season, today, coming: [TrainingRow], past: [TrainingRow] }`; `TrainingRow { date, start, end, status: "not_opened" | "all_there" | "absent" | "cancelled", absent, opened, extra }`.
    - `GET /api/trainings/next` → `TrainingRow` or `null`: the first row of `coming` of the current season.
    - `GET /api/trainings/{date}` → `TrainingDay { date, season, training: { start, end, cancelled } | null, proposed_times: { start, end } | null, absent, players: [AtTrainingView { id, first_name, last_name, role, reason }], reasons: [AbsenceReason] }`; `players` empty while not opened. A path date that is not a date → 400 `invalid_date`.
  - Routes on the protected router; `just prepare`, `just api-types`.
- **Tests**:
  - Core (season 2026-2027, a Tuesday weekly training Tue 1 Sep 2026 – Tue 25 May 2027):
    - `season_list`: on Mon 12 Oct with no trainings, `coming` starts Tue 13 Oct and `past` starts Tue 6 Oct, all `NotOpened`; a training on Tue 6 Oct with 2 absences → `Absent(2)`, none → `AllThere`, cancelled → `Cancelled`; an absence of a player without a role on the date is not counted; a training on Wed 7 Oct → one row, `extra`; a training on Tue 13 Oct → one row, not extra; on Tue 13 Oct its row is in `coming`; no weekly trainings and no trainings → both empty.
    - `check_training`: Thu 22 Oct 19:00–20:30 → ok; a date with a training → `TrainingExists`, ok with `except` = that training; Thu 1 Jul 2027 → `OutsideSeason`; end before start → `EndBeforeStart`.
    - `day_times`: Tue 13 Oct → 19:45–21:00; Wed 14 Oct → `NotATrainingDay`.
    - `proposed_times`: Tue 13 Oct → 19:45–21:00; Tue 1 Jun 2027 → 19:45–21:00; Wed 14 Oct → none.
    - `proposals_from`: last season Tuesday 19:45–21:00 → Tue 1 Sep 2026 – Tue 25 May 2027; two Tuesdays last season (until January, from January with other times) → one proposal with the times of the newest; Tuesday and Thursday → two, sorted by `from`.
    - `players_at`: only players with a role on the date, sorted by first name, with their role; an absence shows its reason; a player who left before the date is not listed; a hidden player with a role is listed.
    - `now_there`: a role from Mon 12 Oct moved to Tue 1 Sep, trainings on 1, 8, 15 Sep and 13 Oct → 1, 8, 15 Sep; a cancelled training left out; a training with an absence of the player left out; unchanged coverage → empty.
    - `check_mark`: no role on the date → `NotInTeam`; a hidden reason → `ReasonHidden`; the same hidden reason as the current absence → ok.
  - Server (`tests/trainings.rs`, new; clock 12 Oct 2026; trainings and absences written through storage in this step): `the_list_has_one_row_per_date` ("Coming", "Past", "2 absent", an extra training, "Cancelled"); `the_card_is_the_next_training` (Tue 13 Oct at 23:59 → Tue 13 Oct; Wed 14 Oct → Tue 20 Oct; a cancelled Tue 20 Oct shows as cancelled; an extra training on Thu 15 Oct comes first; no weekly trainings → `null`); `the_training_page_lists_the_players_with_a_role` (players, roles and reasons; not opened → no players, with `proposed_times`); `invalid_date` and `invalid_season`.
  - Routes: the three routes in the 401 and 403 test (`tests/players.rs`).

### Step 2: The writes of a training
- **Goal**: open, add, change, cancel, undo, delete and marking, with their guarantees.
- **Changes** (`api/trainings.rs`, `trainings.rs`); each changing route answers `TrainingDay` unless said:
  - `PUT /api/trainings/{date}/open`: the training of the date, if any; else `day_times` (weekly trainings of `Season::of(date)`; `NotATrainingDay` → 400 `not_a_training_day`), then `INSERT … ON CONFLICT (date) DO NOTHING`, then read again. Safe to send twice. Log "training opened" with the date.
  - `POST /api/trainings` `{ date, start, end }` ("Add training"): `check_training` with the current season and the stored trainings; insert; a unique violation (SQLSTATE 23505) → 400 `training_exists`.
  - `PUT /api/trainings/{date}` `{ date, start, end }` ("Change"): 404 without a training on the path date; `check_training` with the training's own season and `except` = itself; update; 23505 → `training_exists`. Answers the new date's `TrainingDay`.
  - `POST /api/trainings/{date}/cancel`: a training → `cancelled_at = now()` when not cancelled; no training → `day_times` (else `not_a_training_day`) and `INSERT … ON CONFLICT (date) DO UPDATE SET cancelled_at = coalesce(trainings.cancelled_at, excluded.cancelled_at)`.
  - `POST /api/trainings/{date}/uncancel`: `cancelled_at = NULL`; 404 without a training.
  - `DELETE /api/trainings/{date}` → 204; 404 without a training; the absences go with it (`ON DELETE CASCADE`). Log "training deleted".
  - `PUT /api/trainings/{date}/absences/{player}` `{ reason }` → `Marked { player, reason, absent }`: 404 without a training, player or reason; roles of the player; the current absence; `check_mark` (`not_in_team`, `reason_hidden`); `INSERT … ON CONFLICT (training_id, player_id) DO UPDATE SET reason_id`. `absent` is the training's new count (absences of players with a role on the date).
  - `DELETE /api/trainings/{date}/absences/{player}` → `Marked { player, reason: null, absent }`; 200 also without an absence; 404 without a training.
  - `From<TrainingError>` / `From<AttendanceError>` for `AppError`: `training_exists`, `not_a_training_day`, `outside_season`, `end_before_start`, `not_in_team`, `reason_hidden`.
  - Logs: dates and ids only.
  - `just prepare`, `just api-types`.
- **Tests** (`tests/trainings.rs`):
  - `open_twice_gives_one_training` (two opens with `tokio::join!` → both 200, one row; an absence written between two opens stays); `open_needs_a_training_day` (Wed 14 Oct → `not_a_training_day`).
  - `add_and_change_a_training` (add Thu 22 Oct → listed as extra; change to Tue 20 Oct with an absence → the absence stays; change to a date with a training → `training_exists`; to Thu 1 Jul 2027 → `outside_season`; end before start → `end_before_start`); `two_adds_of_one_date_give_one_training` (`tokio::join!` → one 200, one `training_exists`).
  - `cancel_and_undo_keep_the_absences` (2 absences; cancel → "Cancelled", 2 stored; undo → "2 absent"); `cancel_a_day_that_is_not_opened` (→ a cancelled training; cancel again keeps the first moment).
  - `delete_a_training_deletes_its_absences` (the date is "Not opened" again; no absences stored; again → 404).
  - `mark_absent_and_there_again` (School → `absent: 1`; Ill → one row with Ill; there → `absent: 0`; there again → 200).
  - `mark_errors_have_their_codes` (no role on the date → `not_in_team`; a hidden reason → `reason_hidden`; unknown player, reason or training → 404).
  - `a_training_of_last_season_stays_open` (clock 15 Aug 2027: mark, change, cancel a training of 2026-2027 → 200).
  - `a_weekly_training_change_never_touches_a_training` (the Tuesday weekly training changed to Wednesday: the opened Tuesday training stays with its times and absences, as extra).
  - Routes in the 401 and 403 test; `tests/player_logs.rs`: open, mark and delete a training; no name in the log.
  - Mutation check: without `ON CONFLICT` in open, `open_twice_gives_one_training` fails.

### Step 3: The page "Attendance" and the card
- **Goal**: the coordinator sees the season and the next training, and opens a training with one tap.
- **Changes**:
  - `client.ts`: `loadTrainings`, `loadNextTraining`, `loadTraining`, `openTraining`, `addTraining`, `cancelTraining`; the sentences of `training_exists`, `not_a_training_day`, `reason_hidden`, `invalid_date`.
  - `lib/trainings.ts`: the status words ("Not opened", "All there", "N absent", "Cancelled", "Not opened yet" on the card).
  - `/attendance`: heading "Attendance · 2026-2027"; "Coming" and "Past", each row a button with the date (`formatDay`), the start time and the status; a tap on a row: `openTraining` for a day that is not opened (a cancelled day is already a training), then `goto('/training/{date}')`. "Cancel training" on a row that is not opened. The switch "Show cancelled" (on by default; kept in `localStorage` in `try`/`catch`; works without it). "Add training": a date (inside the season), the times filled from `proposed_times` when a date is chosen, "Save" → the training page.
  - `/training/[date]` (first part): the date and times, the status, the players with their role badge or reason (read only in this step); a training that is not opened shows "Not opened" with "Open training".
  - Start page: the card below the team header in every state (from `GET /api/trainings/next`; no card for `null`), one line, a tap opens the training as a row of the list does; the link "Attendance" first in the line of links.
  - `just api-types`.
- **Tests** (browser, `web/e2e/trainings.e2e.ts`, new, after `players.e2e.ts` in the project `logged-in`; it adds its own Tuesday weekly training and players): the card "Training · Tue 13 Oct · 19:45 · Not opened yet" and the link "Attendance"; `/attendance` with "Coming" above "Past"; a tap on Tue 6 Oct opens it (all players "there"); back in the list the row shows "All there"; "Cancel training" on Tue 20 Oct → "Cancelled"; "Show cancelled" off hides it, on shows it; "Add training" on Thu 22 Oct with typed times and on Tue 3 Nov with the proposed times; a page reached by its address that is not opened shows "Open training"; no sideways scroll at 320 pixels, `expectAccessible`.

### Step 4: Marking and the training page
- **Goal**: taps on the phone, with the actions of a training.
- **Changes** (`/training/[date]`, `client.ts`: `markAbsent`, `markThere`, `changeTraining`, `uncancelTraining`, `deleteTraining`):
  - A tap on a row opens the reasons below it (not hidden ones; "There" on top for an absent player); a tap on a reason or "There" closes them.
  - Each tap: the row shows the new state at once; the request joins the queue of that player (a promise chain per player id: the next request starts after the previous ends). On success the stored state of that player becomes the answer, and the top line uses `absent`. On a failure the row shows the player's last stored state again, with "Not saved. Try again." (`role="alert"`).
  - `visibilitychange` to visible: load the training again when no request is waiting.
  - "Change" (date and times; on success `goto` the new date), "Cancel training" / "Undo cancel", "Delete training" (the confirmation of requirement 5; then back to `/attendance`).
  - "No player has a role on this date."; "No reasons. Add one in Reasons." with only "There" offered.
  - Rows at least 48 pixels high.
- **Tests** (browser): a tap on a player, "Ill" → the row shows "Ill", "1 absent"; a tap, "There" → "All there"; a failed tap (`page.route` answers 500) → the old state and "Not saved. Try again."; a late answer (the first `PUT` held back, a second tap follows) → the second request starts after the first, and the row and the stored state end as the second tap; "Cancel training" and "Undo cancel" on the page; "Change" to Wed 7 Oct → the list shows Wed 7 Oct as extra; "Delete training" with the confirmation; "No player has a role on this date." and "No reasons. Add one in Reasons." (replaced answers); no sideways scroll at 320 pixels, `expectAccessible`.
  - Mutation check: with the queue removed, the test of the late answer fails.

### Step 5: The parts of stories 1 and 4
- **Goal**: what changes elsewhere now that trainings and absences exist.
- **Changes**:
  - `DELETE /api/players/{id}`: a foreign-key violation (SQLSTATE 23503) → 400 `player_has_absences`; `client.ts` sentence "This player has absences and cannot be deleted. Remove the player from the team, or hide the player."
  - `now_there`: the routes "Add player" (`POST /api/players` with a role), "Add to the team" / "Add player" from the list (`POST /api/players/{id}/roles`), "Correct" (`PUT /api/players/{id}/roles/{role}`) and "Add all" (`POST /api/players/carry`) read the player's roles before and after the write, the trainings and absences of the season, and answer `now_there: [{ player, trainings: [date] }]` (only players with one or more). The field is added to `TeamPageView`, `PlayerDetail` and `SavedPlayer` (empty on `GET`). The team page and the player page show "This player now counts as there at N earlier trainings." (with the player's name on the team page when more than one) and the dates.
  - `GET /api/weekly-trainings` gets `proposals: [{ weekday, start, end, from, until }]`: `proposals_from(last season's weekly trainings, season)` when the season has none, else empty. `/settings` shows "Last season's weekly trainings" with each proposal and "Add" (the existing `POST /api/weekly-trainings`).
  - `just api-types`.
- **Tests**:
  - Server: `a_player_with_absences_cannot_be_deleted` (→ `player_has_absences`; the player and the absence stay; a player without absences → 204); `a_role_moved_earlier_counts_as_there` ("Correct" from Mon 12 Oct to Tue 1 Sep with trainings on 1 and 8 Sep → `now_there` with both dates; "Add player" from 1 Sep and "Add all" give the same); `last_seasons_weekly_trainings_are_proposed` (clock 15 Aug 2027: `proposals` with Tue 7 Sep 2027 – Tue 30 May 2028; after "Add" empty); `nothing_saved_yet` gets `"proposals": []`.
  - Mutation check: with `ON DELETE RESTRICT` changed to `CASCADE`, `a_player_with_absences_cannot_be_deleted` fails.
  - Browser: the message on the team page after "Add player" from 1 Sep with trainings since then; the delete error on the player page; "Last season's weekly trainings" and "Add" on `/settings` (replaced answers).

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| Attendance is stored as absences only; "there" follows from the training, the player's role and the absence | No rewrite on role changes; deleting a player without absences loses nothing; totals follow the data (epic) | yes, 0019 at Close |
| One training per date, guarded by a unique date; "Open" is `INSERT … ON CONFLICT DO NOTHING`, safe to send twice | Two opens give one training without a lock to forget (Phase 1, choice 2) | in 0019 |
| A training is opened by the tap on its row (list or card); a page load never writes | No training is made by a reload or a back button; "Not opened" is seen before the tap (Phase 1, choice 1) | in 0019 |
| Absences go with their training (`CASCADE`); a player with absences cannot be deleted (`RESTRICT`) | The database guards both; no race loses an absence | in 0019 |
| The taps of one player are sent one after the other; the screen shows the new state at once and puts the stored state back on a failure | A late answer never overwrites a newer tap; taps of different players do not wait (Phase 1, choice 3) | in 0019 |
| The routes of a training use its date (`/api/trainings/{date}/…`), not an id | The date is unique and is the page's address; differs from the epic's outline | no |
| The card is the first coming row of the current season; no separate rule | One rule for the list and the card | no |
| Last season's weekly trainings are proposed inside `GET /api/weekly-trainings` (`proposals`), saved with the existing "Add" | No new route; differs from the epic's outline | no |
| "Show cancelled" is kept in the browser | A view setting, not data (Phase 1, choice 4) | no |
| A mark for a player with no role on the date is refused | The page never offers it; absences outside a role still stay when a role moves (Phase 1, choice 5) | no |
| The page "Attendance" holds the list in story B; story C adds the totals and the choice of season | The spec's Interface puts the list there (coordinator, 2026-10-17) | no |

## Not in scope
- The totals, "Left the team" in the totals, "N training days not opened", the missed trainings of a player, the choice of an earlier season on the screen (story C).
- "Add training" in an earlier season; a history of marks; the place of a training; attendance at games.
- A backup of the database (backlog).

## Test plan
- **Existing tests that must still pass**: all of `just check`. Changed: `nothing_saved_yet` (`proposals`), the route test (new routes), `tests/player_logs.rs` (trainings), the start page test in `players.e2e.ts` (the link "Attendance").
- **New tests**: per step, above.
- **Checks**: `just check` after each step; `sqlx migrate run` and `just prepare` after steps 1 and 2 (and step 5 if a query changes); `just api-types` after each step that changes the API (1, 2, 3, 5).
- **Mutation checks**: `ON CONFLICT` in open (step 2), the queue per player (step 4), `RESTRICT` (step 5).

## Compatibility and performance
- Migration `0006` adds tables only; the running app works on it (decision 0004).
- The start page gets the card (when the season has a coming row) and the link "Attendance"; the games, the schedule and the list of changes stay as they are.
- The answers of the role routes and `GET /api/weekly-trainings` get new fields; the front end is updated in the same steps.
- Load: about 15 players, about 40 training days and trainings, a few hundred absences a season; the list and the card are worked out at each request.
- The data exists only in the tool and has no backup yet (accepted, backlog).

## Progress
- [x] Step 1: the data and the rules, read only (2026-10-17; `just check` passes apart from `api-types-check` before the commit, the types checked against the code; 159 core tests, 42 browser tests)
- [x] Step 2: the writes of a training (2026-10-17; `just check` passes apart from `api-types-check` before the commit, the types checked against the code; 42 browser tests)
- [x] Step 3: the page "Attendance" and the card (2026-10-17; `just check` passes apart from `api-types-check` before the commit, the types checked against the code; 48 browser tests)
- [x] Step 4: marking and the training page (2026-10-17; `just check` passes; 53 browser tests)
- [x] Step 5: the parts of stories 1 and 4 (2026-10-17; `just check` passes apart from `api-types-check` before the commit, the types checked against the code; 56 browser tests)
- [x] Review build (2026-10-17: R1–R7; all fixed or answered)
- [x] Review accept (2026-10-17: accepted with notes, A1–A6; all fixed or answered)
- [x] Fixes after the reviews (2026-10-17; `just check` passes apart from `api-types-check` before the commit, the types checked against the code; 159 core tests, 58 browser tests)

## Deviations
None from `functional.md`.

The design reads the spec more strictly within its words:
- "Add training" is for a date of the current season (the season of the list); trainings of an earlier season can be opened, marked, changed and cancelled, not added.
- A training day of an earlier season is opened through its address; the list shows the current season (coordinator, 2026-10-17).
- "Open" of a date that is not a training day is refused; the screens offer it only for training days.
- A mark for a player with no role on the date is refused.
- "This player now counts as there at N earlier trainings" lists every training the role now covers and did not cover before, also an opened training in the future (an absence noted in advance), without cancelled trainings and trainings where the player has an absence. "Change role", "Remove from the team" and "Undo" give no such message (they never add a covered date that was not covered before, except "Undo", which restores).
- Last season's weekly trainings are proposed one per day of the week (the newest of that day), so two proposals never overlap.

Implement:
- 2026-10-17, step 1:
  - Core `absent_count(training, absences, roles)` is public: `season_list` uses it, and so does the training page (and step 2's `Marked`), so "N absent" has one rule.
  - `TrainingList` also gives `season_first_day` and `season_last_day`, for the limits of the date field of "Add training" (step 3), as `TeamPageView` does.
  - The server tests of step 1 write trainings and absences with plain SQL, because the write routes come in step 2.
  - Added beyond the step's list: the server test `a_date_or_season_that_cannot_be_read_is_refused` (`invalid_date`, `invalid_season`).
- 2026-10-17, step 2:
  - The two tests of a date saved at the same time do not use `tokio::join!`: two requests on one task do not really run at the same time, so the test passed also without `ON CONFLICT`. Each test holds an insert of the date in an open transaction on another connection; the request must wait 300 ms, and after the commit "open" answers the one training, "Add training" `training_exists`. Mutation checks: without `ON CONFLICT` in open, `open_twice_gives_one_training` fails (500); without the mapping of the unique violation, `two_adds_of_one_date_give_one_training` fails.
  - "Add training" and "Change" take the times as "19:45" text (as the weekly trainings), so a time that cannot be read answers `invalid_time`.
  - "Open" of a date that already has a training (also an extra training) answers that training and writes nothing.
  - Logs: "training opened", "training added", "training changed", "training cancelled", "training cancel undone", "training deleted", "absence marked", "absence removed", with the date and the player id only.
- 2026-10-17, step 3:
  - The season parameter of `GET /api/trainings` was described as a path parameter in the OpenAPI description (step 1); it is now a query parameter (`parameter_in = Query`), as the front end needs.
  - The browser tests of trainings are a Playwright project of their own (`trainings`, after `logged-in`): they share the database, and the settings test of the players expects no weekly training at its start.
  - On the list, the button of a row that is not opened reads "Cancel" (its accessible name is "Cancel training of Tue 20 Oct"), so the row fits at 320 pixels.
  - A season with no training days and no trainings shows "No training days or trainings this season. Add a weekly training in Settings." (the spec names no empty state for the list in story B; story C adds "No trainings yet this season." to the totals).
  - The card is on the start page in every state with a login (it needs no chosen team), below the line of links, which now starts with "Attendance".
- 2026-10-17, step 4:
  - The top line ("N absent", "All there") counts the marks on the screen, so it changes with each tap at once; the answer of a tap (`Marked.absent`) is not needed for it.
  - A failed tap puts back the player's last stored state only when it is the player's newest tap; an older failed tap only shows "Not saved. Try again.", and the newest tap decides the row.
  - The queues are plain objects, not reactive (they are not shown).
  - "Change" to another date goes to the page of the new date (the address is replaced, so "Back" does not lead to the old date).
  - Browser tests: "Change" moves the extra training of Thu 22 Oct to Wed 21 Oct (not Tue 6 Oct to Wed 7 Oct as planned: Tue 6 Oct holds the marks of the other tests); "Delete training" deletes the training of Tue 3 Nov from step 3. With no reasons, a player who is there gets no choices, and an absent player only "There".
  - Mutation check: with the queue removed, the test of the late answer fails (the whole test file run; with `-g` alone the test lacks the player of the tests before it).
- 2026-10-17, step 5:
  - `now_there` is a field of `PlayerDetail` (so "Add player" carries it inside `SavedPlayer.player`, and "Correct" answers it) and of `TeamPageView` ("Add to the team", "Add player" from the list, "Add all"); it is empty on `GET`. The routes read the player's roles before the write and again inside the same transaction after it.
  - `StoreError::is_foreign_key_violation` turns the refused delete into `player_has_absences`, as `is_unique_violation` does for names.
  - The message uses "1 earlier training" for one training ("This player now counts as there at 1 earlier training: …"), and the dates as on the schedule. On the team page, "Add player" for a new player loads the team page and shows the message from the new player's answer.
  - The proposals of last season are shown above the list of weekly trainings, each with "Add" (its accessible name names the day and times); after "Add" the answer has no proposals, so the part goes.
  - Mutation check: with `ON DELETE CASCADE` for the player in migration `0006`, `a_player_with_absences_cannot_be_deleted` fails (the delete answers 204).
- 2026-10-17, fixes after the reviews (wider than the plan; the coordinator chose the answers to A3 and A6):
  - Proposals of last season: only those that ran until its end, one per day of the week without a weekly training in the season (R1, A1, A6).
  - The training page: a "saving…" mark on a row while a tap waits, and a time limit of 10 s for a tap (A3); answers of a reload or an action that were sent before a tap do not undo it (R2).
  - `PlayerDetail.has_absences`; "Delete player" only without absences (A4).
  - `api/format.rs` for the date and time helpers (R4); the sentence of `invalid_date` (R5).
  - Mutation checks: without the check of the reload, and without the time limit, their browser tests fail.

## Review findings

### Build review (2026-10-17)
Input: mode `build`, this change folder; no other text was given. Read: the diff against `main`
(5 commits, nothing uncommitted apart from untracked local files), this folder, the epic's
`functional.md` where the intent was in doubt, `docs/architecture.md`, decisions 0001 and 0018,
`CLAUDE.md`. Run: `cargo test -p team_core -p team` (all pass, `tests/trainings.rs` 17 tests),
`cargo clippy --all-targets -D warnings` (clean), `just api-types-check` (no difference). The
browser tests were not run.

No finding breaks a decision: rules sit in the core (`season_list`, `absent_count`,
`check_training`, `day_times`, `proposed_times`, `proposals_from`, `players_at`, `now_there`,
`check_mark`), the server only moves data, the routes are on the protected router, one training per
date is guarded by the unique index, `CASCADE`/`RESTRICT` guard the absences, and logs carry dates
and ids only.

- **R1 – "Last season's weekly trainings" offers only the first one (bug against the spec's intent).**
  `api/weekly_trainings.rs` `load` gives `proposals` only while the season has no weekly trainings
  (`if stored.is_empty()`). With a Tuesday and a Thursday last season, "Add" on Tuesday makes the
  answer's `proposals` empty, so the settings page drops the Thursday proposal before it is added.
  The epic says "the settings page proposes the weekly trainings of last season" (plural), and
  `proposals_from` deliberately returns one per day of the week. Suggest: leave out only the
  proposals whose day of the week already has a weekly training in the season (or that overlap one
  by `check_weekly_training`), and stop proposing once none is left. Siblings: the browser test
  replaces the answer with a single proposal, and the server test
  `last_seasons_weekly_trainings_are_proposed` checks "after Add empty" with one weekly training,
  so neither can see this. Checked by trace (`load`, `settings/+page.svelte` `runWeekly`
  sets `proposals = answer.proposals`); not run.
  - 2026-10-17: fixed. Core `proposals_from(last, current, season)` leaves out only the days of the week that have a weekly training in the season, so the second proposal stays after "Add" of the first. Core test with Tuesday and Thursday; server test `last_seasons_weekly_trainings_are_proposed` with two days (Thursday still proposed after "Add" of Tuesday).
- **R2 – A reload on "shown again" can overwrite a newer tap on the screen.** In
  `training/[date]/+page.svelte` the `visibilitychange` handler checks `waiting > 0` before it sends
  `loadTraining`, not when the answer comes. Phone unlocked in the hall, a tap at once on a slow
  network: the reload's answer (read before the tap was stored) replaces `day`, and so `shown` and
  `stored`; if it comes after the tap's answer, the row shows the old state although the tap was
  saved (the opposite of "a late answer never overwrites a newer tap"; it never shows an unsaved
  mark as saved). The same holds for the answer of "Cancel training" / "Undo cancel" (`run`) while
  a tap is waiting. Suggest: drop a page answer when a tap started after the request was sent (a
  counter of taps compared before and after), or keep the shown marks of players with a tap since.
  Checked by trace; not reproduced.
  - 2026-10-17: fixed. Every tap is counted; the answer of the reload when the page is shown again is used only when no tap was made since it was sent, and an action ("Open", "Cancel training", "Undo cancel") keeps the marks on the screen when a tap was made while it ran. Browser test `a reload that was sent before a tap does not undo the tap` (the reload held back 1.5 s); it fails without the check.
- **R3 – Decision number 0019 now means two decisions.** Migration `0005` (applied, cannot be
  edited) calls the players decision "decision 0019"; decision 0018 records that this was the
  number planned before Close. The new code uses "decision 0019" for the attendance decision
  (`migrations/0006_trainings.sql`, `crates/server/src/trainings.rs`,
  `crates/core/src/attendance.rs`), and the Decisions table plans 0019 at Close. A reader of `0005`
  who follows "0019" lands on the attendance decision. For Close: either say so in decision 0019
  (as 0018 does for `0005`), or give the new record another number and fix the three new comments.
  Checked by search (`grep -rn 0019 crates web/src migrations`).
  - 2026-10-17: not fixed here: a choice for Close (the header of the applied migration `0005` cannot change).
- **R4 – Small helpers built a third time.** `api/trainings.rs` defines `day`, `clock_time` and
  `time` again; `api/weekly_trainings.rs` has the same three and `api/players.rs` has `day`.
  `trainings::day` and `path_date` are `pub(crate)` but used nowhere else. Not a rule break; one
  shared place (for example in `api/mod.rs`) keeps the date and time formats from drifting. Checked
  by search (`grep -rn "fn day(\|fn clock_time(\|fn time(" crates/server/src`).
  - 2026-10-17: fixed. `api/format.rs` holds `day`, `clock_time`, `time` and `path_date`; the API files use it.
- **R5 – `invalid_date` has no sentence.** Step 3 lists the sentence of `invalid_date` for
  `client.ts`; `messageFor` has none, so a date that cannot be read answers "Something went wrong.
  Please try again." Low weight: the screens send dates from date fields. Checked by search in
  `web/src/lib/api/client.ts`.
  - 2026-10-17: fixed. `client.ts` says "This date cannot be read." for `invalid_date`.
- **R6 – Claims checked.** The "checks pass" lines hold today (tests, clippy and api types run
  above). The concurrency tests of "open" and "Add training" hold an insert open on another
  connection and assert the request waits, so they can fail (read in `tests/trainings.rs`); the
  `tokio::join!` deviation is a real improvement. The mutation checks were not repeated. The browser
  test of the late answer depends on the marks of the tests before it in the file (the builder says
  so); it is not wrong, but it fails alone with `-g`, unlike the pattern of self-contained tests.
  - 2026-10-17: not changed: the training browser tests share one database and run in order on purpose (their own Playwright project after the player tests); each step's notes name what a test uses from the tests before it.
- **R7 – Docs the diff made false (for Close).** `docs/architecture.md` (core `training`,
  `attendance`, server `trainings.rs`, `api/trainings`, migration `0006`, the pages `/attendance`
  and `/training/[date]`, the start page card) and the Architecture list in `CLAUDE.md` do not yet
  name the new parts; Close updates them. In this spec, step 1 names storage `absences_of`; the
  code has `absences_at`. Checked by reading.
  - 2026-10-17: not fixed here: the living documents are updated at Close.

Not findings (checked, intended or documented): a mark is not under the player's lock (a race with
a role change can only leave an absence outside a role, which decision 0018 keeps and does not
count); "Delete player" is shown always and refused with the spec's sentence (the Interface says
"when the player has no absences": for the accept review); `Marked.absent` is unused by the front
end (deviation, step 4).

**Verdict**: no blocker against the project's rules. R1 is a bug against the spec's intent and
should be fixed before Close; R2 is a narrow but real case of the hall scenario; R3 needs a choice
at Close; R4–R7 are small.

### Accept review (2026-10-17)
Input: mode `accept`, this change folder; no other text was given. Read: `change.md`, the epic's
`functional.md` (stories 1–4, Interface, warnings, the changes after approval), this spec's
objective and requirements, the diff against `main` (the pages `/attendance`, `/training/[date]`,
the start page card, `/team`, `/players/[id]`, `/settings`, `client.ts`, the core `training` and
`attendance`, the routes). View formed before reading **Deviations** and the build review. Run:
`cargo test -p team --test trainings` (17 pass). The app and the browser tests were not run (the
browser tests share the dev database).

What matches the intent (checked by reading the pages against the criteria): opening by a tap
stores the training with everyone there, and a page load never writes; one row per date with
"Coming" above "Past"; "Not opened" / "All there" / "N absent" / "Cancelled"; "Cancel" on a day
that is not opened; "Show cancelled" on by default; "Add training" with proposed or typed times;
"Change" keeps the attendance; "Delete training" with the spec's confirmation; "Undo cancel" with
the absences kept; the card with "Not opened yet" and "Cancelled" (no time then), until midnight;
one tap per mark, "There" on top for an absent player, hidden reasons not offered but still shown;
hidden players with a role are at the training; rows 48 pixels high; "No player has a role on this
date." and "No reasons. Add one in Reasons."; "now counts as there" with the dates after "Add
player", "Add to the team", "Add all" and "Correct"; last season's weekly trainings on `/settings`.
The October setup (weekly training, then Tue 1 Sep onward, one tap per Tuesday or "Cancel" for a
holiday) works as the spec's Behaviour tells it.

- **A1 – Concerning deviation: a team that trains twice a week gets only one of last season's
  weekly trainings proposed.** Same cause as R1, judged here against the user: story 2 says "A team
  that trains twice a week has two weekly trainings" and "the settings page proposes the weekly
  trainings of last season". `load` in `api/weekly_trainings.rs` proposes only while the season has
  none (`if stored.is_empty()`), and `/settings` replaces `proposals` with each answer, so after
  "Add" on the first proposal the second is gone for good; the coordinator must type it by hand
  without being told. It does not bite the coordinator's own team (Tuesday only) and not before 1
  July 2027, but it is a wrong result, not a choice. Fix in the build (no spec change): drop only
  the proposals whose day already has a weekly training in the season. Checked by trace; not run.
  - 2026-10-17: fixed with R1.
- **A2 – Neutral drift: an earlier season's training is reachable only by typing its address.**
  The spec's example is "the last training of May found in July". From 1 July the list shows the
  new season, and nothing on the screens leads to 2026-2027 (only `/training/2027-05-25` by hand).
  The deviation says so and the coordinator chose it (2026-10-17), with the choice of season in
  story C. Faithful only if story C is released before 1 July 2027; otherwise the criterion is not
  met for the user. Checked by reading `/attendance` (`loadTrainings` without a season) and
  `+page.ts`.
  - 2026-10-17: not changed (the coordinator's choice, 2026-10-17); a backlog line asks for story C before 1 July 2027.
- **A3 – Neutral drift: a waiting tap looks the same as a saved one.** The spec wants the row to
  change at once and "show the old state again" on a failure, so showing the tap before the answer
  is intended. But nothing marks a tap that is still waiting, and requests have no time limit
  (search `timeout|AbortSignal|signal` in `client.ts`: none). On a weak hall network the row shows
  "Ill" for as long as the browser waits; when it then fails, "Not saved. Try again." appears on a
  phone that may be back in the pocket, and leaving the page drops the message (no warning while a
  tap waits). The list then shows the stored count, so the error is visible later, but not marked
  as such. This is at the edge of the warning "a tap that was not saved must never look saved".
  Suggest for a later step or story C: a quiet "saving" mark on a waiting row, a time limit on
  mark requests, or a warning when leaving with a waiting or failed tap. R2 is the other side of
  the same scenario. Checked by reading `training/[date]/+page.svelte`; not run.
  - 2026-10-17: fixed (coordinator's choice: a "saving" mark and a time limit of 10 s). The row shows "saving…" while a tap of that player waits; a tap without an answer after 10 s (`TAP_TIMEOUT_MS`, `AbortSignal.timeout`) counts as not saved: the row shows the stored state and "Not saved. Try again.". Browser test `a tap without an answer for 10 seconds counts as not saved`; it fails without the time limit. A tap that reaches the server after its time limit is saved there but shown as not saved until the page loads again: never a mark that looks saved when it is not.
- **A4 – Neutral drift: "Delete player" shows also for a player with absences.** The Interface says
  the player page has "Delete player" "when the player has no absences"; the page shows it always,
  asks "This cannot be undone.", and only then refuses with the spec's sentence. The outcome is
  safe and the sentence tells what to do; one useless confirmation. Checked by reading
  `players/[id]/+page.svelte` and `api/players.rs` `delete`.
  - 2026-10-17: fixed. `PlayerDetail.has_absences`; the player page offers "Delete player" only without absences (the server still refuses it). Server test (in `a_player_with_absences_cannot_be_deleted`) and browser test `a player with absences is not offered "Delete player"`.
- **A5 – Faithful adaptations.** "Add training" only in the current season (the spec lists open,
  enter, change and cancel for an earlier season, not add); "Open" refused on a date that is not a
  training day (the screens offer it only for training days; a typed address of such a date gets
  "This date is not a training day."); a mark for a player with no role refused (never offered);
  "now counts as there" also lists an opened future training (the spec says "on or after that
  date"; the word "earlier" in the sentence is the spec's own); one proposal per day of the week;
  the empty list's sentence pointing to Settings; "extra" on the row of an extra training; the
  card in every state with a login. None changes what the coordinator meant.
  - 2026-10-17: nothing to fix.
- **A6 – Neutral, for later.** When last season moved a weekly training ("Tuesday until January,
  Thursday from Thu 7 Jan", the spec's own example of a real change), both days are proposed for
  the new season, also the Tuesday that was given up. The coordinator can leave it, so no harm;
  worth a thought in story C or at Learn. Checked by reading `proposals_from`.
  - 2026-10-17: fixed (coordinator's choice): only the weekly trainings that ran until the end of last season (their window ends in its last week of training days) are proposed. Core test: Tuesday until January and Thursday from January → only Thursday.

**Verdict**: **accepted with notes.** What was built is what the coordinator meant for trainings,
the list, the card and marking. A1 must be fixed in the build before Close (as R1); A2 depends on
story C coming before 1 July 2027; A3 is worth a small follow-up for the hall scenario; A4–A6 need
no action now.
