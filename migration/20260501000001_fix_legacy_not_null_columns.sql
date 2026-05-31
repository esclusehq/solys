-- Fix: Add safe DEFAULT values to legacy NOT NULL columns
-- These columns exist from old migrations and are NOT NULL,
-- but the new Server model no longer includes them in INSERT.
-- Setting defaults lets the DB fill them automatically.

ALTER TABLE servers ALTER COLUMN host SET DEFAULT '';
ALTER TABLE servers ALTER COLUMN username SET DEFAULT '';
ALTER TABLE servers ALTER COLUMN game SET DEFAULT 'minecraft';
ALTER TABLE servers ALTER COLUMN password_auth SET DEFAULT '';
ALTER TABLE servers ALTER COLUMN auto_backup_enabled SET DEFAULT false;
ALTER TABLE servers ALTER COLUMN backup_provider SET DEFAULT 'local';
ALTER TABLE servers ALTER COLUMN auto_restart SET DEFAULT true;
ALTER TABLE servers ALTER COLUMN restart_count SET DEFAULT 0;
ALTER TABLE servers ALTER COLUMN enable_tailscale SET DEFAULT false;

-- environment is a ENUM type - set default
ALTER TABLE servers ALTER COLUMN environment SET DEFAULT 'production';

-- deployment_snapshot may have been added as NOT NULL DEFAULT '{}'
-- ensure the default is set in case it was added without one
ALTER TABLE servers ALTER COLUMN deployment_snapshot SET DEFAULT '{}'::jsonb;
