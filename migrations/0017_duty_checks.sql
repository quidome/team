-- "Checked" of a duty game (story 2 of "Checked for every event"): the coordinator marked the
-- places of the team at a home game of another club team as done in the administration. Duties
-- owns `game_duties`; the check is written under its row lock of the game, as the places.
-- `checked_for`: the date of the game when it was checked; a check for another date has no effect
-- (decision 0029). `checked_at`: the first moment of the check; NULL with `checked_for`.
ALTER TABLE game_duties ADD COLUMN checked_for date;
ALTER TABLE game_duties ADD COLUMN checked_at timestamptz;
