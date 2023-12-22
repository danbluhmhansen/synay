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
        'bd44f000-d7bf-484b-8b31-f71b00104f6d',
        'Game',
        '{"name":"two","description":"foo"}'
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
    event_source (id, source, event)
VALUES
    (
        'bd44f000-d7bf-484b-8b31-f71b00104f6d',
        'Game',
        'Drop'
    );

REFRESH MATERIALIZED VIEW game;