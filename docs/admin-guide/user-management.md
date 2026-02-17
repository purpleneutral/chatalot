# User Management

> **Status: Complete**

The Users tab lets you view, search, and manage every registered account on the instance.

> **Permission Required:** Instance Admin or Instance Owner

## Viewing Users

The user list displays a table with the following columns:

| Column | Description |
|--------|-------------|
| **User** | Avatar, display name, username, and an `admin` badge if applicable |
| **Email** | The user's registered email address |
| **Memberships** | Color-coded badges showing the user's community memberships (purple) and group memberships (blue), each with a tooltip showing the user's role |
| **Status** | `Active` (green) or `Suspended` (red) |
| **Joined** | Registration date |
| **Actions** | Action buttons (or "You" for the currently logged-in admin) |

The user count is displayed in the top-right corner of the section header (e.g., "14 users").

## Searching Users

A search bar at the top of the user list provides real-time filtering. Type a username, display name, or email address, and results update automatically after a 300ms debounce. The search is case-insensitive and matches partial strings.

The API supports pagination with a limit of up to 100 users per request (default: 50).

## Available Actions

Each user row (other than your own account) has the following action buttons:

### Suspend / Unsuspend

Suspending a user locks their account and immediately revokes all active sessions (refresh tokens). The user cannot log in while suspended.

- Click **Suspend** to suspend an active user. You will be prompted for an optional reason (up to 500 characters).
- Click **Unsuspend** to restore access to a suspended user.

> **Note:** You cannot suspend another admin. Revoke their admin status first if needed.

### Reset Password

Click **Reset PW** to set a new password for the user. You will be prompted to enter the new password, which must meet the standard password complexity requirements:

- 8 to 128 characters
- At least one uppercase letter
- At least one lowercase letter
- At least one digit
- At least one special character

Resetting a password immediately revokes all of the user's active sessions, forcing them to log in again with the new credentials.

### Grant / Revoke Admin

Click **Grant Admin** to promote a regular user to instance admin, or **Revoke Admin** to demote an existing admin back to a regular user. A confirmation dialog is shown before the change is applied.

> **Note:** You cannot change your own admin status. Another admin must do it.

### Delete User

Click **Delete** to permanently remove a user account. A confirmation dialog is shown before deletion.

> **Warning:** Deleting a user is permanent and cannot be undone. All associated data (tokens, memberships, keys) will be cascade-deleted.

> **Note:** You cannot delete a user who currently has admin status. Revoke their admin role first. You also cannot delete your own account from the admin panel.

## Safety Constraints

The admin panel enforces the following safety rules:

| Constraint | Reason |
|------------|--------|
| Cannot suspend yourself | Prevents self-lockout |
| Cannot suspend another admin | Requires revoking admin first |
| Cannot delete yourself | Prevents accidental self-deletion |
| Cannot delete another admin | Requires revoking admin first |
| Cannot change your own admin status | Prevents accidental self-demotion |

## Audit Trail

All user management actions are recorded in the [Audit Log](./audit-log.md). The following actions are logged:

- `admin_suspend_user`
- `admin_unsuspend_user`
- `admin_delete_user`
- `admin_grant_admin`
- `admin_revoke_admin`
- `admin_reset_password`

Each log entry includes the admin's user ID and the target user's ID in the metadata.

## API Reference

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/admin/users` | GET | List users with optional `search`, `limit`, `offset` query params |
| `/admin/users/{id}/suspend` | POST | Suspend a user (body: `{ reason }`) |
| `/admin/users/{id}/unsuspend` | POST | Unsuspend a user |
| `/admin/users/{id}` | DELETE | Delete a user account |
| `/admin/users/{id}/admin` | PUT | Set admin status (body: `{ is_admin }`) |
| `/admin/users/{id}/password` | PUT | Reset password (body: `{ new_password }`) |

## Next Step

Continue to [Invite System](./invite-system.md) to learn about managing registration invites.
