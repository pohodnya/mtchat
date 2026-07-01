-- Add free-form JSON metadata to dialogs. Opaque to MTChat: never validated,
-- searched, or filtered. NULL means no metadata.
ALTER TABLE dialogs ADD COLUMN meta JSONB;
