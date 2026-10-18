# Technical spec: Player names stored encrypted

**Status**: in progress
**Size**: story
**Change**: [2026-10-18-encrypted-player-names](./)
**Date**: 2026-10-18

## Objective
Store the first and last names of the players encrypted in the database (AES-256-GCM in the
server, the key from Vault and also kept outside the platform), so a copy of the database or a
backup does not show who the players are.

## Requirements summary
The reason: `players.first_name` and `players.last_name` are plain text. Anyone who reads the
database, its disk or an unencrypted copy (for example a dump on a laptop) sees the names of
children. Since v0.3.0 these names exist only in this database. A nightly dump encrypted with age
exists since 2026-10-18 (built in `gitops-lab`); this change is a second lock on the names
themselves.

The expected result (coordinator, 2026-10-18):
1. The database stores no player name as plain text, also not in an index or a query.
2. Each name is encrypted in the server with AES-256-GCM. The key is a new setting
   `PLAYER_NAME_KEY` from Vault; the coordinator keeps a second copy outside the platform (the
   password manager, next to the backup key).
3. The coordinator sees no change: the same names, the same order (first name, then last name,
   without case, then id), the same warning for the same name.
4. The existing players are encrypted once; after that, no plain-text columns are left.
5. Without the key, or with a wrong key, the app does not start. It never shows a name it cannot
   read and never logs a name.

The limit (accepted in Phase 0): this protects against a copy of the data without the key, not
against someone with access to the cluster, because the app holds the key in its environment.

Answers of the coordinator in the design: only the players' names (not the coordinator's name in
`sessions`); the one-time encryption runs in `team migrate`; two releases; the unencrypted laptop
dumps in `~/backups/team/` are deleted after release 2.

## Current state
- Names are read and written only in `crates/server/src/players.rs`: `all` (sorted in SQL by
  `lower(first_name), lower(coalesce(last_name, '')), id`), `by_id`, `same_name_exists` (compares
  in SQL, without case), `insert`, `update_name`. Callers: `api/players.rs` (`list`, `get`, `add`,
  `change`, `carry`), `api/team.rs`, `api/trainings.rs`, `api/attendance.rs`. They all use the
  core type `team_core::Player { id, first_name, last_name: Option<String>, hidden }`.
- `team_core::Player::sort_key` (first name lower case, last name lower case, id) already sorts in
  the core (`team`, `players_at`, `season_overview`). `team_core::clean_name` trims the name and
  refuses an empty first name (`first_name_required`).
- `add` and `change` in `api/players.rs` write, then call `same_name_exists` in the same
  transaction, and answer `same_name: bool`.
- Migration `0005_players_and_settings.sql`: `first_name text NOT NULL CHECK (btrim(first_name) <> '')`
  (constraint name `players_first_name_check`), `last_name text`.
- Settings: `config.rs` (`Config::from_lookup`, a hand-written `Debug` that leaves secrets out,
  `ConfigError::Missing` / `Invalid { name, reason }`). `main.rs`: `serve` reads `Config`;
  `migrate` reads only `DATABASE_URL` and runs `db::migrate` (the sqlx `MIGRATOR`).
- `StoreError` in `db.rs`: `Database(sqlx::Error)`, `Invalid(String)`.
- The cluster: `team migrate` runs in the init container `migration`, which gets only
  `DATABASE_URL` from the Secret; `team serve` gets the Vault settings. One replica. The cluster
  pulls `latest` at each pod start, so a merge to `main` is deployed at the next pod start.
- Crates in the server already: `rand` 0.8, `base64` 0.22, `sha2`, `hmac`.
- Test settings: `tests/common/mod.rs` (`SESSION_SECRET` const and the lookup), 
  `web/playwright.config.ts`, `scripts/smoke-test.sh`, `.env.example`.

## Steps

Steps 1–3 are one pull request and release 1. Step 4 is a second pull request and release 2,
after release 1 runs on the cluster. Before the merge of release 1 the operator tasks under
**Release steps** must be done.

### Step 1: The encryption and the key setting
- **Goal**: the server can encrypt and decrypt a name, and every command and test has a key. No
  data changes.
