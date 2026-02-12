-- X3DH signed prekeys
CREATE TABLE signed_prekeys (
    id              UUID PRIMARY KEY,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key_id          INTEGER NOT NULL,
    public_key      BYTEA NOT NULL,
    signature       BYTEA NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, key_id)
);

-- X3DH one-time prekeys
CREATE TABLE one_time_prekeys (
    id              UUID PRIMARY KEY,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key_id          INTEGER NOT NULL,
    public_key      BYTEA NOT NULL,
    used            BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, key_id)
);

CREATE INDEX idx_otp_available ON one_time_prekeys(user_id, used) WHERE NOT used;
