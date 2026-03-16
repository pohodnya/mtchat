-- Migration: Replace tenant_uid with scope_level0 array
-- This enables consistent OR logic across all scope levels:
-- Match: (ANY scope_level0) AND (ANY scope_level1) AND (ANY scope_level2)

-- Add new scope_level0 array column
ALTER TABLE dialog_access_scopes
    ADD COLUMN scope_level0 TEXT[] NOT NULL DEFAULT '{}';

-- Migrate existing data: single tenant_uid becomes single-element array
UPDATE dialog_access_scopes
SET scope_level0 = ARRAY[tenant_uid]
WHERE tenant_uid IS NOT NULL AND tenant_uid != '';

-- Drop old constraint and column
ALTER TABLE dialog_access_scopes DROP CONSTRAINT IF EXISTS chk_tenant_uid_length;
ALTER TABLE dialog_access_scopes DROP COLUMN tenant_uid;

-- Drop old tenant index (no longer needed)
DROP INDEX IF EXISTS idx_access_scopes_tenant;

-- Add GIN index for efficient array overlap on scope_level0
CREATE INDEX idx_access_scopes_level0 ON dialog_access_scopes USING GIN(scope_level0);

-- Update comments
COMMENT ON COLUMN dialog_access_scopes.scope_level0 IS 'Top scope level, e.g., tenants/organizations. User must match at least one. Empty array = wildcard (matches all).';
