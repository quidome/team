-- The club's players, their roles in the team of a season, the reasons for an absence, and the
-- weekly trainings of each season (attendance at training, story A; decision 0019).

-- A player is a person of the club: a name, with no dates. A hidden player is no longer offered as
-- a choice; its history stays.
CREATE TABLE players (
    id          bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    first_name  text NOT NULL CHECK (btrim(first_name) <> ''),
    last_name   text,
    hidden      boolean NOT NULL DEFAULT false,
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

-- A weekly training: a day of the week (1 = Monday) with a start and an end time, in a window
-- inside one season. It only suggests training days; a training exists only when it is opened
-- (story B). The core checks that two weekly trainings on the same day never overlap, under an
-- advisory lock.
CREATE TABLE weekly_trainings (
    id           bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    season_year  integer NOT NULL,
    weekday      smallint NOT NULL CHECK (weekday BETWEEN 1 AND 7),
    start_time   time NOT NULL,
    end_time     time NOT NULL,
    from_date    date NOT NULL,
    until_date   date NOT NULL,
    CHECK (end_time > start_time),
    CHECK (until_date >= from_date)
);
CREATE INDEX weekly_trainings_season ON weekly_trainings (season_year);
