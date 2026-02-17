# Warnings

> **Status: Complete**

Warnings are the lightest moderation action. They formally notify a user that their behavior violates community expectations, without restricting their ability to participate.

## Issuing a Warning

> **Permission Required:** Community Moderator or higher

To warn a user:

1. Navigate to the channel where the problematic behavior occurred.
2. Click on the user's avatar or name to open their **profile card**.
3. Use the moderation option to issue a warning with a reason.

Alternatively, warnings can be issued from the community member management panel via the community settings page.

### Warning Reasons

Every warning **requires a reason**. The reason must be between 1 and 1,000 characters. Be specific and constructive:

- **Good:** "Please keep discussion in #general on-topic. This channel is for project updates only."
- **Poor:** "Bad behavior"

The reason is visible to the warned user and recorded in the warning history, so write something that clearly explains the issue and what behavior is expected going forward.

### API Details

Warnings are scoped to a **specific channel**. The API endpoint is:

```
POST /api/communities/{communityId}/channels/{channelId}/warn
```

Request body:
```json
{
  "user_id": "<target-user-uuid>",
  "reason": "Please keep discussions on-topic in this channel."
}
```

## What Users See When Warned

When a warning is issued, a real-time **WebSocket notification** is broadcast to all members in the channel:

- The notification includes the **channel**, the **warned user**, the **reason**, and the user's **total warning count** in that channel.
- The warned user sees the warning appear in the channel, making it clear that a moderator has formally addressed their behavior.

This transparency ensures the community understands that moderation is active and fair.

## Warning History

> **Permission Required:** Community Moderator or higher

Moderators can view the complete warning history for any user within a specific channel. This helps you understand whether a user has a pattern of problematic behavior before deciding to escalate.

To view warning history:

```
GET /api/communities/{communityId}/channels/{channelId}/warnings/{userId}
```

The response includes:
- **Warning ID** -- unique identifier for each warning
- **Reason** -- the text provided by the issuing moderator
- **Issued by** -- which moderator issued the warning
- **Created at** -- timestamp of when the warning was issued

Warnings are listed in reverse chronological order (newest first).

## Warning Count

After issuing a warning, the system automatically counts the total warnings for that user in that channel. This count is included in the broadcast notification, making it easy for the moderation team to see at a glance whether this is a first offense or a recurring pattern.

## Automatic Escalation

> **Status: Planned**

Automatic escalation from warnings to timeouts is a planned feature. In the current version, moderators must manually escalate if a user accumulates too many warnings. A future update will allow community admins to configure rules like "auto-timeout after 3 warnings in 24 hours."

For now, the recommended practice is:

1. **1st warning** -- Verbal notice with explanation
2. **2nd warning** -- Formal warning with a clear statement that further violations will result in a timeout
3. **3rd warning** -- Manually apply a timeout (see [Timeouts](./timeouts.md))
4. **Continued violations** -- Consider a kick or ban (see [Kicks and Bans](./kicks-and-bans.md))

## Deleting Warnings

Warnings can be removed from a user's record if they were issued in error. This is done via the API:

```
DELETE /api/warnings/{warningId}
```

> **Note:** Deleting a warning removes it from the database entirely. There is no undo. Use this only for warnings that were issued by mistake.

## Next Steps

- [Timeouts](./timeouts.md) -- Escalate from warnings to temporary muting
- [Permissions Reference](./permissions-reference.md) -- See who can issue and view warnings
