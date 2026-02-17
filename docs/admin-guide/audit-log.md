# Audit Log

> **Status: Complete**

The Audit Log tab provides a chronological record of administrative actions, security events, and user activity on the instance.

> **Permission Required:** Instance Admin or Instance Owner

## Overview

Every significant action in Chatalot is recorded in the audit log. This includes admin operations (suspending users, deleting content), authentication events (logins, failed attempts), and system actions. The log is stored in PostgreSQL and is queryable through the admin panel.

## Log Table

The audit log is displayed as a paginated table with the following columns:

| Column | Description |
|--------|-------------|
| **Time** | Timestamp of the event (localized to the browser's timezone) |
| **Action** | Machine-readable action identifier (monospace, e.g., `admin_suspend_user`) |
| **User** | Truncated user ID of who performed the action (click to copy), or "system" for automated events |
| **IP** | IP address of the request, if captured |
| **Details** | JSON metadata with additional context (e.g., target user ID, file ID, report status) |

## Filtering

Two filter inputs are available in the top-right corner of the Audit Log section:

| Filter | Description |
|--------|-------------|
| **Action** | Text input to filter by action name (exact match, e.g., `admin_suspend_user`) |
| **User ID** | Text input to filter by the UUID of the user who performed the action |

After entering filter values, click the **Filter** button to apply them. Clear the fields and click **Filter** again to reset.

## Pagination

Audit log entries are displayed 50 per page. Pagination controls at the bottom show the current page, total pages, and total entry count, with **Prev** and **Next** buttons.

## Logged Actions

### Admin Actions

| Action | Description |
|--------|-------------|
| `admin_suspend_user` | An admin suspended a user account |
| `admin_unsuspend_user` | An admin unsuspended a user account |
| `admin_delete_user` | An admin deleted a user account |
| `admin_grant_admin` | An admin promoted a user to instance admin |
| `admin_revoke_admin` | An admin revoked a user's admin status |
| `admin_reset_password` | An admin reset a user's password |

### Content Moderation Actions

| Action | Description |
|--------|-------------|
| `admin_purge_message` | An admin permanently deleted a single message |
| `admin_purge_user_messages` | An admin purged all messages and files from a user |
| `admin_purge_channel` | An admin purged all messages and files in a channel |
| `admin_quarantine_file` | An admin quarantined a file |
| `admin_unquarantine_file` | An admin unquarantined a file |
| `admin_quarantine_message` | An admin quarantined a message |
| `admin_unquarantine_message` | An admin unquarantined a message |
| `admin_delete_file` | An admin permanently deleted a file |
| `admin_block_hash` | An admin added a file hash to the blocklist |
| `admin_unblock_hash` | An admin removed a file hash from the blocklist |

### Report Actions

| Action | Description |
|--------|-------------|
| `report_reviewed` | An admin reviewed a user report |

### Authentication Events

| Action | Description |
|--------|-------------|
| `register` | A new user registered an account |
| `login` | A user logged in successfully |
| `login_failed` | A login attempt failed (wrong password) |
| `login_failed_2fa` | A login attempt failed (wrong 2FA code) |
| `login_backup_code_used` | A user logged in using a TOTP backup code |

## Metadata

Each audit entry may include a `metadata` field containing a JSON object with additional context. For example:

- **User actions:** `{ "target_user_id": "..." }`
- **Purge actions:** `{ "messages_deleted": 42, "files_deleted": 5, "hashes_blocked": 3 }`
- **Report reviews:** `{ "report_id": "...", "new_status": "resolved" }`
- **File actions:** `{ "file_id": "...", "uploader_id": "...", "checksum": "..." }`

## Log Retention

Audit log entries are stored indefinitely in the database. There is currently no automatic pruning or retention policy. For instances with high activity, consider implementing periodic database maintenance to archive or remove old entries.

## Export

There is no built-in export feature in the admin UI. To export audit log data, query the `audit_log` table directly from PostgreSQL:

```sql
-- Export all entries from the last 30 days
COPY (
  SELECT * FROM audit_log
  WHERE created_at > NOW() - INTERVAL '30 days'
  ORDER BY created_at DESC
) TO '/tmp/audit_log_export.csv' WITH CSV HEADER;
```

## API Reference

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/admin/audit-log` | GET | Query audit log with optional `action`, `user_id`, `page`, `per_page` params |

## Next Step

Continue to [Security Settings](./security-settings.md) to learn about purge tools, quarantine, and the file hash blocklist.
