-- Security Suite: blocked hashes, file/message quarantine, admin query indexes

-- Blocked file hashes (prevent re-upload of known-bad content)
CREATE TABLE blocked_hashes (
    id          UUID PRIMARY KEY,
    hash        VARCHAR(128) NOT NULL UNIQUE,
    reason      TEXT,
    blocked_by  UUID NOT NULL REFERENCES users(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_blocked_hashes_hash ON blocked_hashes(hash);

-- Quarantine columns on files (hide without deleting, preserve for evidence)
ALTER TABLE files ADD COLUMN quarantined_at TIMESTAMPTZ;
ALTER TABLE files ADD COLUMN quarantined_by UUID REFERENCES users(id);

-- Quarantine columns on messages
ALTER TABLE messages ADD COLUMN quarantined_at TIMESTAMPTZ;
ALTER TABLE messages ADD COLUMN quarantined_by UUID REFERENCES users(id);

-- Indexes for admin queries and background tasks
CREATE INDEX idx_files_quarantined ON files(quarantined_at) WHERE quarantined_at IS NOT NULL;
CREATE INDEX idx_messages_quarantined ON messages(quarantined_at) WHERE quarantined_at IS NOT NULL;
CREATE INDEX idx_files_created ON files(created_at DESC);
CREATE INDEX idx_files_size ON files(size_bytes DESC);
CREATE INDEX idx_messages_deleted ON messages(deleted_at) WHERE deleted_at IS NOT NULL;
