CREATE TABLE log_level_warning (
    id          SERIAL PRIMARY KEY,
    timestamp   TIMESTAMP NOT NULL,
    user_id     INTEGER,
    title       VARCHAR(64),
    body        TEXT
);

CREATE TABLE log_level_critical (
    id          SERIAL PRIMARY KEY,
    timestamp   TIMESTAMP NOT NULL,
    user_id     INTEGER,
    title       VARCHAR(64),
    body        TEXT
);
