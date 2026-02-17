# Database Schema

> **Status: Complete**

Chatalot uses PostgreSQL as its primary data store. The schema is managed through 39 sequential SQL migration files in the `migrations/` directory. Migrations run automatically on server startup.

> **Note:** Chatalot uses sqlx with runtime-checked queries (not compile-time macros). Set `SQLX_OFFLINE=true` for builds without a live database.

## Entity Relationship Overview

```
Instance
├── users (authentication, profiles)
│   ├── identity_keys (E2E encryption)
│   ├── signed_prekeys / one_time_prekeys
│   ├── refresh_tokens (sessions)
│   ├── user_preferences (JSONB settings)
│   └── user_blocks
├── communities
│   ├── community_members (with roles)
│   ├── community_invites
│   ├── community_bans
│   ├── custom_emojis
│   └── groups
│       ├── group_members (with roles)
│       └── channels
│           ├── channel_members (with roles)
│           ├── channel_bans
│           ├── messages
│           │   ├── message_edits (edit history)
│           │   ├── reactions
│           │   ├── pinned_messages
│           │   └── files (attachments)
│           ├── webhooks
│           ├── polls → poll_votes
│           ├── voice_sessions → voice_session_participants
│           ├── read_cursors
│           ├── sender_key_distributions (E2E)
│           ├── scheduled_messages
│           ├── user_timeouts
│           └── warnings
├── dm_pairs (DM channel mapping)
├── reports (content/user reports)
├── audit_log (admin activity)
├── announcements → announcement_dismissals
├── bookmarks (personal saved messages)
└── blocked_hashes (file security)
```

---

## Core Tables

### `users`

The primary user table. Stores authentication credentials, profile information, and admin flags.

| Column | Type | Notes |
|--------|------|-------|
| `id` | `UUID` PK | |
| `username` | `VARCHAR(32)` UNIQUE | 3-32 chars, letters/numbers/underscores/hyphens/dots |
| `display_name` | `VARCHAR(64)` | |
| `email` | `VARCHAR(255)` UNIQUE | |
| `password_hash` | `TEXT` | Argon2id |
| `avatar_url` | `TEXT` | |
| `banner_url` | `TEXT` | |
| `status` | `VARCHAR(16)` | online/idle/dnd/invisible/offline |
| `custom_status` | `VARCHAR(128)` | |
| `bio` | `TEXT` | |
| `pronouns` | `VARCHAR(50)` | |
| `totp_secret` | `BYTEA` | Encrypted if `TOTP_ENCRYPTION_KEY` set |
| `totp_enabled` | `BOOLEAN` | |
| `totp_backup_codes` | `TEXT[]` | SHA-256 hashed |
| `is_admin` | `BOOLEAN` | Instance admin |
| `is_owner` | `BOOLEAN` | Instance owner (god role) |
| `suspended_at` | `TIMESTAMPTZ` | |
| `suspended_reason` | `TEXT` | |
| `recovery_code_hash` | `TEXT` | |
| `upload_bytes_used` | `BIGINT` | File quota tracking |
| `created_at` | `TIMESTAMPTZ` | |
| `updated_at` | `TIMESTAMPTZ` | |

### `identity_keys`

Ed25519 identity keys for E2E encryption. One per user.

| Column | Type | Notes |
|--------|------|-------|
| `user_id` | `UUID` PK FK→users | CASCADE delete |
| `identity_key` | `BYTEA` | 32 bytes Ed25519 public key |
| `fingerprint` | `VARCHAR(64)` | SHA-256 hex |
| `created_at` | `TIMESTAMPTZ` | |
| `rotated_at` | `TIMESTAMPTZ` | |

### `signed_prekeys`

X3DH signed prekeys (X25519 public key + Ed25519 signature).

| Column | Type | Notes |
|--------|------|-------|
| `id` | `UUID` PK | |
| `user_id` | `UUID` FK→users | |
| `key_id` | `INTEGER` | UNIQUE with `user_id` |
| `public_key` | `BYTEA` | 32 bytes X25519 |
| `signature` | `BYTEA` | 64 bytes Ed25519 |
| `created_at` | `TIMESTAMPTZ` | |

