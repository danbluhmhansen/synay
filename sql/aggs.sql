CREATE MATERIALIZED VIEW game AS SELECT id, data, added, updated FROM event_agg();

CREATE UNIQUE INDEX game_id_idx ON game(id);

COMMENT ON MATERIALIZED VIEW game IS 'Projection of games over the `events` table.';

COMMENT ON COLUMN game.id IS 'The unique game identifier.';
COMMENT ON COLUMN game.added IS 'Timestamp of when this game was first recorded.';
COMMENT ON COLUMN game.updated IS 'Timestamp of when this game was last updated.';
