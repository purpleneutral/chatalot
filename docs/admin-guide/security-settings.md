# Security Settings

> **Status: Complete**

The Security tab provides powerful content moderation tools: purge operations for permanent content deletion, message quarantine, and a file hash blocklist to prevent re-uploads of prohibited content.

> **Permission Required:** Instance Admin or Instance Owner

## Purge Tools

The Purge Tools section allows you to permanently hard-delete messages and files from the database and disk. This section is visually highlighted with a red border to indicate its destructive nature.

> **Warning:** Purge operations are irreversible. Messages are hard-deleted from the database and files are removed from disk. This cannot be undone.

### Purge Targets

Select the target type from the dropdown:

| Target Type | Description |
|-------------|-------------|
| **Single Message** | Delete one specific message by its UUID |
| **All from User** | Delete ALL messages and ALL uploaded files for a given user |
| **All in Channel** | Delete ALL messages and ALL files associated with a given channel |

### How to Purge

1. Select the **Target Type** from the dropdown.
2. Enter the UUID of the target (message, user, or channel) in the ID field.
3. Optionally check **Block file hashes** to automatically add the SHA-256 hashes of any associated files to the blocklist, preventing the same file content from being re-uploaded.
4. Click **Purge**.
5. A confirmation dialog will appear describing exactly what will be deleted. Confirm to proceed.

### Purge Results

After a purge completes, a toast notification displays the results:

- **Messages deleted** -- Number of messages permanently removed
- **Files deleted** -- Number of files removed from disk
- **Hashes blocked** -- Number of file hashes added to the blocklist (if the option was checked)

### Notes on Single Message Purge

When purging a single message, files are **not** automatically deleted. This is because message content is end-to-end encrypted and the server cannot determine which files a message references. Use the [File Management](./file-management.md) tab to delete specific files separately.

### Audit Logging

All purge operations are recorded in the [Audit Log](./audit-log.md):

| Action | Metadata |
|--------|----------|
| `admin_purge_message` | message_id, channel_id, block_hashes |
| `admin_purge_user_messages` | target_user_id, messages_deleted, files_deleted, hashes_blocked |
| `admin_purge_channel` | channel_id, messages_deleted, files_deleted, hashes_blocked |

## Quick Quarantine

The Quick Quarantine section provides a fast way to hide a message from view without deleting it. Quarantined messages are preserved in the database but are not visible to users. This is useful for preserving evidence while removing offensive content from view.

### How to Quarantine a Message

1. Enter the message UUID in the **Message ID** field.
2. Click **Quarantine** to hide the message, or **Unquarantine** to restore a previously quarantined message.

> **Tip:** For file quarantine, use the [File Management](./file-management.md) tab instead.

Quarantine actions are recorded in the audit log as `admin_quarantine_message` and `admin_unquarantine_message`.

## Blocked File Hashes

The Blocked File Hashes section maintains a list of SHA-256 file hashes that are rejected on upload. If any user attempts to upload a file whose checksum matches a blocked hash, the upload is denied.

### Viewing Blocked Hashes

The hash list displays a table with:

| Column | Description |
|--------|-------------|
| **Hash** | Truncated SHA-256 hash (click to copy the full 64-character hex string) |
| **Reason** | Optional reason for blocking (up to 500 characters) |
| **Added** | Date the hash was added |
| **Actions** | Remove button |

### Adding a Hash Manually

1. Enter the 64-character hex SHA-256 hash in the **SHA256 Hash** field.
2. Optionally enter a **Reason** describing why the hash is blocked.
3. Click **Block**.

The hash must be exactly 64 hexadecimal characters. If a hash is already blocked, the reason will be updated.

### Automatic Hash Blocking

Hashes can also be blocked automatically during:

- **File deletion** -- When deleting a file from the Files tab, you can opt to block its hash.
- **Purge operations** -- When running a purge with the "Block file hashes" option checked, all associated file hashes are added to the blocklist.

Automatic blocks are labeled with the reason "auto-blocked via admin purge" or "blocked via admin file delete".

### Removing a Hash

Click **Remove** next to any blocked hash to unblock it. A confirmation dialog is shown before removal. After unblocking, users can upload files matching that hash again.

### Audit Logging

Hash operations are recorded in the audit log as `admin_block_hash` and `admin_unblock_hash`.

## OIDC / Single Sign-On

Chatalot supports external authentication through any OpenID Connect provider (Authentik, Keycloak, Authelia, Zitadel, etc.). When OIDC is configured, users see a **Log in with SSO** button on the login page.

