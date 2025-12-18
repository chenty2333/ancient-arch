-- Add up migration script here
CREATE TABLE IF NOT EXISTS architectures (
    id              BIGSERIAL PRIMARY KEY,
    category        TEXT NOT NULL,      -- e.g., 'Palace', 'Bridge'
    name            TEXT NOT NULL,
    dynasty         TEXT NOT NULL,
    location        TEXT NOT NULL,
    description     TEXT NOT NULL,
    cover_img       TEXT NOT NULL,      -- URL path
    carousel_imgs   JSONB NOT NULL       -- JSONB Array of URL paths
);