# Technical spec: Checked trainings

**Status**: approved
**Size**: story
**Change**: [2026-10-25-checked-trainings](./)
**Date**: 2026-10-25

## Objective
A check on each training: the coordinator confirms that its attendance is complete, so that a
past training nobody looked at can stay in view (the open items of story B use it). At the release
every training before today counts as checked.

## Requirements summary
Story C of the epic [Start page and season](../2026-10-25-start-page-and-season/) (`functional.md`
approved 2026-10-25, story C; epic `technical.md` approved 2026-10-25):

- A training (opened or added) whose date is today or earlier has "Checked" on its page. A tap saves at once; the page then shows "Checked" (a state, not a button) with "Undo check". "Undo check" saves at once and shows "Checked" (the button) again.
- A save that fails shows "Not saved. Try again." and the page shows what is stored, as for marking.
- A checked training stays checked when the coordinator marks a player, changes its times or date (also to a date after today: it then shows "Checked" with "Undo check", and is no open item when that date passes), cancels it or undoes the cancel.
- "Delete training" removes the training with its check; opened again, it is not checked.
- A training after today that is not checked has no "Checked".
- "Season" shows "Checked" on the row of a checked training, next to "All there" or "N absent".
- A training of an earlier season can be checked and unchecked as well.
- At the release, every training before today (Dutch date) counts as checked; trainings of today and later start not checked.
- Two screens: a check and a mark at the same time are both kept; "Checked" and "Undo check" on two screens: the last one wins.
- Not in this story: the open item "Training Thu 22 Oct not checked" on the start page (story B); this story gives the core function `not_checked` that B uses.

## Current state
- **Table `trainings`** (migration `0006`): `id`, `date` (unique), `start_time`, `end_time`, `cancelled_at`, `created_at`. `absences` has a foreign key to it (`ON DELETE CASCADE`). The last migration is `0009_vacuum_players`.
- **Core** `training.rs`: `Training { id, date, start, end, cancelled }`; `TrainingError` (mapped to 400 codes in `api/trainings.rs`); `season_list` gives `SeasonRow { date, start, end, training: Option<&Training>, extra, status }`; `not_opened(list)`. Test helpers `training(id, on)` in `training.rs` and `attendance.rs` build `Training` values.
- **Store** `trainings.rs`: `TrainingRow` and `From<TrainingRow> for Training`; `for_season` and `on` select `cancelled_at IS NOT NULL AS cancelled`; `open`, `insert`, `update` (Change), `cancel` (an insert `ON CONFLICT (date) DO UPDATE` for a day not opened), `uncancel`, `delete`, `mark`, `unmark`.
- **API** `api/trainings.rs`: every action on a training answers the training page `TrainingDay` (with `training: Option<TrainingView { start, end, cancelled }>`) through `day_of`; `TrainingRow` (the row of a list) through `row_view`; `GET /api/season` uses `row_view`.
- **Front end**: the training page `web/src/routes/training/[date]/+page.svelte` (marks with "saving…" and "Not saved. Try again.", and a path for "an action that does not change the marks" that keeps a tap that is still being sent); `SeasonEvents.svelte` (rows of "Season"); `trainings.ts` (`statusText`); `client.ts` (`messageFor`).
- **Patterns**: the race tests of `docs/architecture.md` ("Patterns to follow"); migrations forward only (decision 0004); `tests/health.rs` uses `#[sqlx::test(migrations = false)]`; logs carry dates and ids only.

## Steps

### Step 1: The migration
- **Goal**: the column, and the trainings before the release day checked once.
- **Changes**: `migrations/0010_trainings_checked.sql`:
  ```sql
  -- A training is checked when the coordinator confirmed that its attendance is complete (story C
  -- of start page and season). At the release every training before today (Dutch date) counts as
  -- checked, so the start page does not show them all as open items.
  ALTER TABLE trainings ADD COLUMN checked_at timestamptz;
  UPDATE trainings SET checked_at = now()
  WHERE date < (now() AT TIME ZONE 'Europe/Amsterdam')::date;
  ```
  One transaction (the default): a failure changes nothing, and the next start runs it again. The old app ignores the column. Apply locally with `sqlx migrate run` (or `just migrate`).
- **Tests**: `crates/server/tests/migrations.rs` (new), `#[sqlx::test(migrations = false)]`:
  - `the_release_checks_the_trainings_before_today`: run the `sql` of the migrations before 0010 from `team::db::MIGRATOR` with `sqlx::raw_sql`; insert trainings on yesterday, today and tomorrow (Dutch date, `(now() AT TIME ZONE 'Europe/Amsterdam')::date ± 1` in SQL); run the `sql` of 0010; yesterday has `checked_at`, today and tomorrow have none.
  - Once by hand: without the `UPDATE` the test fails (write the result under **Progress**).

