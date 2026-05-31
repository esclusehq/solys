CREATE TYPE server_environment AS ENUM ('production', 'staging', 'development');

ALTER TABLE servers ADD COLUMN environment server_environment NOT NULL DEFAULT 'production';
