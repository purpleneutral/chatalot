-- Account recovery codes (self-service password reset without email)
ALTER TABLE users ADD COLUMN recovery_code_hash TEXT;

-- TOTP 2FA backup codes (one-time use, hashed)
ALTER TABLE users ADD COLUMN totp_backup_codes TEXT[];
