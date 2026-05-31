-- Migration: Fix audit_logs constraints
-- Description: Fix tenant_id NOT NULL constraint issue

-- Make tenant_id nullable (it references users which may not exist)
ALTER TABLE audit_logs 
ALTER COLUMN tenant_id DROP NOT NULL;

-- Set default for severity
ALTER TABLE audit_logs 
ALTER COLUMN severity SET DEFAULT 'info';
