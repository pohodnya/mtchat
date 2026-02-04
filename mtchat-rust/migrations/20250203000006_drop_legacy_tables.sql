-- Migration: Drop legacy tables (employees, tenants)
-- In the new architecture, user identity comes from JWT tokens, not internal tables

-- Step 1: Remove FK from attachments to employees
ALTER TABLE attachments DROP CONSTRAINT IF EXISTS attachments_uploader_id_fkey;

-- Step 2: Update attachments.uploader_id comment
COMMENT ON COLUMN attachments.uploader_id IS 'External user identifier (from JWT token)';

-- Step 3: Drop employees table (depends on tenants)
DROP TABLE IF EXISTS employees CASCADE;

-- Step 4: Drop tenants table
DROP TABLE IF EXISTS tenants CASCADE;
