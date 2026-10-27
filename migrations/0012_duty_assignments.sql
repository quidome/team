-- The players on the duty places of the team (story B of duties). One row for each filled place;
-- an open place has no row. The primary key keeps one player on a place; the unique key keeps one
-- place for a player at a game. A table or referee place gets a player only while it names the
-- team, which the server checks under the row lock of the game.
CREATE TABLE duty_assignments (
    club_number    bigint   NOT NULL,
    season_year    integer  NOT NULL,
    source_id      bigint   NOT NULL,
    kind           text     NOT NULL CHECK (kind IN ('drive', 'table', 'referee')),
    number         smallint NOT NULL CHECK (number BETWEEN 1 AND 10),
    planned_player bigint   NOT NULL REFERENCES players ON DELETE RESTRICT,
    -- After the game: the planned player did it, another player did it, or nobody did it.
    outcome        text     NOT NULL DEFAULT 'as_planned'
                            CHECK (outcome IN ('as_planned', 'done_by', 'not_done')),
    done_by        bigint   REFERENCES players ON DELETE RESTRICT,
    CHECK ((outcome = 'done_by') = (done_by IS NOT NULL)),
    PRIMARY KEY (club_number, season_year, source_id, kind, number),
    UNIQUE (club_number, season_year, source_id, planned_player),
    FOREIGN KEY (club_number, season_year, source_id) REFERENCES game_duties
);
CREATE INDEX duty_assignments_planned ON duty_assignments (planned_player);
CREATE INDEX duty_assignments_done_by ON duty_assignments (done_by);
