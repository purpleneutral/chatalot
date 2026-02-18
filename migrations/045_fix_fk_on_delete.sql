-- Fix missing ON DELETE behavior on foreign keys that reference users(id).
-- Without these, deleting a user fails with FK violations.

-- voice_sessions.started_by: allow NULL (session history preserved)
ALTER TABLE voice_sessions ALTER COLUMN started_by DROP NOT NULL;
ALTER TABLE voice_sessions DROP CONSTRAINT voice_sessions_started_by_fkey;
ALTER TABLE voice_sessions ADD CONSTRAINT voice_sessions_started_by_fkey
    FOREIGN KEY (started_by) REFERENCES users(id) ON DELETE SET NULL;

-- voice_session_participants.user_id: CASCADE (user_id is part of PK)
ALTER TABLE voice_session_participants DROP CONSTRAINT voice_session_participants_user_id_fkey;
ALTER TABLE voice_session_participants ADD CONSTRAINT voice_session_participants_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

-- registration_invites.created_by: allow NULL (audit trail preserved)
ALTER TABLE registration_invites ALTER COLUMN created_by DROP NOT NULL;
ALTER TABLE registration_invites DROP CONSTRAINT registration_invites_created_by_fkey;
ALTER TABLE registration_invites ADD CONSTRAINT registration_invites_created_by_fkey
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE SET NULL;

-- group_invites.created_by: allow NULL (audit trail preserved)
ALTER TABLE group_invites ALTER COLUMN created_by DROP NOT NULL;
ALTER TABLE group_invites DROP CONSTRAINT group_invites_created_by_fkey;
ALTER TABLE group_invites ADD CONSTRAINT group_invites_created_by_fkey
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE SET NULL;

-- blocked_hashes.blocked_by: allow NULL (audit trail preserved)
ALTER TABLE blocked_hashes ALTER COLUMN blocked_by DROP NOT NULL;
ALTER TABLE blocked_hashes DROP CONSTRAINT blocked_hashes_blocked_by_fkey;
ALTER TABLE blocked_hashes ADD CONSTRAINT blocked_hashes_blocked_by_fkey
    FOREIGN KEY (blocked_by) REFERENCES users(id) ON DELETE SET NULL;

-- files.quarantined_by: already nullable, just add ON DELETE
ALTER TABLE files DROP CONSTRAINT IF EXISTS files_quarantined_by_fkey;
ALTER TABLE files ADD CONSTRAINT files_quarantined_by_fkey
    FOREIGN KEY (quarantined_by) REFERENCES users(id) ON DELETE SET NULL;

-- messages.quarantined_by: already nullable, just add ON DELETE
ALTER TABLE messages DROP CONSTRAINT IF EXISTS messages_quarantined_by_fkey;
ALTER TABLE messages ADD CONSTRAINT messages_quarantined_by_fkey
    FOREIGN KEY (quarantined_by) REFERENCES users(id) ON DELETE SET NULL;

-- groups.assigned_member_id: already nullable, just add ON DELETE
ALTER TABLE groups DROP CONSTRAINT IF EXISTS groups_assigned_member_id_fkey;
ALTER TABLE groups ADD CONSTRAINT groups_assigned_member_id_fkey
    FOREIGN KEY (assigned_member_id) REFERENCES users(id) ON DELETE SET NULL;