### `one_time_prekeys`

X3DH one-time prekeys (consumed on use).

| Column | Type | Notes |
|--------|------|-------|
| `id` | `UUID` PK | |
| `user_id` | `UUID` FK→users | |
| `key_id` | `INTEGER` | UNIQUE with `user_id` |
| `public_key` | `BYTEA` | 32 bytes X25519 |
| `used` | `BOOLEAN` | DEFAULT FALSE |
| `created_at` | `TIMESTAMPTZ` | |

Index: `(user_id, used) WHERE NOT used`

### `refresh_tokens`

JWT refresh tokens. SHA-256 hashed for storage, rotated on use.

| Column | Type | Notes |
|--------|------|-------|
| `id` | `UUID` PK | |
| `user_id` | `UUID` FK→users | |
| `token_hash` | `BYTEA` UNIQUE | SHA-256 of token |
| `device_name` | `VARCHAR(128)` | Parsed from User-Agent |
| `ip_address` | `TEXT` | |
| `expires_at` | `TIMESTAMPTZ` | 30 days |
| `created_at` | `TIMESTAMPTZ` | |
| `revoked_at` | `TIMESTAMPTZ` | Soft revoke for rotation |

---

## Community Tables

### `communities`

Top-level organizational containers.

| Column | Type | Notes |
|--------|------|-------|
| `id` | `UUID` PK | |
| `name` | `VARCHAR(64)` | |
| `description` | `TEXT` | |
| `icon_url` | `TEXT` | |
| `banner_url` | `TEXT` | |
| `accent_color` | `VARCHAR(7)` | Hex color |
| `owner_id` | `UUID` FK→users | |
| `discoverable` | `BOOLEAN` | |
| `community_theme` | `JSONB` | |
| `welcome_message` | `TEXT` | |
| `who_can_create_groups` | `VARCHAR(16)` | DEFAULT 'admin' |
| `who_can_create_invites` | `VARCHAR(16)` | DEFAULT 'admin' |
| `warn_escalation` | `JSONB` | |
| `created_at` | `TIMESTAMPTZ` | |
| `updated_at` | `TIMESTAMPTZ` | |

### `community_members`

Community membership with roles.

| Column | Type | Notes |
|--------|------|-------|
| `community_id` | `UUID` FK→communities | Composite PK |
| `user_id` | `UUID` FK→users | Composite PK |
| `role` | `VARCHAR(16)` | DEFAULT 'member' |
| `nickname` | `VARCHAR(64)` | Community-specific nickname |
| `joined_at` | `TIMESTAMPTZ` | |

### `community_invites`

Invite codes for joining a community.

| Column | Type | Notes |
|--------|------|-------|
| `id` | `UUID` PK | |
| `community_id` | `UUID` FK→communities | |
| `code` | `VARCHAR(16)` UNIQUE | |
| `created_by` | `UUID` FK→users | |
| `max_uses` | `INTEGER` | NULL = unlimited |
| `used_count` | `INTEGER` | |
| `expires_at` | `TIMESTAMPTZ` | |
| `created_at` | `TIMESTAMPTZ` | |

### `community_bans`

Banned users per community.

| Column | Type | Notes |
|--------|------|-------|
| `community_id` | `UUID` FK→communities | Composite PK |
| `user_id` | `UUID` FK→users | Composite PK |
| `banned_by` | `UUID` FK→users | |
| `reason` | `TEXT` | |
| `created_at` | `TIMESTAMPTZ` | |

---

## Group Tables

### `groups`

Groups within a community, containing channels.

| Column | Type | Notes |
|--------|------|-------|
| `id` | `UUID` PK | |
| `name` | `VARCHAR(64)` | |
| `description` | `TEXT` | |
| `community_id` | `UUID` FK→communities | NOT NULL |
| `owner_id` | `UUID` FK→users | |
| `created_at` | `TIMESTAMPTZ` | |
| `updated_at` | `TIMESTAMPTZ` | |

