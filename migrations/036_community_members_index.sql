-- Index for listing community members by community_id (used by auto-join and member listing)
CREATE INDEX IF NOT EXISTS idx_community_members_community ON community_members(community_id);
