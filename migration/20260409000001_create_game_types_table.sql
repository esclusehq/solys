-- Create game_types table for database-driven game type configuration
-- Allows adding new game types without code deployment
-- Plan: 05-01 Game Types

CREATE TABLE IF NOT EXISTS game_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    identifier VARCHAR(100) NOT NULL UNIQUE,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    docker_image VARCHAR(500) NOT NULL,
    default_ports JSONB DEFAULT '{"game": 25565, "rcon": 25575}'::jsonb,
    default_env JSONB DEFAULT '{}'::jsonb,
    startup_command TEXT,
    capabilities JSONB DEFAULT '{"rcon": true, "mods": true, "backup": true}'::jsonb,
    is_active BOOLEAN DEFAULT true,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Insert default game types
INSERT INTO game_types (identifier, display_name, description, docker_image, default_ports, default_env, capabilities, sort_order) VALUES
    ('minecraft', 'Minecraft', 'Minecraft Java Edition server with vanilla or modded support', 'itzg/minecraft-server:latest', '{"game": 25565, "rcon": 25575}', '{"EULA": "TRUE", "MODE": "survival"}', '{"rcon": true, "mods": true, "backup": true}', 1),
    ('palworld', 'Palworld', 'Palworld survival game server', 'ghcr.io/axllent/minecraft-palworld:latest', '{"game": 8211, "rcon": 25575}', '{"PLAYERS": 32}', '{"rcon": true, "backup": true}', 2),
    ('valheim', 'Valheim', 'Valheim dedicated server', 'lloesche/valheim-server:latest', '{"game": 2456, "rcon": 2457}', '{"WORLD_NAME": "world", "SERVER_NAME": "Valheim Server"}', '{"backup": true}', 3),
    ('fabric', 'Fabric', 'Fabric modloader for Minecraft', 'itzg/minecraft-server:latest', '{"game": 25565, "rcon": 25575}', '{"MODS": "fabric", "FABRIC_LAUNCHER_VERSION": "0.14.21"}', '{"rcon": true, "mods": true, "backup": true}', 4),
    ('forge', 'Forge', 'Forge modloader for Minecraft', 'itzg/minecraft-server:latest', '{"game": 25565, "rcon": 25575}', '{"MODS": "forge", "FORGE_VERSION": "1.20.1"}', '{"rcon": true, "mods": true, "backup": true}', 5)
ON CONFLICT (identifier) DO NOTHING;

-- Add index for identifier lookups
CREATE INDEX IF NOT EXISTS idx_game_types_identifier ON game_types(identifier) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_game_types_active ON game_types(is_active, sort_order);