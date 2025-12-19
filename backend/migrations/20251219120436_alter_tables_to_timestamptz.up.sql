ALTER TABLE users 
    ALTER COLUMN created_at TYPE TIMESTAMPTZ 
    USING created_at AT TIME ZONE 'UTC';

ALTER TABLE questions 
    ALTER COLUMN created_at TYPE TIMESTAMPTZ 
    USING created_at AT TIME ZONE 'UTC';

ALTER TABLE exam_records 
    ALTER COLUMN created_at TYPE TIMESTAMPTZ 
    USING created_at AT TIME ZONE 'UTC';