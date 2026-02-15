-- Performance indexes for commonly queried columns missing indexes

-- group_members: list_user_groups queries WHERE user_id = $1
CREATE INDEX idx_group_members_user ON group_members(user_id);

-- channels: list_group_channels queries WHERE group_id = $1
CREATE INDEX idx_channels_group ON channels(group_id) WHERE group_id IS NOT NULL;

-- community_invites: lookup by code for invite join flow
CREATE INDEX idx_community_invites_code ON community_invites(code);

-- read_cursors: unread count queries join on channel_id
CREATE INDEX idx_read_cursors_channel ON read_cursors(channel_id);

-- messages: all queries filter deleted_at IS NULL, partial index avoids scanning deleted rows
CREATE INDEX idx_messages_channel_active ON messages(channel_id, created_at DESC) WHERE deleted_at IS NULL;
