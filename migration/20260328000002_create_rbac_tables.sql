-- Migration: Create RBAC tables
-- Description: Role-based access control for multi-tenant SaaS

-- Roles table (tenant-scoped)
CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(50) NOT NULL,
    description TEXT,
    is_default BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, name)
);

CREATE INDEX IF NOT EXISTS idx_roles_user_id ON roles(user_id);

-- Permissions table
CREATE TABLE IF NOT EXISTS permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    resource VARCHAR(50) NOT NULL,
    action VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(role_id, resource, action)
);

CREATE INDEX IF NOT EXISTS idx_permissions_role_id ON permissions(role_id);

-- User roles junction table
CREATE TABLE IF NOT EXISTS user_roles (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    assigned_by UUID REFERENCES users(id),
    PRIMARY KEY(user_id, role_id)
);

CREATE INDEX IF NOT EXISTS idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX IF NOT EXISTS idx_user_roles_role_id ON user_roles(role_id);

-- Create default roles for existing users
DO $$
DECLARE
    rec RECORD;
    owner_role_id UUID;
BEGIN
    -- For each existing user, create owner role
    FOR rec IN SELECT id FROM users LOOP
        -- Insert owner role
        INSERT INTO roles (user_id, name, description, is_default, created_at, updated_at)
        VALUES (rec.id, 'owner', 'Full access to all resources', true, NOW(), NOW())
        ON CONFLICT (user_id, name) DO NOTHING
        RETURNING id INTO owner_role_id;
        
        -- Insert all permissions for owner
        IF owner_role_id IS NOT NULL THEN
            INSERT INTO permissions (role_id, resource, action) VALUES
            (owner_role_id, 'nodes', 'read'),
            (owner_role_id, 'nodes', 'create'),
            (owner_role_id, 'nodes', 'update'),
            (owner_role_id, 'nodes', 'delete'),
            (owner_role_id, 'servers', 'read'),
            (owner_role_id, 'servers', 'create'),
            (owner_role_id, 'servers', 'update'),
            (owner_role_id, 'servers', 'delete'),
            (owner_role_id, 'servers', 'execute'),
            (owner_role_id, 'billing', 'read'),
            (owner_role_id, 'billing', 'write'),
            (owner_role_id, 'users', 'read'),
            (owner_role_id, 'users', 'create'),
            (owner_role_id, 'users', 'update'),
            (owner_role_id, 'users', 'delete'),
            (owner_role_id, 'audit', 'read'),
            (owner_role_id, 'roles', 'read'),
            (owner_role_id, 'roles', 'create'),
            (owner_role_id, 'roles', 'update'),
            (owner_role_id, 'roles', 'delete')
            ON CONFLICT DO NOTHING;
            
            -- Assign owner role to user
            INSERT INTO user_roles (user_id, role_id, assigned_at)
            VALUES (rec.id, owner_role_id, NOW())
            ON CONFLICT DO NOTHING;
        END IF;
    END LOOP;
END $$;
