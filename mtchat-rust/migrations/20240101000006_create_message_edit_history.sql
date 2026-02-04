-- Create message edit history table
CREATE TABLE message_edit_history (
    id UUID PRIMARY KEY,
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    old_content TEXT NOT NULL,
    edited_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for fetching edit history by message
CREATE INDEX idx_message_edit_history_message_id ON message_edit_history(message_id, edited_at DESC);
