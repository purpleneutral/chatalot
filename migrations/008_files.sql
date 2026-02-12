-- Encrypted file storage metadata
CREATE TABLE files (
    id              UUID PRIMARY KEY,
    uploader_id     UUID NOT NULL REFERENCES users(id),
    encrypted_name  VARCHAR(512) NOT NULL,
    size_bytes      BIGINT NOT NULL,
    content_type    VARCHAR(128),
    storage_path    TEXT NOT NULL,
    checksum        VARCHAR(128) NOT NULL,
    channel_id      UUID REFERENCES channels(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_files_uploader ON files(uploader_id);
CREATE INDEX idx_files_channel ON files(channel_id);
