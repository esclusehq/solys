-- App-level settings (key-value store for global configuration)
CREATE TABLE IF NOT EXISTS app_settings (
    key   TEXT PRIMARY KEY,
    value JSONB NOT NULL DEFAULT '{}'::jsonb,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed default S3 config (empty)
INSERT INTO app_settings (key, value) VALUES ('s3_config', '{
    "endpoint": "",
    "region": "",
    "bucket": "",
    "access_key": "",
    "secret_key": ""
}'::jsonb)
ON CONFLICT (key) DO NOTHING;
