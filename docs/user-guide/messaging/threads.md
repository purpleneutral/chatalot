# Threads

> **Status: Complete**

Threads let you have focused conversations about a specific message without cluttering the main channel. Thread replies are grouped together in a dedicated panel.

## Starting a Thread

1. Right-click on a message in the channel.
2. Select **Start Thread** (or **Reply in Thread**).
3. The thread panel opens on the right side of the screen, showing the original message at the top.
4. Type your reply in the thread message input at the bottom of the panel.
5. Press **Enter** to send.

## Thread Panel

The thread panel displays:

- **Root message:** The original message that started the thread, shown at the top.
- **Thread replies:** All replies to that message, in chronological order.
- **Message input:** A text input at the bottom for composing replies.
- **Close button:** Click to close the thread panel.

> **Tip:** Press `Esc` to close the thread panel.

## Thread Indicators in the Channel

When a message has thread replies, a thread indicator appears below the message in the main channel showing:

- The number of replies (e.g., "3 replies")
- The time of the most recent reply

Click the thread indicator to open the thread panel for that message.

## Thread Features

Thread messages support all the same features as regular channel messages:

| Feature | Supported |
|---------|-----------|
| Reactions | Yes |
| Editing | Yes (within 15-minute window) |
| Deleting | Yes |
| Markdown formatting | Yes |
| File sharing | Not directly (use the main channel) |

## Thread Notifications

When someone replies to a thread you have participated in:

- A real-time WebSocket event delivers the reply to your thread panel (if open).
- Thread reply count and timestamp update on the root message in the main channel.
- If the thread panel is closed, you will see the updated reply count on the root message.

## Nested Threads

Threads are always one level deep. If you reply to a message that is itself a thread reply, your reply is placed in the same thread as the root message -- it does not create a nested thread.

## What Happens When the Root Message Is Deleted

If the root message of a thread is deleted:

- The thread panel automatically closes.
- Thread replies are no longer accessible through the thread indicator.

## Closing the Thread Panel

The thread panel is mutually exclusive with other side panels (member list, bookmarks, scheduled messages). Opening a different panel automatically closes the thread panel, and vice versa.

## Related Pages

- [Sending Messages](./sending-messages.md)
- [Reactions](./reactions.md)
- [Editing and Deleting](./editing-and-deleting.md)
