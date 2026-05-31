-- Migration: Add tenant_id to nodes table
-- Description: Multi-tenant isolation - nodes belong to users (tenants)

-- Add user_id (tenant_id) to nodes (allow NULL for system nodes)
ALTER TABLE nodes 
ADD COLUMN IF NOT EXISTS user_id UUID REFERENCES users(id) ON DELETE SET NULL;

-- Add user_id to server_nodes relationship
ALTER TABLE server_nodes 
ADD COLUMN IF NOT EXISTS user_id UUID REFERENCES users(id) ON DELETE SET NULL;

-- Create indexes for tenant isolation
CREATE INDEX IF NOT EXISTS idx_nodes_user_id ON nodes(user_id);
CREATE INDEX IF NOT EXISTS idx_server_nodes_user_id ON server_nodes(user_id);

-- Add user_id to servers if not exists
ALTER TABLE servers 
ADD COLUMN IF NOT EXISTS user_id UUID REFERENCES users(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_servers_user_id ON servers(user_id);