- **Changes**:
  - Crate `aes-gcm = "0.10"` in `crates/server/Cargo.toml` (RustCrypto; its `OsRng` is the
    operating system's random source, as `rand` 0.8 uses).
  - New module `crates/server/src/cipher.rs`:
    ```rust
    /// The key of the player names: 32 bytes. Its Debug output never shows the bytes.
    pub struct NameKey(Key<Aes256Gcm>);
    impl NameKey {
        /// Standard base64 of exactly 32 bytes.
        pub fn from_base64(text: &str) -> Result<Self, KeyError>;
    }
    /// Which column a value belongs to; the column name is the AAD, so a value moved to the
    /// other column does not decrypt.
    #[derive(Clone, Copy)]
    pub enum NameField { First, Last }   // AAD b"players.first_name" / b"players.last_name"
    pub fn seal(key: &NameKey, field: NameField, name: &str) -> Result<Vec<u8>, CipherError>;
    pub fn open(key: &NameKey, field: NameField, sealed: &[u8]) -> Result<String, CipherError>;

    #[derive(Debug, thiserror::Error)]
    pub enum CipherError {
        #[error("unknown version {0}")] UnknownVersion(u8),
        #[error("too short")] TooShort,
        #[error("not authentic (wrong key or changed data)")] NotAuthentic,
        #[error("not text")] NotText,
    }
    ```
    The stored form: version byte `1`, 12 bytes nonce (new from `OsRng` for each `seal`), then
    the AES-256-GCM output (encrypted name and 16-byte tag). `seal` returns a `Result` because
    the crate's `encrypt` does; no `unwrap`/`expect`. No error contains a name or the key.
  - `Config` gets `pub player_name_key: NameKey` from `PLAYER_NAME_KEY` (required). An invalid
    value gives `ConfigError::Invalid { name: "PLAYER_NAME_KEY", reason }` without the value.
    The hand-written `Debug` does not show it. `NameKey` is `Clone` (for the `Arc<Config>`).
  - `main.rs` `migrate`: reads `PLAYER_NAME_KEY` too (with `NameKey::from_base64`), so it fails
    early without a key. It does nothing with it yet.
  - Test key everywhere `SESSION_SECRET` is set: `tests/common/mod.rs` (a const
    `PLAYER_NAME_KEY` in the lookup), `web/playwright.config.ts`, `scripts/smoke-test.sh`,
    `.env.example` (with how to make one: `openssl rand -base64 32`). `README.md`: the cluster
    contract table lists `PLAYER_NAME_KEY` under "Settings from Vault", and says the migration
    command needs it too.
- **Tests** (unit tests in `cipher.rs` and `config.rs`):
  1. `seal` then `open` gives the name back, for "Anna", "Zoë", "Şükrü" and a name of 100
     characters.
  2. Two `seal` calls on the same name give different bytes.
  3. `open` with another key gives `NotAuthentic`.
  4. `open` of a `First` value as `Last` gives `NotAuthentic`.
  5. One changed byte after the nonce gives `NotAuthentic`; version byte 2 gives
     `UnknownVersion(2)`; 5 bytes give `TooShort`.
  6. `from_base64` refuses text that is not base64, and 16 or 31 bytes; the error names
     `PLAYER_NAME_KEY` (in `Config`) and does not contain the value.
  7. `debug_output_hides_secrets` also checks that the key text is not in the debug output.
  8. A missing `PLAYER_NAME_KEY` gives `setting PLAYER_NAME_KEY is required`.

### Step 2: Encrypted names in the store
- **Goal**: the database holds only encrypted names; the existing players are encrypted by
  `team migrate`; the coordinator sees no change.
- **Changes**:
  - Migration `migrations/0007_encrypted_player_names.sql`:
    ```sql
    -- The player names, encrypted by the app (AES-256-GCM, decision NNNN). The plain columns
    -- are emptied by `team migrate` and removed in a later release (decision 0004).
    ALTER TABLE players ADD COLUMN first_name_enc bytea, ADD COLUMN last_name_enc bytea;
    ALTER TABLE players ALTER COLUMN first_name DROP NOT NULL;
    ALTER TABLE players DROP CONSTRAINT players_first_name_check;
    ALTER TABLE players ADD CHECK (first_name IS NOT NULL OR first_name_enc IS NOT NULL);
    ```
    The rule "a first name is not empty" stays in `team_core::clean_name`.
  - `players.rs` (each function that touches names gets `key: &NameKey`):
    - `all(db, key)`: `SELECT id, first_name_enc, last_name_enc, hidden FROM players`, decrypt,
      sort with `sort_by_cached_key(Player::sort_key)`. A row without `first_name_enc` (a plain
      row written by the old pod during an update, until the next `team migrate`) is an error
      `StoreError::Invalid` with the player id only.
    - `by_id(db, key, id)`: the same for one row.
    - `insert(db, key, first, last, now)` and `update_name(db, key, id, first, last)`: write
      `first_name_enc`, `last_name_enc` (NULL when there is no last name), and `first_name =
      NULL, last_name = NULL`.
    - `same_name_exists` is removed.
    - New `encrypt_plain_names(tx: &mut PgConnection, key) -> Result<u64, StoreError>`:
      `SELECT id, first_name, last_name FROM players WHERE first_name_enc IS NULL FOR UPDATE`;
      for each row `UPDATE players SET first_name_enc = $2, last_name_enc = $3, first_name =
      NULL, last_name = NULL WHERE id = $1`. Returns the count.
  - `StoreError` gets `Name { player: PlayerId, error: CipherError }`, Display "the name of
    player {id} cannot be read: {error}". It maps to 500 like the other store errors.
  - Core `crates/core/src/players.rs`: `pub fn same_name(players: &[Player], first: &str, last:
    Option<&str>, except: Option<PlayerId>) -> bool` — equal first name and equal last name
    (no last name equals no last name), both without case, the player `except` left out.
    Compare with `to_lowercase`, as `sort_key` does.
  - `api/players.rs` `add` and `change`: after the write, `players::all(&mut *tx, key)` in the
    same transaction, then `same_name(...)`. The other callers pass `&state.config.player_name_key`.
  - `main.rs` `migrate`: after `db::migrate`, in one transaction, `encrypt_plain_names`; log
    `tracing::info!(count, "player names encrypted")` (only the count).
- **Tests**:
  9. Core `same_name`: "anna" without last name equals "Anna" without last name; "Anna de
     Vries" does not equal "Anna"; the player in `except` does not count; an empty list gives
     `false`.
  10. `names_are_not_stored_as_plain_text`: add "Anna de Vries" through the API; the raw row
      has `first_name` and `last_name` NULL, and the encrypted columns do not contain the bytes
      of "Anna" or "Vries"; `players::all` gives "Anna" and "de Vries".
  11. Sort order with encrypted data: "bram", "Anna de Vries", "anna Bakker" are listed as
      "anna Bakker", "Anna de Vries", "bram".
  13. A stored value with one changed byte: `GET /api/players` gives 500; the log has the
      player id and no name.
  14. `encrypt_plain_names_encrypts_each_player_once`: a test database at migration `0006`
      (a `Migrator` with only the first 6 of `team::db::MIGRATOR`'s migrations; the list is
      public in sqlx — if not possible, a deviation), three players with plain names by SQL (one
      with a last name, one without, one hidden) with a role and an absence; then the remaining
      migrations and `encrypt_plain_names`: count 3, plain columns NULL, `players::all` gives
      the same names, hidden state and ids; the role and the absence still point at the
      players; a second run gives 0.
  - The existing tests (`tests/players.rs` with `same_name`, `tests/player_logs.rs`, the browser
    test "the same name again shows a warning") pass unchanged.

### Step 3: The key check at start
- **Goal**: a wrong key stops the app before it serves a page or writes a name.
- **Changes**:
  - `players::check_key(db, key) -> Result<(), StoreError>`: decrypts the first name of the
    player with the lowest id that has `first_name_enc`, if one exists; else OK.
  - `serve`: after the migrations check and before it listens, `check_key`; an error stops the
    command (anyhow context "the player name key cannot read the stored names").
  - `migrate`: `check_key` before `encrypt_plain_names`, so a wrong key never mixes two keys in
    the table.
- **Tests**:
  12. `check_key`: the right key OK; an empty table OK; another key gives `StoreError::Name`
      with the player id and no name.
  15. `migrate` with a wrong key on a database with encrypted names stops with an error, and no
      row changes (test the function that `migrate` calls, `players::check_key` then
      `encrypt_plain_names`, with a plain row present: the plain row stays plain).

### Step 4: The plain-text columns removed (release 2)
- **Goal**: no plain-text name column, and no old row versions with names in the table file.
- **Changes**:
  - `migrations/0008_drop_plain_player_names.sql`:
    ```sql
    ALTER TABLE players DROP COLUMN first_name, DROP COLUMN last_name;
    ALTER TABLE players ALTER COLUMN first_name_enc SET NOT NULL;
    ```
    The CHECK of `0007` goes with the columns. If a plain row is still there (release 1 was
    skipped), `SET NOT NULL` fails, `migrate` stops, the pod does not start, nothing is lost.
  - `migrations/0009_vacuum_players.sql`: first line `-- no-transaction`, then
    `VACUUM FULL players;` (writes the table file again without the old row versions).
  - Remove `encrypt_plain_names`, its call in `migrate`, test 14, and the "row without
    `first_name_enc`" case in `all`/`by_id` (the column is NOT NULL now). `just prepare`.
- **Tests**:
  16. All migrations: `players` has no `first_name` / `last_name` columns, and
      `first_name_enc` is NOT NULL (read from `information_schema.columns`).
  17. A database at `0007` with a plain row (no `first_name_enc`): running `0008` fails, and
      the row is still there.

## Release steps
Before the merge of release 1 (operator):
1. Make the key: `openssl rand -base64 32`. Put it in the password manager and in Vault
   (`kv/productivity/team`, `PLAYER_NAME_KEY`). Compare a checksum of both copies
   (`printf %s "$KEY" | sha256sum`). The key lost = the names lost, also in every backup.
2. In a `gitops-lab` session: the Secret and both containers (`migration` and `team`) get
   `PLAYER_NAME_KEY`. Without it, a pod with the new image does not start.
3. A fresh backup (as the recovery steps in `gitops-lab` say), and check that it exists.

Release 1: merge, green `main`, restart the pod, check, tag (as `README.md`, "Making a release").
After release 1:
4. Check on the cluster: `SELECT count(*) FROM players WHERE first_name IS NOT NULL OR
   last_name IS NOT NULL` gives 0; "Players", "Team" and "Attendance" show the same names in the
   same order as before. When the count is not 0 (the old pod added or renamed a player during
   the update), restart the pod once more: `team migrate` encrypts those rows (review R1). During
   the update (about a minute) the old pod cannot list players, because the plain columns are
   empty (review A7).
5. Wait for the first nightly dump with encrypted names.

Release 2 (step 4): merge, green `main`, restart the pod, check, tag. After release 2:
6. Hand check: the files of the database on the cluster do not contain a known name (for
   example `grep -c` on the PostgreSQL data directory in the pod).
7. Delete the unencrypted laptop dumps in `~/backups/team/` (coordinator, 2026-10-18).

## Decisions
| Decision | Reason | Decision record? |
|----------|--------|------------------|
| Encrypt in the app with AES-256-GCM, random nonce, version byte, column name as AAD | The database and every copy hold only ciphertext; GCM detects a wrong key and changed data; the version byte keeps a later key change possible; the AAD stops a swap of first and last name | Yes: player names encrypted in the app |
| Not `pgcrypto` | The key would go to the database with each query and can show in its logs and statistics | In the same record |
| Sorting and the same-name check in the core | The database cannot sort or compare ciphertext; the core already has `sort_key`; about 20 players | In the same record |
| The one-time encryption in `team migrate` | It runs before `serve` on each pod start, so the app never reads plain names; also catches a plain row written by the old pod during an update | In the same record |
| Two releases (add, then remove the plain columns) | Decision 0004: the old pod must not meet a removed column during an update | No (follows 0004) |
| The key in `Config`, required for `serve` and `migrate`, checked at start against a stored name | A wrong or missing key stops the pod at once, not at the first page | In the same record |

## Not in scope
- Changing the key, and more than one key (the version byte makes it possible later).
- The coordinator's name in `sessions` (coordinator, 2026-10-18).
- Encryption of the disk on TrueNAS; old disk blocks in TrueNAS snapshots.
- Other data: reason names, dates, absences, games.

## Test plan
- **Existing tests that must still pass**: all of `tests/players.rs` (sort order, `same_name`,
  rename, hide, delete), `tests/player_logs.rs`, `tests/team.rs`, `tests/trainings.rs`,
  `tests/attendance.rs`, `tests/players_storage.rs`, the browser tests (players, team, trainings),
  `scripts/smoke-test.sh`, `api-types-check` (the API description must not change).
- **New tests**: 1–17, per step above.
- Not tested automatically: that `VACUUM FULL` removes the old rows from the disk (release
  step 6), and the nonce source itself (test 2 shows that it is used).

## Compatibility and performance
- No change for the coordinator, the API or the front end.
- New required setting `PLAYER_NAME_KEY` for `team serve` and `team migrate` (cluster contract in
  `README.md`; `gitops-lab` change before release 1).
- After release 1, v0.3.0 cannot read the players: the way back is a restore (decision 0004).
- Every list of players decrypts all players (about 20; microseconds each). The same-name check
  reads all players once per add or change.
- Known debt after Close: old nightly dumps (made before release 1) keep plain names inside their
  age encryption until retention removes them; TrueNAS snapshots may keep old disk blocks; no key
  change.

## Progress
- [x] Step 1 – the encryption and the key setting, 2026-10-18: `cipher.rs` (`NameKey`, `NameField`, `seal`, `open`, `KeyError`, `CipherError`) with 7 unit tests (tests 1–6 and the key's debug output); `Config::player_name_key` and `config::player_name_key` with 2 new settings tests and the debug test extended (tests 7, 8); `team migrate` reads the key; test key in `tests/common/mod.rs`, `web/playwright.config.ts`, `scripts/smoke-test.sh`; `.env.example` and the cluster contract in `README.md`. `just check` passes.
- [x] Step 2 – encrypted names in the store, 2026-10-18: migration `0007_encrypted_player_names.sql`; `players.rs` reads and writes only the encrypted columns, sorts with `Player::sort_key`, `encrypt_plain_names`; `same_name_exists` removed for the core rule `team_core::same_name`; `StoreError::Name` and `StoreError::Seal`; every caller passes `config.player_name_key`; `team migrate` encrypts after the SQL migrations and logs only the count. Tests: core `the_same_name_is_found_without_case` (9); `tests/player_names.rs` with `names_are_not_stored_as_plain_text` (10), `encrypted_players_are_listed_in_order_and_the_same_name_is_found` (11) and `encrypt_plain_names_encrypts_each_player_once` (14); `tests/player_name_logs.rs` (13). `.sqlx/` prepared; the local database encrypted by `just migrate`. `just check` passes. Note: the browser test "a cancelled training does not count, and counts again after \"Undo cancel\"" failed 3 times with "TypeError: Failed to fetch" in the first runs (two of them during or right after a full `just check`), then passed 11 runs in a row; the cause was not found (the debug output of the console watcher caught nothing).
- [x] Step 3 – the key check at start, 2026-10-18: `players::check_key` (the player with the lowest id that has an encrypted name) and `players::encrypt_after_migrations` (`check_key`, then `encrypt_plain_names` in one transaction), which `team migrate` calls; `team serve` calls `check_key` before it listens. Tests: `check_key_reads_a_stored_name_with_the_right_key_only` (12) and `migrate_with_a_wrong_key_stops_and_changes_nothing` (15) in `tests/player_names.rs`. By hand on the local database: `serve` with the database down starts (liveness 200, readiness 503, the warning in the log); `serve` and `migrate` with another key stop with "the name of player 1 cannot be read: not authentic (wrong key or changed data)" and no name in the output. `just check` passes.
- [x] Release 1, 2026-10-18: PR #13 (steps 1–3) and PR #14 (review fixes R1, R2) merged; live at 11:29 UTC. On the database `team`: migration 0007, the migration log "player names encrypted" count 14, 14 players with encrypted names, 0 plain names, 14 roles, 71 games, 1 training and 2 absences (the same as before); the coordinator saw the names right on "Players", "Team" and "Attendance"; the next pod start encrypted 0. Backup before: `team-20261018T102433Z.dump.age`. A problem outside this change was found on the way: since 08:09 UTC the Vault value `DATABASE_URL` named the empty database `team_v2`, so the app had shown an empty tool and the first two starts of release 1 ran on `team_v2`; the coordinator set it back to `team` in Vault, and a hard refresh and a full sync in Argo CD applied it. Not tagged yet.
- [ ] Step 4 – the plain-text columns removed
- [ ] Release 2
- [ ] All checks pass

## Deviations
- 2026-10-18, step 1: `CipherError` has a fifth case, `Encrypt` ("cannot encrypt"), because the crate's `encrypt` returns a `Result` (only for an input far larger than a name); the spec listed four cases.
- 2026-10-18, step 1: `NameKey::from_base64` ignores spaces and line ends around the value (a key pasted with a line end is not refused); the bytes must still be exactly 32.
- 2026-10-18, step 2: the comment in migration `0007` names the decision record as **0021** (the next free number; 0019 is not used), not `NNNN`: the file is applied on the cluster at release 1, before Close writes the record, and an applied file is never edited (decision 0004). Close must give the record number 0021.
- 2026-10-18, step 2: `StoreError` has a second new case, `Seal(CipherError)` ("a name cannot be encrypted"), for a write: the player has no id yet, so `Name { player, … }` does not fit.
- 2026-10-18, step 2: test 14 starts the database at `0006` with `Migrator::new` on a temporary folder with copies of the first 6 migration files, not with a cut list of `MIGRATOR`'s migrations: the fields of `Migrator` are hidden and "semver-exempt" in sqlx 0.8. Test 13 is in its own file (`tests/player_name_logs.rs`), because a test that reads the log needs its own process (`LogBuffer`).
- 2026-10-18, step 3 (coordinator): `team serve` stops only when the key cannot read a stored name. When the database cannot be reached, it logs "cannot check the player name key now: the database is not reachable" and starts, as before; readiness reports the database. Reason: `serve` is built to start without the database (`db::lazy_pool`), and stopping would make the pod restart in a loop while the database is down. `team migrate` (the init container) always checks the key with a working database.
- 2026-10-18, step 3: the data step of `team migrate` is a store function, `players::encrypt_after_migrations`, so test 15 tests the order (check, then encrypt) that `main.rs` uses.

## Review findings

### Build review, 2026-10-18 (steps 1–3, release 1)
Input: the diff of `feat/encrypted-player-names` against `main` (commit 8eb37f2, nothing
uncommitted in the code), this change folder, `docs/architecture.md`, the valid decision records,
`CLAUDE.md`, `AGENTS.md`. No other input was given. Summary: no blocker; the structure follows the
project's rules (decision 0001: `same_name` and the sort in the core, the cipher and the key in the
server; decision 0004: add first, remove in release 2, the data step in `team migrate`, not in
`serve`). R1 is worth fixing before the merge of release 1; the rest is notes and work for Close.

- **R1 – a rename by the old pod during the update leaves a plain name, and the rename is lost
  (twin of the "plain row" case).** The team deployment is `RollingUpdate` with `maxSurge: 1`,
  `maxUnavailable: 0` (`gitops-lab/applications/productivity/team/helm-chart/templates/deployment.yaml`),
  so the v0.3.0 pod keeps serving after the new init container applied `0007` and encrypted the
  names. The spec handles one case of that window: a player *added* by the old pod (a row without
  `first_name_enc`). It does not handle the twin: a *rename* by the old pod (`PUT /api/players/{id}`
  from a page that was already open) runs `UPDATE players SET first_name = $2, last_name = $3`, which
  the 0007 CHECK allows, on a row that already has `first_name_enc`. Then: (a) `encrypt_plain_names`
  selects only `WHERE first_name_enc IS NULL`, so this row is never encrypted again and the plain
  name stays in the table until release 2 drops the column (expected result 1 broken, silently);
  (b) the new pod reads the old encrypted name, so the coordinator's rename is lost without a
  message. The old pod answers 200 (its `detail` reads the plain name it just wrote). How checked:
  trace of the old `change` (main) and the new `encrypt_plain_names`, and the deployment strategy
  read in `gitops-lab`; not reproduced by a run. The window is short and one coordinator does the
  release, so the chance is small, but the fix is cheap: select
  `WHERE first_name_enc IS NULL OR first_name IS NOT NULL OR last_name IS NOT NULL` (a plain value
  is always newer than the encrypted one, because new code writes both plain columns as NULL) and a
  test for that row. Release step 4 (`count(*) WHERE first_name IS NOT NULL` = 0) would show the
  leftover, but the release steps do not say what to do then (restart the pod once more, so
  `team migrate` runs again). The accepted case (an added plain row) has the same gap: until the next
  pod start, Players, Team, Attendance, the training page and add/change all answer 500 (`all`
  gives `StoreError::Invalid`); release step 4 should say "if not 0: restart the pod again".
  - 2026-10-18: Fixed (branch `fix/encrypted-names-review`): `encrypt_plain_names` takes every row with a plain first name (`WHERE first_name IS NOT NULL`), so a rename or an add by the old pod is encrypted over the old value at the next `team migrate`; new test `a_rename_by_an_older_version_is_encrypted_over_the_old_name` (fails with the old query, checked). Release step 4 now checks both plain columns and says to restart the pod once more when the count is not 0.
- **R2 – `serve` calls every database error "not reachable".** `main.rs` `serve` treats any
  `StoreError::Database` from `check_key` as "cannot check the player name key now: the database
  is not reachable". A reachable database without migration `0007` (for example `just serve` before
  `just migrate`) gives an SQL error (column `first_name_enc` does not exist) and the same warning.
  The behaviour is right (it starts, readiness reports the missing migration); only the log text
  misleads. Low. How checked: trace of `check_key` and the `match` in `serve`; not run.
  - 2026-10-18: Fixed: the warning says "cannot check the player name key now: the database cannot be read".
