-- Add missing disk_usage_mb column to server_metrics
ALTER TABLE server_metrics ADD COLUMN IF NOT EXISTS disk_usage_mb BIGINT NOT NULL DEFAULT 0;