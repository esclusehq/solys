-- Create nodes table for Node Agent management
CREATE TABLE IF NOT EXISTS nodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    
    -- Connection info
    ip_address VARCHAR(45) NOT NULL,
    port INTEGER NOT NULL DEFAULT 8080,
    
    -- Status tracking
    status VARCHAR(50) DEFAULT 'offline',
    last_seen TIMESTAMP,
    first_seen TIMESTAMP DEFAULT NOW(),
    
    -- Node capabilities
    podman_version VARCHAR(50),
    os_info VARCHAR(255),
    total_memory BIGINT,
    cpu_cores INTEGER,
    
    -- Agent info
    agent_version VARCHAR(50),
    agent_capabilities JSONB DEFAULT '["podman", "metrics"]',
    
    -- Security
    api_key_hash VARCHAR(255),
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_nodes_status ON nodes(status);
CREATE INDEX IF NOT EXISTS idx_nodes_ip ON nodes(ip_address);

-- Create server_nodes relationship table
CREATE TABLE IF NOT EXISTS server_nodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    node_id UUID NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    is_primary BOOLEAN DEFAULT true,
    
    created_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(server_id, node_id)
);

CREATE INDEX IF NOT EXISTS idx_server_nodes_server ON server_nodes(server_id);
CREATE INDEX IF NOT EXISTS idx_server_nodes_node ON server_nodes(node_id);

-- Create node_events table for logging
CREATE TABLE IF NOT EXISTS node_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    node_id UUID NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL,
    severity VARCHAR(20) DEFAULT 'info',
    message TEXT,
    data JSONB,
    
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_node_events_node ON node_events(node_id);
CREATE INDEX IF NOT EXISTS idx_node_events_type ON node_events(event_type);
