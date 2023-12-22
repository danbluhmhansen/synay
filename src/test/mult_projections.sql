INSERT INTO
    event_source (id, source, data)
VALUES
    (
        'bd44f000-d7bf-484b-8b31-f71b00104f6d',
        'Game',
        '{"name":"one"}'
    );

INSERT INTO
    event_source (id, source, data)
VALUES
    (
        '9e6e4c7b-5980-4369-a671-3b0c9998e47e',
        'Game',
        '{"name":"another one"}'
    );

INSERT INTO
    event_source (id, source, data)
VALUES
    (
        'bd44f000-d7bf-484b-8b31-f71b00104f6d',
        'Game',
        '{"name":"two","description":"foo"}'
    );

INSERT INTO
    event_source (id, source, data)
VALUES
    (
        'a11fbe6c-4a56-423f-accd-d9262e131e76',
        'Game',
        '{"name":"and another one"}'
    );

INSERT INTO
    event_source (id, source, data)
VALUES
    (
        '9e6e4c7b-5980-4369-a671-3b0c9998e47e',
        'Game',
        '{"name":"another two","description":"bar"}'
    );

INSERT INTO
    event_source (id, source, data)
VALUES
    (
        '9e6e4c7b-5980-4369-a671-3b0c9998e47e',
        'Game',
        '{"name":"another three"}'
    );

INSERT INTO
    event_source (id, source, data)
VALUES
    (
        'bd44f000-d7bf-484b-8b31-f71b00104f6d',
        'Game',
        '{"name":"three"}'
    );

INSERT INTO
    event_source (id, source, data)
VALUES
    (
        'a11fbe6c-4a56-423f-accd-d9262e131e76',
        'Game',
        '{"name":"and another two"}'
    );

REFRESH MATERIALIZED VIEW game;