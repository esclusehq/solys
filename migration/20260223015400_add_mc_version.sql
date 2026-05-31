-- Add mc_version column to servers table
ALTER TABLE servers
ADD COLUMN mc_version VARCHAR(255) DEFAULT 'LATEST';
