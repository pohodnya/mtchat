-- Remove unique constraint on (object_id, object_type) to allow multiple dialogs per object

-- Drop the unique index
DROP INDEX IF EXISTS idx_dialogs_object;

-- Create a regular (non-unique) index for query performance
CREATE INDEX idx_dialogs_object ON dialogs(object_id, object_type);
