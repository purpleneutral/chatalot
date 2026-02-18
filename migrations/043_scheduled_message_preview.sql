-- Add encrypted content preview to scheduled messages (client-side encrypted, server stores opaque blob)
ALTER TABLE scheduled_messages ADD COLUMN content_preview TEXT;
