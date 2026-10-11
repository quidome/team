-- The chosen club and team, one row for each season. Rows of earlier seasons stay. The current
-- choice is the row with the newest season. For more teams later, UNIQUE (season) goes and a link
-- to the coordinator comes; existing rows stay valid.
CREATE TABLE followed_teams (
    id           bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    season       text NOT NULL UNIQUE,          -- '2026-2027'
    club_number  bigint NOT NULL,
    club_name    text NOT NULL,
    club_town    text,
    team_number  bigint,
    team_name    text,
    chosen_at    timestamptz NOT NULL,
    CHECK ((team_number IS NULL) = (team_name IS NULL))
);
