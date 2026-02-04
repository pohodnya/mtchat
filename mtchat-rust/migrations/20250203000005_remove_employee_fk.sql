-- Migration: Remove FK constraints to employees table
-- In the new architecture, user_id/sender_id are external identifiers, not references to employees table

-- Remove FK from dialog_participants
ALTER TABLE dialog_participants DROP CONSTRAINT IF EXISTS participants_employee_id_fkey;

-- Remove FK from messages (sender_id no longer references employees)
ALTER TABLE messages DROP CONSTRAINT IF EXISTS messages_sender_id_fkey;

-- Add comments explaining the change
COMMENT ON COLUMN dialog_participants.user_id IS 'External user identifier (from JWT token), not a reference to employees table';
COMMENT ON COLUMN messages.sender_id IS 'External user identifier (from JWT token), not a reference to employees table';
