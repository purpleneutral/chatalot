# Communities Overview

> **Status: Complete**

A community is the top-level organizational unit in Chatalot -- similar to a "server" in Discord. It is the container for everything: groups, channels, members, roles, custom emoji, and settings.

## What is a Community?

When you open Chatalot, the community picker on the left side of the screen shows all the communities you belong to. Clicking a community loads its groups and channels in the sidebar.

![Community Picker](../../screenshots/02-community-picker.png)

Each community is an independent space with its own:

- **Members** -- users who have joined the community, each with a role
- **Groups** -- organizational folders that contain channels (like Discord categories)
- **Channels** -- text and voice channels where conversations happen
- **Roles** -- a hierarchy that determines who can do what (Owner, Admin, Moderator, Member)
- **Custom emoji** -- community-specific emoji uploaded by admins
- **Settings** -- name, description, icon, banner, policies, theme, and welcome message

## Hierarchy

Chatalot uses a three-level hierarchy:

```
Community
  └── Group
        └── Channel (text or voice)
```

- A community contains one or more **groups**.
- Each group contains one or more **channels**.
- Channels are where actual messaging and voice calls take place.

When you create a community, a default "General" group with a `#general` text channel is automatically created for you.

## Community Membership

You must be a member of a community to see its groups and channels. There are two ways to join:

1. **Invite link** -- someone in the community generates an invite code and shares it with you.
2. **Public groups** -- if you are already a community member, you can freely join any public group within that community.

Joining a community does not automatically add you to all groups. Public groups auto-add all community members when created, but private groups require a separate group invite.

## Discoverability

Communities have a `discoverable` setting. When enabled, the community name and description are shown to anyone who receives an invite link. When disabled, invite links show "Private Community" instead.

## Key Concepts

| Concept | Description |
|---------|-------------|
| Community | Top-level container for all organizational structure |
| Group | An organizational folder within a community (see [Groups](../groups/overview.md)) |
| Channel | A text or voice conversation space within a group (see [Channels](../channels/README.md)) |
| Role | A permission level assigned to each member (Owner, Admin, Moderator, Member) |
| Invite | A shareable code that allows new users to join the community |

## Next Steps

- [Creating a Community](./creating-a-community.md) -- set up your own community
- [Managing Members](./managing-members.md) -- invite users and manage roles
- [Community Settings](./community-settings.md) -- configure your community
