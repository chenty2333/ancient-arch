-- Create questions table
CREATE TABLE IF NOT EXISTS questions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    type        TEXT NOT NULL,          -- 'single', 'multiple'
    content     TEXT NOT NULL,
    options     JSON NOT NULL,          -- JSON Array
    answer      TEXT NOT NULL,          -- Correct answer
    analysis    TEXT,                   -- Explanation
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Create exam_records table
CREATE TABLE IF NOT EXISTS exam_records (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id     INTEGER NOT NULL,
    score       INTEGER NOT NULL,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);