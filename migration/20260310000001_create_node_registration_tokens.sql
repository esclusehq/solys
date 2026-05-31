-- Create node_registration_tokens table for agent auto-registration
CREATE TABLE IF NOT EXISTS node_registration_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    node_id UUID REFERENCES nodes(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(100),
    is_active BOOLEAN DEFAULT true,
    used_at TIMESTAMP,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Index for token lookup
CREATE INDEX IF NOT EXISTS idx_node_registration_tokens_hash ON node_registration_tokens(token_hash);

-- Index for node lookup
CREATE INDEX IF NOT EXISTS idx_node_registration_tokens_node ON node_registration_tokens(node_id);
