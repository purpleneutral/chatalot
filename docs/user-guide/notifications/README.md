# Notifications

How Chatalot keeps you informed about new messages, mentions, and activity -- and how to control what gets your attention.

## In This Section

| Page | Description |
|------|-------------|
| [Notification Settings](./notification-settings.md) | Global and per-channel notification configuration |
| [Unread Indicators](./unread-indicators.md) | Badge counts, bold text, mention highlights, and mark-as-read |
| [Do Not Disturb](./do-not-disturb.md) | Suppressing notifications when you need focus |

## Overview

Chatalot has three layers of notification:

1. **Visual indicators** -- Unread badges, bold channel names, and unread separators in the message list. These are always active.
2. **Sound notifications** -- Short tonal sounds for DMs, channel messages, mentions, and voice join/leave events. Configurable per event type.
3. **Desktop notifications** -- OS-level popup notifications (e.g., macOS Notification Center, Windows toast, Linux notification daemon). Requires browser permission.

All notification settings are stored locally and synced to the server as part of your preferences, so they follow you across devices.

## Quick Reference

| What happened | Visual | Sound | Desktop |
|--------------|--------|-------|---------|
| New message in channel | Unread badge + bold name | Optional (off by default) | Optional (off by default) |
| New direct message | Unread badge + bold name | On by default | On by default |
| @mention | Unread badge + bold name | On by default | On by default |
| Someone joined voice | -- | On by default | -- |
| Someone left voice | -- | On by default | -- |

## Related Sections

- [Profiles and Presence](../profiles-and-presence/README.md) -- Do Not Disturb is a presence status
- [Channels](../channels/README.md) -- per-channel notification overrides
