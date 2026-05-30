-- Migration: Add retention_rules and s3_profile_id to servers table
-- Description: D-07 retention rules (label-based) + D-14 S3 profile reference
-- Related: Phase 55 Scheduled Backups

ALTER TABLE servers
  ADD COLUMN IF NOT EXISTS retention_rules JSONB DEFAULT '{"daily": 7, "weekly": 4, "monthly": 3}',
  ADD COLUMN IF NOT EXISTS retention_mode TEXT DEFAULT 'hybrid' CHECK (retention_mode IN ('count', 'label', 'hybrid')),
  ADD COLUMN IF NOT EXISTS s3_profile_id UUID REFERENCES s3_profiles(id);
