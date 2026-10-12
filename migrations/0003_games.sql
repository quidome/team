-- The games of the club's game list, their places, and every reading of the game list.
-- A season is stored as its start year: 2026 means "2026-2027" (decision 0014).
CREATE TABLE places (
    number  bigint PRIMARY KEY,                -- the source's loc_id
    hall    text NOT NULL,
    town    text NOT NULL
);

CREATE TABLE games (
    club_number       bigint NOT NULL,         -- the club whose game list it came from
    season_year       integer NOT NULL,
    source_id         bigint NOT NULL,         -- the key
    game_number       text NOT NULL,
    date              date NOT NULL,           -- Dutch local date
    start_time        time,                    -- Dutch local time; NULL = not known
    home_team_number  bigint NOT NULL,
    home_team_name    text NOT NULL,
    home_club_number  bigint NOT NULL,
    away_team_number  bigint NOT NULL,
    away_team_name    text NOT NULL,
    away_club_number  bigint NOT NULL,
    place_number      bigint NOT NULL REFERENCES places,
    court             text,
    competition       text NOT NULL,
    in_list           boolean NOT NULL,        -- false: no longer in the game list
    first_read_at     timestamptz NOT NULL,
    PRIMARY KEY (club_number, season_year, source_id)
);

CREATE TABLE readings (
    id             bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    club_number    bigint NOT NULL,
    season_year    integer NOT NULL,
    started_at     timestamptz NOT NULL,
    finished_at    timestamptz NOT NULL,
    result         text NOT NULL CHECK (result IN ('good', 'failed', 'held_back')),
    error          text CHECK (error IN ('source_unavailable', 'source_format')),
    games_read     integer,
    first_reading  boolean NOT NULL DEFAULT false,
    counts         jsonb,                      -- Counts of a good reading that is not the first
    hold_reason    text CHECK (hold_reason IN ('empty', 'far_fewer')),
    hold_before    integer
);
CREATE INDEX readings_club_season ON readings (club_number, season_year, id DESC);

-- followed_teams keeps writing the text season until a later release (the old app may run on this
-- schema during a rollout); new code reads only season_year.
ALTER TABLE followed_teams
    ADD COLUMN season_year integer GENERATED ALWAYS AS (left(season, 4)::integer) STORED;
CREATE UNIQUE INDEX followed_teams_season_year ON followed_teams (season_year);
