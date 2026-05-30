-- Add restart policy and health check fields
-- Phase 57: Auto Restart Policies

-- Restart history tracking
ALTER TABLE servers ADD COLUMN IF NOT EXISTS last_restart_at TIMESTAMPTZ;
ALTER TABLE servers ADD COLUMN IF NOT EXISTS last_restart_reason TEXT;

-- Health check configuration
ALTER TABLE servers ADD COLUMN IF NOT EXISTS health_check_timeout_seconds INTEGER NOT NULL DEFAULT 5;
