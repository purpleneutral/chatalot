-- Identity keys for E2E encryption
CREATE TABLE identity_keys (
    user_id         UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    identity_key    BYTEA NOT NULL,
    fingerprint     VARCHAR(64) NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    rotated_at      TIMESTAMPTZ
);
