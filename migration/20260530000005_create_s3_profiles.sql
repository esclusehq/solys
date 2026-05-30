-- Migration: Create s3_profiles table for named S3 storage profiles
-- Description: D-14 platform-level S3 credential profiles with name, endpoint, bucket, keys
-- Related: Phase 55 Scheduled Backups

CREATE TABLE IF NOT EXISTS s3_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    endpoint TEXT NOT NULL,
    region TEXT NOT NULL DEFAULT '',
    bucket TEXT NOT NULL,
    access_key TEXT NOT NULL,
    secret_key TEXT NOT NULL,
    is_default BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_s3_profiles_name ON s3_profiles(name);
CREATE INDEX IF NOT EXISTS idx_s3_profiles_is_default ON s3_profiles(is_default);
