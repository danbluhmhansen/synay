SELECT
    save_game(
        'bd44f000-d7bf-484b-8b31-f71b00104f6d',
        '{"name":"one"}'
    );

SELECT
    save_game(
        'bd44f000-d7bf-484b-8b31-f71b00104f6d',
        '{"name":"two","description":"foo"}'
    );

SELECT
    save_game(
        'bd44f000-d7bf-484b-8b31-f71b00104f6d',
        '{"name":"three"}'
    );

SELECT
    drop_game('bd44f000-d7bf-484b-8b31-f71b00104f6d');

REFRESH MATERIALIZED VIEW game;