-- Add missing game and password_auth columns
ALTER TABLE servers ADD COLUMN IF NOT EXISTS game TEXT NOT NULL DEFAULT 'minecraft';
ALTER TABLE servers ADD COLUMN IF NOT EXISTS password_auth TEXT NOT NULL DEFAULT '';
