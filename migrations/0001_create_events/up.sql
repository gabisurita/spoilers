CREATE TABLE events (
    id          SERIAL PRIMARY KEY,
    timestamp   TIMESTAMP NOT NULL,
    body        JSONB
)
