-- Voice/video call sessions
CREATE TABLE voice_sessions (
    id              UUID PRIMARY KEY,
    channel_id      UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    started_by      UUID NOT NULL REFERENCES users(id),
    started_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at        TIMESTAMPTZ
);

CREATE TABLE voice_session_participants (
    session_id      UUID NOT NULL REFERENCES voice_sessions(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id),
    joined_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    left_at         TIMESTAMPTZ,
    PRIMARY KEY (session_id, user_id)
);

CREATE INDEX idx_voice_sessions_channel ON voice_sessions(channel_id) WHERE ended_at IS NULL;
CREATE INDEX idx_voice_participants_user ON voice_session_participants(user_id) WHERE left_at IS NULL;
