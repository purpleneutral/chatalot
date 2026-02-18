-- Fix remaining foreign keys referencing users that lacked ON DELETE actions.
-- Without these, deleting a user would fail with FK constraint violations.

-- files.uploader_id: preserve file record, clear uploader reference
ALTER TABLE files DROP CONSTRAINT files_uploader_id_fkey;
ALTER TABLE files ALTER COLUMN uploader_id DROP NOT NULL;
ALTER TABLE files ADD CONSTRAINT files_uploader_id_fkey
    FOREIGN KEY (uploader_id) REFERENCES users(id) ON DELETE SET NULL;

-- announcements.created_by: preserve announcement, clear author
ALTER TABLE announcements DROP CONSTRAINT announcements_created_by_fkey;
ALTER TABLE announcements ADD CONSTRAINT announcements_created_by_fkey
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE SET NULL;

-- custom_emojis.uploaded_by: preserve emoji, clear uploader
ALTER TABLE custom_emojis DROP CONSTRAINT custom_emojis_uploaded_by_fkey;
ALTER TABLE custom_emojis ADD CONSTRAINT custom_emojis_uploaded_by_fkey
    FOREIGN KEY (uploaded_by) REFERENCES users(id) ON DELETE SET NULL;

-- dm_pairs: delete the DM pair when either user is deleted
ALTER TABLE dm_pairs DROP CONSTRAINT dm_pairs_user_a_fkey;
ALTER TABLE dm_pairs ADD CONSTRAINT dm_pairs_user_a_fkey
    FOREIGN KEY (user_a) REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE dm_pairs DROP CONSTRAINT dm_pairs_user_b_fkey;
ALTER TABLE dm_pairs ADD CONSTRAINT dm_pairs_user_b_fkey
    FOREIGN KEY (user_b) REFERENCES users(id) ON DELETE CASCADE;

-- groups.owner_id: preserve group, clear owner (can be reassigned)
ALTER TABLE groups DROP CONSTRAINT groups_owner_id_fkey;
ALTER TABLE groups ADD CONSTRAINT groups_owner_id_fkey
    FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE SET NULL;

-- pinned_messages.pinned_by: preserve pin, clear who pinned it
ALTER TABLE pinned_messages DROP CONSTRAINT pinned_messages_pinned_by_fkey;
ALTER TABLE pinned_messages ADD CONSTRAINT pinned_messages_pinned_by_fkey
    FOREIGN KEY (pinned_by) REFERENCES users(id) ON DELETE SET NULL;

-- polls.created_by: preserve poll, clear creator
ALTER TABLE polls DROP CONSTRAINT polls_created_by_fkey;
ALTER TABLE polls ADD CONSTRAINT polls_created_by_fkey
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE SET NULL;

-- reports.reviewed_by: preserve report, clear reviewer
ALTER TABLE reports DROP CONSTRAINT reports_reviewed_by_fkey;
ALTER TABLE reports ADD CONSTRAINT reports_reviewed_by_fkey
    FOREIGN KEY (reviewed_by) REFERENCES users(id) ON DELETE SET NULL;

-- user_timeouts.issued_by: preserve timeout record, clear issuer
ALTER TABLE user_timeouts DROP CONSTRAINT user_timeouts_issued_by_fkey;
ALTER TABLE user_timeouts ADD CONSTRAINT user_timeouts_issued_by_fkey
    FOREIGN KEY (issued_by) REFERENCES users(id) ON DELETE SET NULL;

-- warnings.issued_by: preserve warning record, clear issuer
ALTER TABLE warnings DROP CONSTRAINT warnings_issued_by_fkey;
ALTER TABLE warnings ADD CONSTRAINT warnings_issued_by_fkey
    FOREIGN KEY (issued_by) REFERENCES users(id) ON DELETE SET NULL;

-- webhooks.created_by: preserve webhook, clear creator
ALTER TABLE webhooks DROP CONSTRAINT webhooks_created_by_fkey;
ALTER TABLE webhooks ADD CONSTRAINT webhooks_created_by_fkey
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE SET NULL;

-- communities.owner_id: preserve community, clear owner (can be reassigned)
ALTER TABLE communities DROP CONSTRAINT communities_owner_id_fkey;
ALTER TABLE communities ADD CONSTRAINT communities_owner_id_fkey
    FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE SET NULL;
