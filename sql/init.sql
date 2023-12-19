CREATE TABLE event_source (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    data jsonb NULL,
    event EventType NOT NULL DEFAULT 'Save',
    added timestamptz DEFAULT clock_timestamp(),
    PRIMARY KEY (id, added)
);

COMMENT ON TABLE event_source IS 'Collection of game data events to be projected to a game view.';

COMMENT ON COLUMN event_source.id IS 'The unique game projection the event should aggregate to.';
COMMENT ON COLUMN event_source.data IS 'JSON data representing the atomic change to the game projection.';
COMMENT ON COLUMN event_source.event IS 'Represents the type of the event.';
COMMENT ON COLUMN event_source.added IS 'Timestamp of when this particular change happened. To be used for replaying the projection in the correct order.';
