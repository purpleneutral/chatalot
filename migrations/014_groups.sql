-- Groups: named communities containing channels
CREATE TABLE groups (
    id          UUID PRIMARY KEY,
    name        VARCHAR(64) NOT NULL,
    description TEXT,
    owner_id    UUID NOT NULL REFERENCES users(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE group_members (
    group_id    UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role        VARCHAR(16) NOT NULL DEFAULT 'member',
    joined_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (group_id, user_id)
);

-- Link channels to groups (nullable for standalone channels)
ALTER TABLE channels ADD COLUMN group_id UUID REFERENCES groups(id) ON DELETE CASCADE;
