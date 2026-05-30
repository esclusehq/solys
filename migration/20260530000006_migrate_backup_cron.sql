-- Migration: Copy existing server.backup_cron values into cron_tasks table
-- Description: D-03 one-time migration from deprecated server.backup_cron to cron_tasks
-- Related: Phase 55 Scheduled Backups
-- NOTE: After this migration, old BackupScheduler in API is disabled (D-02).
--       Worker cron evaluation replaces it entirely.

INSERT INTO cron_tasks (id, server_id, user_id, task_type, schedule_cron, command, enabled, last_run, created_at, updated_at)
SELECT
    gen_random_uuid(),
    s.id,
    COALESCE(s.user_id, (SELECT id FROM users LIMIT 1)),
    'backup',
    s.backup_cron,
    NULL,
    s.auto_backup_enabled,
    NULL,
    NOW(),
    NOW()
FROM servers s
WHERE s.backup_cron IS NOT NULL AND s.backup_cron != ''
AND NOT EXISTS (
    SELECT 1 FROM cron_tasks ct
    WHERE ct.server_id = s.id AND ct.task_type = 'backup'
);
