-- "Checked" for games (story C of attendance at games). Attendance at games owns these tables:
-- only `crates/server/src/game_attendance.rs` reads or writes them (decision 0030).

-- The date of the game when "Checked" was tapped; a check for another date has no effect
-- (decision 0029). NULL: not checked.
ALTER TABLE game_attendance ADD COLUMN checked_for date;

-- An answer that "Checked" made (a "no answer" that became absent). "Undo check" deletes only
-- these; a tap on the player clears it.
ALTER TABLE game_answers ADD COLUMN settled boolean NOT NULL DEFAULT false;

-- From this date on (the day of the release), a past game of the team that is not checked is an
-- open item, and its list opens at once. One row.
CREATE TABLE game_check_start (
    only_row  boolean PRIMARY KEY DEFAULT true CHECK (only_row),
    from_date date    NOT NULL
);
INSERT INTO game_check_start (from_date)
VALUES ((now() AT TIME ZONE 'Europe/Amsterdam')::date);
