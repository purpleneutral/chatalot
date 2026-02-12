-- Site-wide admin role and account suspension support.

ALTER TABLE users ADD COLUMN is_admin BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE users ADD COLUMN suspended_at TIMESTAMPTZ;
ALTER TABLE users ADD COLUMN suspended_reason TEXT;
