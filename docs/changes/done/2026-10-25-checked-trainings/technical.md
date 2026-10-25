# Technical spec: Checked trainings

**Status**: done
**Size**: story
**Change**: [2026-10-25-checked-trainings](./)
**Date**: 2026-10-25

## Objective
A check on each training: the coordinator confirms that its attendance is complete, so that a
past training nobody looked at can stay in view (the open items of story B use it). At the release
every training before today counts as checked.

## Requirements summary
Story C of the epic [Start page and season](../../2026-10-25-start-page-and-season/) (`functional.md`
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
| The check reads the training with a weak row lock (`FOR NO KEY UPDATE`) and writes in one transaction; "Undo check" is one `UPDATE` | The rule and the write cannot be split by another check or undo (build review R3); a mark is not blocked; a delete or move gives `not_opened` | No |
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
- [x] Step 1 – the migration (2026-10-25: the test fails without the `UPDATE`: all three trainings stay unchecked)
- [x] Step 2 – core, and the store reads the check
- [x] Step 3 – the routes (2026-10-25: with a `SELECT … FOR UPDATE` of the training in the check, `a_check_does_not_wait_for_a_mark` fails with "the check waited for the mark")
- [x] Step 4 – the training page and the row on "Season"
- [x] All checks pass (`just check`, 2026-10-25: 84 browser tests)

## Deviations
None from `functional.md`. Differences from the epic's `technical.md` and readings of the spec:
- The two routes answer the training page (`TrainingDay`), not `{ checked }` (the pattern of the other actions; agreed with the coordinator, 2026-10-25).
- The migration test runs the SQL of the migrations by hand, as the epic allowed ("the statement on its own, written down").
- A cancelled training that is not checked cannot be checked (after the build review, R4, coordinator): it has no attendance to confirm. A check made before "Cancel" stays, as the spec says.

Found while building (2026-10-25):
- Step 2: the mapping of `TrainingError` to API codes is in `api/weekly_trainings.rs` (one `From<TrainingError> for AppError` for both files), not in `api/trainings.rs`; `NotOpened` and `NotYet` are added there.
- Step 3: four tests compare an exact JSON of a training and got `"checked": false`, not two: also `add_and_change_a_training` and `cancel_a_day_that_is_not_opened`.
- Step 4: the training page (`TrainingView`) has one more field, `can_check` ("Checked" is offered: not checked, and its date today or earlier, from `check_checking` with the server's Dutch today). The page cannot work out "after today" itself: in the browser tests the server's clock is 12 October 2026 and the browser has the real date. Tested in `check_and_undo_check_a_training` and `check_errors_have_their_codes`.
- Step 4: "Checked" and "Undo check" use their own `saveCheck` next to `run`: on a failure they show "Not saved. Try again." and load the stored training again (as a mark), where `run` shows the error message.

## Review findings
Build review, 2026-10-25 (fresh agent; input: mode `build`, this folder). Read: the diff against
`main` (committed and not), this folder, the epic's `functional.md` (story C) and `technical.md`,
`docs/architecture.md`, `CLAUDE.md`. Run: `cargo test -p team_core` (189 pass);
`cargo test -p team --test migrations --test trainings --test players --test player_logs --test season`
(all pass); `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check` (clean);
`just api-types-check` (clean); `just check-web`, then `npx playwright test --project trainings`
(66 pass). Not run: `just check-sqlx`, the full `just check`.

What holds (checked by trace): the rules are in the core (`check_checking`, `not_checked`), the API
only maps them; no `unwrap`/`expect` outside tests; the logs carry the date only; the new routes are
in the login test. The race test is sound: the absence insert takes `FOR KEY SHARE` on the training
row (foreign key), the check's `UPDATE` of a column outside any index takes `FOR NO KEY UPDATE`,
which does not conflict; a `SELECT … FOR UPDATE` does, so the claim under **Progress** that the test
then fails is right. The migration test's claim (fails without the `UPDATE`) is right: all three
rows keep `checked_at` `NULL`. `not_checked` uses `past` of `season_list`, which is strictly before
today and includes extra trainings.

- **R1 – a test that cannot fail on the part it names** (trace of `saveCheck` and the test
  'a check that cannot be saved …'). `saveCheck` changes nothing on the screen before the answer, and
  the mocked 500 never reaches the server, so the stored state equals the state before the tap. The
  test passes just as well when the page does not load the stored training again. The reload matters
  when the server stored the check but the answer was lost; that path is not tested. Suggestion: in
  `page.route`, `await route.fetch()` first (the server stores the check), then fulfill 500, and
  expect "Not saved. Try again." with the `checked` line (and "Undo check").
- **R2 – the two new error texts are never shown** (search `messageFor(` in `web/src`; trace of
  `saveCheck`). `saveCheck` drops the error and always shows "Not saved. Try again.", so
  `not_opened` and `not_yet` in `messageFor` cannot reach the screen; Step 4 calls them "a safety
  net". `not_opened` is reachable from two screens: the training was deleted or moved on the other
  screen (`not_yet` is not: the page offers "Checked" only for a date of today or earlier, and that
  stays so). The page then says "Try again", and after the reload there is nothing to try again. Small: the reload shows the right state. Either show `message(failure)` for
  an `ApiError` with status 400 (the stored state still loads), or remove the two entries and the
  claim.
- **R3 – the rule and the write are two statements** (trace of `api/trainings.rs::check`; the
  epic's `technical.md` says "one `UPDATE` … after `check_checking`"). `trainings::on` and
  `trainings::check` run outside a transaction. In between, another screen can "Undo check" on a
  checked training after today; the `UPDATE` then checks a training after today that is not checked,
  which is the state `NotYet` forbids. Afterwards it shows "Checked" with "Undo check", and when
  its date passes it is not an open item. The page cannot send this request: it offers "Checked"
  only for a date of today or earlier, so only a direct API call (or a later client) reaches the
  window. The rule therefore holds for the product, but the route does not enforce it on its own.
  The fix keeps the race test
  green: one transaction with `SELECT … FOR NO KEY UPDATE` of the training (that lock does not
  conflict with the `FOR KEY SHARE` of a mark), `check_checking`, then the `UPDATE`. Or accept it
  and say so in the decision table, whose line "No lock" currently claims only delete and move.
- **R4 – a cancelled training offers "Checked"** (trace of `fill`: `can_check` ignores
  `cancelled`). This is written under **Deviations** and does not break a project rule. A cancelled
  training has no attendance to confirm, though, and "Season" can then show "Cancelled · Checked".
  The coordinator should decide on it in the accept review or at the close, not leave it to the
  silence of the spec.
- **R5 – docs that are now incomplete or false** (search `checked` in `docs/architecture.md`; read
  the comment at the top of `web/src/routes/training/[date]/+page.svelte`). `architecture.md` does
  not have `checked_at` in the `trainings` table, `0010_trainings_checked` in the migrations list,
  `check_checking` / `not_checked` in the core row, or the routes `…/checked` with their "no lock"
  and race test. This is for `/close`. The page's header comment lists the actions below the top
  line without "Checked" / "Undo check", and the docstring of `run` still names only "Open", "Cancel
  training" and "Undo cancel" (correct, but `saveCheck` is now a second path). Fix the header comment
  with the code.
  - 2026-10-25: Fixed: the test lets the check reach the server (`route.fetch()`) and then answers 500, so the check is stored and its answer lost; the page must show "Checked". With the reload of the stored training removed (and the front end built), the test fails.
  - 2026-10-25: Fixed: a refusal of the server (a code other than `internal`) shows its message, for `not_opened` "This training no longer exists. Load the page again."; only other failures show "Not saved. Try again.". Test "a check of a training deleted on another screen says so".
  - 2026-10-25: Fixed (coordinator: fix it): the check reads the training with `trainings::lock_for_check` (`FOR NO KEY UPDATE`) and writes in the same transaction. Test `a_check_waits_for_an_undo_check_on_another_screen`: with the plain read it answers 200 and leaves a training after today checked; with the lock it waits and answers `not_yet`. `a_check_does_not_wait_for_a_mark` still passes (the lock does not block `FOR KEY SHARE`).
  - 2026-10-25: Fixed (coordinator: no "Checked" when cancelled): `check_checking` refuses a cancelled training that is not checked (`TrainingError::Cancelled`, code `cancelled`); a check made before "Cancel" stays. Core test `a_cancelled_training_is_not_checked_but_a_check_stays`, server test `a_cancelled_training_is_not_checked`, browser test "a cancelled training has no Checked".
  - 2026-10-25: The comment at the top of the training page is fixed. `docs/architecture.md` is updated at `/close`.

Verdict: no blocking finding. R1 and R2 are small fixes in the front end and its test, R3 needs a
decision (fix or accept in writing), R4 is for the coordinator, R5 is for the close.
