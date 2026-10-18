# 0021 – Player names encrypted in the app

**Status**: valid
**Date**: 2026-10-18
**Change**: [2026-10-18-encrypted-player-names](../changes/done/2026-10-18-encrypted-player-names/)

## Context
The names of the players are personal data of children. Since v0.3.0 they exist only in the
team database, and they were stored as plain text: anyone with a copy of the database, of its
disk, or of an unencrypted dump saw who the players are. A nightly dump encrypted with age exists
since 2026-10-18 (built in `gitops-lab`). The coordinator wanted the names themselves encrypted
too, so that a copy of the database or a backup without the key does not show them. This does
not protect against someone with access to the cluster: the app must read the names, so it holds
the key.

## Decision
- **Encrypted in the server** (`crates/server/src/cipher.rs`): AES-256-GCM. A stored name is a
  version byte (`1`), a new random 12-byte nonce from the operating system for each value, then
  the encrypted name with its 16-byte tag. The column name (`players.first_name`,
  `players.last_name`) is the additional authenticated data, so a value moved to the other
  column does not decrypt. Errors never contain a name or the key.
- **The key** is the setting `PLAYER_NAME_KEY` (32 random bytes in base64) from Vault
  (`kv/productivity/team`), with a second copy in the coordinator's password manager. `team
  serve` and `team migrate` both need it.
- **Columns** `players.first_name_enc` (required) and `last_name_enc` (`bytea`). Only
  `crates/server/src/players.rs` reads and writes them.
- **Sorting and the same-name check in the core**: the database cannot sort or compare
  ciphertext. `players::all` sorts with `Player::sort_key`; `team_core::same_name` replaces the
  SQL check. With about 20 players this costs nothing.
- **A wrong key stops the app**: `players::check_key` decrypts the name of the player with the
  lowest id. `team migrate` stops on any failure. `team serve` stops on a wrong key, but starts
  with a warning when the database cannot be read, as it always did (`db::lazy_pool`); readiness
  then reports the database, and `team migrate` in the init container has checked the key with a
  working database.
- **Two releases** (decision 0004: add first, remove later): v0.3.1 added the encrypted columns,
  and `team migrate` encrypted every row with a plain name once (also a rename by the old pod
  during the update). v0.3.2 removed the plain columns (`0008`) and wrote the table file again
  without the old row versions (`0009`, `VACUUM FULL players`).
- **`0009` runs outside a transaction** (`-- no-transaction`): `VACUUM` cannot run in one. It is
  the one exception to "each migration in a transaction" (decision 0004); it can run twice.

## Options considered
| Option | For | Against |
|--------|-----|---------|
| AES-256-GCM in the app (chosen) | The database, its disk and every dump hold only ciphertext; GCM detects a wrong key and changed data; the version byte keeps a later key change possible | The database cannot sort or search names; a lost key loses every name |
| `pgcrypto` in PostgreSQL | Less app code; SQL can still compare | The key goes to the database with each query and can show in its logs and statistics |
| An encrypted disk on TrueNAS | No code | Protects the disk only, not a dump or a copy of the database |
| Plain names (as before) | Simple | Every copy shows the children's names |
| One release (add and remove at once) | Faster | The old pod meets removed columns during the update; against decision 0004 |

## Consequences
- **The key is the names.** Without `PLAYER_NAME_KEY` the names in the database and in every
  backup made since v0.3.1 cannot be read. Vault and the password manager must keep the same key.
- **Limits** (outside the accepted threat, a copy without the key):
  - a copy shows how long each name is, and an encrypted name copied to another player's row
    still decrypts (the AAD names the column, not the player; at insert the id does not exist
    yet);
  - old plain names can stay on the database disk: in WAL segments until new writes fill them
    (seen in two segments on 2026-10-18), and in TrueNAS snapshots or freed disk blocks;
  - the coordinator's own name in `sessions` (from Pocket ID) is not encrypted (coordinator,
    2026-10-18);
  - no command changes the key yet.
- **A backup from before v0.3.1** has plain names, which v0.3.2 refuses: restore it with the v0.3.1
  migration image once (`README.md`, "Restoring a backup from before v0.3.1"; the `gitops-lab`
  recovery steps link to it). Such backups leave the share after about a week.
- New code that shows or stores a player name goes through `players.rs` with the key; logs keep
  ids only.
