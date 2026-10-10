-- Login sessions (decision 0007). The cookie holds a random token; only its SHA-256 hash is stored,
-- so a copy of this table cannot be used to log in.
CREATE TABLE sessions (
    token_hash  bytea PRIMARY KEY,
    subject     text NOT NULL,
    name        text NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT now(),
    expires_at  timestamptz NOT NULL
);

CREATE INDEX sessions_expires_at ON sessions (expires_at);
