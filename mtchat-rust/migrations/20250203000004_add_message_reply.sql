-- Migration: Add reply support to messages

-- Add reply_to_id column
ALTER TABLE messages ADD COLUMN reply_to_id UUID REFERENCES messages(id) ON DELETE SET NULL;

-- Index for finding replies to a message
CREATE INDEX idx_messages_reply_to ON messages(reply_to_id) WHERE reply_to_id IS NOT NULL;

-- Comment
COMMENT ON COLUMN messages.reply_to_id IS 'Reference to the message this is a reply to';
