-- Migration: Update attachments table for v3 architecture
-- - Remove uploader_id (sender is determined by message.sender_id)
-- - Rename columns to standard naming
-- - Add image metadata fields

-- Drop old indexes that reference uploader_id
DROP INDEX IF EXISTS idx_attachments_pending;
DROP INDEX IF EXISTS idx_attachments_cleanup;

-- Remove FK constraint to employees (if exists)
ALTER TABLE attachments DROP CONSTRAINT IF EXISTS attachments_uploader_id_fkey;

-- Drop uploader_id column
ALTER TABLE attachments DROP COLUMN IF EXISTS uploader_id;

-- Rename columns to standard naming
ALTER TABLE attachments RENAME COLUMN file_name TO filename;
ALTER TABLE attachments RENAME COLUMN file_type TO content_type;
ALTER TABLE attachments RENAME COLUMN file_size TO size;
ALTER TABLE attachments RENAME COLUMN storage_key TO s3_key;

-- Add image metadata columns
ALTER TABLE attachments ADD COLUMN IF NOT EXISTS width INTEGER;
ALTER TABLE attachments ADD COLUMN IF NOT EXISTS height INTEGER;
ALTER TABLE attachments ADD COLUMN IF NOT EXISTS thumbnail_s3_key VARCHAR(500);

-- Add default for id
ALTER TABLE attachments ALTER COLUMN id SET DEFAULT gen_random_uuid();

-- Make message_id NOT NULL (attachments must be linked to a message)
-- First, delete any orphaned attachments
DELETE FROM attachments WHERE message_id IS NULL;
ALTER TABLE attachments ALTER COLUMN message_id SET NOT NULL;

-- Add constraints
ALTER TABLE attachments ADD CONSTRAINT attachments_s3_key_unique UNIQUE (s3_key);
ALTER TABLE attachments ADD CONSTRAINT attachments_valid_size CHECK (size > 0 AND size <= 104857600);

-- Add index for content type (for filtering by type)
CREATE INDEX IF NOT EXISTS idx_attachments_content_type ON attachments(content_type);

-- Update existing index name for consistency
DROP INDEX IF EXISTS idx_attachments_message_id;
CREATE INDEX IF NOT EXISTS idx_attachments_message ON attachments(message_id);

-- Comments
COMMENT ON TABLE attachments IS 'File attachments linked to messages';
COMMENT ON COLUMN attachments.s3_key IS 'S3 object key (bucket is configured via S3_BUCKET env var)';
COMMENT ON COLUMN attachments.width IS 'Image width in pixels (NULL for non-images)';
COMMENT ON COLUMN attachments.height IS 'Image height in pixels (NULL for non-images)';
COMMENT ON COLUMN attachments.thumbnail_s3_key IS 'S3 key for thumbnail (images only)';
