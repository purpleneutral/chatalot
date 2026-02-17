# Channel Permissions

> **Status: Complete**

Chatalot uses a role hierarchy to control who can perform which actions in a channel. Permissions flow down from the Community level through Groups to individual Channels.

## Role Hierarchy

There are three role levels at the channel/group level:

| Role | Level | Description |
|------|-------|-------------|
| **Owner** | 2 | Full control over the channel and all its settings |
| **Admin** | 1 | Can moderate members and manage most channel settings |
| **Member** | 0 | Can send messages and use standard features |

At the Community level, there are additional roles:

| Role | Description |
|------|-------------|
| **Owner** | Full control of the community |
| **Admin** | Community-wide administration |
| **Moderator** | Moderation across the community's groups |
| **Member** | Standard community member |

## Permission Inheritance

Permissions flow through the hierarchy:

```
Community Role
  └── Group Role (owner of group, or inherited from community role)
        └── Channel Permissions (inherited from group role)
```

- If you are a **Community admin**, you receive admin-level permissions in all Groups within that Community.
- If you are the **Group owner**, you have owner-level permissions in all channels within that Group.
- A Community **moderator** receives admin-level permissions in personal (assigned) groups.

## Permission Matrix

### Message Permissions

| Action | Member | Admin | Owner |
|--------|--------|-------|-------|
| Send messages | Yes | Yes | Yes |
| Send messages (read-only channel) | No | Yes | Yes |
| Send messages (slow mode active) | Rate-limited | Exempt | Exempt |
| Send messages (archived channel) | No | No | No |
| Edit own messages (within 15 min) | Yes | Yes | Yes |
| Delete own messages | Yes | Yes | Yes |
| Delete others' messages | No | Yes | Yes |

### Channel Management

| Action | Member | Admin | Owner |
|--------|--------|-------|-------|
| Edit channel topic | No | Yes | Yes |
| Rename channel | No | No | Yes |
| Toggle read-only | No | Yes | Yes |
| Set slow mode | No | Yes | Yes |
| Archive/unarchive | No | Yes | Yes |
| Delete channel | No | No | Yes |

### Moderation

| Action | Member | Admin | Owner |
|--------|--------|-------|-------|
| Kick members | No | Members only | Admins and members |
| Ban members | No | Members only | Admins and members |
| Unban members | No | Yes | Yes |
| Change member roles | No | No | Yes |
| Pin/unpin messages | No | Yes | Yes |
| Kick from voice | No | Members only | Admins and members |

> **Important:** Moderation actions follow a strict hierarchy. You can only moderate users with a **strictly lower** role level than your own. An admin cannot kick another admin; only an owner can.

### Member Management

| Action | Member | Admin | Owner |
|--------|--------|-------|-------|
| View member list | Yes | Yes | Yes |
| Transfer ownership | No | No | Yes |

## How Roles Are Determined

When you interact with a channel, your effective role is determined by checking (in order):

1. **Group ownership:** If you are the Group owner, you have owner-level permissions.
2. **Personal group assignment:** If you are the assigned member of a personal group, you have owner-level permissions in that group's channels.
3. **Community role:** Your Community role (owner, admin, moderator, member) is inherited as a group-level role.
4. **Explicit channel role:** If you have been given a specific role in the channel (admin or member), that role applies.

## Related Pages

- [Channel Settings](./channel-settings.md)
- [Text Channels](./text-channels.md)
- [Groups](../groups/README.md)
- [Communities](../communities/README.md)
