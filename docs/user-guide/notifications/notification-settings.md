# Notification Settings

> **Status: Complete**

How to configure sound notifications, desktop notifications, and per-channel overrides in Chatalot.

## Where to Configure

Notification settings are split across two locations:

| Location | What you configure |
|----------|-------------------|
| **Settings > Notifications** | Sound toggles, sound volume, desktop notification toggle, default channel notification level |
| **Channel header (bell icon)** | Per-channel notification level override |

## Sound Notifications

Sound notifications are short synthesized tones that play when certain events occur. Configure them at **Settings > Notifications > Sounds**.

### Available Sound Events

| Event | Default | Description |
|-------|---------|-------------|
| **DM message** | On | Plays a two-tone ascending sound when you receive a direct message |
| **Channel message** | Off | Plays a single tone for messages in channels |
| **Mention** | On | Plays a three-tone ascending sound when someone @mentions you |
| **Voice join/leave** | On | Plays an ascending tone when someone joins voice and a descending tone when they leave |

Each event has a toggle switch and a **Test** button so you can hear the sound before enabling it.

### Sound Volume

A global volume slider controls the loudness of all notification sounds. The range is 0% to 100%, with a default of 50%.

> **Tip:** Sound notifications are generated using the Web Audio API (synthesized tones), not audio files. They work even without an internet connection.

## Desktop Notifications

Desktop notifications are OS-level popups that appear outside the browser window. Configure them at **Settings > Notifications > Desktop Notifications**.

### Enabling Desktop Notifications

1. Toggle the **Desktop notifications** switch to On.
2. Your browser will prompt you to allow notifications from the Chatalot site.
3. Once granted, notifications will appear even when the Chatalot tab is in the background.

### Permission States

| State | Meaning |
|-------|---------|
| Toggle available | Browser has not yet been asked, or permission was previously granted |
| "Blocked by browser" (red text) | You previously denied the permission. Reset it in your browser's site settings. |
| "Not supported" | Your browser does not support the Notification API |

### What Triggers Desktop Notifications

Desktop notifications are controlled by two layers:

1. **Per-event preferences** (in **Settings > Notifications**):
   - DM notifications (on by default)
   - @mention notifications (on by default)
   - Channel message notifications (off by default)

2. **Per-channel level** (see below) -- overrides the defaults for specific channels.

Desktop notifications only fire when the Chatalot tab is **not focused** (page hidden). If you are actively looking at the chat, only visual and sound notifications are used.

### Notification Behavior

When a desktop notification appears:

- **Title:** The sender's display name (or channel name).
- **Body:** A preview of the message content.
- **Click:** Clicking the notification brings the Chatalot tab to focus and navigates to the relevant channel.
- **Tag:** Notifications are tagged by channel ID, so multiple messages in the same channel replace the previous notification rather than stacking.
- **Sound:** Desktop notifications are set to `silent: true` to avoid double-playing with Chatalot's own sound notifications.

## Default Channel Notification Level

At **Settings > Notifications > Desktop Notifications**, you can set the default notification level for all channels:

| Level | Behavior |
|-------|----------|
| **All messages** | Notify for every message in every channel (default) |
| **Only @mentions** | Only notify when you are directly mentioned with @username |
| **Nothing** | No desktop notifications for any channel |

This default applies to all channels unless overridden per-channel.

## Per-Channel Notification Overrides

You can override the notification level for individual channels:

1. Open the channel you want to configure.
2. Click the **bell icon** in the channel header toolbar.
3. Select a level from the dropdown:
   - **All Messages** -- Notify for every message
   - **Only @mentions** -- Only notify for @mentions
   - **Nothing** -- No notifications for this channel

A checkmark indicates the currently active level. The bell icon shows a slash when the channel is set to "Nothing."

> **Tip:** This is useful for muting busy channels while keeping notifications active for important ones.

Per-channel overrides are available for community channels only (not DMs). When a channel is set to the same level as your global default, the override is automatically removed.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Shift+Esc` | Mark all channels as read |

## Related Pages

- [Unread Indicators](./unread-indicators.md) -- visual indicators for unread messages
- [Do Not Disturb](./do-not-disturb.md) -- suppressing all notifications
