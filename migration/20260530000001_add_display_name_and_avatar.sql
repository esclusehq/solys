-- Migration: Add display_name, avatar_url, scheduled_deletion_at to users
-- Description: Phase 53 profile management fields

ALTER TABLE users ADD COLUMN IF NOT EXISTS display_name VARCHAR(255);
ALTER TABLE users ADD COLUMN IF NOT EXISTS avatar_url TEXT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS scheduled_deletion_at TIMESTAMPTZ;

-- Index for the deletion cleanup cron job (D-07)
CREATE INDEX IF NOT EXISTS idx_users_scheduled_deletion_at ON users(scheduled_deletion_at);
