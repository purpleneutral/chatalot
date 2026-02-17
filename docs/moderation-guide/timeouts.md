# Timeouts

> **Status: Complete**

A timeout temporarily prevents a user from sending messages in a specific channel. Timeouts are the recommended escalation step after warnings, providing a cool-down period without removing the user from the community.

## What a Timeout Does

When a user is timed out in a channel:

- They **cannot send messages** in that channel until the timeout expires.
- They **can still read** messages in the channel.
- They **can still participate** in other channels within the community.
- If they attempt to send a message, they receive an error showing how many seconds remain on the timeout.

Timeouts are **per-channel**, not community-wide. A user timed out in `#general` can still chat freely in `#off-topic`.

## Duration Options

> **Permission Required:** Community Moderator or higher

The following timeout durations are available from the user profile card in the UI:

| Duration | Seconds | Use Case |
|----------|---------|----------|
| **1 minute** | 60 | Quick cool-down for heated moments |
| **5 minutes** | 300 | Minor disruptions or off-topic chatter |
| **15 minutes** | 900 | Repeated minor violations after a warning |
| **1 hour** | 3,600 | Persistent disruptive behavior |
| **1 day** | 86,400 | Serious rule violations requiring a significant break |
| **1 week** | 604,800 | Major infractions; one step below a kick |

The API accepts any value between **60 seconds** (1 minute) and **2,592,000 seconds** (30 days). The UI provides the preset options listed above, but custom durations within this range can be set via the API.

## How to Apply a Timeout

### From the User Profile Card

1. Click on a user's avatar or name in the channel to open their **profile card**.
2. If you have moderation permissions, you will see a **Timeout** button.
3. Click **Timeout** to expand the duration picker.
4. Select the desired duration from the list.
5. The timeout takes effect immediately.

### From the API

```
POST /api/communities/{communityId}/channels/{channelId}/timeout
```

Request body:
```json
{
  "user_id": "<target-user-uuid>",
  "duration_seconds": 3600,
  "reason": "Repeated spam after two warnings."
}
```

The `reason` field is optional (up to 500 characters) but strongly recommended for record-keeping.

## What the User Sees

When timed out, the user experiences the following:

1. A **real-time notification** is broadcast to all members of the channel, announcing the timeout with the affected user, expiry time, and reason (if provided).
2. If the timed-out user attempts to send a message, they receive an error indicating how many seconds remain before the timeout expires.
3. The message input area may reflect the timeout state, preventing submission.

## Replacing an Existing Timeout

If a user already has an active timeout in a channel and a moderator applies a new timeout:

- The **old timeout is removed** and replaced with the new one.
- The new timeout duration starts from the current time.
- This allows moderators to extend or shorten a timeout as needed.

## Removing a Timeout Early

> **Permission Required:** Community Moderator or higher

If a timeout was issued in error, or if the user has shown they understand the issue, you can lift it before it expires:

```
DELETE /api/communities/{communityId}/channels/{channelId}/timeout/{userId}
```

Removing a timeout:

- Takes effect immediately -- the user can send messages again.
- Creates an entry in the **audit log** recording who lifted the timeout and when.
- Does not delete the original timeout record from history.

If no active timeout exists for the specified user and channel, the API returns a 404 error.

## Timeout Expiry and Cleanup

Timeouts expire automatically when their `expires_at` timestamp is reached. The server runs a periodic cleanup task (`cleanup_expired`) to remove stale timeout records from the database. No moderator action is required for expired timeouts.

## Best Practices

- **Always explain the reason.** Even though the reason field is optional, providing one helps the user understand what they did wrong and what to change.
- **Start short.** A 5-minute timeout for a first escalation gives the user a chance to cool down without feeling punished.
- **Escalate progressively.** Move from short timeouts to longer ones before considering a kick or ban.
- **Check warning history first.** Before issuing a timeout, review the user's warning history to make an informed decision about the appropriate duration.

## Next Steps

- [Kicks and Bans](./kicks-and-bans.md) -- When timeouts are not enough
- [Warnings](./warnings.md) -- The step before timeouts
- [Permissions Reference](./permissions-reference.md) -- Who can issue and remove timeouts
