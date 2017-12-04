CREATE TABLE log_level_warning (
    id          IDENTITY(1,1),
    timestamp   TIMESTAMP,
    user_id     INTEGER,
    title       VARCHAR(64),
    body        TEXT
);

CREATE TABLE log_level_critical (
    id          IDENTITY(1,1),
    timestamp   TIMESTAMP,
    user_id     INTEGER,
    title       VARCHAR(64),
    body        TEXT
);
