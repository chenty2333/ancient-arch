-- Add up migration script here
CREATE TABLE IF NOT EXISTS architectures (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    category        TEXT NOT NULL,      -- e.g., 'Palace', 'Bridge'
    name            TEXT NOT NULL,
    dynasty         TEXT NOT NULL,
    location        TEXT NOT NULL,
    description     TEXT NOT NULL,
    cover_img       TEXT NOT NULL,      -- URL path
    carousel_imgs   JSON NOT NULL       -- JSON Array of URL paths
);
