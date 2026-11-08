-- The day from which a past game of the team that is not checked is an open item: 4 November
-- 2026, the day of release 1 (v0.8.0), from which the coordinator entered answers at games. The
-- migration 0015 stored the day it ran; the games between the two would never be an open item.
UPDATE game_check_start SET from_date = DATE '2026-11-04';
