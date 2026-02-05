-- Add participant profile fields to dialog_participants
-- Allows storing display name, company, email, and phone for each participant

ALTER TABLE dialog_participants
  ADD COLUMN display_name VARCHAR(255),
  ADD COLUMN company VARCHAR(255),
  ADD COLUMN email VARCHAR(255),
  ADD COLUMN phone VARCHAR(50);

-- Add comments for documentation
COMMENT ON COLUMN dialog_participants.display_name IS 'Display name (full name, initials, or anonymous)';
COMMENT ON COLUMN dialog_participants.company IS 'Company/organization name';
COMMENT ON COLUMN dialog_participants.email IS 'Contact email (optional, can be hidden)';
COMMENT ON COLUMN dialog_participants.phone IS 'Contact phone (optional, can be hidden)';