### `group_members`

Group membership with roles.

| Column | Type | Notes |
|--------|------|-------|
| `group_id` | `UUID` FK→groups | Composite PK |
| `user_id` | `UUID` FK→users | Composite PK |
| `role` | `VARCHAR(16)` | DEFAULT 'member' |
| `joined_at` | `TIMESTAMPTZ` | |

---

## Channel Tables

### `channels`

Text, voice, and DM channels.

| Column | Type | Notes |
|--------|------|-------|
| `id` | `UUID` PK | |
| `name` | `VARCHAR(64)` | |
| `channel_type` | ENUM | `text`, `voice`, `dm` |
| `group_id` | `UUID` FK→groups | Nullable |
| `topic` | `TEXT` | |
| `read_only` | `BOOLEAN` | |
| `slow_mode_seconds` | `INTEGER` | 0 = off |
| `archived` | `BOOLEAN` | |
| `message_ttl_seconds` | `INTEGER` | Auto-delete after N seconds |
| `voice_background` | `TEXT` | Voice channel background URL |
| `created_by` | `UUID` FK→users | |
| `created_at` | `TIMESTAMPTZ` | |
| `updated_at` | `TIMESTAMPTZ` | |

### `channel_members`

Channel membership with roles.

| Column | Type | Notes |
|--------|------|-------|
| `channel_id` | `UUID` FK→channels | Composite PK |
| `user_id` | `UUID` FK→users | Composite PK |
| `role` | `VARCHAR(16)` | DEFAULT 'member' |
| `joined_at` | `TIMESTAMPTZ` | |

### `channel_bans`

Banned users per channel.

| Column | Type | Notes |
|--------|------|-------|
| `channel_id` | `UUID` FK→channels | Composite PK |
| `user_id` | `UUID` FK→users | Composite PK |
| `banned_by` | `UUID` FK→users | |
| `reason` | `TEXT` | |
| `banned_at` | `TIMESTAMPTZ` | |

### `channel_slowmode_tracker`

Tracks when users last sent a message in slow-mode channels.

| Column | Type | Notes |
|--------|------|-------|
| `channel_id` | `UUID` FK→channels | Composite PK |
| `user_id` | `UUID` FK→users | Composite PK |
| `last_sent` | `TIMESTAMPTZ` | |

---

## Messaging Tables

### `messages`

Encrypted messages. Supports text, file, system, and webhook message types.

| Column | Type | Notes |
|--------|------|-------|
| `id` | `UUID` PK | |
| `channel_id` | `UUID` FK→channels | |
| `sender_id` | `UUID` FK→users | |
| `ciphertext` | `BYTEA` | 64 KiB max |
| `nonce` | `BYTEA` | |
| `message_type` | `VARCHAR(16)` | text/file/system/webhook |
| `sender_key_id` | `UUID` FK→sender_key_distributions | Nullable |
| `reply_to_id` | `UUID` FK→messages | Nullable |
| `thread_id` | `UUID` FK→messages | Nullable |
| `plaintext` | `TEXT` | |
| `expires_at` | `TIMESTAMPTZ` | Auto-delete (message TTL) |
| `edited_at` | `TIMESTAMPTZ` | |
| `deleted_at` | `TIMESTAMPTZ` | Soft delete |
| `quarantined_at` | `TIMESTAMPTZ` | |
| `quarantined_by` | `UUID` FK→users | |
| `created_at` | `TIMESTAMPTZ` | |

Indexes: `(channel_id, created_at DESC)`, `(sender_id, created_at DESC)`

### `message_edits`

Tracks message edit history. Stores the previous ciphertext before each edit.

| Column | Type | Notes |
|--------|------|-------|
| `id` | `UUID` PK | |
| `message_id` | `UUID` FK→messages | |
| `old_ciphertext` | `BYTEA` | |
| `old_nonce` | `BYTEA` | |
| `edited_at` | `TIMESTAMPTZ` | |

