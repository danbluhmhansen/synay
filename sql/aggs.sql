CREATE MATERIALIZED VIEW game AS SELECT id, name, description, added, updated FROM game_agg();

CREATE UNIQUE INDEX game_id_idx ON game(id);

COMMENT ON MATERIALIZED VIEW game IS 'Projection of games over the `game_event` table.';

COMMENT ON COLUMN game.id IS 'The unique game identifier.';
COMMENT ON COLUMN game.name IS 'Name of the game.';
COMMENT ON COLUMN game.description IS 'Short overview or description of the game.';
COMMENT ON COLUMN game.added IS 'Timestamp of when this game was first recorded.';
COMMENT ON COLUMN game.updated IS 'Timestamp of when this game was last updated.';