### Step 2: Core, and the store reads the check
- **Goal**: `Training.checked` everywhere a training is read; the rules of a check and of "not checked".
- **Changes**:
  - `crates/core/src/training.rs`:
    ```rust
    pub struct Training { /* … */ pub cancelled: bool,
        /// The coordinator confirmed that the attendance is complete.
        pub checked: bool }
    pub enum TrainingError { /* … */
        /// No training on the date (a training day that is not opened).
        NotOpened,
        /// The date is after today, and the training is not checked yet.
        NotYet }
    /// A check of the training on its date: there must be one, and its date must be today or
    /// earlier unless it is already checked (a checked training moved later stays checked).
    pub fn check_checking(training: Option<&Training>, today: NaiveDate) -> Result<(), TrainingError>;
    /// The trainings before today that are not cancelled and not checked, oldest first.
    pub fn not_checked(list: &SeasonList<'_>) -> Vec<NaiveDate>;
    ```
    Exported in `lib.rs`. The test helpers `training(id, on)` in `training.rs` and `attendance.rs` get `checked: false`.
  - `crates/server/src/trainings.rs`: `TrainingRow` gets `checked: Option<bool>`; `for_season` and `on` select `checked_at IS NOT NULL AS checked`; `From` fills `checked`. `just prepare`.
  - `api/trainings.rs`: the new `TrainingError` variants map to `BadRequest("not_opened")` and `BadRequest("not_yet")`.
- **Tests** (core):
  - `checking_needs_a_training` (`None` → `NotOpened`)
  - `checking_a_training_after_today_waits_for_its_date` (tomorrow not checked → `NotYet`; today and yesterday → `Ok`)
  - `a_checked_training_after_today_can_be_checked_again` (tomorrow checked → `Ok`)
  - `not_checked_leaves_out_checked_cancelled_and_coming_trainings` (only the past training that is not checked, oldest first; no training day not opened, no training of today)
  - All existing tests pass.

### Step 3: The routes
- **Goal**: check and undo through the API.
- **Changes**:
  - `crates/server/src/trainings.rs`:
    ```rust
    /// Checks the training on `date`; the first moment stays. False: no training on the date.
    pub async fn check(db, date: NaiveDate, now: DateTime<Utc>) -> Result<bool, StoreError>;
    //   UPDATE trainings SET checked_at = COALESCE(checked_at, $2) WHERE date = $1
    /// Undoes the check. False: no training on the date.
    pub async fn uncheck(db, date: NaiveDate) -> Result<bool, StoreError>;
    //   UPDATE trainings SET checked_at = NULL WHERE date = $1
    ```
  - `crates/server/src/api/trainings.rs`: `PUT /api/trainings/{date}/checked` (`check`): `path_date`; `check_checking(trainings::on(date).as_ref(), today)`; `trainings::check`, and `not_opened` when it finds no row (deleted or moved meanwhile); `tracing::info!(%date, "training checked")`; answer `day_of(date)`. `DELETE /api/trainings/{date}/checked` (`uncheck`): `trainings::uncheck`, `not_opened` without a row; log "training check undone"; answer `day_of(date)`. `TrainingView` and `TrainingRow` get `checked: bool` (in `fill` and `row_view`). No lock: neither touches `absences`.
  - `app.rs`: `.route("/trainings/{date}/checked", put(trainings::check).delete(trainings::uncheck))`; `api/mod.rs`: both paths; `just api-types`; `just prepare`.