### `reactions`

Emoji reactions on messages.

| Column | Type | Notes |
|--------|------|-------|
| `id` | `UUID` PK | |
| `message_id` | `UUID` FK→messages | |
| `user_id` | `UUID` FK→users | |
| `emoji` | `VARCHAR(32)` | |
| `created_at` | `TIMESTAMPTZ` | |

UNIQUE: `(message_id, user_id, emoji)`

### `pinned_messages`

Pinned messages within a channel.

| Column | Type | Notes |
|--------|------|-------|
| `message_id` | `UUID` PK FK→messages | |
| `channel_id` | `UUID` FK→channels | |
| `pinned_by` | `UUID` FK→users | |
| `pinned_at` | `TIMESTAMPTZ` | |

Index: `(channel_id, pinned_at DESC)`

### `read_cursors`

Per-user unread message tracking.

| Column | Type | Notes |
|--------|------|-------|
| `user_id` | `UUID` FK→users | Composite PK |
| `channel_id` | `UUID` FK→channels | Composite PK |
| `last_read_message_id` | `UUID` FK→messages | |
| `last_read_at` | `TIMESTAMPTZ` | |

---

## File Tables

### `files`

Uploaded file records.

| Column | Type | Notes |
|--------|------|-------|
| `id` | `UUID` PK | |
| `uploader_id` | `UUID` FK→users | |
| `encrypted_name` | `VARCHAR(512)` | |
| `size_bytes` | `BIGINT` | |
| `content_type` | `VARCHAR(128)` | |
| `storage_path` | `TEXT` | |
| `checksum` | `VARCHAR(128)` | |
| `channel_id` | `UUID` FK→channels | Nullable |
| `quarantined_at` | `TIMESTAMPTZ` | |
| `quarantined_by` | `UUID` FK→users | |
| `created_at` | `TIMESTAMPTZ` | |

### `blocked_hashes`

SHA-256 hashes of blocked file content. Prevents re-upload of banned files.

| Column | Type | Notes |
|--------|------|-------|
| `id` | `UUID` PK | |
| `hash` | `VARCHAR(128)` UNIQUE | |
| `reason` | `TEXT` | |
| `blocked_by` | `UUID` FK→users | |
| `created_at` | `TIMESTAMPTZ` | |

---

## DM Tables

### `dm_pairs`

Maps direct message conversations to their channel. Pair is stored with `user_a < user_b` to ensure uniqueness.

| Column | Type | Notes |
|--------|------|-------|
| `user_a` | `UUID` FK→users | Composite PK (lower UUID) |
| `user_b` | `UUID` FK→users | Composite PK (higher UUID) |
| `channel_id` | `UUID` FK→channels | UNIQUE |
| `created_at` | `TIMESTAMPTZ` | |

---

## Voice Tables

### `voice_sessions`

Tracks voice/video sessions.

| Column | Type | Notes |
|--------|------|-------|
| `id` | `UUID` PK | |
| `channel_id` | `UUID` FK→channels | |
| `started_by` | `UUID` FK→users | |
| `started_at` | `TIMESTAMPTZ` | |
| `ended_at` | `TIMESTAMPTZ` | NULL while active |

### `voice_session_participants`

Tracks participants within a voice session.

| Column | Type | Notes |
|--------|------|-------|
| `session_id` | `UUID` FK→voice_sessions | Composite PK |
| `user_id` | `UUID` FK→users | Composite PK |
| `joined_at` | `TIMESTAMPTZ` | |
| `left_at` | `TIMESTAMPTZ` | NULL while connected |

---

## Encryption Tables

### `sender_key_distributions`

Sender keys for group E2E encryption (Signal Sender Keys protocol).

