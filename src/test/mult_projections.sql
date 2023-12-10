SELECT
    save_game(
        'bd44f000-d7bf-484b-8b31-f71b00104f6d',
        '{"name":"one"}'
    );

SELECT
    save_game(
        '9e6e4c7b-5980-4369-a671-3b0c9998e47e',
        '{"name":"another one"}'
    );

SELECT
    save_game(
        'bd44f000-d7bf-484b-8b31-f71b00104f6d',
        '{"name":"two","description":"foo"}'
    );

SELECT
    save_game(
        'a11fbe6c-4a56-423f-accd-d9262e131e76',
        '{"name":"and another one"}'
    );

SELECT
    save_game(
        '9e6e4c7b-5980-4369-a671-3b0c9998e47e',
        '{"name":"another two","description":"bar"}'
    );

SELECT
    save_game(
        '9e6e4c7b-5980-4369-a671-3b0c9998e47e',
        '{"name":"another three"}'
    );

SELECT
    save_game(
        'bd44f000-d7bf-484b-8b31-f71b00104f6d',
        '{"name":"three"}'
    );

SELECT
    save_game(
        'a11fbe6c-4a56-423f-accd-d9262e131e76',
        '{"name":"and another two"}'
    );

REFRESH MATERIALIZED VIEW game;