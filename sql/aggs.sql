CREATE MATERIALIZED VIEW game AS
SELECT
    id,
    data ->> 'name' AS name,
    data ->> 'description' AS description,
    added,
    updated
FROM
    event_agg()
WHERE
    source = 'Game';

CREATE UNIQUE INDEX game_id_idx ON game(id);

COMMENT ON MATERIALIZED VIEW game IS 'Projection of games over the `events` table.';

COMMENT ON COLUMN game.id IS 'The unique game identifier.';

COMMENT ON COLUMN game.name IS 'Name of the game.';

COMMENT ON COLUMN game.description IS 'Short overview or description of the game.';

COMMENT ON COLUMN game.added IS 'Timestamp of when this game was first recorded.';

COMMENT ON COLUMN game.updated IS 'Timestamp of when this game was last updated.';