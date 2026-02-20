# API Reference

All REST API endpoints are served under `/api`. Authentication is via Bearer JWT in the `Authorization` header unless noted otherwise.

## Authentication

Public endpoints (no auth required).

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/auth/register` | Register a new account |
| `POST` | `/auth/login` | Log in with username + password |
| `POST` | `/auth/refresh` | Refresh an access token |
| `POST` | `/auth/recover` | Recover account via recovery code |
| `GET` | `/auth/config` | Get server configuration (registration mode, public URL) |

### POST /auth/register

```json
{
  "username": "alice",
  "display_name": "Alice",
  "email": "alice@example.com",
  "password": "SecureP@ss1",
  "identity_key": [/* 32 bytes */],
  "signed_prekey": {
    "key_id": 1,
    "public_key": [/* 32 bytes */],
    "signature": [/* 64 bytes */]
  },
  "one_time_prekeys": [
    { "key_id": 1, "public_key": [/* 32 bytes */] }
  ],
  "invite_code": "ABC123"
}
```

Response: `AuthResponse` with `access_token`, `refresh_token`, `user`, and `recovery_code`.

### POST /auth/login

```json
{
  "username": "alice",
  "password": "SecureP@ss1",
  "totp_code": "123456"
}
```

The `totp_code` field is required only when TOTP 2FA is enabled. TOTP backup codes are also accepted in this field.

### POST /auth/refresh

```json
{
  "refresh_token": "hex-encoded-token"
}
```

Returns new `access_token` and `refresh_token` (old refresh token is revoked on use -- rotation).

### POST /auth/recover

```json
{
  "username": "alice",
  "recovery_code": "ABCD-EFGH-JKLM-NPQR",
  "new_password": "NewSecureP@ss1"
}
```

Resets the password, revokes all refresh tokens, and returns a new `recovery_code`.

---

## Users

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/users/search?q=alice` | Search users by username |
| `GET` | `/users/{user_id}` | Get a user's public profile |
| `POST` | `/users/block` | Block a user |
| `POST` | `/users/unblock/{user_id}` | Unblock a user |
| `GET` | `/users/blocked` | List blocked users |
| `POST` | `/reports` | Report a user or content |

User visibility requires shared community membership (or instance owner status).

---

## Account

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/account/me` | Get own profile (public, no auth for initial load) |
| `PATCH` | `/account/profile` | Update display name, avatar, bio, pronouns, etc. |
| `POST` | `/account/change-password` | Change password (requires current password) |
| `GET` | `/account/sessions` | List active sessions (device, IP, last used) |
| `DELETE` | `/account/sessions/{id}` | Revoke a specific session |
| `POST` | `/account/sessions/revoke-all` | Revoke all sessions except current |
| `DELETE` | `/account/delete` | Delete own account |

---

## Communities

Communities are the top-level organizational unit. They contain groups, which contain channels.

### Public Routes (auth required, no community gate)

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/communities` | List communities the caller belongs to |
| `POST` | `/communities` | Create a new community (admin-only or open, per config) |
| `GET` | `/community-invites/{code}` | Get invite info (community name, member count) |
| `POST` | `/community-invites/{code}/accept` | Accept a community invite |
| `GET` | `/emojis/{id}` | Serve custom emoji image |
| `GET` | `/community-assets/{filename}` | Serve community icon/banner |

