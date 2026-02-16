-- Thread support: group replies under a root message
ALTER TABLE messages ADD COLUMN thread_id UUID REFERENCES messages(id);
CREATE INDEX idx_messages_thread ON messages(thread_id, created_at ASC) WHERE thread_id IS NOT NULL;
