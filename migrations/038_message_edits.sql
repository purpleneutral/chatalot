-- Message edit history: store old content before each edit
CREATE TABLE message_edits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    old_ciphertext BYTEA NOT NULL,
    old_nonce BYTEA NOT NULL,
    edited_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_message_edits_message_id ON message_edits(message_id);
