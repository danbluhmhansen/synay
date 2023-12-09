CREATE TABLE e_game (
    id uuid NOT NULL,
    data jsonb NULL,
    event EventType NOT NULL DEFAULT 'Updated',
    added timestamptz DEFAULT NOW(),
    PRIMARY KEY (id, added)
);
