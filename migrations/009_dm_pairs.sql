-- Track DM channel pairs for quick lookup.
-- When user A wants to DM user B, we check if a DM channel already exists.
CREATE TABLE dm_pairs (
    user_a      UUID NOT NULL REFERENCES users(id),
    user_b      UUID NOT NULL REFERENCES users(id),
    channel_id  UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Always store (min_id, max_id) to ensure uniqueness
    CONSTRAINT dm_pairs_ordered CHECK (user_a < user_b),
    PRIMARY KEY (user_a, user_b)
);

CREATE UNIQUE INDEX idx_dm_pairs_channel ON dm_pairs(channel_id);
