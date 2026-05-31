-- Add public_host column to servers table
ALTER TABLE servers ADD COLUMN public_host VARCHAR(255);
