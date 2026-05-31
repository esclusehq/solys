-- Add Git remote configuration fields per server
ALTER TABLE servers ADD COLUMN IF NOT EXISTS git_remote_url TEXT;
ALTER TABLE servers ADD COLUMN IF NOT EXISTS git_remote_username TEXT;
ALTER TABLE servers ADD COLUMN IF NOT EXISTS git_remote_token TEXT;
