-- Attendance at games (story A of attendance at games). A row is made at the first save to the
-- game and never deleted.
CREATE TABLE game_attendance (
    club_number   bigint   NOT NULL,
    season_year   integer  NOT NULL,
    source_id     bigint   NOT NULL,
    extra_players smallint NOT NULL DEFAULT 0 CHECK (extra_players BETWEEN 0 AND 10),
    -- The date of the game when its answers were given or confirmed.
    answers_for   date,
    PRIMARY KEY (club_number, season_year, source_id),
    FOREIGN KEY (club_number, season_year, source_id) REFERENCES games
);

-- One row for each player who answered; no row is "no answer". The reason stays when the answer
-- goes to "coming" and back.
CREATE TABLE game_answers (
    club_number bigint  NOT NULL,
    season_year integer NOT NULL,
    source_id   bigint  NOT NULL,
    player_id   bigint  NOT NULL REFERENCES players ON DELETE RESTRICT,
    answer      text    NOT NULL CHECK (answer IN ('coming', 'not_coming')),
    reason_id   bigint  REFERENCES reasons ON DELETE RESTRICT,
    PRIMARY KEY (club_number, season_year, source_id, player_id),
    FOREIGN KEY (club_number, season_year, source_id) REFERENCES game_attendance
);
CREATE INDEX game_answers_player ON game_answers (player_id);