| Column | Type | Notes |
|--------|------|-------|
| `id` | `UUID` PK | |
| `channel_id` | `UUID` FK→channels | |
| `user_id` | `UUID` FK→users | |
| `chain_id` | `INTEGER` | |
| `distribution` | `JSONB` | Serialized SenderKeyDistribution |
| `created_at` | `TIMESTAMPTZ` | |

UNIQUE: `(channel_id, user_id)`

---

## Webhook Tables

### `webhooks`

Channel webhooks for external integrations.

| Column | Type | Notes |
|--------|------|-------|
| `id` | `UUID` PK | |
| `channel_id` | `UUID` FK→channels | |
| `name` | `VARCHAR(64)` | |
| `token` | `VARCHAR(128)` UNIQUE | |
| `created_by` | `UUID` FK→users | |
| `avatar_url` | `TEXT` | |
| `active` | `BOOLEAN` | |
| `created_at` | `TIMESTAMPTZ` | |

---

## Moderation Tables

### `user_timeouts`

Temporary mutes within a channel.

| Column | Type | Notes |
|--------|------|-------|
| `id` | `UUID` PK | |
| `user_id` | `UUID` FK→users | |
| `channel_id` | `UUID` FK→channels | |
| `issued_by` | `UUID` FK→users | |
| `reason` | `TEXT` | |
| `expires_at` | `TIMESTAMPTZ` | |
| `created_at` | `TIMESTAMPTZ` | |

### `warnings`

Formal warnings issued to users.

| Column | Type | Notes |
|--------|------|-------|
| `id` | `UUID` PK | |
| `user_id` | `UUID` FK→users | |
| `channel_id` | `UUID` FK→channels | |
| `issued_by` | `UUID` FK→users | |
| `reason` | `TEXT` | |
| `created_at` | `TIMESTAMPTZ` | |

### `reports`

Content and user reports.

| Column | Type | Notes |
|--------|------|-------|
| `id` | `UUID` PK | |
| `reporter_id` | `UUID` FK→users | |
| `report_type` | `VARCHAR(32)` | |
| `target_id` | `UUID` | Generic target reference |
| `reason` | `TEXT` | |
| `status` | `VARCHAR(32)` | pending/reviewed/resolved/dismissed |
| `reviewed_by` | `UUID` FK→users | Nullable |
| `reviewed_at` | `TIMESTAMPTZ` | |
| `admin_notes` | `TEXT` | |
| `created_at` | `TIMESTAMPTZ` | |

### `user_blocks`

Per-user block list.

| Column | Type | Notes |
|--------|------|-------|
| `blocker_id` | `UUID` FK→users | Composite PK |
| `blocked_id` | `UUID` FK→users | Composite PK |
| `created_at` | `TIMESTAMPTZ` | |

---

## Poll Tables

### `polls`

In-channel polls.

| Column | Type | Notes |
|--------|------|-------|
| `id` | `UUID` PK | |
| `channel_id` | `UUID` FK→channels | |
| `created_by` | `UUID` FK→users | |
| `question` | `TEXT` | |
| `options` | `JSONB` | |
| `multi_select` | `BOOLEAN` | |
| `anonymous` | `BOOLEAN` | |
| `closed` | `BOOLEAN` | |
| `expires_at` | `TIMESTAMPTZ` | |
| `created_at` | `TIMESTAMPTZ` | |

### `poll_votes`

Individual votes on a poll.

| Column | Type | Notes |
|--------|------|-------|
| `id` | `UUID` PK | |
| `poll_id` | `UUID` FK→polls | |
| `user_id` | `UUID` FK→users | |
| `option_index` | `INTEGER` | |
| `created_at` | `TIMESTAMPTZ` | |

UNIQUE: `(poll_id, user_id, option_index)`

---

## Scheduled Messages

### `scheduled_messages`

Messages scheduled for future delivery.

| Column | Type | Notes |
|--------|------|-------|
| `id` | `UUID` PK | |
| `channel_id` | `UUID` FK→channels | |
| `user_id` | `UUID` FK→users | |
| `ciphertext` | `TEXT` | |
| `nonce` | `TEXT` | |
| `scheduled_for` | `TIMESTAMPTZ` | |
| `created_at` | `TIMESTAMPTZ` | |

