-- One row for each game with a change the coordinator has not seen yet (story 4, decision 0016).
-- The row keeps the game as the coordinator last saw it; the kind of change is worked out from it
-- and the stored game. "Seen" deletes the row.

-- Every version comes from this sequence, also after a row was deleted and made again, so a
-- "seen" from an old page never matches a newer change of the same game.
CREATE SEQUENCE change_versions;

CREATE TABLE changes (
    club_number  bigint NOT NULL,
    season_year  integer NOT NULL,
    source_id    bigint NOT NULL,
    last_seen    jsonb,                  -- the game as last seen; NULL = a new game
    version      bigint NOT NULL DEFAULT nextval('change_versions'),
    updated_at   timestamptz NOT NULL,
    PRIMARY KEY (club_number, season_year, source_id),
    FOREIGN KEY (club_number, season_year, source_id) REFERENCES games
);
