-- Messages (server stores only ciphertext)
CREATE TABLE messages (
    id              UUID PRIMARY KEY,
    channel_id      UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    sender_id       UUID NOT NULL REFERENCES users(id),
    ciphertext      BYTEA NOT NULL,
    nonce           BYTEA NOT NULL,
    message_type    VARCHAR(16) NOT NULL DEFAULT 'text',
    sender_key_id   UUID,
    reply_to_id     UUID REFERENCES messages(id),
    edited_at       TIMESTAMPTZ,
    deleted_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_messages_channel ON messages(channel_id, created_at DESC);
CREATE INDEX idx_messages_sender ON messages(sender_id, created_at DESC);
