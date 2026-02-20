-- E2E encryption feature flag (admin-toggleable)
INSERT INTO instance_settings (key, value) VALUES
    ('e2e_enabled', 'true')
ON CONFLICT (key) DO NOTHING;
