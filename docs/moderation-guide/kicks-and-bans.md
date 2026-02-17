# Kicks and Bans

> **Status: Complete**

Kicks and bans are the most severe community-level moderation actions. Use them when warnings and timeouts have not resolved the issue, or when the behavior is severe enough to warrant immediate removal.

## Kicking a User

> **Permission Required:** Community Moderator or higher

Kicking removes a user from the community entirely. The kicked user loses access to all channels, groups, and group channels within the community.

### What Happens When a User Is Kicked

- The user is removed from the **community membership**.
- The user is removed from all **groups** within the community.
- The user is removed from all **channels** within those groups.
- The action is recorded in the **audit log** (action: `community_kick`).
- The kicked user **can rejoin** if they receive a new invite link.

### How to Kick a User

**From the community settings page:**

1. Open the community settings and navigate to the **Members** tab.
2. Find the user you want to kick.
3. Click the **Kick** button next to their name.
4. Confirm the action in the dialog.

**From the API:**

```
DELETE /api/communities/{communityId}/members/{userId}
```

### Kick Restrictions

- You **cannot kick yourself**. Use **Leave Community** instead.
- You **cannot kick the Owner**. The Owner must transfer ownership or delete the community.
- **Moderators** can only kick users with the **Member** role.
- **Admins** can kick Members and Moderators, but not other Admins or the Owner.

## Banning a User

> **Permission Required:** Community Moderator or higher

Banning is a permanent removal. A banned user is removed from the community and **cannot rejoin** via invite links until unbanned.

> **Warning:** Banning is a serious action that permanently prevents the user from participating in your community. Make sure you have considered warnings, timeouts, and kicks first -- unless the violation is severe enough to warrant an immediate ban.

### What Happens When a User Is Banned

- A **ban record** is created with the ban reason, the banning moderator, and a timestamp.
- The user is removed from all **channels** in community groups.
- The user is removed from all **groups** within the community.
- The user is removed from the **community membership**.
- If the banned user attempts to accept an invite to the community, they receive a **Forbidden** error.

All of these steps happen atomically in a single database transaction.

### How to Ban a User

**From the community settings page:**

1. Open the community settings and navigate to the **Members** tab.
2. Find the user you want to ban.
3. Click the **Ban** button next to their name.
4. Enter an optional **reason** for the ban (up to 500 characters).
5. Confirm the action.

**From the API:**

```
POST /api/communities/{communityId}/bans/{userId}
```

Request body:
```json
{
  "reason": "Repeated harassment after multiple warnings and timeouts."
}
```

The `reason` field is optional but strongly recommended. It is stored with the ban record and visible to other moderators in the ban list.

### Ban Restrictions

- You **cannot ban yourself**.
- You **cannot ban the Owner**.
- **Moderators** can only ban users with the **Member** role.
- **Admins** can ban Members and Moderators, but not other Admins or the Owner.
- If the user is not currently a member (already left or was kicked), the ban record is still created to prevent future rejoins.

## Viewing the Ban List

> **Permission Required:** Community Moderator or higher

The ban list shows all users who are currently banned from the community.

**From the community settings page:**

1. Open the community settings.
2. Navigate to the **Bans** tab.
3. View the list of banned users with their username, display name, ban reason, and ban date.

**From the API:**

```
GET /api/communities/{communityId}/bans
```

The response includes:
- `user_id` -- the banned user's ID
- `username` -- the banned user's username
- `display_name` -- the banned user's display name
- `reason` -- the ban reason (if provided)
- `created_at` -- when the ban was issued

The ban list is limited to the 500 most recent bans, sorted by date (newest first).

## Unbanning a User

> **Permission Required:** Community Moderator or higher

Unbanning removes the ban record, allowing the user to accept future invite links.

**From the community settings page:**

1. Open the community settings and navigate to the **Bans** tab.
2. Find the user you want to unban.
3. Click the **Unban** button next to their name.

**From the API:**

```
DELETE /api/communities/{communityId}/bans/{userId}
```

Unbanning:

- Removes the ban record from the database.
- Records the action in the **audit log** (action: `community_unban`).
- Does **not** automatically re-add the user to the community. They must use a new invite link to rejoin.

If no ban record exists for the specified user, the API returns a 404 error.

## Community Ban vs. Instance-Level Suspension

It is important to understand the difference between these two actions:

| Aspect | Community Ban | Instance Suspension |
|--------|--------------|---------------------|
| **Scope** | One community | Entire instance (all communities) |
| **Who can do it** | Community Moderator+ | Instance Admin or Owner |
| **Effect** | Cannot rejoin that community | Cannot log in at all |
| **Reversible** | Yes (unban) | Yes (unsuspend) |
| **User's other communities** | Unaffected | All inaccessible |
| **Where to manage** | Community settings > Bans | Admin panel > User Management |

For instance-level suspensions, see the [Admin Guide -- User Management](../admin-guide/user-management.md).

## Best Practices

- **Document the reason.** Always provide a ban reason. Future moderators reviewing the ban list will need context about why the decision was made.
- **Consider a kick first.** If you think the user might reform with a second chance, kick them instead of banning. They can rejoin via invite and start fresh.
- **Coordinate with your team.** For borderline cases, discuss the situation with other moderators or admins before issuing a permanent ban.
- **Review periodically.** Check your ban list from time to time. Some bans from months ago may no longer be necessary.

## Next Steps

- [Content Moderation](./content-moderation.md) -- Deleting messages and managing content
- [Timeouts](./timeouts.md) -- The step before kicks and bans
- [Permissions Reference](./permissions-reference.md) -- Who can kick, ban, and unban
