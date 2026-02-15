-- Community discoverability
ALTER TABLE communities ADD COLUMN discoverable BOOLEAN NOT NULL DEFAULT TRUE;

-- Group discoverability
ALTER TABLE groups ADD COLUMN discoverable BOOLEAN NOT NULL DEFAULT TRUE;

-- Channel discoverability
ALTER TABLE channels ADD COLUMN discoverable BOOLEAN NOT NULL DEFAULT TRUE;
