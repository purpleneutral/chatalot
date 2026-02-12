-- Users table
CREATE TABLE users (
    id              UUID PRIMARY KEY,
    username        VARCHAR(32) NOT NULL UNIQUE,
    display_name    VARCHAR(64) NOT NULL,
    email           VARCHAR(255) NOT NULL UNIQUE,
    password_hash   TEXT NOT NULL,
    avatar_url      TEXT,
    status          VARCHAR(16) NOT NULL DEFAULT 'offline',
    custom_status   VARCHAR(128),
    totp_secret     BYTEA,
    totp_enabled    BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
