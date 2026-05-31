-- Add new fields for enhanced server features
-- Auto-restart, Tailscale, Custom IP, Templates, Networks

-- Auto-restart configuration
ALTER TABLE servers ADD COLUMN IF NOT EXISTS auto_restart BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE servers ADD COLUMN IF NOT EXISTS restart_count INTEGER NOT NULL DEFAULT 0;

-- Tailscale per-server configuration
ALTER TABLE servers ADD COLUMN IF NOT EXISTS enable_tailscale BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE servers ADD COLUMN IF NOT EXISTS tailscale_auth_key TEXT;

-- Custom container configuration
ALTER TABLE servers ADD COLUMN IF NOT EXISTS custom_container_name TEXT;
ALTER TABLE servers ADD COLUMN IF NOT EXISTS ip_binding TEXT DEFAULT '0.0.0.0';

-- Template and network configuration
ALTER TABLE servers ADD COLUMN IF NOT EXISTS template TEXT DEFAULT 'paper';
ALTER TABLE servers ADD COLUMN IF NOT EXISTS network_name TEXT DEFAULT 'devnode-minecraft';
