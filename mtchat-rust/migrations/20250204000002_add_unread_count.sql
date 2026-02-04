-- Migration: Add unread_count to dialog_participants
-- For fast unread message counting without JOINs

-- Add unread_count column
ALTER TABLE dialog_participants
ADD COLUMN unread_count INTEGER NOT NULL DEFAULT 0;

-- Partial index for fast lookups of dialogs with unread messages
CREATE INDEX idx_dialog_participants_unread
ON dialog_participants(user_id)
WHERE unread_count > 0;
