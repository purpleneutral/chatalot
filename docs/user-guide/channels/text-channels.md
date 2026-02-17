# Text Channels

> **Status: Complete**

Text channels are the primary way to communicate within a Group. Each text channel has a persistent message history, supports rich formatting, and provides features like reactions, threads, pins, and file sharing.

![Chat conversation](../../screenshots/01-chat-conversation.png)

## Creating a Text Channel

To create a new text channel inside a Group:

1. Expand the Group in the sidebar by clicking its name.
2. Click the **+** button next to the Group name.
3. Enter a channel name and select **Text** as the channel type.
4. Optionally provide a topic (description).
5. Click **Create**.

> **Permission:** Only Group owners and admins can create channels.

### Channel Name Format

- Channel names can be 1 to 64 characters long.
- Names are displayed as-is in the sidebar (lowercase with hyphens is conventional but not enforced).
- Names must be unique within a Group.

## Message History

When you open a text channel, the most recent 50 messages are loaded automatically. As you scroll upward, older messages are fetched in batches of 50 (infinite scroll).

- **Scroll position** is preserved per channel -- switching away and back restores your position.
- A **scroll-to-bottom** button appears when you are scrolled up from the latest messages.
- **Unread separators** mark where new messages begin when you return to a channel with unread messages.

## Channel Topic

Each channel can have an optional topic that appears in the channel header. Topics can be up to 512 characters.

To edit the topic:

1. Click the topic text in the channel header (or click the pencil icon).
2. Type the new topic.
3. Press **Enter** to save or **Escape** to cancel.

> **Permission:** Only Group owners and admins can edit channel topics.

## Read-Only Mode

Admins can set a channel to **read-only**, which prevents regular members from sending messages. Admins and owners are exempt from this restriction and can still post.

## Slow Mode

Slow mode limits how often non-privileged members can send messages. Available intervals:

| Setting | Cooldown |
|---------|----------|
| Off | No limit |
| 5s | 5 seconds between messages |
| 10s | 10 seconds |
| 30s | 30 seconds |
| 1m | 1 minute |
| 5m | 5 minutes |
| 10m | 10 minutes |

A countdown timer appears in the message input area when slow mode is active.

> **Permission:** Admins and owners are exempt from slow mode.

## Archived Channels

Channels can be **archived** to prevent any new messages. Archived channels remain visible in the sidebar but cannot receive new content from any user, including admins.

## Message TTL (Auto-Delete)

Admins can configure a **message TTL** (Time To Live) on a channel. Messages older than the TTL are automatically deleted by the server. The TTL can be set from 0 (disabled) up to 2,592,000 seconds (30 days).

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+K` | Open Quick Switcher to jump to any channel or DM |
| `Ctrl+F` | Search messages in the current channel |
| `Ctrl+T` | Focus the message input box |
| `End` | Scroll to the latest message |
| `Home` | Scroll to the top of loaded messages |
| `Shift+Esc` | Mark all channels as read |
| `Esc` | Close the current panel or modal |

## Related Pages

- [Sending Messages](../messaging/sending-messages.md)
- [Voice Channels](./voice-channels.md)
- [Channel Settings](./channel-settings.md)
- [Channel Permissions](./channel-permissions.md)
