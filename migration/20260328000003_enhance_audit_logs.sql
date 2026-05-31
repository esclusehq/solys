-- Migration: Enhance audit_logs table
-- Description: Add tenant isolation and full audit trail

-- Add tenant_id (already have user_id, but ensure consistency)
ALTER TABLE audit_logs 
ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE;

-- Add old/new values for full audit trail
ALTER TABLE audit_logs 
ADD COLUMN IF NOT EXISTS old_value JSONB;

ALTER TABLE audit_logs 
ADD COLUMN IF NOT EXISTS new_value JSONB;

-- Add actor info (if different from user_id, e.g., system actions)
ALTER TABLE audit_logs 
ADD COLUMN IF NOT EXISTS actor_id UUID REFERENCES users(id);

-- Add severity
ALTER TABLE audit_logs 
ADD COLUMN IF NOT EXISTS severity VARCHAR(20) DEFAULT 'info';

-- Add indexes for tenant isolation and queries
CREATE INDEX IF NOT EXISTS idx_audit_logs_tenant_id ON audit_logs(tenant_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_tenant_created ON audit_logs(tenant_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_logs_resource ON audit_logs(tenant_id, resource_type, resource_id);

-- Backfill tenant_id from user_id if NULL
UPDATE audit_logs 
SET tenant_id = user_id 
WHERE tenant_id IS NULL AND user_id IS NOT NULL;

-- Set default tenant_id for rows where both are NULL
ALTER TABLE audit_logs 
ALTER COLUMN tenant_id SET DEFAULT NULL;
