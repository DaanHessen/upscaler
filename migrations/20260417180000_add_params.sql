-- Add user-tunable parameters to the upscale job
ALTER TABLE upscales
    ADD COLUMN temperature REAL NOT NULL DEFAULT 0.0,
    ADD COLUMN quality VARCHAR(20) NOT NULL DEFAULT '2K';
