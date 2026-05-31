-- Add backup scheduling columns to servers table
ALTER TABLE servers ADD COLUMN auto_backup_enabled BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE servers ADD COLUMN backup_cron TEXT;
ALTER TABLE servers ADD COLUMN backup_provider TEXT NOT NULL DEFAULT 'local';
ALTER TABLE servers ADD COLUMN backup_path TEXT;
ALTER TABLE servers ADD COLUMN max_retained_backups INTEGER NOT NULL DEFAULT 5;
