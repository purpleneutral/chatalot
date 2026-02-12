-- Audit log for security events
CREATE TABLE audit_log (
    id              UUID PRIMARY KEY,
    user_id         UUID REFERENCES users(id),
    action          VARCHAR(64) NOT NULL,
    ip_address      TEXT,
    user_agent      TEXT,
    metadata        JSONB,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_user ON audit_log(user_id, created_at DESC);
CREATE INDEX idx_audit_action ON audit_log(action, created_at DESC);
