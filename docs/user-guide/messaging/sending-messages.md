# Sending Messages

> **Status: Complete**

Messages are sent through the message input area at the bottom of any text channel or DM. Messages are encrypted on your device before being transmitted over WebSocket to the server.

## Typing and Sending

1. Click in the message input area at the bottom of the channel (or press `Ctrl+T` to focus it).
2. Type your message.
3. Press **Enter** to send.

### Send Behavior

The send behavior is configurable in Settings:

| Mode | Send | New Line |
|------|------|----------|
| **Enter to send** (default) | `Enter` | `Shift+Enter` |
| **Ctrl+Enter to send** | `Ctrl+Enter` or `Cmd+Enter` | `Enter` |

> **Tip:** You can change the send behavior in **Settings > Preferences > Send Behavior**.

### Message Input Auto-Resize

The message input area automatically grows as you type multi-line messages, up to a maximum height. It resets after sending.

## Message Size Limits

- Maximum message size: **64 KiB** of ciphertext (approximately 64,000 characters of plaintext).
- Messages cannot be empty.
- If a message exceeds the size limit, the server rejects it with an error.

## @Mentions

Type `@` followed by a username to mention a user. An autocomplete popup appears as you type, showing matching members from the current channel.

### User Mentions

- Start typing `@` followed by a username or display name.
- Use arrow keys to navigate the autocomplete list.
- Press **Enter** or **Tab** to insert the mention.
- The mentioned user receives a notification.

### Special Mentions

| Mention | Description |
|---------|-------------|
| `@everyone` | Notify all members of the channel |
| `@here` | Notify all currently online members |
| `@channel` | Notify all channel members |

## Emoji Autocomplete

Type `:` followed by an emoji name (at least 2 characters) to trigger emoji autocomplete. This searches both standard Unicode emojis and custom community emojis.

- Use arrow keys to navigate results.
- Press **Enter** or **Tab** to insert the emoji.
- Up to 8 results are shown (custom emojis are prioritized).

## Replying to Messages

To reply to a specific message:

1. Right-click the message and select **Reply**, or hover over the message and click the reply icon.
2. A reply preview bar appears above the message input showing the original message.
3. Type your reply and send as usual.
4. The reply appears with a reference to the original message.

To cancel a reply, click the **X** button on the reply preview bar.

## File Uploads

You can share files in any text channel or DM:

- **Click the attachment button** (paperclip icon) in the message input area to browse for a file.
- **Drag and drop** a file directly onto the channel to upload it.
- **Paste an image** from your clipboard into the message input.

A file message is created with the file name and size. Images are displayed inline with a lightbox viewer.

## Slash Commands

Chatalot supports several slash commands that transform your message before sending:

| Command | Output |
|---------|--------|
| `/me action` | *action* (italicized, like an action/emote) |
| `/shrug` | Appends the shrug emoticon |
| `/tableflip` | Appends the table flip emoticon |
| `/unflip` | Appends the table unflip emoticon |
| `/lenny` | Appends the Lenny face |

You can also include text after the command. For example, `/shrug oh well` produces: `oh well` followed by the shrug emoticon.

## Typing Indicators

When you start typing, other users in the channel see a typing indicator. The indicator is sent once and then suppressed for 3 seconds to avoid flooding.

Typing indicators appear below the message list, showing which users are currently typing.

## Draft Persistence

If you switch channels while composing a message, your draft is automatically saved to local storage. When you return to that channel, the draft is restored.

## Optimistic Sending

Messages appear in the channel immediately after you press send (marked as "pending"). Once the server confirms delivery, the message is updated with its final ID and timestamp. If the connection is lost, the pending message is removed and the text is restored to the input area.

## Encryption

All messages are encrypted before leaving your device:

- **DM channels:** Messages are encrypted using the Double Ratchet protocol (per-recipient key).
- **Group channels:** Messages are encrypted using Sender Keys (one key for the entire group).
- If encryption fails for any reason, the message falls back to plaintext with a warning logged to the console.

## Related Pages

- [Formatting](./formatting.md) -- Markdown and formatting shortcuts
- [Reactions](./reactions.md) -- Adding emoji reactions
- [Threads](./threads.md) -- Replying in threads
- [Scheduled Messages](./scheduled-messages.md) -- Sending messages later
