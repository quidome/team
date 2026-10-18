-- The player names, encrypted by the app (AES-256-GCM; decision 0021, change
-- 2026-10-18-encrypted-player-names). `team migrate` encrypts the plain names and empties the plain
-- columns; a later release removes them (decision 0004: add first, remove later). The rule that a
-- first name is not empty stays in the core (`clean_name`).
ALTER TABLE players ADD COLUMN first_name_enc bytea, ADD COLUMN last_name_enc bytea;
ALTER TABLE players ALTER COLUMN first_name DROP NOT NULL;
ALTER TABLE players DROP CONSTRAINT players_first_name_check;
ALTER TABLE players ADD CHECK (first_name IS NOT NULL OR first_name_enc IS NOT NULL);
