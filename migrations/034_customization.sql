-- Profile banners
ALTER TABLE users ADD COLUMN banner_url TEXT;

-- Community theming
ALTER TABLE communities ADD COLUMN community_theme JSONB;
ALTER TABLE communities ADD COLUMN banner_url TEXT;
ALTER TABLE communities ADD COLUMN welcome_message TEXT;

-- Group customization
ALTER TABLE groups ADD COLUMN icon_url TEXT;
ALTER TABLE groups ADD COLUMN banner_url TEXT;
ALTER TABLE groups ADD COLUMN accent_color VARCHAR(7);

-- Voice channel backgrounds
ALTER TABLE channels ADD COLUMN voice_background TEXT;
