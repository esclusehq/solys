-- Add new columns for Solys-compatible server model
-- This migration adds the columns required by the new Server model in PANEL_BACKEND_PLAN.md

-- Add user_id column (references owner)
ALTER TABLE servers ADD COLUMN IF NOT EXISTS user_id UUID REFERENCES users(id);

-- Add agent_id column (references Solys agent)
ALTER TABLE servers ADD COLUMN IF NOT EXISTS agent_id UUID REFERENCES agents(id);

-- Add job_id column (references active job)
ALTER TABLE servers ADD COLUMN IF NOT EXISTS job_id UUID REFERENCES jobs(id);

-- Add image column (Docker image)
ALTER TABLE servers ADD COLUMN IF NOT EXISTS image TEXT NOT NULL DEFAULT 'nginx:latest';

-- Add remote_id column (ID on the remote agent)
ALTER TABLE servers ADD COLUMN IF NOT EXISTS remote_id TEXT;

-- Add config column (JSONB for server configuration)
ALTER TABLE servers ADD COLUMN IF NOT EXISTS config JSONB DEFAULT '{}'::jsonb;

-- Add resources column (JSONB for resource allocation)
ALTER TABLE servers ADD COLUMN IF NOT EXISTS resources JSONB DEFAULT '{"ram": "1G", "cpu": 1, "disk": "5G"}'::jsonb;

-- Add endpoints column (JSONB for server endpoints)
ALTER TABLE servers ADD COLUMN IF NOT EXISTS endpoints JSONB DEFAULT '[]'::jsonb;

-- Add deleted_at column for soft deletes
ALTER TABLE servers ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- Drop old columns that are no longer needed (optional - comment out if you want to keep them)
-- ALTER TABLE servers DROP COLUMN IF EXISTS host;
-- ALTER TABLE servers DROP COLUMN IF EXISTS port;
-- ALTER TABLE servers DROP COLUMN IF EXISTS executor_type;

-- Set user_id for existing records (would need to be updated with actual user IDs)
-- For now, we'll set a default or null
ALTER TABLE servers ALTER COLUMN user_id DROP NOT NULL;
ALTER TABLE servers ALTER COLUMN image SET DEFAULT 'nginx:latest';
