-- Trainings and absences (attendance at training, story B; decision 0020).

-- A training exists only when the coordinator opens a training day or adds one. At most one
-- training per date. No link to a weekly training: a later change of a weekly training never
-- changes it. The season follows from the date.
CREATE TABLE trainings (
    id            bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    date          date NOT NULL UNIQUE,
    start_time    time NOT NULL,
    end_time      time NOT NULL,
    cancelled_at  timestamptz,
    created_at    timestamptz NOT NULL,
    CHECK (end_time > start_time)
);

-- An absence of a player at a training, with its reason. "There" is not stored: a player is there
-- when the training exists, the player has a role on its date, and there is no absence. Absences go
-- with their training; a player with absences cannot be deleted.
CREATE TABLE absences (
    training_id  bigint NOT NULL REFERENCES trainings ON DELETE CASCADE,
    player_id    bigint NOT NULL REFERENCES players ON DELETE RESTRICT,
    reason_id    bigint NOT NULL REFERENCES reasons,
    PRIMARY KEY (training_id, player_id)
);
CREATE INDEX absences_player ON absences (player_id);
