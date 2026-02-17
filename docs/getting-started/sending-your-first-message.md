# Sending Your First Message

> **Status: Complete**

This page covers the basics of composing and sending messages in Chatalot.

## Navigate to a Channel

Before you can send a message, you need to be in a text channel:

1. Click the channel name dropdown in the header to open the navigation sidebar.
2. Under the **Groups** tab, expand a group to see its channels.
3. Click on a text channel (prefixed with `#`) to open it.

The channel name and a green indicator appear in the sub-header to confirm you are in the right place.

## Composing a Message

Click the message input at the bottom of the screen. The placeholder text reads "Message #channel-name..." to show which channel you are typing in.

### Sending

By default, press **Enter** to send your message. The message appears immediately in the chat (optimistic sending) while it is delivered to the server in the background.

### New Lines

Press **Shift+Enter** to insert a new line without sending.

> **Tip:** You can swap this behavior in Settings > Chat. With the alternative "Ctrl+Enter to send" mode, Enter inserts a new line and Ctrl+Enter (or Cmd+Enter on Mac) sends the message.

### Drafts

If you switch channels before sending, your in-progress message is automatically saved as a draft. When you return to that channel, the draft is restored.

## Formatting

Chatalot supports Markdown formatting for rich text messages. You can type Markdown directly or use keyboard shortcuts:

| Style | Markdown | Shortcut | Example |
|-------|----------|----------|---------|
| **Bold** | `**text**` | `Ctrl+B` | **bold text** |
| *Italic* | `_text_` | `Ctrl+I` | _italic text_ |
| `Code` | `` `text` `` | `Ctrl+E` | `inline code` |
| ~~Strikethrough~~ | `~~text~~` | -- | ~~struck text~~ |
| Link | `[text](url)` | -- | [link text](https://example.com) |

For code blocks, wrap your code in triple backticks with an optional language name:

````
```rust
fn main() {
    println!("Hello, world!");
}
```
````

Chatalot supports syntax highlighting for many languages, including JavaScript, TypeScript, Python, Rust, Go, Java, C/C++, CSS, HTML, JSON, YAML, SQL, Bash, and Markdown.

A formatting toolbar is also available below the message input with buttons for Bold, Italic, Strikethrough, Code, and Link.

## Mentioning Users

Type `@` followed by a username to mention someone. An autocomplete popup appears as you type, showing matching users from the current channel. Select a user from the list to insert the mention.

Mentioned users receive a notification (if they have notifications enabled).

## Replying to Messages

To reply to a specific message:

1. Hover over the message you want to reply to.
2. Click the **reply** arrow icon in the message action bar.
3. A reply preview appears above the input, showing the original message.
4. Type your reply and press Enter to send.

Click the **X** on the reply preview to cancel.

## Adding Reactions

Reactions let you respond to a message with an emoji without typing a reply.

1. Hover over a message to reveal the action bar.
2. Click the **smiley face** icon to open the emoji picker.
3. Select an emoji to add your reaction.

Reactions appear as small chips below the message, showing the emoji and the number of users who reacted. Click an existing reaction chip to toggle your own reaction on or off.

## Uploading Files

Click the **paperclip** icon to the left of the message input to attach a file. You can also **drag and drop** files directly into the chat area.

Uploaded images are displayed inline as previews. Click on an image to open it in the lightbox viewer, where you can navigate between images in the channel.

## Sending GIFs

Click the **GIF** button next to the message input to open the GIF picker. You can browse trending GIFs or search by keyword. Click a GIF to send it immediately.

## Editing Your Messages

To edit a message you already sent:

- **Keyboard shortcut**: Press the **Up Arrow** key when the message input is empty to start editing your most recent message.
- **Action bar**: Hover over your message and click the **pencil** icon.

Make your changes and press Enter to save, or Escape to cancel.

## Deleting Messages

Hover over your message and click the **trash** icon in the action bar. You will be asked to confirm before the message is deleted.

## Threads

To start or continue a threaded conversation:

1. Hover over a message and click the **reply** icon (or the thread icon if a thread already exists).
2. A thread panel opens on the right side with the original message and its replies.
3. Type your reply in the thread input and press Enter.

Threads keep detailed discussions organized without cluttering the main channel.

## Next Step

Ready to talk live? Continue to [Joining a Voice Call](./joining-a-voice-call.md).
