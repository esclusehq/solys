-- Create deployment_configs table for deployment template storage
-- Source of truth for deployment configurations
-- Plan: 05-04 Deployment Config

CREATE TABLE IF NOT EXISTS deployment_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    game_type_id UUID REFERENCES game_types(id) ON DELETE SET NULL,
    docker_image VARCHAR(500),
    ports JSONB DEFAULT '{"game": 25565, "rcon": 25575}'::jsonb,
    env_vars JSONB DEFAULT '{}'::jsonb,
    startup_command TEXT,
    resources JSONB DEFAULT '{"ram": "2G", "cpu": 2, "disk": "10G"}'::jsonb,
    volume_path VARCHAR(500) DEFAULT '/data',
    network_name VARCHAR(100) DEFAULT 'bridge',
    is_default BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    CONSTRAINT unique_config_name UNIQUE (name)
);

-- Insert default Minecraft config
INSERT INTO deployment_configs (name, game_type_id, docker_image, ports, env_vars, resources, volume_path, network_name, is_default)
SELECT 
    'minecraft-default',
    id,
    'itzg/minecraft-server:latest',
    '{"game": 25565, "rcon": 25575}',
    '{"EULA": "TRUE", "MODE": "survival"}',
    '{"ram": "2G", "cpu": 2, "disk": "10G"}',
    '/data',
    'bridge',
    true
FROM game_types
WHERE identifier = 'minecraft'
ON CONFLICT (name) DO NOTHING;

-- Create index for game_type lookups
CREATE INDEX IF NOT EXISTS idx_deployment_configs_game_type ON deployment_configs(game_type_id);
CREATE INDEX IF NOT EXISTS idx_deployment_configs_default ON deployment_configs(is_default, game_type_id) WHERE is_default = true;