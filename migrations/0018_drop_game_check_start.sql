-- Open items from the start of the season (story 3 of "Checked for every event"): every past game
-- of the team of the season that is not checked is an open item, so the day of the release of
-- "Checked" for games (`game_check_start`, migrations 0015 and 0016) is no longer read.
DROP TABLE game_check_start;
