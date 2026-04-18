-- Optimize upscales table for the queue worker
-- This index prevents full table scans when polling for PENDING jobs.
CREATE INDEX IF NOT EXISTS idx_upscales_status_created_at ON upscales (status, created_at);
