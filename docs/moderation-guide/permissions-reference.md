# Permissions Reference

> **Status: Complete**

A comprehensive reference of all moderation-related permissions, organized by role and action category.

## Role Hierarchy

Chatalot has two levels of roles: **instance-level** and **community-level**. Instance-level roles grant server-wide privileges, while community-level roles are scoped to a single community.

### Instance-Level Roles

| Role | Description |
|------|-------------|
| **Instance Owner** | The first registered user. Has unrestricted access to everything on the instance. Bypasses all community membership and role checks. |
| **Instance Admin** | Appointed by the Instance Owner. Has access to the admin panel and all instance-level management features. |
| **Regular User** | A standard registered user with no instance-level privileges. |

### Community-Level Roles

| Role | Level | Description |
|------|-------|-------------|
| **Owner** | 3 (highest) | Created the community or received ownership via transfer. Full community control. |
| **Admin** | 2 | Promoted by the Owner. Can manage settings, members, invites, emoji, and moderate. |
| **Moderator** | 1 | Promoted by an Admin or Owner. Can issue warnings, timeouts, kicks, and bans. |
| **Member** | 0 (default) | Standard community access. Can chat, react, and report. |

## User Actions

Actions that affect user participation.

| Action | Instance Owner | Instance Admin | Community Owner | Community Admin | Community Moderator | Member |
|--------|:-:|:-:|:-:|:-:|:-:|:-:|
| Issue warning | Yes | Yes | Yes | Yes | Yes | No |
| View warning history | Yes | Yes | Yes | Yes | Yes | No |
| Delete warning | Yes | Yes | Yes | Yes | Yes | No |
| Apply timeout | Yes | Yes | Yes | Yes | Yes | No |
| Remove timeout | Yes | Yes | Yes | Yes | Yes | No |
| Kick member | Yes | Yes | Yes | Yes | Yes* | No |
| Ban member | Yes | Yes | Yes | Yes | Yes* | No |
| Unban member | Yes | Yes | Yes | Yes | Yes | No |
| View ban list | Yes | Yes | Yes | Yes | Yes | No |
| Suspend user (instance-wide) | Yes | Yes | No | No | No | No |
| Unsuspend user | Yes | Yes | No | No | No | No |
| Delete user account | Yes | Yes | No | No | No | No |

*Moderators can only kick/ban users with the **Member** role. They cannot act on Admins, other Moderators, or the Owner.

> **Note:** Both Instance Owner and Instance Admin bypass community membership checks and can moderate in any community. The Instance Owner receives the synthetic role `instance_owner` (level 5), and the Instance Admin receives `instance_admin` (level 4).

## Message Actions

Actions related to message content.

| Action | Instance Owner | Instance Admin | Community Owner | Community Admin | Community Moderator | Member |
|--------|:-:|:-:|:-:|:-:|:-:|:-:|
| Delete own messages | Yes | Yes | Yes | Yes | Yes | Yes |
| Delete others' messages | Yes | Yes | Yes | Yes | Yes | No |
| Pin/unpin messages | Yes | Yes | Yes | Yes | Yes | No |
| View edit history | Yes | Yes | Yes | Yes | Yes | Yes |
| Quarantine message | Yes | Yes | No | No | No | No |
| Unquarantine message | Yes | Yes | No | No | No | No |
| Purge single message | Yes | Yes | No | No | No | No |
| Purge user's messages | Yes | Yes | No | No | No | No |
| Purge entire channel | Yes | Yes | No | No | No | No |

> **Note:** "Delete others' messages" requires a channel role of **Moderator** or above (`role_level >= 1`). Instance Owner and Instance Admin can delete messages in any channel, including DMs.

## Report Actions

Actions related to the reporting system.