- **R3 – limits of the stored form, for decision record 0021.** The stored value is
  `1 + 12 + len(name) + 16` bytes (asserted in `the_same_name_is_sealed_differently_each_time`), so
  a copy of the database shows the length in bytes of each first and last name. The AAD is the
  column name only, so a value can be moved to another player's row (same column) and still
  decrypts. Both are outside the accepted threat (a copy without the key, read-only), and no change
  is needed, but the record should name them as known limits next to "not against cluster access".
  How checked: reading `cipher.rs`.
  - 2026-10-18: Not fixed in code (outside the accepted threat, a copy without the key); Close writes both limits in decision record 0021.
- **R4 – "the same order" holds (checked).** The old list was `ORDER BY lower(first_name), …` in
  the database; the new one is `Player::sort_key` (`to_lowercase`, code point order). They match
  only if the database sorts by code point. Both the local and the cluster PostgreSQL are the
  official `postgres:16-alpine` image (`compose.yaml`; `gitops-lab/.../postgresql-chart/values.yaml`),
  whose musl `en_US.utf8` collation sorts by code point: a run on the local database of
  `ORDER BY lower(n)` gave `anna marie, annabel, Eva, fleur, Zoe, Zoë, Émile`, which is also the
  `to_lowercase` order. Not checked on the cluster database itself. The order would differ on a
  glibc PostgreSQL (accents, spaces); worth one line in the record. Test 11 uses only ASCII names.
  - 2026-10-18: No change needed.
