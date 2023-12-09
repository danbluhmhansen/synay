INSERT INTO
    game_event
    (id, data, added)
VALUES
    (
        'bd44f000-d7bf-484b-8b31-f71b00104f6d',
        '{"name":"one"}',
        '2023-01-01T00:00:00Z'
    ),
    (
        'bd44f000-d7bf-484b-8b31-f71b00104f6d',
        '{"name":"two","description":"foo"}',
        '2023-01-01T00:00:01Z'
    ),
    (
        'bd44f000-d7bf-484b-8b31-f71b00104f6d',
        '{"name":"three"}',
        '2023-01-01T00:00:02Z'
    );

REFRESH MATERIALIZED VIEW game;
