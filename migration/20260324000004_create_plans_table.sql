-- Migration: Create plans table
-- Description: Subscription plans with limits and features

CREATE TABLE IF NOT EXISTS plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    price_monthly DECIMAL(10,2) NOT NULL,
    price_yearly DECIMAL(10,2),
    stripe_price_id_monthly VARCHAR(255),
    stripe_price_id_yearly VARCHAR(255),
    limits JSONB NOT NULL DEFAULT '{}',
    features JSONB DEFAULT '[]',
    is_active BOOLEAN NOT NULL DEFAULT true,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_plans_name ON plans(name);
CREATE INDEX IF NOT EXISTS idx_plans_is_active ON plans(is_active);

-- Insert default plans
INSERT INTO plans (name, display_name, description, price_monthly, price_yearly, limits, features, sort_order) VALUES
('free', 'Free', 'Free tier for testing', 0.00, NULL, 
 '{"max_servers": 1, "max_nodes": 1, "max_ram_mb": 2048, "max_cpu_cores": 1, "max_disk_gb": 5, "max_bandwidth_gb": 10, "max_backups": 0, "concurrent_operations": 1, "modpack_support": false, "priority_support": false, "sla": null}',
 '["basic"]', 0),
('hobby', 'Hobby', 'Hobby plan for small projects', 6.99, 69.90,
 '{"max_servers": 5, "max_nodes": 2, "max_ram_mb": 8192, "max_cpu_cores": 4, "max_disk_gb": 100, "max_bandwidth_gb": 1000, "max_backups": 5, "concurrent_operations": 3, "modpack_support": true, "priority_support": false, "sla": null}',
 '["basic", "modpacks"]', 1),
('pro', 'Pro', 'Pro plan for serious projects', 24.99, 249.90,
 '{"max_servers": 20, "max_nodes": 10, "max_ram_mb": 32768, "max_cpu_cores": 8, "max_disk_gb": 500, "max_bandwidth_gb": 5000, "max_backups": 20, "concurrent_operations": 5, "modpack_support": true, "priority_support": true, "sla": null}',
 '["basic", "modpacks", "backups", "priority"]', 2),
('enterprise', 'Enterprise', 'Enterprise plan with unlimited resources', 99.99, 999.90,
 '{"max_servers": -1, "max_nodes": -1, "max_ram_mb": -1, "max_cpu_cores": -1, "max_disk_gb": -1, "max_bandwidth_gb": -1, "max_backups": -1, "concurrent_operations": -1, "modpack_support": true, "priority_support": true, "sla": "99.9"}',
 '["basic", "modpacks", "backups", "priority", "sla"]', 3)
ON CONFLICT (name) DO NOTHING;
