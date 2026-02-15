-- Account lockout tracking (in-memory, not DB — this migration only adds quota tracking)

-- Per-user file upload quota tracking
ALTER TABLE users ADD COLUMN upload_bytes_used BIGINT NOT NULL DEFAULT 0;

-- Index for quota lookups
CREATE INDEX idx_users_upload_bytes ON users(upload_bytes_used DESC);
