-- Pinned messages: tracks which messages are pinned in a channel.
-- PK on message_id ensures a message can only be pinned once.
-- channel_id denormalized from message for query efficiency.
CREATE TABLE pinned_messages (
    message_id   UUID PRIMARY KEY REFERENCES messages(id) ON DELETE CASCADE,
    channel_id   UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    pinned_by    UUID NOT NULL REFERENCES users(id),
    pinned_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_pinned_messages_channel ON pinned_messages(channel_id, pinned_at DESC);
