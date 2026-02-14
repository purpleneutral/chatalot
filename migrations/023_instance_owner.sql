-- Add instance owner (god role) column
ALTER TABLE users ADD COLUMN is_owner BOOLEAN NOT NULL DEFAULT FALSE;

-- Promote the first admin to instance owner
UPDATE users SET is_owner = TRUE
WHERE id = (SELECT id FROM users WHERE is_admin = TRUE ORDER BY created_at ASC LIMIT 1);
