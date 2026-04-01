-- OIDC/SSO support: allow passwordless accounts and track OIDC identity links.

-- Allow NULL password_hash for OIDC-only users
ALTER TABLE users ALTER COLUMN password_hash DROP NOT NULL;

-- Track which OIDC provider a user originally signed up with
ALTER TABLE users ADD COLUMN oidc_provider VARCHAR(64);
ALTER TABLE users ADD COLUMN oidc_subject TEXT;

-- Whether the user has completed E2E key registration
-- Existing users already have keys, so default to true
ALTER TABLE users ADD COLUMN keys_registered BOOLEAN NOT NULL DEFAULT true;

-- Separate table linking OIDC identities to users (supports multiple providers per user)
CREATE TABLE oidc_identities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(64) NOT NULL,
    subject TEXT NOT NULL,
    email TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(provider, subject)
);

CREATE INDEX idx_oidc_provider_subject ON oidc_identities(provider, subject);
CREATE INDEX idx_oidc_user_id ON oidc_identities(user_id);
