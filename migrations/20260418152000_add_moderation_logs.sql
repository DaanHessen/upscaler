-- Add EXPIRED to job_status enum
-- Note: ALTER TYPE ADD VALUE IF NOT EXISTS is supported in Postgres 13+
-- For older versions, we would need to check pg_type.
ALTER TYPE job_status ADD VALUE IF NOT EXISTS 'EXPIRED';

-- Table for tracking rejected content for "owner insights"
CREATE TABLE IF NOT EXISTS moderation_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL, -- references local users table for safety
    path TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for the Janitor cleanup
CREATE INDEX IF NOT EXISTS idx_moderation_logs_created_at ON moderation_logs(created_at);
