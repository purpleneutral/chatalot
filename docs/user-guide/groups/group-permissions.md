# Group Permissions

> **Status: Complete**

Groups have their own role hierarchy that determines who can manage the group, its channels, and its members. Group permissions work alongside -- and sometimes inherit from -- community-level roles.

## Group Role Hierarchy

Each group member has one of three roles:

| Role | Level | Description |
|------|-------|-------------|
| **Owner** | Highest | Full control over the group. Can delete the group, transfer ownership, and manage all settings. |
| **Admin** | High | Can manage group settings, create/edit/delete channels, and manage invites. |
| **Member** | Default | Standard access. Can view channels and participate in conversations. |

> **Note:** Groups do not have a separate "Moderator" role. Moderation actions (timeouts, warnings, kicks, bans) are handled at the community level. See the [Moderation Guide](../../moderation-guide/README.md) for details.

## Permission Matrix

| Action | Owner | Admin | Member |
|--------|:-----:|:-----:|:------:|
| View group and channels | Yes | Yes | Yes |
| Edit group settings (name, description, visibility) | Yes | Yes | No |
| Upload group icon/banner | Yes | Yes | No |
| Create channels | Yes | Yes | No |
| Edit channels | Yes | Yes | No |
| Delete channels | Yes | Yes | No |
| Create group invites | Yes | Yes | No |
| Delete group invites | Yes | Yes | No |
| Delete the group | Yes | No | No |
| Transfer group ownership | Yes | No | No |

## Community Role Inheritance

Community-level roles provide implicit access to groups in certain situations:

### Regular Groups

For standard (non-personal) groups, community roles do **not** automatically grant group-level permissions. A community admin is treated as a regular group member unless they also have an admin role within the group.

### Personal Groups

Personal groups (those with an `assigned_member_id`) have special permission inheritance:

- **Community Moderators, Admins, and Owners** automatically receive **admin-level access** to all personal groups in their community, even if they are not explicitly added as group members.
- **Instance administrators** also receive admin-level access to all personal groups.

This ensures that community staff can always access and manage personal groups for moderation purposes.

| Actor | Regular Group | Personal Group |
|-------|:------------:|:--------------:|
| Community Member | Uses group role | Uses group role |
| Community Moderator | Uses group role | Implicit Admin |
| Community Admin | Uses group role | Implicit Admin |
| Community Owner | Uses group role | Implicit Admin |
| Instance Admin | Uses group role | Implicit Admin |

## Who Can Delete Groups

The rules for deleting a group depend on the group type:

### Regular Groups
- Only the **group Owner** can delete the group.

### Personal Groups
- The **group Owner** (assigned member) can delete the group.
- The **original creator** (`owner_id` on the group record) can delete the group.
- Any **community Moderator or higher** can delete the group.

## Invite Permissions

### Regular Groups
- Group **Owners** and **Admins** can create, view, and delete invites.

### Personal Groups
- The `allow_invites` flag controls whether the assigned member (group owner) can create invites.
- When `allow_invites` is `false` (the default), only **community Moderators or higher** can create invites for the personal group.
- When `allow_invites` is `true`, the group owner and admins can create invites normally.
- Only **community Moderators or higher** can change the `allow_invites` setting.

## Effective Role Resolution

When determining what a user can do in a group, Chatalot checks permissions in this order:

1. **Direct group membership** -- if the user is an explicit member of the group, their group role is used.
2. **Personal group inheritance** -- if the group is a personal group and the user is a community moderator or higher (or an instance admin), they receive implicit admin access.
3. **No access** -- if neither condition is met, the user cannot access the group.

## Ownership Transfer

- **Regular groups:** The group Owner can transfer ownership to any existing group member. The old owner is demoted to Admin, and the new owner is promoted to Owner.
- **Personal groups:** Ownership cannot be transferred. The assigned member remains the owner for the lifetime of the group.

## Next Steps

- [Managing Groups](./managing-groups.md) -- edit, delete, and administer groups
- [Creating Groups](./creating-groups.md) -- create new groups with the right visibility
- [Community Settings](../communities/community-settings.md) -- configure who can create groups
- [Managing Members](../communities/managing-members.md) -- community-level role management
