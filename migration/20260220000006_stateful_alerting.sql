-- Add duration_seconds to alert_rules
ALTER TABLE alert_rules ADD COLUMN IF NOT EXISTS duration_seconds INTEGER NOT NULL DEFAULT 0;

-- Create alert_states table
CREATE TABLE IF NOT EXISTS alert_states (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    rule_id UUID NOT NULL REFERENCES alert_rules(id) ON DELETE CASCADE,
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    status TEXT NOT NULL, -- 'normal' or 'triggered'
    last_triggered_at TIMESTAMPTZ,
    last_recovered_at TIMESTAMPTZ,
    violation_count INTEGER NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(rule_id, server_id) -- Ensure one state per rule per server
);

-- Create alert_history table (audit trail)
CREATE TABLE IF NOT EXISTS alert_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    rule_id UUID NOT NULL REFERENCES alert_rules(id) ON DELETE CASCADE,
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL, -- 'triggered' or 'recovered'
    metric_value DOUBLE PRECISION NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_alert_states_server_id ON alert_states(server_id);
CREATE INDEX idx_alert_history_server_id ON alert_history(server_id);
CREATE INDEX idx_alert_history_created_at ON alert_history(created_at DESC);
