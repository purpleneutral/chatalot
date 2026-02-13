-- Server-side user preferences (syncs across devices).
-- The preferences column stores a JSONB blob; the server is opaque to its contents.
CREATE TABLE user_preferences (
    user_id      UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    preferences  JSONB NOT NULL DEFAULT '{}',
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
