-- Add platform association to API keys
-- WARNING: This migration will set llm_platform_id to NULL for all existing API keys.
-- Existing keys will not work with the proxy until they are regenerated with a platform association.
-- Users must delete old keys and create new ones through the UI after this migration.

ALTER TABLE proxy_api_keys ADD COLUMN llm_platform_id INTEGER REFERENCES llm_platforms(id) ON DELETE CASCADE;

-- Create index for faster lookups
CREATE INDEX idx_proxy_api_keys_platform ON proxy_api_keys(llm_platform_id);
