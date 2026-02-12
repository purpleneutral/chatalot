CREATE TABLE group_invites (
    id          UUID PRIMARY KEY,
    group_id    UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    created_by  UUID NOT NULL REFERENCES users(id),
    code        VARCHAR(12) NOT NULL UNIQUE,
    max_uses    INT,
    used_count  INT NOT NULL DEFAULT 0,
    expires_at  TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_group_invites_code ON group_invites(code);