### Gated Routes (require community membership)

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/communities/{cid}` | Get community details |
| `PATCH` | `/communities/{cid}` | Update community (name, description, theme, welcome message) |
| `DELETE` | `/communities/{cid}` | Delete community (owner only) |
| `POST` | `/communities/{cid}/transfer-ownership` | Transfer ownership |
| `POST` | `/communities/{cid}/leave` | Leave community |
| `GET` | `/communities/{cid}/members` | List community members |
| `PUT` | `/communities/{cid}/members/{uid}/role` | Set member role |
| `PUT` | `/communities/{cid}/members/{uid}/nickname` | Set member nickname |
| `DELETE` | `/communities/{cid}/members/{uid}` | Kick member |
| `GET` | `/communities/{cid}/bans` | List banned users |
| `POST` | `/communities/{cid}/bans/{uid}` | Ban user |
| `DELETE` | `/communities/{cid}/bans/{uid}` | Unban user |
| `GET` | `/communities/{cid}/invites` | List community invites |
| `POST` | `/communities/{cid}/invites` | Create community invite |
| `DELETE` | `/communities/{cid}/invites/{iid}` | Delete invite |
| `GET` | `/communities/{cid}/groups` | List groups in community |
| `POST` | `/communities/{cid}/channels/{chid}/timeout` | Timeout a user |
| `DELETE` | `/communities/{cid}/channels/{chid}/timeout/{uid}` | Remove timeout |
| `POST` | `/communities/{cid}/channels/{chid}/warn` | Warn a user |
| `GET` | `/communities/{cid}/channels/{chid}/warnings/{uid}` | List warnings |
| `POST` | `/communities/{cid}/icon` | Upload community icon |
| `POST` | `/communities/{cid}/banner` | Upload community banner |
| `GET` | `/communities/{cid}/emojis` | List custom emojis |
| `POST` | `/communities/{cid}/emojis` | Upload custom emoji (max 50 per community) |
| `DELETE` | `/communities/{cid}/emojis/{eid}` | Delete custom emoji |

---

## Groups

Groups are containers for channels within a community.

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/groups` | List groups the caller belongs to |
| `POST` | `/groups` | Create a group (max 200 per community) |
| `GET` | `/groups/discover` | Discover joinable groups |
| `GET` | `/groups/{id}` | Get group details |
| `PATCH` | `/groups/{id}` | Update group (name, description, visibility, accent color) |
| `DELETE` | `/groups/{id}` | Delete group |
| `POST` | `/groups/{id}/transfer-ownership` | Transfer group ownership |
| `POST` | `/groups/{id}/join` | Join a discoverable group |
| `POST` | `/groups/{id}/leave` | Leave group |
| `GET` | `/groups/{id}/members` | List group members |
| `GET` | `/groups/{id}/channels` | List channels in group |
| `POST` | `/groups/{id}/channels` | Create channel in group (max 100 per group) |
| `PATCH` | `/groups/{id}/channels/{cid}` | Update channel in group |
| `DELETE` | `/groups/{id}/channels/{cid}` | Delete channel in group |
| `GET` | `/groups/{id}/invites` | List group invites |
| `POST` | `/groups/{id}/invites` | Create group invite |
| `GET` | `/group-invites/{code}` | Get invite info |
| `POST` | `/group-invites/{code}/accept` | Accept group invite |
| `POST` | `/groups/{id}/icon` | Upload group icon |
| `POST` | `/groups/{id}/banner` | Upload group banner |
| `POST` | `/groups/{id}/voice-background` | Upload voice channel background |

---

## Channels

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/channels` | List channels the caller belongs to |
| `POST` | `/channels` | Create a standalone channel |
| `GET` | `/channels/{id}` | Get channel details |
| `PATCH` | `/channels/{id}` | Update channel (name, topic, slow mode, read-only, archived) |
| `POST` | `/channels/{id}/join` | Join channel |
| `POST` | `/channels/{id}/leave` | Leave channel |
| `GET` | `/channels/{id}/members` | List channel members |
| `PATCH` | `/channels/{id}/members/{user_id}/role` | Update member role |
| `POST` | `/channels/{id}/members/{user_id}/kick` | Kick member |
| `POST` | `/channels/{id}/members/{user_id}/ban` | Ban member |
| `POST` | `/channels/{id}/members/{user_id}/unban` | Unban member |
| `POST` | `/channels/{id}/transfer-ownership` | Transfer channel ownership |
| `GET` | `/channels/unread` | Get unread counts for all channels |
| `GET` | `/channels/{id}/read-cursors` | Get read cursors for channel members |

---

## Messages

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/channels/{id}/messages?before=uuid&limit=50` | Get messages (paginated) |
| `GET` | `/channels/{id}/messages/search?q=hello` | Search messages in channel |
| `GET` | `/messages/search?q=hello` | Global search across all accessible channels |
| `GET` | `/channels/{id}/messages/{msg_id}/history` | Get edit history |
| `GET` | `/channels/{id}/threads/{msg_id}` | Get thread messages |
| `GET` | `/channels/{id}/pins` | List pinned messages (max 50 per channel) |
| `POST` | `/channels/{id}/pins/{msg_id}` | Pin a message |
| `DELETE` | `/channels/{id}/pins/{msg_id}` | Unpin a message |

Search supports filters: `sender`, `before`, `after`, `has_file`.

---

## Direct Messages

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/dms` | List DM channels |
| `POST` | `/dms` | Create DM channel with another user |

DMs require shared community membership. Blocked users cannot initiate DMs.

---

## Files

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/files/upload` | Upload file (multipart, max 100 MB default) |
| `GET` | `/files/{file_id}` | Download file |
| `DELETE` | `/files/{file_id}` | Delete own file |
| `GET` | `/files/{file_id}/meta` | Get file metadata (name, size, MIME type) |

Files are encrypted client-side before upload. The server stores opaque ciphertext blobs. Per-user upload quota defaults to 500 MB. Files are stored in a sharded directory structure on disk.

---

## E2E Encryption Keys

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/keys/{user_id}/bundle` | Fetch a user's key bundle for X3DH |
| `POST` | `/keys/prekeys/signed` | Upload/rotate signed prekey |
| `POST` | `/keys/prekeys/one-time` | Upload batch of one-time prekeys |
| `GET` | `/keys/prekeys/count` | Get remaining one-time prekey count |

### Sender Keys (Group E2E)

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/channels/{id}/sender-keys` | Upload sender key distribution |
| `GET` | `/channels/{id}/sender-keys` | Get sender keys for channel |

---

