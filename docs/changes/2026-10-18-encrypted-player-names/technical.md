# Technical spec: Player names stored encrypted

**Status**: approved
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
4. Check on the cluster: `SELECT count(*) FROM players WHERE first_name IS NOT NULL` gives 0;
   "Players", "Team" and "Attendance" show the same names in the same order as before.
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
- [ ] Step 1 – the encryption and the key setting
- [ ] Step 2 – encrypted names in the store
- [ ] Step 3 – the key check at start
- [ ] Release 1
- [ ] Step 4 – the plain-text columns removed
- [ ] Release 2
- [ ] All checks pass

## Deviations

## Review findings
