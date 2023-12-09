CREATE TABLE game_event (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    data jsonb NULL,
    event EventType NOT NULL DEFAULT 'Save',
    added timestamptz DEFAULT clock_timestamp(),
    PRIMARY KEY (id, added)
);

COMMENT ON TABLE game_event IS 'Collection of game data events to be projected to a game view.';

COMMENT ON COLUMN game_event.id IS 'The unique game projection the event should aggregate to.';
COMMENT ON COLUMN game_event.data IS 'JSON data representing the atomic change to the game projection.';
COMMENT ON COLUMN game_event.event IS 'Represents the type of the event.';
COMMENT ON COLUMN game_event.added IS 'Timestamp of when this particular change happened. To be used for replaying the projection in the correct order.';
