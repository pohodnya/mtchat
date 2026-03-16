-- Add pg_trgm extension and GIN indexes for efficient ILIKE searches
-- This improves performance of dialog title and participant company search

-- Enable pg_trgm extension (requires superuser or granted privileges)
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- GIN index on dialogs.title for efficient ILIKE search
CREATE INDEX IF NOT EXISTS idx_dialogs_title_trgm
ON dialogs USING GIN (title gin_trgm_ops);

-- GIN index on dialog_participants.company for efficient ILIKE search
CREATE INDEX IF NOT EXISTS idx_participants_company_trgm
ON dialog_participants USING GIN (company gin_trgm_ops);
