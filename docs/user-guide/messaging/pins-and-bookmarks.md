# Pins and Bookmarks

> **Status: Complete**

Chatalot provides two ways to save important messages: **pins** (shared with the channel) and **bookmarks** (personal and private).

## Pinned Messages

Pinned messages are visible to all members of a channel. Use pins to highlight important information, announcements, or frequently referenced content.

### Pinning a Message

1. Right-click on a message.
2. Select **Pin Message**.
3. The message is pinned and a notification is broadcast to the channel.

> **Permission:** Only **admins** and **owners** can pin and unpin messages.

### Viewing Pinned Messages

1. Click the **pin icon** in the channel header bar.
2. The pinned messages panel opens below the header, showing all pinned messages in the channel.
3. Each pinned message shows the sender, content, timestamp, and who pinned it.

### Unpinning a Message

1. Open the pinned messages panel.
2. Click the **unpin** button on the message you want to unpin.
3. The message is removed from the pinned list.

Alternatively, right-click a pinned message in the channel and select **Unpin**.

### Pin Limits

- Each channel can have a maximum of **50 pinned messages**.
- The current pin count is displayed in the pinned messages panel header (e.g., "12/50").
- When approaching the limit (45+ pins), a warning is included in the success toast.
- If you try to pin a 51st message, you will receive an error asking you to unpin a message first.

### Pin Rules

- You cannot pin a deleted message.
- A message can only be pinned once (attempting to pin an already-pinned message shows an error).
- Pin status is synced in real time -- all channel members see pins and unpins immediately.

## Personal Bookmarks

Bookmarks are private to you. Other users cannot see what you have bookmarked. Use bookmarks to save messages for your own reference.

### Adding a Bookmark

1. Right-click on a message.
2. Select **Bookmark**.
3. The message is saved to your bookmarks.

You can optionally add a **note** to a bookmark (up to 500 characters) to remind yourself why you saved it.

### Viewing Bookmarks

1. Click the **bookmark icon** in the channel header toolbar.
2. The bookmarks panel opens on the right side, showing all your bookmarked messages.
3. Each bookmark shows the message content, timestamp, and your note (if any).

### Removing a Bookmark

1. Open the bookmarks panel.
2. Click the **remove** button next to the bookmark you want to delete.

Alternatively, right-click a bookmarked message in the channel and select **Remove Bookmark** (the bookmark option toggles between add and remove).

### Bookmark vs. Pin

| Feature | Pin | Bookmark |
|---------|-----|----------|
| Visibility | All channel members | Only you |
| Who can create | Admins and owners | Any member |
| Limit | 50 per channel | No limit |
| Notes | No | Yes (up to 500 characters) |
| Location | Channel-specific | Global (all your bookmarks) |

## Related Pages

- [Editing and Deleting](./editing-and-deleting.md)
- [Search](./search.md)
- [Channel Permissions](../channels/channel-permissions.md)
