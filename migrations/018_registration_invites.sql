-- Registration invite codes (server-wide, distinct from group invites)
CREATE TABLE registration_invites (
    id UUID PRIMARY KEY,
    code VARCHAR(32) NOT NULL UNIQUE,
    created_by UUID NOT NULL REFERENCES users(id),
    max_uses INT,
    used_count INT NOT NULL DEFAULT 0,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_registration_invites_code ON registration_invites(code);
