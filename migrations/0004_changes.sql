-- One row for each game with a change the coordinator has not seen yet (story 4, decision 0016).
-- The row keeps the game as the coordinator last saw it; the kind of change is worked out from it
-- and the stored game. "Seen" deletes the row.
CREATE TABLE changes (
    club_number  bigint NOT NULL,
    season_year  integer NOT NULL,
    source_id    bigint NOT NULL,
    last_seen    jsonb,                  -- the game as last seen; NULL = a new game
    version      integer NOT NULL DEFAULT 1,
    updated_at   timestamptz NOT NULL,
    PRIMARY KEY (club_number, season_year, source_id),
    FOREIGN KEY (club_number, season_year, source_id) REFERENCES games
);