- **R5 – the claims about checks.** Run by the reviewer on 8eb37f2: `cargo fmt --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (all pass,
  including the 5 tests of `tests/player_names.rs`, `tests/player_name_logs.rs`, the 7 cipher
  tests, the 2 new settings tests and the core `same_name` test), and
  `cargo sqlx prepare --workspace --check` (passes). Not run: `lint-ci`, `api-types-check`,
  `check-web`, `e2e`, `smoke`, and the hand checks of step 3 (`serve`/`migrate` with another key;
  covered by tests 12 and 15). The flaky browser test noted under step 2 was not looked into. Can
  a new test not fail? Checked by trace for tests 10, 11, 12, 13, 14, 15: each fails if its
  behaviour breaks (test 11 inserts in reverse of the expected order; test 13 flips byte 13, the
  first byte after version and nonce; test 15 keeps a plain row and checks it stays plain). No
  test that cannot fail was found.
  - 2026-10-18: No change needed. The full `just check` (with `lint-ci`, `api-types-check`, `check-web`, `e2e`) ran on the fix branch; the flaky browser test: see A6.
- **R6 – documents the diff made false (for Close).** `docs/architecture.md`: the `crates/server`
  row has no `cipher.rs`, and says nothing of the key check in `serve` or the encryption in
  `migrate`; the `migrations/` row stops at `0006`; "Tables: `players` (first name, last name,
  `hidden`)" no longer says how the names are stored. `CLAUDE.md` "Architecture": the server list
  has no `cipher.rs` (the encryption of the player names) and `team migrate` now also needs
  `PLAYER_NAME_KEY`. Decision record **0021** must exist with that number, because migration
  `0007` names it (deviation of step 2). `README.md`, `.env.example`, the smoke test and the
  Playwright settings are updated in the diff. How checked: search of `docs/architecture.md`,
  `CLAUDE.md`, `README.md`, `Justfile` for `players`, `DATABASE_URL`, `same name`, `sorted`,
  `migrate`.
  - 2026-10-18: For Close: `docs/architecture.md`, `CLAUDE.md`, and decision record number 0021.

### Accept review, 2026-10-18 (steps 1–3, release 1)
Input: the mode `accept` and this change folder; nothing else was given. Read: `change.md`, the
objective and expected results 1–5 in this file, the diff of `feat/encrypted-player-names` against
`main` (commit 8eb37f2), the run of the tests; the **Deviations** and the build review only after
the view below was formed. Scope: release 1 only. Step 4 is not built, so expected result 4 ("no
plain-text columns are left") and the "no old row versions" half of result 1 cannot be judged yet;
they need a look after step 4.

**Verdict: accepted with notes.** What the coordinator meant – a copy of the database without the
key shows no names, and the coordinator sees no change – is what release 1 builds. One gap (A1,
the same as build finding R1) can silently break result 1 during the update; it is a small fix in
the build, not a problem of the spec, and should be closed before the merge of release 1.

Runs by the reviewer: `cargo test -p team --test player_names --test player_name_logs --test players
--test players_storage` (23 pass), `cargo test -p team --lib` (29 pass, among them the 7 cipher and
2 settings tests), the core `same_name` test (pass), and the browser tests `players.e2e.ts` and
`trainings.e2e.ts` twice (run 1: 60 pass, 1 fail, 2 not run – see A6; run 2: 63 pass).

- **A1 – result 1: a rename by the old pod during the update stays plain, and is lost (concerning
  deviation).** The same as R1; confirmed by trace: `main`'s `update_name` writes
  `first_name`/`last_name`, the 0007 CHECK allows it on a row that has `first_name_enc`, and
  `encrypt_plain_names` selects only `WHERE first_name_enc IS NULL`. The result breaks both promises
  at once: a plain name stays in the table without any sign (result 1), and the new pod shows the
  old name, so the coordinator's change is gone (result 3). Small chance, cheap fix (R1's wider
  `WHERE`, and a test). Until it is fixed, release step 4 should at least check
  `first_name IS NOT NULL OR last_name IS NOT NULL` and say what to do when the count is not 0
  (restart the pod once more). How checked: trace of both versions of the code; not run.
  - 2026-10-18: Fixed with R1.
- **A2 – result 3, "the same order": faithful adaptation.** The order moved from SQL
  (`ORDER BY lower(...)`, the database's collation) to `Player::sort_key` (`to_lowercase`, code
  point order). On the local PostgreSQL (`postgres:16-alpine`, musl, `en_US.utf8`) I ran
  `ORDER BY lower(n)` on `Émile, Fleur, Zoë, de Vries, Dekker, Anne-Fleur, Anne Bakker, Annek`:
  code point order (`… Fleur | Zoë | Émile`), the same as the new code; with `COLLATE "en-x-icu"`
  the order is different (`Émile` before `Fleur`). So the order is the same only where the
  database sorts by code point; R4 reads the cluster's chart as the same image. The cluster
  database itself: not checked (a read on the cluster was not allowed in this review). The Team
  page already used `sort_key`, so the lists now agree with each other in any case.
  - 2026-10-18: No change needed.
- **A3 – result 3, "the same warning": faithful adaptation, with one small drift.** `same_name` in
  the core matches the old SQL for every name written through the API (`clean_name` trims, and turns
  an empty last name into none). It does not match for a name written outside the API: the old SQL
  treated a last name `''` as no last name (`coalesce(last_name, '')`), the new code does not, and
  `encrypt_plain_names` encrypts the stored values as they are, without `clean_name`. The cluster's
  players were entered after 2026-10-17; if any came in by hand-written SQL (an import), a last
  name `''` or spaces around a name would be kept, and a same-name warning for that player could be
  missed. Neutral drift; a check on the cluster before release 1
  (`count(*) WHERE last_name = '' OR first_name <> btrim(first_name)`) settles it. How checked:
  trace, and a search for every `INSERT INTO players` (only the API and tests); cluster data not
  checked.
  - 2026-10-18: Not fixed: on the cluster, 0 of the 14 players have an empty last name or spaces around a name (counted, no names shown, 2026-10-18), and both versions clean a name before they save it (`clean_name`), so no such row exists or can come from the app.
- **A4 – result 5, the key: faithful adaptation.** Without `PLAYER_NAME_KEY` neither `serve`
  (`Config`) nor `migrate` starts; a key that is not 32 bytes of base64 is refused without its
  value in the message (tests 6–8 run). A wrong key stops `migrate` before it writes anything
  (test 15) and stops `serve` when the database can be read (test 12 for `check_key`; the `serve`
  path by trace). A name that cannot be read gives 500 and only the player id in the log (test 13
  run). The deviation of step 3 (`serve` starts when the database cannot be reached) keeps the
  intent, because the init container always checks the key against a working database first. One
  case the spec already accepts: with no player stored yet, any key is taken, and the first name
  written fixes the key; with the cluster's existing players this cannot happen at release 1.
  - 2026-10-18: No change needed.
- **A5 – result 2 and the data: faithful adaptation.** AES-256-GCM in the server, a new nonce from
  `OsRng` for each value, version byte `1`, the column name as AAD (`cipher.rs`, tests 1–5 run).
  New and changed names are written only to the encrypted columns, with the plain columns set to
  NULL (test 10 run: no "Anna" or "Vries" bytes in the raw row). The existing players are
  encrypted once with their ids, hidden state, roles and absences kept (test 14 run). No index on
  a name exists (search of `migrations/` for `INDEX`). The old row versions with plain names stay in
  the table file until step 4's `VACUUM FULL`; a `pg_dump` does not copy them.
  - 2026-10-18: No change needed.
- **A6 – a browser test failed once in my runs (neutral, not explained).** "a cancelled training
  does not count, and counts again after \"Undo cancel\"" failed in 1 of 2 runs on
  `expect(errors).toEqual([])`, the same test the builder saw fail (Progress, step 2). The test
  calls `page.goto` while the overview may still be loading, so an aborted fetch is a likely cause,
  unrelated to names; not proven. Whether it also fails on `main`: not checked.
  - 2026-10-18: Not fixed: the cause is still not found (3 failures in the builder's runs, 1 in the reviewer's, many passing runs). Added to `docs/backlog.md` (Small) to check whether it also fails on the commit before this change.
- **A7 – during the update the old pod cannot list players (neutral drift).** After `0007` the
  plain columns are NULL for every player, and the v0.3.0 pod still serving reads them as
  `NOT NULL` text, so its Players, Team and Attendance pages fail until the new pod takes over.
  This is the "no change for the coordinator" promise broken for the minute of the update only,
  and it follows from decision 0004's add-then-remove path; the release is done by the coordinator,
  who can wait. Worth one line in the release steps. How checked: trace of `main`'s `players::all`
  (`query!` on a column that was `NOT NULL` when it was compiled); not run.
  - 2026-10-18: Fixed in the release steps: step 4 says that during the update (about a minute) the old pod cannot list players.
