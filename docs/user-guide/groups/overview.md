# Groups Overview

> **Status: Complete**

Groups are organizational folders within a community, similar to "categories" in other chat platforms. They sit between communities and channels in the hierarchy and help you organize related channels together.

## What is a Group?

A group is a container for channels within a community. For example, a gaming community might have groups like "General," "Game Discussion," and "Voice Rooms," each containing relevant text and voice channels.

```
Community: "My Gaming Community"
  ├── Group: "General"
  │     ├── #welcome
  │     ├── #rules
  │     └── #off-topic
  ├── Group: "Game Discussion"
  │     ├── #strategy
  │     └── #highlights
  └── Group: "Voice Rooms"
        ├── Lobby
        └── Team Chat
```

Groups appear in the sidebar and can be collapsed to hide their channels, keeping the interface organized.

## Group Types

Chatalot supports three types of groups based on visibility and ownership:

### Public Groups

- **Visibility:** `public`
- **Auto-join:** When a public group is created, all current community members are automatically added.
- **Open join:** Community members can freely join public groups at any time.
- **Discoverable:** Visible in the group discovery list (when discoverability is enabled).

### Private Groups

- **Visibility:** `private`
- **Invite-only:** Members must receive a group invite to join. They cannot join directly.
- **Not discoverable by default:** Hidden from discovery unless explicitly made discoverable.
- **Use case:** Restricted conversations, team-specific channels, or sensitive topics.

### Personal Groups

- **Visibility:** Forced to `private`
- **Assigned member:** A personal group is created by a community moderator or admin and assigned to a specific community member.
- **Ownership:** The assigned member becomes the group owner in `group_members`.
- **Invite control:** By default, the assigned member cannot create invites (`allow_invites` is `false`). A community moderator or higher can toggle this setting.
- **Moderation access:** Community moderators and admins automatically have admin-level access to personal groups, even without being an explicit group member.
- **Use case:** Private workspaces, ticket-style support channels, or individual member areas.

## Default Group

When you create a new community, Chatalot automatically creates a default group with:

- **Group name:** (matches the community setup)
- **Channel:** A `#general` text channel

You can rename, reconfigure, or delete this default group.

## Group Properties

Each group has the following properties:

| Property | Description |
|----------|-------------|
| **Name** | Display name (1--64 characters) |
| **Description** | Optional description (up to 2,048 characters) |
| **Icon** | Optional group icon image |
| **Banner** | Optional group banner image |
| **Accent color** | Optional hex color for visual customization |
| **Visibility** | `public` or `private` |
| **Discoverable** | Whether the group appears in group discovery |
| **Allow invites** | Whether group owners/admins can create invites (relevant for personal groups) |
| **Owner** | The user who owns the group |
| **Community** | The parent community |
| **Assigned member** | If set, this is a personal group owned by the assigned user |

## Joining and Leaving Groups

### Joining

- **Public groups:** Any community member can join directly.
- **Private groups:** Requires a group invite link.
- **Personal groups:** Requires a group invite (if `allow_invites` is enabled) or being added by a moderator.

To join a group via invite, you must already be a member of the group's community.

### Leaving

Any group member can leave, except the group owner. When you leave a group:

- You are removed from all channels within that group.
- Your sender keys for those channels are cleaned up.
- Other members are notified that a key rotation is needed (for encryption purposes).

The group owner cannot leave. They must either transfer ownership or delete the group.

## Limits

| Limit | Value |
|-------|-------|
| Groups per community | 200 |
| Channels per group | 100 |
| Group name length | 1--64 characters |
| Group description length | up to 2,048 characters |

## Next Steps

- [Creating Groups](./creating-groups.md) -- create a new group in your community
- [Group Permissions](./group-permissions.md) -- understand who can do what
- [Managing Groups](./managing-groups.md) -- edit, delete, and manage group settings
