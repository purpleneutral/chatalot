-- Instance-level settings (admin-configurable key-value store)
CREATE TABLE IF NOT EXISTS instance_settings (
    key   TEXT        PRIMARY KEY,
    value TEXT        NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed defaults
INSERT INTO instance_settings (key, value) VALUES
    ('max_messages_cache', '500'),
    ('max_pins_per_channel', '50')
ON CONFLICT (key) DO NOTHING;
