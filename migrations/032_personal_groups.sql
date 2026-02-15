-- Personal groups: allow moderators to assign a group to a specific member
ALTER TABLE groups ADD COLUMN assigned_member_id UUID REFERENCES users(id);
ALTER TABLE groups ADD COLUMN allow_invites BOOLEAN NOT NULL DEFAULT FALSE;
