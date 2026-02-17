# File Management

> **Status: Complete**

The Files tab provides a browser for all uploaded files on the instance, along with storage statistics, quarantine controls, and file deletion.

> **Permission Required:** Instance Admin or Instance Owner

## Storage Statistics

At the top of the Files tab, three summary cards display real-time storage information:

| Card | Description |
|------|-------------|
| **Total Files** | Number of files stored on the instance |
| **Total Storage** | Combined size of all files (displayed in B, KB, MB, or GB) |
| **Uploaders** | Number of distinct users who have uploaded files |

These statistics are loaded separately from the file list and update when files are deleted.

## File Browser

Below the statistics cards, the file browser displays a paginated table of all uploaded files.

### Table Columns

| Column | Description |
|--------|-------------|
| **ID** | Truncated file UUID (click to copy the full ID) |
| **Name** | Encrypted file name as stored (may show "--" if not available) |
| **Type** | MIME content type (e.g., `image/png`, `application/pdf`) or "unknown" |
| **Size** | Human-readable file size |
| **Uploader** | Truncated uploader user ID (click to copy) |
| **Status** | `Active` (green) or `Quarantined` (orange) |
| **Uploaded** | Upload date |
| **Actions** | Quarantine and Delete buttons |

### Sorting

Use the **Sort** dropdown in the top-right corner to order files by:

- **Newest** (default) -- Most recently uploaded first
- **Largest** -- Biggest files first

### Pagination

Files are displayed 25 per page. When there are more files than fit on one page, pagination controls appear at the bottom of the table showing the current page, total pages, and total file count, with **Prev** and **Next** buttons.

## Quarantining Files

Quarantining a file hides it from downloads while preserving it on disk for evidence. This is a reversible action -- useful when you want to investigate a file before deciding whether to delete it.

- Click **Quarantine** to hide a file from users. The file's status changes to `Quarantined` (orange).
- Click **Unquarantine** to restore a quarantined file, making it downloadable again.

Quarantine actions are recorded in the [Audit Log](./audit-log.md) with the entries `admin_quarantine_file` and `admin_unquarantine_file`.

## Deleting Files

Click **Delete** on any file to permanently remove it from both the database and disk storage.

Before deletion, you will be prompted:

1. **Block hash?** -- Whether to add the file's SHA-256 checksum to the blocked hash list, preventing anyone from re-uploading the same file content.
2. **Confirm deletion** -- Final confirmation that the file will be permanently removed.

> **Warning:** File deletion is permanent. The file is removed from disk and its database record is erased. This cannot be undone.

Deletion is recorded in the audit log as `admin_delete_file`, with metadata including the file ID, uploader ID, and checksum.

## Per-User Upload Quotas

The server enforces a per-user upload quota configured via the `UPLOAD_QUOTA_MB` environment variable (default: 500 MB). When a user exceeds their quota, further uploads are rejected. Admins can monitor per-user storage usage through the storage statistics, which include a per-user breakdown.

## API Reference

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/admin/files` | GET | List files with `page`, `per_page`, `user_id`, `sort` query params |
| `/admin/files/{id}` | DELETE | Delete a file (optional `block_hashes=true` query param) |
| `/admin/files/{id}/quarantine` | POST | Quarantine a file |
| `/admin/files/{id}/unquarantine` | POST | Unquarantine a file |
| `/admin/storage-stats` | GET | Get storage statistics with per-user breakdown |

## Next Step

Continue to [Reports and Moderation](./reports-and-moderation.md) to learn about handling user reports.
