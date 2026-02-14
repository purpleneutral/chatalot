-- Sender key distributions for group E2E encryption.
-- Each user stores one distribution per channel. Other members fetch it
-- to initialise their ReceiverKeyState for that sender.
CREATE TABLE sender_key_distributions (
    id              UUID PRIMARY KEY,
    channel_id      UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    chain_id        INTEGER NOT NULL,
    distribution    JSONB NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (channel_id, user_id)
);

CREATE INDEX idx_sender_key_dist_channel ON sender_key_distributions(channel_id);
