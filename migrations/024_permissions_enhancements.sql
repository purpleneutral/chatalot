-- Group visibility (fixes bug #9: any community member could join any group)
ALTER TABLE groups ADD COLUMN visibility VARCHAR(16) NOT NULL DEFAULT 'public';

-- Channel settings
ALTER TABLE channels ADD COLUMN read_only BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE channels ADD COLUMN slow_mode_seconds INTEGER NOT NULL DEFAULT 0;

-- Community-level policies
ALTER TABLE communities ADD COLUMN who_can_create_groups VARCHAR(16) NOT NULL DEFAULT 'admin';
ALTER TABLE communities ADD COLUMN who_can_create_invites VARCHAR(16) NOT NULL DEFAULT 'admin';

-- Slow mode tracking (per-user per-channel last message timestamp)
CREATE TABLE channel_slowmode_tracker (
    channel_id UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    last_sent  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (channel_id, user_id)
);
