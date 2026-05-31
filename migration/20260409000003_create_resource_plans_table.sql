-- Create resource_plans table for predefined resource configurations
-- User selects from predefined plans (2GB, 4GB, 8GB, 16GB)
-- Plan: 05-03 Resource Plans

CREATE TABLE IF NOT EXISTS resource_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) NOT NULL UNIQUE,
    display_name VARCHAR(100) NOT NULL,
    ram_mb INTEGER NOT NULL CHECK (ram_mb > 0),
    cpu_cores INTEGER NOT NULL CHECK (cpu_cores > 0),
    disk_gb INTEGER NOT NULL CHECK (disk_gb > 0),
    price_monthly DECIMAL(10, 2),
    is_active BOOLEAN DEFAULT true,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Insert default resource plans with CPU ratios: 2GB→2 cores, 4GB→3 cores, 8GB→4 cores, 16GB→6 cores
INSERT INTO resource_plans (name, display_name, ram_mb, cpu_cores, disk_gb, price_monthly, sort_order) VALUES
    ('2gb', '2 GB', 2048, 2, 10, 5.00, 1),
    ('4gb', '4 GB', 4096, 3, 20, 10.00, 2),
    ('8gb', '8 GB', 8192, 4, 50, 20.00, 3),
    ('16gb', '16 GB', 16384, 6, 100, 40.00, 4)
ON CONFLICT (name) DO NOTHING;

-- Add indexes
CREATE INDEX IF NOT EXISTS idx_resource_plans_name ON resource_plans(name) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_resource_plans_active ON resource_plans(is_active, sort_order);