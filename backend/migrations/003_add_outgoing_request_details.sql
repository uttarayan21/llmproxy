-- Add outgoing request details to request logs
-- This allows us to see both what the user sent us and what we sent to the LLM platform

ALTER TABLE request_logs ADD COLUMN outgoing_url TEXT;
ALTER TABLE request_logs ADD COLUMN outgoing_headers TEXT; -- JSON
ALTER TABLE request_logs ADD COLUMN outgoing_body TEXT; -- JSON

-- Create index for faster lookups by platform
CREATE INDEX IF NOT EXISTS idx_request_logs_platform_id ON request_logs(llm_platform_id);
