-- Create questions table
CREATE TABLE IF NOT EXISTS questions (
    id          BIGSERIAL PRIMARY KEY,
    type        TEXT NOT NULL,          -- 'single', 'multiple'
    content     TEXT NOT NULL,
    options     JSONB NOT NULL,          -- JSONB Array
    answer      TEXT NOT NULL,          -- Correct answer
    analysis    TEXT,                   -- Explanation
    created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create exam_records table
CREATE TABLE IF NOT EXISTS exam_records (
    id          BIGSERIAL PRIMARY KEY,
    user_id     BIGINT NOT NULL UNIQUE, -- Use unique for upsert logic if needed, or index
    score       INTEGER NOT NULL,
    created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);