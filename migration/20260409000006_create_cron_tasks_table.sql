-- Migration: Create cron_tasks table for scheduled task automation
-- Date: 2026-04-09

CREATE TABLE IF NOT EXISTS cron_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    task_type TEXT NOT NULL CHECK (task_type IN ('backup', 'restart', 'stop', 'command')),
    schedule_cron TEXT NOT NULL,
    command TEXT,
    enabled BOOLEAN DEFAULT true,
    last_run TIMESTAMP WITH TIME ZONE,
    next_run TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Index for efficient querying by server_id
CREATE INDEX IF NOT EXISTS idx_cron_tasks_server_id ON cron_tasks(server_id);

-- Index for finding due tasks
CREATE INDEX IF NOT EXISTS idx_cron_tasks_next_run ON cron_tasks(next_run) WHERE enabled = true;

-- Add updated_at trigger
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_cron_tasks_updated_at BEFORE UPDATE ON cron_tasks
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
