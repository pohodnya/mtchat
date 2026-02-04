-- Create messages table
-- Note: id uses UUIDv7 which is time-ordered for efficient sorting
CREATE TABLE messages (
    id UUID PRIMARY KEY,
    dialog_id UUID NOT NULL REFERENCES dialogs(id) ON DELETE CASCADE,
    sender_id UUID NOT NULL REFERENCES employees(id) ON DELETE RESTRICT,
    content TEXT NOT NULL,
    sent_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_edited_at TIMESTAMPTZ
);

-- Index for fetching messages by dialog (with ordering)
CREATE INDEX idx_messages_dialog_id ON messages(dialog_id, id DESC);

-- Index for fetching messages by sender
CREATE INDEX idx_messages_sender_id ON messages(sender_id);

-- Add foreign key from participants.last_read_message_id to messages.id
ALTER TABLE participants
ADD CONSTRAINT fk_participants_last_read_message
FOREIGN KEY (last_read_message_id) REFERENCES messages(id) ON DELETE SET NULL;
