-- Create modpack_templates table for storing pre-configured modpack lists
CREATE TABLE IF NOT EXISTS modpack_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    game_type VARCHAR(50) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    source VARCHAR(20) NOT NULL CHECK (source IN ('curseforge', 'modrinth')),
    project_id VARCHAR(100) NOT NULL,
    version_id VARCHAR(100) NOT NULL,
    version_name VARCHAR(50) NOT NULL,
    mod_count INTEGER NOT NULL DEFAULT 0,
    image_url TEXT,
    min_plan VARCHAR(20) NOT NULL DEFAULT 'hobby' CHECK (min_plan IN ('hobby', 'pro', 'enterprise')),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Index for filtering by game type and active status
CREATE INDEX IF NOT EXISTS idx_modpack_templates_game_type_active 
ON modpack_templates(game_type, is_active);

-- Index for filtering by plan tier
CREATE INDEX IF NOT EXISTS idx_modpack_templates_min_plan 
ON modpack_templates(min_plan);

-- Index for ordering by plan tier
CREATE INDEX IF NOT EXISTS idx_modpack_templates_plan_order 
ON modpack_templates(min_plan, game_type, display_name);