- **Tests** (`tests/trainings.rs`):
  - `check_and_undo_check_a_training` (open yesterday's training; `PUT` gives `training.checked: true`, also in `GET /api/trainings/{date}` and in the row of `GET /api/season`; `DELETE` gives `false`)
  - `check_errors_have_their_codes` (`PUT` on a day not opened → 400 `not_opened`; `PUT` on tomorrow's opened training → 400 `not_yet`; `DELETE` without a training → 400 `not_opened`; a date that cannot be read → 400 `invalid_date`)
  - `a_check_stays_through_marks_cancel_and_change` (check; mark absent, "Cancel training", "Undo cancel", "Change" to next week: each answer `checked: true`)
  - `delete_training_removes_the_check` (check, delete, open again: `checked: false`)
  - `a_check_does_not_wait_for_a_mark`: on a second connection, an open transaction inserts an absence for the training; `PUT …/checked` answers within 300 ms; after the commit both the absence and the check are stored. Once by hand: with a `SELECT … FOR UPDATE` of the training in the check, the test fails (write the result under **Progress**).
  - The exact JSON of `page["training"]` (`the_training_page_lists_the_players_with_a_role`) and of a row (`the_list_has_one_row_per_date`) gets `"checked": false`.
  - `tests/players.rs`: `PUT` and `DELETE /api/trainings/2026-10-13/checked` in the routes that need a login. `tests/player_logs.rs`: a check and an undo, no player names in the log.

### Step 4: The training page and the row on "Season"
- **Goal**: "Checked" and "Undo check" on the page; "Checked" on the row.
- **Changes**:
  - `client.ts`: `checkTraining(date)` and `uncheckTraining(date)` (both `TrainingDay`); `messageFor`: `not_opened` "This training is not opened yet.", `not_yet` "A training after today cannot be checked yet." (the page never offers these; a safety net).
  - The training page, below the top line, when a training exists: not checked and its date today or earlier: the button "Checked"; checked: a line with a check mark and the text "Checked" (`data-testid="checked"`) and the button "Undo check"; not checked and after today: nothing. Both buttons use the page's path for an action that does not change the marks; on a failure the page shows "Not saved. Try again." and loads the stored training again.
  - `trainings.ts`: `checkedText(row)` ("· Checked" or ""); `SeasonEvents.svelte`: after the status of a checked training ("All there · Checked").
- **Tests** (`web/e2e/trainings.e2e.ts`, with the tests of a training):
  - "Checked" on a past training shows the text "Checked" and "Undo check"; after a reload it is still checked.
  - "Undo check" shows the button "Checked" again.
  - A failed save (`page.route` answers 500 to `…/checked`) shows "Not saved. Try again." and the stored state.
  - A coming opened training has no "Checked".
  - "Season" shows "Checked" on the row of the checked training.
  - The training page passes the accessibility check and fits a small phone.

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| `trainings.checked_at timestamptz`, `NULL` = not checked; the first moment stays | Keeps when it was checked at no cost | In 0022 at the close of the epic |
| The migration checks every training before the release day (Dutch date) | Old trainings must not become open items | In 0022 |
| The two routes answer the training page, as the other actions on a training | One path in the page; the coordinator: a larger answer is no problem (2026-10-25) | No |
| No lock: one `UPDATE` after `check_checking`; a delete or move at the same moment gives `not_opened` | One column, the last save wins; nothing is lost without a sign | No |
| The migration test runs the SQL of the migrations by hand | Only public parts of sqlx (`Migration::sql`); the `Migrator` fields are hidden | No |

## Not in scope
- The open item "Training … not checked" and the start page (story B); "Checked" on "Attendance"; any other change of the training page.

## Test plan
- **Existing tests that must still pass**: all core tests (helpers get `checked: false`); `tests/trainings.rs` (two exact JSON values get `"checked": false`), `tests/season.rs`, `tests/attendance.rs`, `tests/players.rs`, `tests/player_logs.rs`; `web/e2e/trainings.e2e.ts`, `season.e2e.ts`.
- **New tests**: per step, above.
- **Not testable here**: the migration on the cluster's release day (the image runs it once through `team migrate`, and `scripts/smoke-test.sh` runs the image).
- **Checks while working**: `cargo test -p team_core`; `cargo test -p team --test migrations --test trainings --test season`; `just check-web`, then `cd web && npx playwright test --project trainings`; `cargo clippy --workspace --all-targets -- -D warnings` at the end of each step; `just check` once before the commit.

## Compatibility and performance
- The first migration since v0.3.2: one column and one `UPDATE` of about 30 rows. The old app runs on it. A wrong check is undone one by one with "Undo check".
- Release story A and this story together (release 1 of the epic). After the release, check in the cluster that the trainings before the release day have a check.
- New routes; `TrainingView` and `TrainingRow` get one field (the front-end types are generated).

## Progress
- [ ] Step 1 – the migration
- [ ] Step 2 – core, and the store reads the check
- [ ] Step 3 – the routes
- [ ] Step 4 – the training page and the row on "Season"
- [ ] All checks pass

## Deviations
None from `functional.md`. Differences from the epic's `technical.md` and readings of the spec:
- The two routes answer the training page (`TrainingDay`), not `{ checked }` (the pattern of the other actions; agreed with the coordinator, 2026-10-25).
- The migration test runs the SQL of the migrations by hand, as the epic allowed ("the statement on its own, written down").
- A cancelled training can be checked: the spec does not forbid it, and a cancelled training is never an open item.

## Review findings
