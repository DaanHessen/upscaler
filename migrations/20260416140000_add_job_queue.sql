-- Create job status enum
CREATE TYPE job_status AS ENUM ('PENDING', 'PROCESSING', 'COMPLETED', 'FAILED');

-- Add new columns to support async processing
ALTER TABLE upscales 
    ADD COLUMN status job_status NOT NULL DEFAULT 'PENDING',
    ADD COLUMN error_msg TEXT;

-- existing fields might be unknown at intake time
ALTER TABLE upscales 
    ALTER COLUMN style DROP NOT NULL,
    ALTER COLUMN output_path DROP NOT NULL;
