CREATE TABLE IF NOT EXISTS server_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    server_id UUID NOT NULL,
    cpu_usage REAL NOT NULL,
    memory_usage_mb BIGINT NOT NULL,
    disk_usage_mb BIGINT NOT NULL DEFAULT 0,
    tps REAL,
    players INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_server_metrics_server_id_created_at ON server_metrics(server_id, created_at DESC);
