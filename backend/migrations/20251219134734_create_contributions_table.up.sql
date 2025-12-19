CREATE TABLE contributions (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type TEXT NOT NULL,
    data JSONB NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    admin_comment TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reviewed_at TIMESTAMPTZ
);

-- Use AT TIME ZONE 'UTC' to make the date cast IMMUTABLE for the index.
CREATE UNIQUE INDEX idx_user_daily_contribution ON contributions (user_id, (CAST(created_at AT TIME ZONE 'UTC' AS DATE)));
