# Managing Groups

> **Status: Complete**

This page covers editing group settings, managing group invites, deleting groups, and transferring ownership.

## Editing Group Settings

> **Permission Required:** Group Owner or Admin

Group owners and admins can edit the following settings:

### Name

- **Length:** 1--64 characters
- Displayed in the sidebar as the group's header.

### Description

- **Length:** up to 2,048 characters
- An optional description for the group.

### Visibility

- **Options:** `public` or `private`
- Changing a public group to private does not remove existing members.
- Changing a private group to public does not automatically add all community members.

### Discoverability

- Toggle whether the group appears in the group discovery list.
- When disabled, the group is hidden from discovery and invite previews show "Private Group" instead of the actual name and description.

### Icon

Upload an icon image for the group.

- **Formats:** PNG, JPEG, WebP, GIF
- **Max size:** 10 MB

### Banner

Upload a banner image for the group.

- **Formats:** PNG, JPEG, WebP, GIF
- **Max size:** 10 MB

### Accent Color

Set a custom accent color for the group.

- **Format:** Hex color code (e.g., `#ff5500`)

### Allow Invites (Personal Groups)

> **Permission Required:** Community Moderator or higher

For personal groups, this setting controls whether the assigned member (group owner) can create invites. When disabled, only community moderators and above can generate invites for the group.

## Managing Channels

Groups contain channels. Group owners and admins can:

- **Create channels** -- add text or voice channels to the group
- **Edit channels** -- change name, topic, read-only mode, slow mode, discoverability, and archive status
- **Delete channels** -- permanently remove a channel and all its messages
- **Upload voice background** -- set a background image for voice channels

For details, see [Channels](../channels/README.md).

### Channel Limits

| Limit | Value |
|-------|-------|
| Channels per group | 100 |
| Channel name | 1--64 characters |
| Channel topic | up to 512 characters |
| Slow mode | 0--86,400 seconds (0 = off, max = 24 hours) |
| Voice background size | 10 MB |

## Managing Group Invites

Group invites allow you to add new members to private groups (or to share a direct join link for public groups).

### Creating an Invite

> **Permission Required:** Group Owner or Admin

1. Navigate to the group's invite management area.
2. Create a new invite with optional parameters:
   - **Max uses** -- limit how many times the invite can be used (leave blank for unlimited)
   - **Expiry** -- set how long the invite remains valid, from 1 hour to 8,760 hours (1 year)
3. Share the generated invite code or link.

For personal groups with `allow_invites` set to `false`, only community moderators or higher can create invites.

### Accepting an Invite

When someone receives a group invite:

1. They must already be a member of the group's community.
2. They see a preview showing the group name, description, and member count (if discoverable; otherwise "Private Group").
3. They click **Accept** to join the group and all its channels.

Invite acceptance fails if:
- The invite has expired
- The invite has reached its max uses
- The user is not a member of the group's community
- The user is already a group member

### Viewing and Deleting Invites

> **Permission Required:** Group Owner or Admin

Owners and admins can:
- View all active invites with their usage counts and expiry times.
- Delete invites that are no longer needed.

## Deleting a Group

Deleting a group permanently removes:

- The group and all its settings
- All channels within the group
- All messages and data in those channels
- All group memberships and invites

This action cannot be undone.

### Who Can Delete

| Group type | Who can delete |
|-----------|---------------|
| **Regular group** | Group Owner only |
| **Personal group** | Group Owner (assigned member), original creator, or Community Moderator+ |

## Transferring Ownership

> **Permission Required:** Group Owner

The group Owner can transfer ownership to another group member.

When ownership is transferred:
- The target user becomes the new Owner.
- The former Owner is demoted to Admin.
- The target must already be a member of the group.

### Restrictions

- **Personal groups** cannot have their ownership transferred. The assigned member remains the owner for the lifetime of the group.
- Only the current Owner can initiate a transfer.

## Leaving a Group

Any group member (except the Owner) can leave a group at any time.

When you leave:
- You are removed from all channels in the group.
- Your encryption sender keys for those channels are deleted.
- Remaining members are notified that a key rotation is required.

The group Owner cannot leave. They must either:
- **Transfer ownership** to another member, then leave
- **Delete the group** entirely

## Group Discovery

Community members can browse discoverable groups using the group discovery feature.

- **Public, discoverable groups** are shown to all community members.
- **Private, discoverable groups** are shown but require an invite to join.
- **Non-discoverable groups** are hidden from the discovery list entirely.
- Instance administrators can see all groups across all communities.

## Settings Reference

| Setting | Type | Default | Editable by |
|---------|------|---------|-------------|
| Name | String (1--64 chars) | Set at creation | Group Owner/Admin |
| Description | String (up to 2,048 chars) | None | Group Owner/Admin |
| Visibility | `public` or `private` | `public` | Group Owner/Admin |
| Discoverable | Boolean | Varies | Group Owner/Admin |
| Allow invites | Boolean | `true` (regular) / `false` (personal) | Community Moderator+ |
| Icon | Image (PNG/JPEG/WebP/GIF, 10 MB) | None | Group Owner/Admin |
| Banner | Image (PNG/JPEG/WebP/GIF, 10 MB) | None | Group Owner/Admin |
| Accent color | Hex color string | None | Group Owner/Admin |

## Next Steps

- [Group Permissions](./group-permissions.md) -- understand the permission model
- [Creating Groups](./creating-groups.md) -- create new groups
- [Channels](../channels/README.md) -- manage channels within your groups
- [Community Settings](../communities/community-settings.md) -- configure community-level policies
