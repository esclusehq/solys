-- Add deployment_snapshot column to servers table for immutable runtime config
-- Plan: 05-04 Deployment Config

ALTER TABLE servers 
ADD COLUMN IF NOT EXISTS deployment_snapshot JSONB NOT NULL DEFAULT '{}'::jsonb;