-- Migration: Make audit_logs immutable (append-only)
-- Description: Ensure audit logs cannot be modified or deleted

-- 1. Revoke UPDATE and DELETE permissions from all roles
REVOKE UPDATE, DELETE ON audit_logs FROM PUBLIC;

-- 2. Create trigger function to prevent updates/deletes
CREATE OR REPLACE FUNCTION audit_logs_prevent_modification()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'Audit logs are immutable and cannot be modified or deleted';
END;
$$ LANGUAGE plpgsql;

-- 3. Create trigger for DELETE prevention
DROP TRIGGER IF EXISTS audit_logs_no_delete ON audit_logs;
CREATE TRIGGER audit_logs_no_delete
    BEFORE DELETE ON audit_logs
    FOR EACH ROW
    EXECUTE FUNCTION audit_logs_prevent_modification();

-- 4. Create trigger for UPDATE prevention  
DROP TRIGGER IF EXISTS audit_logs_no_update ON audit_logs;
CREATE TRIGGER audit_logs_no_update
    BEFORE UPDATE ON audit_logs
    FOR EACH ROW
    EXECUTE FUNCTION audit_logs_prevent_modification();

-- 5. Add comment
COMMENT ON TABLE audit_logs IS 'Immutable audit log table - INSERT only, no UPDATE or DELETE allowed';
