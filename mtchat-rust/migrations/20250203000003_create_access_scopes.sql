-- Migration: Create dialog_access_scopes table
-- This table defines rules for potential participants (who can see and join the chat)

CREATE TABLE dialog_access_scopes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dialog_id UUID NOT NULL REFERENCES dialogs(id) ON DELETE CASCADE,

    -- Tenant that this scope applies to
    tenant_uid UUID NOT NULL,

    -- Scope levels for matching (AND between levels, OR within level)
    -- scope_level1: departments/divisions (e.g., ["dept_logistics", "dept_sales"])
    -- scope_level2: permissions/roles (e.g., ["tender:manager", "tender:admin"])
    scope_level1 TEXT[] NOT NULL DEFAULT '{}',
    scope_level2 TEXT[] NOT NULL DEFAULT '{}',

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for finding scopes by dialog
CREATE INDEX idx_access_scopes_dialog ON dialog_access_scopes(dialog_id);

-- Index for finding scopes by tenant
CREATE INDEX idx_access_scopes_tenant ON dialog_access_scopes(tenant_uid);

-- GIN indexes for efficient array overlap operations
CREATE INDEX idx_access_scopes_level1 ON dialog_access_scopes USING GIN(scope_level1);
CREATE INDEX idx_access_scopes_level2 ON dialog_access_scopes USING GIN(scope_level2);

-- Comment on table
COMMENT ON TABLE dialog_access_scopes IS 'Defines who can potentially access a dialog (before joining)';
COMMENT ON COLUMN dialog_access_scopes.scope_level1 IS 'First scope level, e.g., departments. User must match at least one.';
COMMENT ON COLUMN dialog_access_scopes.scope_level2 IS 'Second scope level, e.g., permissions. User must match at least one.';
