-- Fix FK constraints for community_bans.banned_by and community_invites.created_by
-- so that deleting a user who banned/invited someone doesn't fail with a FK violation.

-- community_bans.banned_by: SET NULL on user deletion
ALTER TABLE community_bans DROP CONSTRAINT community_bans_banned_by_fkey;
ALTER TABLE community_bans ALTER COLUMN banned_by DROP NOT NULL;
ALTER TABLE community_bans ADD CONSTRAINT community_bans_banned_by_fkey
    FOREIGN KEY (banned_by) REFERENCES users(id) ON DELETE SET NULL;

-- community_invites.created_by: SET NULL on user deletion
ALTER TABLE community_invites DROP CONSTRAINT community_invites_created_by_fkey;
ALTER TABLE community_invites ALTER COLUMN created_by DROP NOT NULL;
ALTER TABLE community_invites ADD CONSTRAINT community_invites_created_by_fkey
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE SET NULL;
