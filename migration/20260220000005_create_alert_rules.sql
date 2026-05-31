-- Create alert_rules table
CREATE TABLE IF NOT EXISTS alert_rules (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    metric_type TEXT NOT NULL, -- cpu, memory, tps
    operator TEXT NOT NULL, -- >, <, >=, <=, ==
    threshold DOUBLE PRECISION NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for faster lookups during evaluation
CREATE INDEX idx_alert_rules_server_id ON alert_rules(server_id);
