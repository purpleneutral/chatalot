# Creating Groups

> **Status: Complete**

Groups let you organize channels within a community. This page covers how to create groups, the different visibility options, and what happens automatically when a group is created.

## Who Can Create Groups

Group creation is controlled by the community's **"Who can create groups"** policy setting. The community owner or admin configures this in [Community Settings](../communities/community-settings.md).

| Policy | Who can create groups |
|--------|----------------------|
| `everyone` (default) | Any community member |
| `moderator` | Community Moderators, Admins, and the Owner |
| `admin` | Community Admins and the Owner only |

Instance administrators can always create groups in any community.

## Creating a Standard Group

1. Navigate to the community where you want to create a group.
2. Use the group creation option in the sidebar (right-click menu or a dedicated button).
3. Fill in the group details:
   - **Name** (required) -- 1 to 64 characters
   - **Description** (optional) -- up to 2,048 characters
   - **Visibility** -- `public` or `private` (defaults to `public`)
4. Confirm creation.

### What Happens on Creation

When a group is created:

1. The group is added to the community.
2. A **#general text channel** is automatically created inside the group.
3. The creator becomes the group **Owner**.
4. **For public groups:** All current community members are automatically added as group members.
5. **For private groups:** Only the creator is added as the group owner.

> **Tip:** If your community has more than 5,000 members, public group creation with auto-join is disabled. Create a private group and use invites instead.

## Creating a Personal Group

Personal groups are private groups assigned to a specific community member. They are typically used for ticket-style support, individual workspaces, or moderation purposes.

> **Permission Required:** Community Moderator or higher

To create a personal group:

1. Use the group creation interface.
2. Set the **assigned member** to the target community member.
3. Provide a name and optional description.

Personal groups have special behavior:

- **Visibility** is forced to `private`.
- **Discoverability** is disabled.
- The **assigned member** becomes the group owner in the membership table (not the creator).
- The **#general channel** is created with the assigned member as the channel creator.
- The assigned member must be a community member.
- By default, the assigned member **cannot create invites** (`allow_invites` defaults to `false`). A community moderator or higher can change this.

## Visibility Options

| Visibility | Behavior |
|-----------|----------|
| **Public** | All community members are auto-added on creation. New members who join the community later are also auto-added. Any community member can join freely. Visible in group discovery (if discoverable). |
| **Private** | Only the creator is added. Other members must receive a group invite. Not visible in discovery by default. |

You can change a group's visibility after creation (see [Managing Groups](./managing-groups.md)).

## After Creating a Group

Once your group exists, you can:

- **Create channels** -- add text and voice channels to the group (see [Channels](../channels/README.md))
- **Invite members** -- for private groups, generate invite links (see [Managing Groups](./managing-groups.md))
- **Customize** -- upload an icon, banner, or set an accent color
- **Adjust settings** -- change the name, description, visibility, or discoverability

## Limits

| Limit | Value |
|-------|-------|
| Groups per community | 200 |
| Group name | 1--64 characters |
| Group description | up to 2,048 characters |
| Max community size for public auto-join | 5,000 members |

## Next Steps

- [Group Permissions](./group-permissions.md) -- understand who can do what within groups
- [Managing Groups](./managing-groups.md) -- edit, delete, and configure groups
- [Channels](../channels/README.md) -- create channels within your new group
