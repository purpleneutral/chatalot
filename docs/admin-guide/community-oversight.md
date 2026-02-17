# Community Oversight

> **Status: Complete**

Instance administrators have elevated visibility and control over all communities on the server, even those they have not explicitly joined.

> **Permission Required:** Instance Admin or Instance Owner

## Overview

While communities are primarily managed by their own owners, admins, and moderators, instance administrators need the ability to oversee all communities for safety, compliance, and conflict resolution. Chatalot provides this through the **instance admin bypass** -- instance owners (and, in community context, admins acting as `instance_admin`) can access any community without being a member.

## Instance Owner Bypass

The instance owner has a "god role" that bypasses all community membership and ban checks. When an instance owner accesses a community's API endpoints, the community gate middleware assigns them the `instance_admin` role, which grants full access:

| Permission Level | Roles with Access |
|-----------------|-------------------|
| **can_manage** | Community owner, community admin, instance admin |
| **can_moderate** | Community owner, community admin, community moderator, instance admin |
| **is_owner** | Community owner, instance admin |

This means the instance owner can:

- View any community and its channels
- Manage community settings
- Moderate content in any community
- Remove or ban members
- Delete communities

## Viewing Communities Through the User List

The [User Management](./user-management.md) tab shows each user's community and group memberships. This provides a cross-reference view of which users belong to which communities, including their roles (displayed as tooltips on the membership badges).

Community memberships are shown as purple badges, and group memberships as blue badges.

## Managing Communities

Instance admins can take the following actions on communities:

### Through the Admin Panel

- **Suspend community owners** -- If a community owner is violating instance rules, suspending their account effectively disables their ability to manage the community. See [User Management](./user-management.md).
- **Purge community content** -- Use the [Security Settings](./security-settings.md) purge tools to delete all messages in a channel belonging to a community.
- **Review reports** -- Community-related reports appear in the [Reports](./reports-and-moderation.md) tab alongside all other reports.

### Through the Community Interface

Instance owners can navigate to any community in the regular chat interface and use the standard community management tools (settings, member management, channel management) as if they were the community owner.

## Community Creation Control

The `COMMUNITY_CREATION_MODE` environment variable controls who can create new communities:

| Mode | Behavior |
|------|----------|
| `admin_only` (default) | Only instance admins and owners can create communities |
| `open` | Any registered user can create communities |

This setting is configured in the server's `.env` file or Docker Compose environment:

```env
COMMUNITY_CREATION_MODE=admin_only
```

## Best Practices

- **Respect community autonomy.** Use instance-level intervention only when community-level moderation has failed or when instance-wide rules are violated.
- **Document interventions.** When taking action on a community, add notes to reports and use the audit log to maintain a paper trail.
- **Communicate with community owners.** Before taking drastic action (deleting a community, suspending its owner), reach out to the community owner when possible.

## Related

- [User Management](./user-management.md) -- Suspend or delete community owners at the instance level
- [Security Settings](./security-settings.md) -- Purge tools for channel and user content
- [Reports and Moderation](./reports-and-moderation.md) -- Review cross-community reports
- [Role Hierarchy](./role-hierarchy.md) -- Full breakdown of role levels and permissions

## Next Step

Continue to [Registration Settings](./registration-settings.md) to learn about configuring registration modes.
