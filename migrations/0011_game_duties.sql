-- The duties of a game (story A of duties). A row is made at the first save to the game, under
-- its row lock, and never deleted.
CREATE TABLE game_duties (
    club_number         bigint      NOT NULL,
    season_year         integer     NOT NULL,
    source_id           bigint      NOT NULL,
    -- "Schedule entered": the first moment it was tapped; NULL = not entered.
    schedule_entered_at timestamptz,
    -- The number of drivers the coordinator set; NULL = the proposal.
    drivers             smallint    CHECK (drivers BETWEEN 0 AND 10),
    saved_at            timestamptz NOT NULL,
    PRIMARY KEY (club_number, season_year, source_id),
    FOREIGN KEY (club_number, season_year, source_id) REFERENCES games
);

-- One row for each place of the referee committee that names a club team; an empty place has no
-- row. The primary key keeps one team for each place.
CREATE TABLE duty_teams (
    club_number  bigint   NOT NULL,
    season_year  integer  NOT NULL,
    source_id    bigint   NOT NULL,
    kind         text     NOT NULL CHECK (kind IN ('referee', 'table')),
    number       smallint NOT NULL CHECK (number BETWEEN 1 AND 2),
    team_number  bigint   NOT NULL,
    -- The short name ("M16 1"); it stays when the team leaves the game list.
    team_name    text     NOT NULL,
    PRIMARY KEY (club_number, season_year, source_id, kind, number),
    FOREIGN KEY (club_number, season_year, source_id) REFERENCES game_duties
);
