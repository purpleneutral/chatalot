-- Allow user deletion by relaxing foreign key constraints on ownership/reference columns.
-- Tables with ON DELETE CASCADE (channel_members, group_members, refresh_tokens, identity_keys,
-- signed_prekeys, one_time_prekeys) already handle user deletion correctly.

-- channels.created_by → SET NULL (preserve channel, clear creator reference)
ALTER TABLE channels DROP CONSTRAINT channels_created_by_fkey;
ALTER TABLE channels ADD CONSTRAINT channels_created_by_fkey
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE SET NULL;

-- audit_log.user_id → SET NULL (preserve audit trail)
ALTER TABLE audit_log DROP CONSTRAINT audit_log_user_id_fkey;
ALTER TABLE audit_log ADD CONSTRAINT audit_log_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL;

-- messages.sender_id → SET NULL (preserve messages as "[deleted user]")
ALTER TABLE messages DROP CONSTRAINT messages_sender_id_fkey;
ALTER TABLE messages ADD CONSTRAINT messages_sender_id_fkey
    FOREIGN KEY (sender_id) REFERENCES users(id) ON DELETE SET NULL;
ALTER TABLE messages ALTER COLUMN sender_id DROP NOT NULL;

-- channel_bans.banned_by → SET NULL (preserve ban record)
ALTER TABLE channel_bans DROP CONSTRAINT channel_bans_banned_by_fkey;
ALTER TABLE channel_bans ADD CONSTRAINT channel_bans_banned_by_fkey
    FOREIGN KEY (banned_by) REFERENCES users(id) ON DELETE SET NULL;
ALTER TABLE channel_bans ALTER COLUMN banned_by DROP NOT NULL;