## TOTP (Two-Factor Authentication)

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/totp/setup` | Generate TOTP secret and QR code URI |
| `POST` | `/totp/verify` | Verify TOTP code to enable 2FA |
| `POST` | `/totp/disable` | Disable 2FA (requires password) |
| `GET` | `/totp/backup-codes` | Get remaining backup code count |

---

## Webhooks

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/channels/{id}/webhooks` | Create webhook for channel |
| `GET` | `/channels/{id}/webhooks` | List webhooks for channel |
| `PATCH` | `/webhooks/{id}` | Update webhook (name, avatar) |
| `DELETE` | `/webhooks/{id}` | Delete webhook |
| `POST` | `/webhooks/execute/{token}` | Execute webhook (public, no auth) |

---

## Polls

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/channels/{id}/polls` | Create a poll |
| `POST` | `/polls/{id}/vote` | Vote on a poll option |
| `POST` | `/polls/{id}/close` | Close a poll (creator or admin) |
| `GET` | `/channels/{id}/polls` | List polls in channel |

---

## Scheduled Messages

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/messages/schedule` | Schedule a message for future delivery |
| `GET` | `/messages/scheduled` | List own scheduled messages |
| `DELETE` | `/messages/scheduled/{id}` | Cancel a scheduled message |

---

## Bookmarks

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/bookmarks/{message_id}` | Bookmark a message |
| `DELETE` | `/bookmarks/{message_id}` | Remove bookmark |
| `GET` | `/bookmarks` | List bookmarked messages |

---

## Announcements

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/announcements` | List announcements |
| `POST` | `/announcements/{id}/dismiss` | Dismiss an announcement |

---

## Miscellaneous

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/feedback` | Submit feedback (creates GitHub issue if configured) |
| `GET` | `/gifs/search?q=hello` | Search GIFs (Giphy API proxy) |
| `GET` | `/gifs/trending` | Get trending GIFs |
| `POST` | `/link-preview` | Fetch link preview metadata (title, description, image) |

---

## Admin

All admin endpoints require `is_admin` or `is_owner` in the JWT claims.

### User Management

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/admin/users?q=search&page=1` | List/search users |
| `POST` | `/admin/users/{id}/suspend` | Suspend user (with reason) |
| `POST` | `/admin/users/{id}/unsuspend` | Unsuspend user |
| `DELETE` | `/admin/users/{id}` | Delete user account |
| `PUT` | `/admin/users/{id}/admin` | Grant/revoke admin role |
| `PUT` | `/admin/users/{id}/password` | Admin password reset |

### Registration Invites

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/admin/invites` | List registration invite codes |
| `POST` | `/admin/invites` | Create registration invite code |
| `DELETE` | `/admin/invites/{id}` | Delete invite code |

### Purge

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/admin/purge/message/{id}` | Hard-delete a specific message |
| `POST` | `/admin/purge/user/{id}/messages` | Hard-delete all messages from a user |
| `POST` | `/admin/purge/channel/{id}` | Hard-delete all messages in a channel |

### File Management

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/admin/files` | List all files (with search, pagination) |
| `DELETE` | `/admin/files/{id}` | Delete a file |
| `POST` | `/admin/files/{id}/quarantine` | Quarantine a file |
| `POST` | `/admin/files/{id}/unquarantine` | Unquarantine a file |
| `GET` | `/admin/storage-stats` | Get storage statistics |

### Content Moderation

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/admin/messages/{id}/quarantine` | Quarantine a message |
| `POST` | `/admin/messages/{id}/unquarantine` | Unquarantine a message |
| `GET` | `/admin/blocked-hashes` | List blocked file hashes |
| `POST` | `/admin/blocked-hashes` | Add a blocked file hash |
| `DELETE` | `/admin/blocked-hashes/{id}` | Remove a blocked file hash |

### Audit and Reports

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/admin/audit-log?action=login&user_id=uuid` | Query audit log |
| `GET` | `/admin/reports` | List user/content reports |
| `POST` | `/admin/reports/{id}/review` | Review (resolve/dismiss) a report |

### Announcements (Admin)

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/admin/announcements` | Create instance-wide announcement |
| `GET` | `/admin/announcements` | List all announcements |

---

## Health

Public endpoint (no auth required).

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/health` | Server health check |

Response:

```json
{
  "status": "ok",
  "version": "0.17.0",
  "uptime_secs": 3600,
  "db_healthy": true
}
```

---

## Error Responses

All errors follow a consistent format:

```json
{
  "error": "description of the error"
}
```

Common HTTP status codes:

| Code | Meaning |
|------|---------|
| `400` | Validation error (bad input) |
| `401` | Unauthorized (missing or invalid token) |
| `403` | Forbidden (insufficient permissions) |
| `404` | Not found |
| `409` | Conflict (duplicate username, email, etc.) |
| `500` | Internal server error |

---

## Rate Limiting

- **Auth endpoints** (`/auth/*`): Stricter rate limiting via `auth_rate_limit_middleware`
- **General API**: Token bucket rate limiter applied to all routes
- **WebSocket messages**: 10 messages/second burst, 5/second sustained refill
- **Body size limit**: 110 MB (slightly above the default 100 MB file upload limit)
