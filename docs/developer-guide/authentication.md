# Authentication

> **Status: Complete**

Chatalot uses a JWT-based authentication system with Ed25519 signing, Argon2id password hashing, refresh token rotation, and optional TOTP two-factor authentication.

## Overview

- Access tokens: Short-lived JWTs (15 minutes), signed with Ed25519
- Refresh tokens: Long-lived (30 days), stored as SHA-256 hashes in PostgreSQL
- Token rotation: Each refresh token use issues a new pair and revokes the old refresh token
- Password hashing: Argon2id (m=65536/64MiB, t=3, p=4, output=32 bytes)
- 2FA: TOTP with 10 backup codes

## JWT Access Token

Algorithm: EdDSA (Ed25519)
Key files: `./secrets/jwt_private.pem` (signing), `./secrets/jwt_public.pem` (verification)

### Claims

| Claim | Type | Description |
|-------|------|-------------|
| `sub` | UUID | User ID |
| `username` | String | Username |
| `is_admin` | bool | Instance admin flag |
| `is_owner` | bool | Instance owner (god role) |
| `iat` | i64 | Issued-at timestamp |
| `exp` | i64 | Expiration (iat + 900s) |
| `jti` | UUID | Unique token ID |

Lifetime: 15 minutes (900 seconds)
Clock skew tolerance: 60 seconds

## Refresh Token

- 32 bytes of cryptographic randomness (from OsRng)
- Returned to client as hex-encoded string (64 characters)
- Stored in DB as SHA-256 hash (never stored in plaintext)
- Lifetime: 30 days
- Tracks: device_name (from User-Agent), ip_address, created_at, expires_at, revoked_at

### Token Rotation

On each refresh, the old token is immediately revoked (revoked_at set) and a new pair is issued. This ensures each refresh token is single-use. If a revoked token is presented, all of that user's sessions may be invalidated (replay detection).

## Password Requirements

- Length: 8-128 characters
- Must contain: 1 uppercase, 1 lowercase, 1 digit, 1 special character
- Hashed with Argon2id before storage

## Username Rules

- 3-32 characters
- Allowed: letters, numbers, underscores, hyphens, dots
- Must start with letter or number
- Must not end with dot
- No consecutive dots

## Account Lockout

- Max failed attempts: 10
- Lockout duration: 15 minutes
- Tracked in memory (resets on server restart)
- Shared between login and recovery endpoints

## Registration Flow

1. Validate username, email, password
2. Validate cryptographic keys (identity key, signed prekey, one-time prekeys)
3. Check uniqueness (username, email)
4. Validate and consume invite code (if registration mode is invite_only)
5. Hash password with Argon2id
6. Compute identity key fingerprint (SHA-256 of public key)
7. Create user record
8. Store signed prekey and one-time prekeys (100 initial)
9. First user automatically becomes admin + owner
10. Issue access token (15 min) + refresh token (30 days)
11. Generate one-time recovery code
12. Log audit event (`register`)

## Login Flow

1. Check account lockout (10 attempts / 15 min)
2. Find user by username
3. Verify password (Argon2id constant-time comparison)
4. If TOTP enabled: verify TOTP code or backup code
5. Check account suspension (suspended_at not null → reject)
6. Clear lockout tracking on success
7. Issue new access + refresh tokens
8. Log audit event (`login` or `login_failed`)

## Refresh Flow

1. Client sends hex-encoded refresh token to `POST /auth/refresh`
2. Server SHA-256 hashes the token
3. Looks up hash in refresh_tokens table
4. Validates: not expired, not revoked, user not suspended
5. Revokes old refresh token (sets revoked_at)
6. Issues new access token (15 min) + new refresh token (30 days)
7. Returns both tokens

## TOTP Two-Factor Authentication

- Secret stored as BYTEA (optionally encrypted with TOTP_ENCRYPTION_KEY)
- 10 backup codes generated in XXXX-XXXX format
- Backup codes SHA-256 hashed before storage
- Backup codes are one-time use (removed from array on use)
- Enable via `POST /auth/totp/setup`, verify with `POST /auth/totp/verify`
- Disable via `POST /auth/totp/disable`

## Account Recovery

- Recovery code shown once at registration (XXXX-XXXX-XXXX-XXXX format, base32 charset)
- Stored as SHA-256 hash in recovery_code_hash column
- Recovery resets password, revokes ALL refresh tokens, generates new recovery code
- Rate-limited same as login (10 attempts / 15 min)

## Auth Middleware

The auth middleware:
1. Extracts Bearer token from `Authorization` header
2. Decodes JWT with Ed25519 public key (with 60s clock skew leeway)
3. Inserts JWT claims into request extensions
4. Skipped for public routes: `/auth/*`, `/health`, `/legal`

## WebSocket Authentication

WebSocket connections authenticate via the first message (not headers):
1. Client opens WS connection to `/ws`
2. Client sends `{"type": "authenticate", "token": "..."}`
3. Server validates JWT and responds with `authenticated` or `error`
4. 10-second timeout for auth message

## Session Management

- One refresh token per device/login
- Device name parsed from User-Agent
- IP from X-Forwarded-For or X-Real-IP headers
- Users can view active sessions and revoke individual or all-except-current

## Key Constants

| Constant | Value |
|----------|-------|
| ACCESS_TOKEN_LIFETIME | 900s (15 min) |
| REFRESH_TOKEN_LIFETIME | 2,592,000s (30 days) |
| LOCKOUT_MAX_ATTEMPTS | 10 |
| LOCKOUT_DURATION | 900s (15 min) |
| CLOCK_SKEW_TOLERANCE | 60s |
| INITIAL_ONE_TIME_PREKEYS | 100 |
| PREKEY_LOW_THRESHOLD | 20 |

## Related Pages

- [WebSocket Protocol](./websocket-protocol.md)
- [API Reference](./api-reference.md)
- [Database Schema](./database-schema.md)
- [Crypto Implementation](./crypto-implementation.md)
