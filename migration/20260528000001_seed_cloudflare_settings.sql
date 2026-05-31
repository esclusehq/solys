-- Seed default Cloudflare config (empty, configured via dashboard)
INSERT INTO app_settings (key, value) VALUES ('cloudflare_config', '{
    "api_token": "",
    "zone_id": "",
    "zone_name": "esluce.com",
    "wildcard_domain": "esluce.com",
    "auto_refresh": true,
    "refresh_interval_secs": 300
}'::jsonb)
ON CONFLICT (key) DO NOTHING;
