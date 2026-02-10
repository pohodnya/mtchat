-- Add object_url field to dialogs table
-- This stores a URL to the object page in the host system

ALTER TABLE dialogs ADD COLUMN object_url TEXT;

COMMENT ON COLUMN dialogs.object_url IS 'URL to the object page in the host system';
