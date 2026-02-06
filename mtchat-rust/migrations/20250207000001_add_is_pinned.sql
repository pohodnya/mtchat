-- Add is_pinned column to dialog_participants
ALTER TABLE dialog_participants
ADD COLUMN is_pinned BOOLEAN NOT NULL DEFAULT FALSE;

-- Index for efficient querying (user's pinned dialogs)
CREATE INDEX idx_dialog_participants_is_pinned
ON dialog_participants(user_id, is_pinned);
