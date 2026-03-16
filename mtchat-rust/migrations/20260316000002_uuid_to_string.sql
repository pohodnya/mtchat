-- Convert UUID columns to TEXT for flexible external identifiers
-- Existing UUID values are preserved as hyphenated strings (e.g., "550e8400-e29b-41d4-a716-446655440000")

-- dialogs table
ALTER TABLE dialogs
    ALTER COLUMN object_id TYPE TEXT USING object_id::text,
    ALTER COLUMN created_by TYPE TEXT USING created_by::text;

-- dialog_participants table
ALTER TABLE dialog_participants
    ALTER COLUMN user_id TYPE TEXT USING user_id::text;

-- messages table
ALTER TABLE messages
    ALTER COLUMN sender_id TYPE TEXT USING sender_id::text;

-- dialog_access_scopes table
ALTER TABLE dialog_access_scopes
    ALTER COLUMN tenant_uid TYPE TEXT USING tenant_uid::text;

-- Add length constraints (255 chars max for identifiers)
ALTER TABLE dialogs ADD CONSTRAINT chk_object_id_length CHECK (length(object_id) <= 255);
ALTER TABLE dialogs ADD CONSTRAINT chk_created_by_length CHECK (created_by IS NULL OR length(created_by) <= 255);
ALTER TABLE dialog_participants ADD CONSTRAINT chk_user_id_length CHECK (length(user_id) <= 255);
ALTER TABLE messages ADD CONSTRAINT chk_sender_id_length CHECK (sender_id IS NULL OR length(sender_id) <= 255);
ALTER TABLE dialog_access_scopes ADD CONSTRAINT chk_tenant_uid_length CHECK (length(tenant_uid) <= 255);
