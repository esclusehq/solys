-- Create port_pools table for dynamic port allocation
-- Prevents port conflicts across servers on same node
-- Plan: 05-02 Port Allocation

CREATE TABLE IF NOT EXISTS port_pools (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    node_id UUID REFERENCES nodes(id) ON DELETE CASCADE,
    port_range_start INTEGER NOT NULL CHECK (port_range_start > 0 AND port_range_start < 65536),
    port_range_end INTEGER NOT NULL CHECK (port_range_end > 0 AND port_range_end <= 65535),
    current_port INTEGER NOT NULL,
    allocated_ports JSONB DEFAULT '[]'::jsonb,
    protocol VARCHAR(10) DEFAULT 'tcp' CHECK (protocol IN ('tcp', 'udp', 'both')),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    CONSTRAINT valid_port_range CHECK (port_range_start <= port_range_end)
);

-- Create unique index for node pools
CREATE UNIQUE INDEX IF NOT EXISTS idx_port_pools_node ON port_pools(node_id) WHERE node_id IS NOT NULL;

-- Create global pool (node_id = NULL)
INSERT INTO port_pools (id, node_id, port_range_start, port_range_end, current_port, protocol, is_active)
SELECT gen_random_uuid(), NULL, 25565, 25665, 25565, 'tcp', true
WHERE NOT EXISTS (SELECT 1 FROM port_pools WHERE node_id IS NULL);

-- Add indexes
CREATE INDEX IF NOT EXISTS idx_port_pools_active ON port_pools(is_active);