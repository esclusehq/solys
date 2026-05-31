-- Migration: Add missing columns to servers table for new schema
-- These columns are needed by the Rust Server model

ALTER TABLE servers ADD COLUMN IF NOT EXISTS user_id UUID REFERENCES users(id);
ALTER TABLE servers ADD COLUMN IF NOT EXISTS agent_id UUID;
ALTER TABLE servers ADD COLUMN IF NOT EXISTS job_id UUID;
ALTER TABLE servers ADD COLUMN IF NOT EXISTS image TEXT DEFAULT 'minecraft';
ALTER TABLE servers ADD COLUMN IF NOT EXISTS node_id UUID REFERENCES nodes(id);
ALTER TABLE servers ADD COLUMN IF NOT EXISTS remote_id TEXT;
ALTER TABLE servers ADD COLUMN IF NOT EXISTS config JSONB DEFAULT '{}';
ALTER TABLE servers ADD COLUMN IF NOT EXISTS resources JSONB DEFAULT '{}';
ALTER TABLE servers ADD COLUMN IF NOT EXISTS endpoints JSONB DEFAULT '[]';
ALTER TABLE servers ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMP;

-- Update existing records with user_id if needed
UPDATE servers SET user_id = '11111111-1111-1111-1111-111111111110' WHERE user_id IS NULL;
