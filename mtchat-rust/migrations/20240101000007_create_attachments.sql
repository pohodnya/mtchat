-- Create attachments table
CREATE TABLE attachments (
    id UUID PRIMARY KEY,
    message_id UUID REFERENCES messages(id) ON DELETE CASCADE,
    uploader_id UUID NOT NULL REFERENCES employees(id) ON DELETE SET NULL,
    file_name VARCHAR(255) NOT NULL,
    file_type VARCHAR(100) NOT NULL,
    file_size BIGINT NOT NULL,
    storage_key VARCHAR(512) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for fetching attachments by message
CREATE INDEX idx_attachments_message_id ON attachments(message_id) WHERE message_id IS NOT NULL;

-- Index for fetching pending (not yet attached) files by uploader
CREATE INDEX idx_attachments_pending ON attachments(uploader_id, created_at DESC) WHERE message_id IS NULL;

-- Index for cleanup job (old pending attachments)
CREATE INDEX idx_attachments_cleanup ON attachments(created_at) WHERE message_id IS NULL;
