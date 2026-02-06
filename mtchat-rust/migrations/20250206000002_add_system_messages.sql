-- Add system messages support
-- System messages have message_type = 'system' and sender_id = NULL

-- Add message_type column (default 'user' for existing messages)
ALTER TABLE messages ADD COLUMN message_type VARCHAR(20) NOT NULL DEFAULT 'user';

-- Make sender_id nullable for system messages
ALTER TABLE messages ALTER COLUMN sender_id DROP NOT NULL;

-- Add check constraint for valid message types
ALTER TABLE messages ADD CONSTRAINT chk_message_type
    CHECK (message_type IN ('user', 'system'));

-- System messages must have NULL sender_id, user messages must have sender_id
ALTER TABLE messages ADD CONSTRAINT chk_system_sender
    CHECK ((message_type = 'system' AND sender_id IS NULL) OR (message_type = 'user' AND sender_id IS NOT NULL));
