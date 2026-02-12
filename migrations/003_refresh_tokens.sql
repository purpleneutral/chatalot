-- Refresh token tracking (allows revocation and rotation)
CREATE TABLE refresh_tokens (
    id              UUID PRIMARY KEY,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash      BYTEA NOT NULL UNIQUE,
    device_name     VARCHAR(128),
    ip_address      TEXT,
    expires_at      TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at      TIMESTAMPTZ
);

CREATE INDEX idx_refresh_active ON refresh_tokens(user_id) WHERE revoked_at IS NULL;
