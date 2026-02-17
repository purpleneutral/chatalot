-- Improve unread count query performance: get_all_unread_counts() joins on user_id
CREATE INDEX IF NOT EXISTS idx_read_cursors_user_channel ON read_cursors(user_id, channel_id);