---

## Bookmark Tables

### `bookmarks`

Per-user saved messages.

| Column | Type | Notes |
|--------|------|-------|
| `id` | `UUID` PK | |
| `user_id` | `UUID` FK→users | |
| `message_id` | `UUID` FK→messages | |
| `note` | `TEXT` | |
| `created_at` | `TIMESTAMPTZ` | |

UNIQUE: `(user_id, message_id)`

---

## Custom Emoji Tables

### `custom_emojis`

Community custom emojis.

| Column | Type | Notes |
|--------|------|-------|
| `id` | `UUID` PK | |
| `community_id` | `UUID` FK→communities | |
| `uploaded_by` | `UUID` FK→users | |
| `shortcode` | `VARCHAR(32)` | |
| `file_path` | `TEXT` | |
| `content_type` | `VARCHAR(64)` | |
| `created_at` | `TIMESTAMPTZ` | |

UNIQUE: `(community_id, shortcode)`

---

## Announcement Tables

### `announcements`

Instance-wide announcements from admins.

| Column | Type | Notes |
|--------|------|-------|
| `id` | `UUID` PK | |
| `title` | `VARCHAR(200)` | |
| `body` | `TEXT` | |
| `created_by` | `UUID` FK→users | |
| `created_at` | `TIMESTAMPTZ` | |

### `announcement_dismissals`

Tracks which users have dismissed an announcement.

| Column | Type | Notes |
|--------|------|-------|
| `user_id` | `UUID` FK→users | Composite PK |
| `announcement_id` | `UUID` FK→announcements | Composite PK |
| `dismissed_at` | `TIMESTAMPTZ` | |

---

## Preference Tables

### `user_preferences`

Per-user preference storage.

| Column | Type | Notes |
|--------|------|-------|
| `user_id` | `UUID` PK FK→users | |
| `preferences` | `JSONB` | DEFAULT '{}' |
| `updated_at` | `TIMESTAMPTZ` | |

---

## Administrative Tables

### `audit_log`

Records administrative and security-relevant events.

| Column | Type | Notes |
|--------|------|-------|
| `id` | `UUID` PK | |
| `user_id` | `UUID` FK→users | Nullable |
| `action` | `VARCHAR(64)` | |
| `ip_address` | `TEXT` | |
| `user_agent` | `TEXT` | |
| `metadata` | `JSONB` | |
| `created_at` | `TIMESTAMPTZ` | |

---

## Migrations

Migrations are in `/migrations/` numbered 001 through 039. They run automatically on server startup via sqlx. Key migrations:

| Migration | Description |
|-----------|-------------|
| 001 | `users` table |
| 002 | `identity_keys` (E2E) |
| 003 | `refresh_tokens` |
| 004 | `audit_log` |
| 005 | Prekeys (signed + one-time) |
| 006 | `channels` + `channel_members` |
| 007 | `messages` |
| 008 | `files` |
| 009 | `dm_pairs` |
| 010 | `voice_sessions` |
| 011 | `reactions` |
| 012 | Unread tracking (`read_cursors`) |
| 013 | `channel_bans` |
| 014 | `groups` + `group_members` |
| 019 | `communities` + `community_members` / `community_invites` / `community_bans` |
| 020 | `user_preferences` |
| 021 | `pinned_messages` |
| 022 | `sender_key_distributions` |
| 024 | Permissions (slow mode, `voice_background`) |
| 026 | Security suite (`blocked_hashes`) |
| 028 | Blocking and reports |
| 029 | New features (`webhooks`, `timeouts`, `warnings`, `scheduled_messages`, `polls`, `bookmarks`, `custom_emojis`, `announcements`) |
| 034 | Community customization |
| 038 | `message_edits` |

---

## Related Pages

- [Architecture](./architecture.md)
- [API Reference](./api-reference.md)
- [Authentication](./authentication.md)