| Action | Instance Owner | Instance Admin | Community Owner | Community Admin | Community Moderator | Member |
|--------|:-:|:-:|:-:|:-:|:-:|:-:|
| Submit a report | Yes | Yes | Yes | Yes | Yes | Yes |
| View all reports | Yes | Yes | No | No | No | No |
| Review/resolve reports | Yes | Yes | No | No | No | No |
| Dismiss reports | Yes | Yes | No | No | No | No |

> **Note:** The report review system is instance-level only. Community moderators and admins cannot see or manage reports directly. They should coordinate with instance admins when needed.

## Community Management

Actions related to community settings and structure.

| Action | Instance Owner | Instance Admin | Community Owner | Community Admin | Community Moderator | Member |
|--------|:-:|:-:|:-:|:-:|:-:|:-:|
| Edit community settings | Yes | Yes | Yes | Yes | No | No |
| Delete community | Yes | Yes | Yes | No | No | No |
| Transfer ownership | Yes | Yes | Yes | No | No | No |
| Set member role | Yes | Yes | Yes | Yes** | No | No |
| View/manage invites | Yes | Yes | Yes | Yes | No | No |
| Manage custom emoji | Yes | Yes | Yes | Yes | No | No |
| Set member nicknames | Yes | Yes | Yes | Yes | No | Own only |

**Only the Owner can promote members to Admin. Admins can set roles to `member` or `moderator`.

## File Management

Actions related to uploaded files and storage.

| Action | Instance Owner | Instance Admin | Community Owner | Community Admin | Community Moderator | Member |
|--------|:-:|:-:|:-:|:-:|:-:|:-:|
| Upload files | Yes | Yes | Yes | Yes | Yes | Yes |
| Delete own files | Yes | Yes | Yes | Yes | Yes | Yes |
| View all files (admin) | Yes | Yes | No | No | No | No |
| Delete any file | Yes | Yes | No | No | No | No |
| Quarantine file | Yes | Yes | No | No | No | No |
| Unquarantine file | Yes | Yes | No | No | No | No |
| Manage hash blocklist | Yes | Yes | No | No | No | No |
| View storage stats | Yes | Yes | No | No | No | No |

## Audit and Logging

| Action | Instance Owner | Instance Admin | Community Owner | Community Admin | Community Moderator | Member |
|--------|:-:|:-:|:-:|:-:|:-:|:-:|
| View audit log | Yes | Yes | No | No | No | No |

## Key Concepts

### `can_moderate()`

The `can_moderate()` check is used for warnings, timeouts, kicks, and bans. It passes when the actor's role level is strictly higher than the target's. Roles that can moderate:
- Instance Owner (level 5, via `instance_owner` synthetic role)
- Instance Admin (level 4, via `instance_admin` synthetic role)
- Community Owner (level 3)
- Community Admin (level 2)
- Community Moderator (level 1, can only act on Members)

### `can_manage()`

The `can_manage()` check is used for community settings, invites, emoji, and role management. It passes for:
- Instance Owner (via the `instance_owner` synthetic role)
- Instance Admin (via the `instance_admin` synthetic role)
- Community Owner
- Community Admin

### `can_delete_others_messages()`

This check is used for deleting other users' messages and pinning/unpinning. It passes for channel roles at level 1 or above:
- Instance Owner / Instance Admin (bypass all checks)
- Channel/group Owner
- Channel/group Admin
- Channel/group Moderator

### Role Level Enforcement

When a moderator acts on a target user, the system enforces:
- The actor's role level must be **strictly higher** than the target's role level.
- This means Moderators (level 1) can only act on Members (level 0).
- Admins (level 2) can act on Members and Moderators.
- The Owner (level 3) can act on anyone.

## Next Steps

- [Moderation Overview](./moderation-overview.md) -- Return to the moderation overview
- [Managing Members](../user-guide/communities/managing-members.md) -- Community role hierarchy and management
- [Admin Guide -- Role Hierarchy](../admin-guide/role-hierarchy.md) -- Full instance and community role breakdown
