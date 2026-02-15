-- Channel archiving: archived channels are read-only and hidden by default
ALTER TABLE channels ADD COLUMN archived BOOLEAN NOT NULL DEFAULT FALSE;
