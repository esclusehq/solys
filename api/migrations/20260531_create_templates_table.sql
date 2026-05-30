-- Create templates table for storing pre-configured server configurations
CREATE TABLE IF NOT EXISTS templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    game_type VARCHAR(50) NOT NULL,
    category VARCHAR(100) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    config JSONB NOT NULL DEFAULT '{}',
    visibility VARCHAR(20) NOT NULL DEFAULT 'private' CHECK (visibility IN ('public', 'private')),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    is_builtin BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Indexes for filtering and performance
CREATE INDEX IF NOT EXISTS idx_templates_game_type ON templates(game_type, is_active);
CREATE INDEX IF NOT EXISTS idx_templates_user_id ON templates(user_id);
CREATE INDEX IF NOT EXISTS idx_templates_visibility ON templates(visibility);

-- Seed built-in templates (per D-12)
INSERT INTO templates (game_type, category, display_name, description, config, visibility, is_builtin, is_active) VALUES
('minecraft', 'vanilla', 'Minecraft Vanilla', 'Default vanilla Minecraft server',
 '{"docker_image": "itzg/minecraft-server:latest", "default_port": 25565, "env": {"TYPE": "VANILLA", "MEMORY": "2G", "MAX_PLAYERS": "20"}}'::jsonb,
 'public', true, true),
('minecraft', 'paper', 'Minecraft Paper', 'Paper server with optimized performance',
 '{"docker_image": "itzg/minecraft-server:latest", "default_port": 25565, "env": {"TYPE": "PAPER", "MEMORY": "2G", "MAX_PLAYERS": "50"}}'::jsonb,
 'public', true, true),
('minecraft', 'forge', 'Minecraft Forge', 'Forge server for modded Minecraft',
 '{"docker_image": "itzg/minecraft-server:latest", "default_port": 25565, "env": {"TYPE": "FORGE", "MEMORY": "4G", "MAX_PLAYERS": "20"}, "startup_command": "java -Xms2G -Xmx4G -jar server.jar nogui"}'::jsonb,
 'public', true, true),
('minecraft', 'fabric', 'Minecraft Fabric', 'Fabric server for lightweight mods',
 '{"docker_image": "itzg/minecraft-server:latest", "default_port": 25565, "env": {"TYPE": "FABRIC", "MEMORY": "2G", "MAX_PLAYERS": "30"}}'::jsonb,
 'public', true, true),
('minecraft', 'spigot', 'Minecraft Spigot', 'Spigot server with API support',
 '{"docker_image": "itzg/minecraft-server:latest", "default_port": 25565, "env": {"TYPE": "SPIGOT", "MEMORY": "2G", "MAX_PLAYERS": "40"}}'::jsonb,
 'public', true, true),
('minecraft', 'bedrock', 'Minecraft Bedrock', 'Minecraft Bedrock Edition dedicated server',
 '{"docker_image": "itzg/minecraft-bedrock-server:latest", "default_port": 19132, "env": {"GAMEMODE": "survival", "DIFFICULTY": "normal", "LEVEL_NAME": "Bedrock Server"}}'::jsonb,
 'public', true, true),
('palworld', 'default', 'Palworld', 'Palworld dedicated server',
 '{"docker_image": "ghcr.io/axllent/minecraft-palworld:latest", "default_port": 8211, "env": {"MAX_PLAYERS": "32", "COMMUNITY_SERVER": "false"}}'::jsonb,
 'public', true, true),
('rust', 'default', 'Rust', 'Rust dedicated server',
 '{"docker_image": "cm2network/rust:latest", "default_port": 28015, "env": {"RUST_PORT": "28015", "RUST_ADMIN_PORT": "28016"}}'::jsonb,
 'public', true, true),
('valheim', 'default', 'Valheim', 'Valheim dedicated server',
 '{"docker_image": "lloesche/valheim-server:latest", "default_port": 2456, "env": {"WORLD_NAME": "world", "SERVER_PASSWORD": ""}}'::jsonb,
 'public', true, true);
