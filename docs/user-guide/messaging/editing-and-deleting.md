# Editing and Deleting Messages

> **Status: Complete**

Chatalot allows you to edit and delete your own messages, and provides moderation tools for admins to manage other users' messages.

## Editing Messages

### How to Edit

There are two ways to start editing a message:

1. **Right-click context menu:** Right-click on your message and select **Edit**.
2. **Arrow Up shortcut:** When the message input is empty, press the **Up Arrow** key to edit your most recent message in the channel.

When editing:

- The message content is replaced with an editable text input.
- Press **Enter** to save your edit.
- Press **Escape** to cancel without saving.

### Edit Window

Messages can only be edited within **15 minutes** of being sent. After that window closes, the edit option is no longer available.

> The 15-minute limit is enforced server-side. Even if the client shows an edit option, the server will reject edits on messages older than 15 minutes.

### "Edited" Indicator

When a message is edited, an **(edited)** label appears next to the timestamp. This is visible to all channel members.

### Edit History

Every edit is recorded. Any channel member can view the full edit history of a message:

1. Right-click on an edited message.
2. Select **View Edit History**.
3. A modal displays all previous versions of the message with timestamps.

Edit history preserves the old ciphertext, so previous versions can be decrypted and viewed.

## Deleting Messages

### Deleting Your Own Messages

1. Right-click on your message.
2. Select **Delete**.
3. Confirm the deletion in the dialog that appears.

Deleted messages are permanently removed and cannot be recovered.

### Deleting Others' Messages

Admins and owners can delete messages from other users:

1. Right-click on the message.
2. Select **Delete**.
3. Confirm the deletion.

> **Permission:** Only users with **admin** or **owner** role can delete other users' messages. See [Channel Permissions](../channels/channel-permissions.md) for the full permission matrix.

### What Happens When a Message Is Deleted

- The message is removed from the channel for all users in real time (via WebSocket broadcast).
- If the deleted message was a **thread root**, the thread panel closes.
- If the deleted message had **replies**, those replies remain but the "replying to" reference shows the message as unavailable.
- If the message was **pinned**, the pin is also removed.

## Message Context Menu

Right-click on any message to access the context menu with the following options:

| Action | Availability |
|--------|-------------|
| **Reply** | All members |
| **Edit** | Own messages only (within 15 min) |
| **Delete** | Own messages, or any message for admins/owners |
| **Pin / Unpin** | Admins and owners |
| **Bookmark** | All members |
| **Copy Text** | All members |
| **View Edit History** | All members (on edited messages) |
| **React** | All members |
| **Start Thread** | All members |
| **Report** | All members |

## Related Pages

- [Sending Messages](./sending-messages.md)
- [Pins and Bookmarks](./pins-and-bookmarks.md)
- [Channel Permissions](../channels/channel-permissions.md)
