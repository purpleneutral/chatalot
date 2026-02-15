-- Group discoverability
ALTER TABLE groups ADD COLUMN discoverable BOOLEAN NOT NULL DEFAULT TRUE;
