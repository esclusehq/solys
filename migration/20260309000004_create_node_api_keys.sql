-- Create node_api_keys table for API key management
CREATE TABLE IF NOT EXISTS node_api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    node_id UUID NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    key_hash VARCHAR(255) NOT NULL,
    name VARCHAR(100),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT NOW(),
    expires_at TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_node_api_keys_node_id ON node_api_keys(node_id);
CREATE INDEX IF NOT EXISTS idx_node_api_keys_hash ON node_api_keys(key_hash);

-- Also add api_key to nodes table for initial/legacy support (if not exists)
ALTER TABLE nodes ADD COLUMN IF NOT EXISTS api_key_hash VARCHAR(255);
