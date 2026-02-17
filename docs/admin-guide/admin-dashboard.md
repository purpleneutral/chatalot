# Admin Dashboard

> **Status: Complete**

The admin dashboard is the central hub for managing your Chatalot instance. It provides a tabbed interface for user management, invites, files, reports, audit logs, security tools, and announcements.

> **Permission Required:** Instance Admin or Instance Owner

![Admin Panel - User Management tab](../../screenshots/08-admin-panel.png)

## Accessing the Admin Panel

Navigate to `/admin` in your browser, or click the **Admin** link in the sidebar (visible only to admins and owners). You can also reach it from the user settings page via the navigation buttons.

If you are not an instance admin or owner, navigating to `/admin` will redirect you back to the main chat view.

## Panel Layout

The admin panel consists of:

- **Header** -- Displays "Admin Panel" with navigation buttons to **Settings** and **Back to Chat** in the top-right corner.
- **Tab Bar** -- A horizontal row of tabs for switching between admin sections.
- **Content Area** -- The main body, which changes based on the selected tab.

## Tabs

The admin panel is organized into seven tabs:

| Tab | Description |
|-----|-------------|
| **Users** | View and manage all registered users on the instance |
| **Invites** | Generate and manage registration invite codes |
| **Files** | Browse all uploaded files with storage statistics |
| **Reports** | Review content reports submitted by users |
| **Audit Log** | View a time-ordered log of admin and security events |
| **Security** | Purge tools, message quarantine, and file hash blocklist |
| **Announcements** | Publish server-wide announcements |

Each tab loads its data on first access and caches it for the duration of the session. Switching between tabs does not reload data unless you explicitly refresh.

## Who Can Access

Access to the admin panel requires one of the following flags on the user account:

| Role | Access | How It Is Assigned |
|------|--------|--------------------|
| **Instance Owner** | Full access | Automatically assigned to the first registered user; can also be set via the `ADMIN_USERNAME` environment variable |
| **Instance Admin** | Full access | Granted by an existing admin via the Users tab or the `ADMIN_USERNAME` environment variable |

Both roles have identical access to the admin panel. The instance owner flag serves as a "god role" that additionally bypasses community membership checks (see [Role Hierarchy](./role-hierarchy.md)).

## Quick Navigation

From the admin panel, you can:

- Click **Settings** to go to your personal account settings.
- Click **Back to Chat** to return to the main chat interface.

## Next Step

Continue to [User Management](./user-management.md) to learn about managing user accounts.
