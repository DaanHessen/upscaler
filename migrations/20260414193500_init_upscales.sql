-- Create upscales table
CREATE TABLE upscales (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    style VARCHAR(50) NOT NULL,
    input_path TEXT NOT NULL,
    output_path TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for faster lookups by user
CREATE INDEX idx_upscales_user_id ON upscales(user_id);
