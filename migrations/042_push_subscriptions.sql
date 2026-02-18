-- Web Push notification subscriptions (one per browser/device per user)
CREATE TABLE push_subscriptions (
    id            UUID PRIMARY KEY,
    user_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    endpoint      TEXT NOT NULL,
    p256dh_key    TEXT NOT NULL,
    auth_key      TEXT NOT NULL,
    user_agent    TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used     TIMESTAMPTZ,
    failure_count INT NOT NULL DEFAULT 0,
    UNIQUE (user_id, endpoint)
);

CREATE INDEX idx_push_subs_user ON push_subscriptions(user_id);
