-- Channel bans (prevents rejoining after kick)
CREATE TABLE channel_bans (
    channel_id  UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    banned_by   UUID NOT NULL REFERENCES users(id),
    reason      TEXT,
    banned_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (channel_id, user_id)
);
