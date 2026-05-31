-- Create node_metrics table for storing historical node metrics
CREATE TABLE IF NOT EXISTS node_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    node_id UUID NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    cpu_usage DOUBLE PRECISION DEFAULT 0,
    memory_used BIGINT DEFAULT 0,
    memory_total BIGINT DEFAULT 0,
    disk_used BIGINT DEFAULT 0,
    disk_total BIGINT DEFAULT 0,
    container_count INTEGER DEFAULT 0,
    
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_node_metrics_node_id ON node_metrics(node_id);
CREATE INDEX IF NOT EXISTS idx_node_metrics_created_at ON node_metrics(created_at);

-- Add current_metrics JSONB to nodes table for quick access
ALTER TABLE nodes ADD COLUMN IF NOT EXISTS current_metrics JSONB;
