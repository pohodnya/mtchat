-- Create participants table (many-to-many between employees and dialogs)
CREATE TABLE participants (
    employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    dialog_id UUID NOT NULL REFERENCES dialogs(id) ON DELETE CASCADE,
    last_read_message_id UUID,
    archived_at TIMESTAMPTZ,
    notifications_disabled_since TIMESTAMPTZ,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (employee_id, dialog_id)
);

-- Index for listing dialogs by employee
CREATE INDEX idx_participants_employee_id ON participants(employee_id);

-- Index for listing participants by dialog
CREATE INDEX idx_participants_dialog_id ON participants(dialog_id);

-- Index for filtering non-archived dialogs
CREATE INDEX idx_participants_not_archived ON participants(employee_id, dialog_id) WHERE archived_at IS NULL;
