# Status and Presence

> **Status: Complete**

How to set your online status and how Chatalot's presence system works.

## Status Options

Chatalot supports five presence statuses:

| Status | Indicator | Description |
|--------|-----------|-------------|
| **Online** | Green dot | You are active and available. This is the default when you connect. |
| **Idle** | Yellow dot | You are away from your keyboard. Can be set manually or triggered automatically. |
| **Do Not Disturb** | Red dot | You are busy. Suppresses all notifications. |
| **Invisible** | Gray dot (you see hollow circle) | You appear offline to others but can still use Chatalot normally. |
| **Offline** | Gray dot / no dot | You are not connected. Set automatically when you close Chatalot or lose connection. |

## How to Change Your Status

### Using the Status Picker

1. In the main channel view, locate the status picker in the sidebar (near your user info at the bottom).
2. Click to open the status picker dropdown.
3. Select a status: Online, Idle, Do Not Disturb, or Invisible.

Your selection is saved to local storage and restored when you reconnect.

### What Happens When You Change Status

When you select a status:

1. A `update_presence` message is sent to the server via WebSocket.
2. The server broadcasts your new status to all users who share a community with you.
3. Your status is persisted locally so it is restored on reconnect.

> **Tip:** Setting your status to Invisible lets you read messages and participate without others knowing you are online.

## Automatic Status Changes

### Idle Detection

Chatalot includes automatic idle detection:

- After **5 minutes** of inactivity (no mouse movement, keyboard input, touch, or scroll), your status is automatically changed from Online to **Idle**.
- When you interact with the page again, your status is automatically restored to **Online**.
- Idle detection only activates if your manually-set status is "Online." If you manually set your status to Idle, DND, or Invisible, idle detection does not override it.

### Connection-Based Presence

- When you connect via WebSocket, the server broadcasts your status as **Online** (unless you have a saved status other than Online).
- When your WebSocket connection closes (browser closed, network lost, etc.) and you have no other active sessions, the server broadcasts you as **Offline**.
- Chatalot supports up to **8 concurrent sessions** per user (multi-device). You remain online as long as at least one session is connected.

## Do Not Disturb Behavior

When your status is set to **Do Not Disturb (DND)**:

- Desktop notifications are suppressed.
- Sound notifications are suppressed.
- Other users see a red DND indicator on your avatar, signaling that you prefer not to be disturbed.
- You can still receive and read messages normally -- only the notification delivery is affected.

See [Do Not Disturb](../notifications/do-not-disturb.md) for full details on which notifications are affected.

## Presence Visibility

Your presence status is only visible to users who share at least one community with you. This is a privacy measure -- strangers or users in unrelated communities cannot see your online status.

### Where Status Is Shown

- **Members panel** -- An indicator dot next to each member's name.
- **Profile card** -- Shown when you click a user's name.
- **DM list** -- Status indicators appear next to DM contacts.

## Custom Status vs. Presence Status

These are two different features:

| Feature | Example | Where to set |
|---------|---------|-------------|
| **Presence status** | Online, Away, DND, Invisible | Status picker in sidebar |
| **Custom status** | "In a meeting" | Settings > Profile > Custom Status |

Custom status is a free-text message (up to 128 characters). Presence status is one of the predefined states. Both are visible to other users on your profile card.

## Related Pages

- [Your Profile](./your-profile.md) -- editing your custom status and other profile fields
- [Viewing Profiles](./viewing-profiles.md) -- seeing other users' status
- [Do Not Disturb](../notifications/do-not-disturb.md) -- notification suppression details
