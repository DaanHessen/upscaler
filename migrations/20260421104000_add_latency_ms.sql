-- Add latency_ms to track processing duration
ALTER TABLE upscales ADD COLUMN latency_ms INTEGER DEFAULT 0;
