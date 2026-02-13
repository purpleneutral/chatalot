-- Communities: top-level organizational units containing groups and channels.
-- Hierarchy: Instance → Communities → Groups → Channels

CREATE TABLE communities (
    id          UUID PRIMARY KEY,
    name        VARCHAR(64) NOT NULL,
    description TEXT,
    icon_url    TEXT,
    owner_id    UUID NOT NULL REFERENCES users(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Community membership with roles and optional per-community nickname
CREATE TABLE community_members (
    community_id UUID NOT NULL REFERENCES communities(id) ON DELETE CASCADE,
    user_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role         VARCHAR(16) NOT NULL DEFAULT 'member',
    nickname     VARCHAR(64),
    joined_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (community_id, user_id)
);
CREATE INDEX idx_community_members_user ON community_members(user_id);

-- Invite codes for joining a community
CREATE TABLE community_invites (
    id           UUID PRIMARY KEY,
    community_id UUID NOT NULL REFERENCES communities(id) ON DELETE CASCADE,
    code         VARCHAR(16) NOT NULL UNIQUE,
    created_by   UUID NOT NULL REFERENCES users(id),
    max_uses     INT,
    used_count   INT NOT NULL DEFAULT 0,
    expires_at   TIMESTAMPTZ,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Community-level bans
CREATE TABLE community_bans (
    community_id UUID NOT NULL REFERENCES communities(id) ON DELETE CASCADE,
    user_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    banned_by    UUID NOT NULL REFERENCES users(id),
    reason       TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (community_id, user_id)
);

-- Link groups to communities (initially nullable for migration)
ALTER TABLE groups ADD COLUMN community_id UUID REFERENCES communities(id) ON DELETE CASCADE;

-- Migrate existing data: create a default community and move all groups under it.
-- The community name and member roles can be customized after migration via the API
-- or by updating the database directly.
DO $$
DECLARE
    community_uuid UUID;
    owner_id UUID;
BEGIN
    -- Pick the first instance admin as community owner (fallback: first user)
    SELECT id INTO owner_id FROM users WHERE is_admin = true
        ORDER BY created_at LIMIT 1;
    IF owner_id IS NULL THEN
        SELECT id INTO owner_id FROM users ORDER BY created_at LIMIT 1;
    END IF;

    -- Only create default community if there are existing users
    IF owner_id IS NOT NULL THEN
        community_uuid := gen_random_uuid();

        INSERT INTO communities (id, name, description, owner_id)
        VALUES (community_uuid, 'General', 'Default community', owner_id);

        -- Move all existing groups under the default community
        UPDATE groups SET community_id = community_uuid WHERE community_id IS NULL;

        -- Add all existing users as community members
        INSERT INTO community_members (community_id, user_id, role)
        SELECT community_uuid, u.id,
            CASE WHEN u.id = owner_id THEN 'owner'
                 WHEN u.is_admin THEN 'admin'
                 ELSE 'member'
            END
        FROM users u;
    END IF;
END $$;

-- Now make community_id NOT NULL (all groups have been migrated)
ALTER TABLE groups ALTER COLUMN community_id SET NOT NULL;
