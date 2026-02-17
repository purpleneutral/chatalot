# Role Hierarchy

> **Status: Complete**

Chatalot has a three-tier role system spanning the instance, community, and group levels. Each level has its own set of roles and permissions, and higher-level roles can override lower-level access in specific cases.

## Role Levels Overview

```
Instance Level (server-wide)
  |
  +-- Community Level (per community)
        |
        +-- Group Level (per group within a community)
              |
              +-- Channel Level (inherits from group)
```

## Instance Roles

Instance roles apply server-wide and are managed through the [Admin Panel](./admin-dashboard.md).

| Role | Description | How Assigned |
|------|-------------|--------------|
| **Owner** | Highest authority on the instance; bypasses all community/group access checks | Automatically assigned to the first registered user; can be set via database |
| **Admin** | Full access to the admin panel and all admin API endpoints | Granted by another admin via the Users tab, or via `ADMIN_USERNAME` env var |
| **User** | Standard registered user with no admin privileges | Default for all new registrations |

### Instance Role Permissions

| Permission | Owner | Admin | User |
|------------|:-----:|:-----:|:----:|
| Access admin panel | Yes | Yes | No |
| Manage users (suspend, delete, promote) | Yes | Yes | No |
| Manage invites | Yes | Yes | No |
| View/manage all files | Yes | Yes | No |
| Review reports | Yes | Yes | No |
| View audit log | Yes | Yes | No |
| Use purge/quarantine tools | Yes | Yes | No |
| Manage announcements | Yes | Yes | No |
| Bypass community membership checks | Yes | No | No |
| Cannot be suspended or deleted by other admins | N/A | Yes (must revoke admin first) | No |

> **Note:** The Owner and Admin roles have identical access to the admin panel. The key difference is that the Owner role acts as a "god role" that bypasses community-level access restrictions.

## Community Roles

Community roles apply within a specific community and are managed by the community owner and admins.

| Role | Description | How Assigned |
|------|-------------|--------------|
| **Owner** | Full control of the community; can delete it, transfer ownership, manage all settings | Assigned at community creation; transferable |
| **Admin** | Can manage community settings, members, groups, channels, and moderation | Promoted by the community owner |
| **Moderator** | Can moderate content and members (warn, timeout, kick, ban) | Promoted by community owner or admin |
| **Member** | Standard community member; can participate in channels they have access to | Joins via invite or by browsing discoverable communities |

### Community Role Permissions

| Permission | Owner | Admin | Moderator | Member |
|------------|:-----:|:-----:|:---------:|:------:|
| Delete community | Yes | No | No | No |
| Transfer ownership | Yes | No | No | No |
| Edit community settings | Yes | Yes | No | No |
| Manage groups | Yes | Yes | No | No |
| Manage channels | Yes | Yes | No | No |
| Create invites | Yes | Yes | Configurable | Configurable |
| Manage members (promote, demote) | Yes | Yes | No | No |
| Ban/unban users | Yes | Yes | Yes | No |
| Kick users | Yes | Yes | Yes | No |
| Warn users | Yes | Yes | Yes | No |
| Timeout users | Yes | Yes | Yes | No |
| Send messages | Yes | Yes | Yes | Yes |
| Join voice channels | Yes | Yes | Yes | Yes |

> **Note:** Community settings `who_can_create_groups` and `who_can_create_invites` control whether moderators and/or members can perform these actions.

## Group Roles

Group roles apply within a specific group (a subcategory within a community) and govern access to the group's channels.

| Role | Description | How Assigned |
|------|-------------|--------------|
| **Owner** | Full control of the group | Assigned at group creation; transferable |
| **Admin** | Can manage group settings and channels | Promoted by group owner |
| **Member** | Standard group member | Joins the group or is added by a group admin |

### Group Role Permissions

| Permission | Owner | Admin | Member |
|------------|:-----:|:-----:|:------:|
| Delete group | Yes | No | No |
| Transfer ownership | Yes | No | No |
| Edit group settings | Yes | Yes | No |
| Manage channels | Yes | Yes | No |
| Manage members | Yes | Yes | No |
| Access group channels | Yes | Yes | Yes |

## Channel Access

Channels exist within groups and inherit the group's membership. There are no separate channel-level roles; access is determined by group membership and group-level roles. Channel-specific permissions (such as who can send messages in read-only channels) are controlled through channel settings.

## Instance Admin Override

Instance owners have a special override that affects community-level access. When an instance owner accesses any community endpoint, the middleware assigns them the synthetic role `instance_admin`, which grants:

- `can_manage` -- Equivalent to community owner/admin
- `can_moderate` -- Equivalent to community moderator
- `is_owner` -- Equivalent to community owner

This allows the instance owner to intervene in any community without needing to be a member.

Instance admins (non-owner) do **not** have this bypass. They must be explicit members of a community to access it, unless they are also the instance owner.

## Role Assignment Summary

| Role | Who Can Assign | Where |
|------|---------------|-------|
| Instance Owner | Auto (first user), database | Server-level |
| Instance Admin | Existing admin via admin panel | Admin panel > Users tab |
| Community Owner | Auto (creator), current owner (transfer) | Community settings |
| Community Admin | Community owner | Community member management |
| Community Moderator | Community owner, community admin | Community member management |
| Community Member | Self (join), invite | Community invite system |
| Group Owner | Auto (creator), current owner (transfer) | Group settings |
| Group Admin | Group owner | Group member management |
| Group Member | Self (join), group admin | Group management |

## Visual Role Hierarchy

```
Instance Owner
  |
  +-- Instance Admin
  |     |
  |     +-- (No community override)
  |
  +-- Community Owner  <-- Instance Owner can act as this in any community
        |
        +-- Community Admin
              |
              +-- Community Moderator
                    |
                    +-- Community Member
                          |
                          +-- Group Owner
                                |
                                +-- Group Admin
                                      |
                                      +-- Group Member
```

## Safety Constraints

Several safety rules prevent accidental privilege escalation or self-lockout:

| Constraint | Enforced At |
|------------|-------------|
| Cannot change your own admin status | Admin panel |
| Cannot suspend yourself | Admin panel |
| Cannot delete yourself | Admin panel |
| Cannot suspend another admin (must revoke admin first) | Admin panel |
| Cannot delete another admin (must revoke admin first) | Admin panel |
| Community owner cannot be kicked or banned from their own community | Community moderation |
| Group owner cannot be removed from their own group | Group management |

## Related

- [Admin Dashboard](./admin-dashboard.md) -- Accessing the admin panel
- [User Management](./user-management.md) -- Promoting and demoting instance admins
- [Community Oversight](./community-oversight.md) -- Instance-level community management
- [Moderation Guide](../moderation-guide/README.md) -- Community-level moderation tools
