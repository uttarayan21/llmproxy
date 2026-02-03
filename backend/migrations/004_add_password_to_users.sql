-- Migration: Add password hash field to users table for authentication
-- This migration adds password support while maintaining backwards compatibility
-- Existing users will have NULL passwords (can still use Remote-User header auth)

-- Add password_hash column (nullable for backwards compatibility)
ALTER TABLE users ADD COLUMN password_hash TEXT;

-- Create index for faster password lookups
CREATE INDEX idx_users_password_hash ON users(password_hash);

-- Create sessions table for tower-sessions
CREATE TABLE IF NOT EXISTS tower_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    data BLOB NOT NULL,
    expiry_date INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS tower_sessions_expiry_date_idx ON tower_sessions(expiry_date);