### Enabling OIDC

Set the following environment variables (see [Configuration](../self-hosting/configuration.md#oidc--sso) for full details):

| Variable | Description |
|----------|-------------|
| `OIDC_ISSUER_URL` | Provider discovery URL (must expose `/.well-known/openid-configuration`) |
| `OIDC_CLIENT_ID` | Client ID from your identity provider |
| `OIDC_CLIENT_SECRET` | Client secret from your identity provider |
| `OIDC_REDIRECT_URI` | Callback URL (e.g. `https://chat.example.com/auth/oidc/callback`) |
| `OIDC_DISABLE_PASSWORD_LOGIN` | Set to `true` to enforce SSO-only login (default: `false`) |

SSO is active when all three of `OIDC_ISSUER_URL`, `OIDC_CLIENT_ID`, and `OIDC_CLIENT_SECRET` are set.

### User Accounts

- First-time SSO users get a local account created automatically from the provider's `email` and `preferred_username` claims.
- OIDC users appear in the admin Users list like any other user. You can suspend, promote, or delete them normally.
- If `OIDC_DISABLE_PASSWORD_LOGIN=true`, the email/password login form is hidden and all users must authenticate through the identity provider.
- Invite-only mode still applies: first-time SSO users need a valid invite code unless their account was pre-created by an admin.

### Security Considerations

- OIDC login is subject to the same rate limiting and account lockout rules as password login.
- The OIDC client secret should be treated with the same care as any other credential. Store it in `.env` or Docker secrets, never in version control.
- Disabling password login (`OIDC_DISABLE_PASSWORD_LOGIN=true`) is recommended for instances where all users are managed through a central identity provider.

## Instance-Wide Security Features

Beyond the Security tab, Chatalot enforces several security measures at the infrastructure level:

### Rate Limiting

All API requests are rate-limited using a token-bucket algorithm per IP address:

| Scope | Rate | Burst |
|-------|------|-------|
| General API | 20 requests/second | 50 burst |
| Auth endpoints (login/register) | 5 requests/second | 10 burst |

When the rate limit is exceeded, the server returns HTTP 429 with the message "too many requests, please slow down."

### Account Lockout

After **10 consecutive failed login attempts**, the account is locked for **15 minutes**. The lockout is tracked in-memory per username and resets on server restart.

### Security Headers

The server automatically applies the following HTTP security headers to all responses:

| Header | Value |
|--------|-------|
| `X-Content-Type-Options` | `nosniff` |
| `X-Frame-Options` | `DENY` |
| `X-XSS-Protection` | `1; mode=block` |
| `Strict-Transport-Security` | `max-age=31536000; includeSubDomains` |
| `Referrer-Policy` | `strict-origin-when-cross-origin` |
| `Permissions-Policy` | `camera=(self), microphone=(self), geolocation=()` |
| `Content-Security-Policy` | Restrictive policy allowing self-origin scripts, WebSocket connections, and specific media sources |

### Password Policy

All passwords (set during registration, password change, or admin reset) must meet these requirements:

- 8 to 128 characters
- At least one uppercase letter (A-Z)
- At least one lowercase letter (a-z)
- At least one digit (0-9)
- At least one special character

Passwords are hashed using **Argon2id** with the following parameters: 64 MB memory, 3 iterations, 4 parallelism, 32-byte output.

### Session Management

- JWT access tokens are signed with Ed25519 (EdDSA algorithm) and have a short lifetime.
- Refresh tokens are 32-byte random values, stored as SHA-256 hashes in the database.
- Refresh token rotation: each refresh request invalidates the old token and issues a new one.
- Suspended users have all refresh tokens revoked immediately.

## API Reference

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/admin/purge/message/{id}` | POST | Purge a single message (optional `block_hashes` query param) |
| `/admin/purge/user/{id}/messages` | POST | Purge all messages and files from a user |
| `/admin/purge/channel/{id}` | POST | Purge all messages and files in a channel |
| `/admin/messages/{id}/quarantine` | POST | Quarantine a message |
| `/admin/messages/{id}/unquarantine` | POST | Unquarantine a message |
| `/admin/blocked-hashes` | GET | List blocked hashes with `page`, `per_page` query params |
| `/admin/blocked-hashes` | POST | Add a hash (body: `{ hash, reason }`) |
| `/admin/blocked-hashes/{id}` | DELETE | Remove a hash from the blocklist |

## Next Step

Continue to [Announcements](./announcements.md) to learn about publishing server-wide announcements.
