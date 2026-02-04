-- Migration: Update dialogs table for object-bound architecture
-- This migration transforms the tenant-to-tenant model to object-bound model

-- Step 1: Drop old indexes
DROP INDEX IF EXISTS idx_dialogs_chat_key;
DROP INDEX IF EXISTS idx_dialogs_tenant_a;
DROP INDEX IF EXISTS idx_dialogs_tenant_b;
DROP INDEX IF EXISTS idx_dialogs_context_id;

-- Step 2: Drop old constraints
ALTER TABLE dialogs DROP CONSTRAINT IF EXISTS dialogs_different_tenants;

-- Step 3: Drop old columns
ALTER TABLE dialogs DROP COLUMN IF EXISTS chat_key;
ALTER TABLE dialogs DROP COLUMN IF EXISTS tenant_a_id;
ALTER TABLE dialogs DROP COLUMN IF EXISTS tenant_b_id;

-- Step 4: Rename context_id to object_id (it already exists, just rename)
ALTER TABLE dialogs RENAME COLUMN context_id TO object_id;

-- Step 5: Add new columns
ALTER TABLE dialogs ADD COLUMN object_type VARCHAR(100);
ALTER TABLE dialogs ADD COLUMN title VARCHAR(500);
ALTER TABLE dialogs ADD COLUMN created_by UUID;

-- Step 6: Make object_id NOT NULL (set default for existing rows first)
UPDATE dialogs SET object_id = id WHERE object_id IS NULL;
ALTER TABLE dialogs ALTER COLUMN object_id SET NOT NULL;

-- Step 7: Set defaults for new columns on existing rows
UPDATE dialogs SET object_type = 'legacy' WHERE object_type IS NULL;
ALTER TABLE dialogs ALTER COLUMN object_type SET NOT NULL;

-- Step 8: Create new indexes
CREATE UNIQUE INDEX idx_dialogs_object ON dialogs(object_id, object_type);
CREATE INDEX idx_dialogs_object_type ON dialogs(object_type);
CREATE INDEX idx_dialogs_created_by ON dialogs(created_by) WHERE created_by IS NOT NULL;
