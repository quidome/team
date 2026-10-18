-- The plain name columns go (release 2 of the encrypted player names; decision 0021, change
-- 2026-10-18-encrypted-player-names). Release 1 encrypted every player and emptied them. If a plain
-- row is still there (release 1 was skipped), SET NOT NULL fails, `team migrate` stops and nothing
-- is lost. The CHECK of 0007 goes with the columns.
ALTER TABLE players DROP COLUMN first_name, DROP COLUMN last_name;
ALTER TABLE players ALTER COLUMN first_name_enc SET NOT NULL;
