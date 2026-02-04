-- Migration: Update participants table for new architecture
-- Rename to dialog_participants and add joined_as column

-- Step 1: Rename table
ALTER TABLE participants RENAME TO dialog_participants;

-- Step 2: Rename employee_id to user_id for clarity
ALTER TABLE dialog_participants RENAME COLUMN employee_id TO user_id;

-- Step 3: Add joined_as column
ALTER TABLE dialog_participants ADD COLUMN joined_as VARCHAR(50) NOT NULL DEFAULT 'participant';
-- Possible values: 'creator', 'participant', 'joined'

-- Step 4: Rename notifications_disabled_since to notifications_enabled (flip logic)
ALTER TABLE dialog_participants ADD COLUMN notifications_enabled BOOLEAN NOT NULL DEFAULT TRUE;
UPDATE dialog_participants SET notifications_enabled = (notifications_disabled_since IS NULL);
ALTER TABLE dialog_participants DROP COLUMN notifications_disabled_since;

-- Step 5: Drop archived_at (will handle archiving differently if needed)
ALTER TABLE dialog_participants DROP COLUMN IF EXISTS archived_at;

-- Step 6: Recreate indexes with new names
DROP INDEX IF EXISTS idx_participants_employee_id;
DROP INDEX IF EXISTS idx_participants_dialog_id;
DROP INDEX IF EXISTS idx_participants_not_archived;

CREATE INDEX idx_dialog_participants_user ON dialog_participants(user_id);
CREATE INDEX idx_dialog_participants_dialog ON dialog_participants(dialog_id);

-- Step 7: Update foreign key constraint name
ALTER TABLE dialog_participants DROP CONSTRAINT IF EXISTS fk_participants_last_read_message;
ALTER TABLE dialog_participants
ADD CONSTRAINT fk_dialog_participants_last_read_message
FOREIGN KEY (last_read_message_id) REFERENCES messages(id) ON DELETE SET NULL;
