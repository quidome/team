-- A game that was not played on a date (story E of attendance at games). Events owns this table:
-- only `crates/server/src/games.rs` reads or writes it, and a reading of the game list never
-- writes it. A row is deleted by "Undo".
CREATE TABLE games_not_played (
    club_number    bigint  NOT NULL,
    season_year    integer NOT NULL,
    source_id      bigint  NOT NULL,
    -- The date of the game when "Not played" was tapped; a mark for another date has no effect
    -- (decision 0029).
    not_played_for date    NOT NULL,
    PRIMARY KEY (club_number, season_year, source_id),
    FOREIGN KEY (club_number, season_year, source_id) REFERENCES games
);

-- The switch "The drivers drove" of a not played game (duties owns `game_duties`).
ALTER TABLE game_duties ADD COLUMN drivers_drove boolean NOT NULL DEFAULT false;
