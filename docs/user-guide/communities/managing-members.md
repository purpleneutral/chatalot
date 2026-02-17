# Managing Members

> **Status: Complete**

Communities are built around their members. This page covers inviting users, understanding the role hierarchy, managing roles, and removing members.

## Inviting Users

To invite someone to your community, you need to generate an **invite link**.

### Creating an Invite

1. Open the community you want to invite someone to.
2. Navigate to the community settings or invite management area.
3. Create a new invite with optional parameters:
   - **Max uses** -- limit how many times the invite can be used (leave blank for unlimited)
   - **Expiry** -- set how long the invite remains valid, from 1 hour to 8,760 hours (1 year)
4. Share the generated invite code or link with the person you want to invite.

> **Permission Required:** Who can create invites is controlled by the community's `who_can_create_invites` policy. This can be set to `everyone`, `moderator`, or `admin`. See [Community Settings](./community-settings.md).

### Accepting an Invite

When someone receives an invite link:

1. They open the link or enter the invite code.
2. They see a preview showing the community name, description, and member count (if the community is discoverable; otherwise it shows "Private Community").
3. They click **Accept** to join the community.

Banned users cannot accept invites. Users who are already members will see a message indicating they have already joined.

> **Tip:** Joining a community does not automatically add the new member to private groups. They will need a separate group invite for those. Public groups, however, auto-include all community members when created.

### Managing Invites

Community admins and owners can view and manage all active invites:

- See each invite's code, usage count, max uses, and expiry
- Delete invites that are no longer needed

> **Permission Required:** Community Admin or higher

## Role Hierarchy

Each community member has one of four roles, listed from most to least privileged:

| Role | Level | Description |
|------|-------|-------------|
| **Owner** | Highest | Full control over the community. Only one owner per community. |
| **Admin** | High | Can manage settings, members, invites, emoji, and moderate. |
| **Moderator** | Medium | Can kick and ban members, manage timeouts and warnings. |
| **Member** | Default | Standard access. Can chat and participate in channels. |

### What Each Role Can Do

| Action | Owner | Admin | Moderator | Member |
|--------|:-----:|:-----:|:---------:|:------:|
| Edit community settings | Yes | Yes | No | No |
| Delete the community | Yes | No | No | No |
| Transfer ownership | Yes | No | No | No |
| Create invites (default policy) | Yes | Yes | Yes | Yes |
| Manage invites (view/delete) | Yes | Yes | No | No |
| Promote to Admin | Yes | No | No | No |
| Set member roles (member/moderator) | Yes | Yes | No | No |
| Kick members | Yes | Yes | Yes* | No |
| Ban/unban members | Yes | Yes | Yes* | No |
| Manage custom emoji | Yes | Yes | No | No |
| Set member nicknames | Yes | Yes | No | Own only |
| View ban list | Yes | Yes | Yes | No |
| Issue timeouts and warnings | Yes | Yes | Yes | No |

*Moderators can only kick/ban members with the **Member** role. They cannot kick or ban Admins or other Moderators.

## Changing Member Roles

> **Permission Required:** Community Admin or higher (only the Owner can promote to Admin)

1. Open the members panel for the community.
2. Find the member whose role you want to change.
3. Select a new role: `member`, `moderator`, or `admin`.

Rules:
- You cannot change your own role.
- You cannot change the Owner's role. Use **Transfer Ownership** instead.
- Only the Owner can promote members to Admin.
- Admins can set members to `member` or `moderator`.

## Setting Nicknames

Members can set their own nickname within a community. Admins and owners can set nicknames for any member.

- Nicknames are 1--64 characters.
- Setting an empty nickname clears it, reverting to the user's display name.
- Nicknames are community-specific and do not affect the user's global profile.

## Kicking Members

> **Permission Required:** Community Moderator or higher

Kicking removes a member from the community and all of its groups and channels. The kicked user can rejoin if they receive a new invite.

- You cannot kick yourself (use **Leave** instead).
- You cannot kick the Owner.
- Moderators can only kick members with the **Member** role.
- Admins can kick Members and Moderators.

## Banning Members

> **Permission Required:** Community Moderator or higher

Banning removes a member and prevents them from rejoining via invite links.

- Provide an optional reason (up to 500 characters) for the ban.
- You cannot ban yourself or the Owner.
- Moderators can only ban members with the **Member** role.
- Admins can ban Members and Moderators.
- The ban list is visible to Moderators, Admins, and the Owner.

### Unbanning

> **Permission Required:** Community Moderator or higher

Unbanning a user removes them from the ban list, allowing them to accept future invites.

## Viewing the Members Panel

The members panel shows all community members, organized by role. Owners appear first, followed by Admins, Moderators, and then regular Members.

![Members Panel](../../screenshots/04-members-panel.png)

## Leaving a Community

Any member (except the Owner) can leave a community at any time. Leaving removes you from the community and all of its groups and channels.

The Owner cannot leave. To step down, the Owner must either:

- **Transfer ownership** to another community member, or
- **Delete the community** entirely

## Transferring Ownership

> **Permission Required:** Community Owner

The Owner can transfer ownership to any other community member. When ownership is transferred:

- The new user becomes the Owner.
- The former Owner is demoted to Admin.
- The target user must already be a member of the community.

## Next Steps

- [Community Settings](./community-settings.md) -- configure policies and appearance
- [Custom Emoji](./custom-emoji.md) -- upload emoji for your community
- [Groups Overview](../groups/overview.md) -- learn how groups organize channels
