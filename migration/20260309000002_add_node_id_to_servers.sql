-- Add node_id column to servers table for Node Agent support
ALTER TABLE servers ADD COLUMN IF NOT EXISTS node_id UUID REFERENCES nodes(id) ON DELETE SET NULL;
CREATE INDEX IF NOT EXISTS idx_servers_node_id ON servers(node_id);
