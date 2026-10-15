-- Players, their roles in the team of a season, the reasons for an absence, and the training days
-- of each season (attendance at training, story A; decision 0019).

-- A player is the person: a name, with no dates.
CREATE TABLE players (
    id          bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    first_name  text NOT NULL CHECK (btrim(first_name) <> ''),
    last_name   text,
    created_at  timestamptz NOT NULL
);

-- A role in the team of one season: plays games or only trains, from a date until a date (NULL
-- while it continues). The core checks that a role stays inside its season and that the roles of
-- one player never overlap, under a lock on the player's row.
CREATE TABLE player_roles (
    id           bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    player_id    bigint NOT NULL REFERENCES players ON DELETE CASCADE,
    season_year  integer NOT NULL,
    role         text NOT NULL CHECK (role IN ('plays_games', 'only_trains')),
    from_date    date NOT NULL,
    until_date   date,
    CHECK (until_date IS NULL OR until_date >= from_date)
);
CREATE INDEX player_roles_player ON player_roles (player_id);
CREATE INDEX player_roles_season ON player_roles (season_year);

-- Reasons for an absence, in the order of `position`. Names are unique without case and outer
-- spaces.
CREATE TABLE reasons (
    id        bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name      text NOT NULL CHECK (btrim(name) <> ''),
    valid     boolean NOT NULL,
    hidden    boolean NOT NULL DEFAULT false,
    position  integer NOT NULL
);
CREATE UNIQUE INDEX reasons_name ON reasons (lower(btrim(name)));

INSERT INTO reasons (name, valid, position) VALUES
    ('School', true, 1),
    ('Ill', true, 2),
    ('Injured', true, 3),
    ('Holiday', false, 4),
    ('Work', false, 5),
    ('No reason', false, 6);

-- The first and the last training date of a season.
CREATE TABLE training_seasons (
    season_year  integer PRIMARY KEY,
    first_date   date NOT NULL,
    last_date    date NOT NULL,
    CHECK (last_date >= first_date)
);

-- The training days of a season, in sets: a set applies from `valid_from` until the next set, so
-- a change of days keeps the past. One training time for each day of the week (1 = Monday).
CREATE TABLE training_days (
    season_year  integer NOT NULL REFERENCES training_seasons,
    valid_from   date NOT NULL,
    weekday      smallint NOT NULL CHECK (weekday BETWEEN 1 AND 7),
    start_time   time NOT NULL,
    end_time     time NOT NULL,
    CHECK (end_time > start_time),
    PRIMARY KEY (season_year, valid_from, weekday)
);
