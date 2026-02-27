-- Thumbnail storage path for image files (generated on upload)
ALTER TABLE files ADD COLUMN thumbnail_path TEXT;
-- Track if EXIF metadata was stripped from the image
ALTER TABLE files ADD COLUMN exif_stripped BOOLEAN NOT NULL DEFAULT FALSE;
