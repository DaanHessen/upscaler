-- Add prompt_settings to support dynamic prompt building
ALTER TABLE upscales ADD COLUMN prompt_settings JSONB DEFAULT '{}'::jsonb;
