# Unread Indicators

> **Status: Complete**

How Chatalot shows you which channels have new messages, how many, and where the unread messages begin.

## Unread Channel Indicators

The sidebar uses two visual cues to show channels with unread messages:

### Bold Channel Name

When a channel has unread messages, its name appears in **bold white text** (instead of the default muted secondary color). This makes unread channels easy to spot at a glance.

### Badge Count

A numeric badge appears to the right of the channel name showing the number of unread messages:

- The badge shows the exact count up to 99.
- For 100 or more unread messages, the badge shows **99+**.
- The badge is hidden for the currently active channel (since you are already reading it).
- The badge disappears when you open the channel and scroll to the bottom.

### Where Badges Appear

Unread badges appear in three places:

| Location | What it shows |
|----------|--------------|
| **Community channels** | Unread count per channel within each community's channel list |
| **Ungrouped channels** | Unread count per channel in the ungrouped section |
| **Direct Messages** | Unread count per DM conversation |

## Mention Highlights

When someone mentions you with `@username` in a message:

- The mention text is highlighted with a distinct color in the message body.
- A mention-specific notification sound plays (if enabled in **Settings > Notifications**).
- A desktop notification appears (if enabled and the tab is not focused).

Mentions are tracked as part of the unread count -- there is no separate "mentions" counter in the sidebar.

## Unread Separator

When you open a channel that has unread messages, an **unread separator line** appears in the message list marking where your last read position was. All messages below the separator are ones you have not yet seen.

- The separator text typically says "New Messages" or similar visual divider.
- It appears only when there are unread messages and you have previously read messages in that channel.
- Scrolling to the bottom of the channel clears the unread state.

## Mark as Read

### Automatic

Messages are automatically marked as read when:

1. You open a channel and view its messages.
2. You scroll to the bottom of a channel while new messages arrive.
3. A `mark_read` message is sent to the server with the latest message ID you have seen.

### Manual: Mark All as Read

To clear all unread indicators across every channel at once:

- Press **Shift+Esc** anywhere in the app.
- Or click the **Mark all read** button in the sidebar header.

This sends a `mark_all_read` command to the server, which resets your unread position for every channel to the latest message.

## Read Receipts

Chatalot supports read receipts that let other users see when you have read their messages:

- When you view a message, a read receipt is broadcast to the channel.
- Other participants can see who has read up to which message.
- In DMs, you can see the exact message the other person has read up to.

### Disabling Read Receipts

If you prefer not to share your read status:

1. Go to **Settings > Chat > Display**.
2. Toggle **Send read receipts** off.

When disabled, your read position is still tracked for your own unread indicators, but the receipt is not broadcast to others.

## Community-Level Indicators

While Chatalot does not show a single aggregate badge on each community, you can quickly identify communities with unread activity by scanning the channel list within each community -- any bold channel name or visible badge indicates unread content.

> **Tip:** Use **Shift+Esc** to clear all unreads at once when you are catching up after being away. This resets every channel to "read" without needing to open each one individually.

## Related Pages

- [Notification Settings](./notification-settings.md) -- configuring sounds and desktop notifications
- [Do Not Disturb](./do-not-disturb.md) -- suppressing notification delivery
