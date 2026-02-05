-- Add is_archived column to dialog_participants
-- Allows users to archive dialogs individually (per-participant)

ALTER TABLE dialog_participants
ADD COLUMN is_archived BOOLEAN NOT NULL DEFAULT FALSE;

-- Index for filtering archived dialogs
CREATE INDEX idx_participants_archived
ON dialog_participants(user_id, is_archived);
