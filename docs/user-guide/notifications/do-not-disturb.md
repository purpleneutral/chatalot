# Do Not Disturb

> **Status: Complete**

How Do Not Disturb (DND) mode works in Chatalot and what it suppresses.

## What DND Does

When your status is set to **Do Not Disturb**, Chatalot suppresses active notification delivery:

| Notification Type | Behavior with DND |
|-------------------|-------------------|
| **Desktop notifications** | Suppressed -- no OS-level popups appear |
| **Sound notifications** | Suppressed -- no tones play for messages, mentions, or voice events |
| **Unread badges** | **Still shown** -- channel badge counts and bold names continue to update |
| **Unread separators** | **Still shown** -- the "New Messages" line still appears when you open a channel |
| **Message delivery** | **Not affected** -- all messages are still delivered and stored; you just are not actively notified |

In short, DND silences sounds and popups but does not hide any information. When you return from DND, all unread indicators accurately reflect what you missed.

## How to Enable DND

### Using the Status Picker

1. Open the **status picker** in the sidebar (near your user info area).
2. Select **Do Not Disturb**.
3. Your presence indicator changes to a red dot, visible to other users.

Your DND status is saved locally and persisted across page reloads and reconnections.

### When Does DND End?

DND stays active until you manually change your status to something else (Online, Idle, or Invisible). There is no automatic timer or schedule for DND -- you must explicitly turn it off.

To turn off DND:

1. Open the status picker.
2. Select a different status (e.g., **Online**).

## Visual Indicator to Others

When you are in DND mode, other users see:

- A **red dot** on your avatar in the members panel, DM list, and profile card.
- This signals that you prefer not to be disturbed and may not respond immediately.

> **Note:** DND is a social signal as well as a functional setting. Other users can still send you messages -- the messages just will not produce notifications on your end.

## Which Notifications Are Still Shown

Even in DND mode, the following continue to work normally:

- **Unread badge counts** in the sidebar update in real time.
- **Bold channel names** for channels with unread messages.
- **Unread separator lines** appear when you open a channel.
- **Typing indicators** still show when others are typing.
- **Voice state updates** still show who is in voice channels.
- **Messages appear in real time** if you have a channel open.

The only things suppressed are the active interruptions: sounds and OS-level notification popups.

## DND and Idle Detection

If your status is DND, Chatalot's idle detection does **not** override it. Even after 5 minutes of inactivity, your status remains DND (it does not switch to Idle). Idle detection only activates when your status is set to Online.

## DND Across Devices

Your DND status is stored locally per browser/device and also broadcast via WebSocket. If you set DND on one device:

- Other users see you as DND regardless of which device you set it on.
- Your other devices restore their saved status independently on reconnect, so you may want to set DND on each device where you want notifications suppressed.

## Related Pages

- [Status and Presence](../profiles-and-presence/status-and-presence.md) -- all presence statuses explained
- [Notification Settings](./notification-settings.md) -- configuring what notifications you receive
- [Unread Indicators](./unread-indicators.md) -- visual indicators that remain active during DND
