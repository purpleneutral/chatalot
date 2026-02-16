-- Additional indexes for queries that lack index coverage

-- group_invites: list invites per group
CREATE INDEX IF NOT EXISTS idx_group_invites_group ON group_invites(group_id);

-- dm_pairs: PK is (user_a, user_b) which covers user_a lookups but not user_b
CREATE INDEX IF NOT EXISTS idx_dm_pairs_user_b ON dm_pairs(user_b);

-- scheduled_messages: list by user
CREATE INDEX IF NOT EXISTS idx_scheduled_messages_user ON scheduled_messages(user_id);

-- signed_prekeys: fetch latest by user (UNIQUE(user_id, key_id) exists but not ordered by created_at)
CREATE INDEX IF NOT EXISTS idx_signed_prekeys_user_latest ON signed_prekeys(user_id, created_at DESC);
