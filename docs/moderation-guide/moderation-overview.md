# Moderation Overview

> **Status: Complete**

An introduction to Chatalot's moderation tools, who can use them, and the overall moderation philosophy.

## What Moderation Tools Are Available

Chatalot provides a layered set of moderation actions, ordered from lightest to most severe:

| Action | Severity | Scope | Effect |
|--------|----------|-------|--------|
| **Warning** | Low | Per channel | Notifies the user with a reason; recorded in warning history |
| **Timeout** | Medium | Per channel | Prevents the user from sending messages for a set duration |
| **Kick** | High | Community-wide | Removes the user from the community (they can rejoin via invite) |
| **Ban** | Highest | Community-wide | Removes the user and prevents them from rejoining |

In addition to user-targeted actions, moderators can also:

- **Delete messages** -- Remove individual messages from a channel (soft-delete)
- **Pin/unpin messages** -- Highlight important messages in a channel
- **Review reports** -- Handle reports submitted by community members (Instance Admins only)

## Who Can Moderate

Moderation permissions are based on your **community role**. The following roles have moderation capabilities:

| Role | Can Moderate? | Notes |
|------|:------------:|-------|
| **Community Owner** | Yes | Full moderation power over all members |
| **Community Admin** | Yes | Can moderate Members and Moderators; can delete others' messages |
| **Community Moderator** | Yes | Can moderate Members only; can issue warnings and timeouts |
| **Community Member** | No | Can report messages to admins, but cannot take moderation action |
| **Instance Owner** | Yes | Bypasses all community checks (god role); can moderate in any community |

### Role Hierarchy Rules

- You can only moderate users with a **strictly lower** role level than your own.
- Moderators can kick/ban **Members** only. They cannot act on Admins or other Moderators.
- Admins can kick/ban Members and Moderators, but not the Owner or other Admins.
- Only the Owner (or Instance Owner) can act on Admins.

For the complete permissions matrix, see the [Permissions Reference](./permissions-reference.md).

## Moderation Philosophy

Chatalot is designed around an **escalating response** model:

```
Warning  -->  Timeout  -->  Kick  -->  Ban
(lightest)                          (most severe)
```

### When to Warn

- First offense for minor rule violations
- Off-topic behavior in a focused channel
- Mild language or tone issues
- Accidental disruptions

### When to Timeout

- Repeated warnings that go unheeded
- Heated arguments that need a cool-down period
- Spam or flooding behavior

### When to Kick

- Persistent rule violations after warnings and timeouts
- Users who are disrupting the community but may reform
- Situations where a "fresh start" via rejoin might help

### When to Ban

- Severe violations (harassment, hate speech, illegal content)
- Users who rejoin after being kicked and continue violating rules
- Spam bots or malicious accounts
- Any situation where the user should permanently lose access

## Quick Reference: How to Take Action

| Action | How to Access |
|--------|---------------|
| Warn a user | Click the user's profile card in a channel, or use the community member management panel |
| Timeout a user | Click the user's profile card, then select **Timeout** and choose a duration |
| Kick a member | Open the community settings members tab, click **Kick** next to the member |
| Ban a member | Open the community settings members tab, click **Ban** next to the member |
| Delete a message | Right-click (or long-press) a message and select **Delete** |
| Report a message | Right-click (or long-press) a message and select **Report** |

## Next Steps

- [Warnings](./warnings.md) -- Learn how to issue and track warnings
- [Timeouts](./timeouts.md) -- Understand timeout durations and how to apply them
- [Kicks and Bans](./kicks-and-bans.md) -- Remove or permanently exclude users
