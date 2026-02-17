# Content Moderation

> **Status: Complete**

Beyond managing users, moderators also need tools to manage the content itself. This page covers deleting messages, the quarantine system, and using message history for moderation evidence.

## Deleting Messages

### Deleting Your Own Messages

Any user can delete their own messages. Right-click (or long-press on mobile) a message you sent, then select **Delete** from the context menu. You will be asked to confirm before the message is removed.

Own-message deletion does not require any special permissions.

### Deleting Others' Messages

> **Permission Required:** Community Admin or higher (channel role: admin or owner)

Admins and owners can delete messages from any user in channels they have access to. This is useful for removing rule-violating content, spam, or off-topic messages.

To delete another user's message:

1. Right-click (or long-press) the message.
2. Select **Delete** from the context menu.
3. Confirm the deletion.

When a moderator deletes another user's message:

- The message is **soft-deleted** (the `deleted_at` timestamp is set, but the record is preserved in the database).
- All channel members see the message disappear in real-time via WebSocket.
- The soft-deleted message is excluded from future queries and search results.
- After a configurable retention period, soft-deleted messages are permanently removed by the server's garbage collection process.

> **Note:** Moderators with the **Moderator** role cannot delete others' messages. This action requires the **Admin** or **Owner** role. If you need a Moderator to be able to manage content, promote them to Admin.

## Quarantine System

> **Permission Required:** Instance Admin or Instance Owner

The quarantine system allows instance administrators to hide messages and files without permanently deleting them. Quarantined content is preserved for evidence but hidden from regular users.

### Quarantining a Message

Instance admins can quarantine a message via the admin panel:

```
POST /api/admin/messages/{messageId}/quarantine
```

Quarantined messages:

- Are **hidden** from all channel views, search results, and thread listings.
- Are **preserved** in the database with a `quarantined_at` timestamp and the ID of the admin who quarantined it.
- Can be **restored** if the quarantine was issued in error.

### Unquarantining a Message

```
POST /api/admin/messages/{messageId}/unquarantine
```

This restores the message to normal visibility. The action is recorded in the audit log.

### Quarantining Files

Files can also be quarantined to prevent downloads while preserving the evidence:

```
POST /api/admin/files/{fileId}/quarantine
POST /api/admin/files/{fileId}/unquarantine
```

Quarantined files cannot be downloaded by any user until the quarantine is lifted.

> **Note:** The quarantine system is an **instance-level** feature, managed through the admin panel. Community moderators do not have access to quarantine tools. For community-level content removal, use the message delete feature.

## Purge Tools

> **Permission Required:** Instance Admin or Instance Owner

For severe situations (spam attacks, compromised accounts, illegal content), instance admins have access to bulk purge tools:

| Purge Action | API Endpoint | Effect |
|-------------|-------------|--------|
| **Purge single message** | `POST /api/admin/purge/message/{id}` | Hard-deletes one message permanently |
| **Purge user's messages** | `POST /api/admin/purge/user/{id}/messages` | Hard-deletes all messages from a user, plus their uploaded files |
| **Purge channel** | `POST /api/admin/purge/channel/{id}` | Hard-deletes all messages in a channel, plus associated files |

All purge endpoints support an optional `?block_hashes=true` query parameter, which adds the SHA-256 hashes of deleted files to the blocklist. This prevents the same files from being re-uploaded.

> **Warning:** Purge actions perform **hard-deletes** that cannot be undone. There is no recovery mechanism. Use purge tools only for content that must be permanently removed.

For more details, see the [Admin Guide -- Security Settings](../admin-guide/security-settings.md).

## Message Edit History

Any channel member can view the edit history of a message. This is useful for moderation because it shows what the message originally said before it was changed.

To view edit history:

1. Right-click (or long-press) an edited message (indicated by an "(edited)" label).
2. Select **Edit History** from the context menu.
3. A modal displays all previous versions of the message with timestamps.

**From the API:**

```
GET /api/channels/{channelId}/messages/{messageId}/history
```

Each entry includes:
- `old_ciphertext` -- the previous encrypted message content
- `old_nonce` -- the encryption nonce for the previous version
- `edited_at` -- when the edit was made

> **Tip:** When investigating a report, always check the edit history. Users sometimes edit messages after they are reported to hide the original content.

## File Management

### Admin File Browser

Instance admins can browse, inspect, and manage all uploaded files through the admin panel. The file browser supports:

- Filtering by uploader
- Sorting by date or size
- Viewing file metadata (size, content type, checksum, channel, quarantine status)
- Deleting individual files with optional hash blocking
- Quarantining files to hide them from download

### Hash Blocklist

Instance admins can maintain a blocklist of SHA-256 file hashes. Any file matching a blocked hash will be rejected on upload. This prevents re-upload of known-bad content.

Hashes can be blocked:

- **Manually** via the admin panel blocked hashes management section.
- **Automatically** when using purge tools with the `?block_hashes=true` flag.

For more details, see the [Admin Guide -- File Management](../admin-guide/file-management.md).

## Best Practices

- **Soft-delete first, purge only if necessary.** Standard message deletion is reversible during the retention period. Purging is permanent.
- **Use quarantine for evidence preservation.** If you need to hide content but may need it later for a report or legal matter, quarantine it instead of deleting it.
- **Check edit history before acting.** An edited message may have been more severe than what is currently visible.
- **Coordinate with instance admins.** Community moderators should escalate to instance admins when quarantine, purge, or file management is needed.

## Next Steps

- [Reports](./reports.md) -- How users report content and how admins review reports
- [Kicks and Bans](./kicks-and-bans.md) -- Removing users responsible for problematic content
- [Permissions Reference](./permissions-reference.md) -- Who can delete, quarantine, and purge content
