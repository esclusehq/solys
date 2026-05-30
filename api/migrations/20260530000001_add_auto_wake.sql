-- Add sleep/wake and auto-restart backoff fields
-- Phase 56: Auto Online & Sleep Recovery

-- Auto-wake (sleep mode)
ALTER TABLE servers ADD COLUMN IF NOT EXISTS auto_wake BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE servers ADD COLUMN IF NOT EXISTS sleep_timeout_minutes INTEGER NOT NULL DEFAULT 30;
ALTER TABLE servers ADD COLUMN IF NOT EXISTS last_player_activity TIMESTAMPTZ;

-- Auto-restart backoff
ALTER TABLE servers ADD COLUMN IF NOT EXISTS max_restart_attempts INTEGER NOT NULL DEFAULT 5;
ALTER TABLE servers ADD COLUMN IF NOT EXISTS restart_cooldown_seconds INTEGER NOT NULL DEFAULT 300;
