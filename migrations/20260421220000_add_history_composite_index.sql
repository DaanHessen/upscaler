-- Add composite index for user history queries to optimize sorting by date
CREATE INDEX idx_upscales_user_created ON upscales(user_id, created_at DESC);
