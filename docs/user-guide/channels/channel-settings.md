# Channel Settings

> **Status: Complete**

Channel settings allow authorized users to configure a channel's name, topic, moderation options, and more. Settings are accessed by right-clicking a channel in the sidebar and selecting the settings option.

## Accessing Channel Settings

1. Right-click on a channel name in the sidebar.
2. The Channel Settings card appears as a popup near the click location.

Alternatively, admins can edit the channel topic directly by clicking the topic area in the channel header.

## Available Settings

### Name

- Channel names can be between 1 and 64 characters.
- To rename: click the name in the settings card, edit it, and press Enter.

> **Permission:** Only the channel/Group **owner** can rename a channel (the `can_manage_roles` permission, which requires owner role level).

### Topic (Description)

- The topic appears below the channel name in the header area.
- Topics can be up to 512 characters.
- To edit: click the topic area in the settings card, or click the topic in the channel header.

### Channel Type

Channels are created as either **text** or **voice**. The type cannot be changed after creation.

### Read-Only

When enabled, only admins and owners can send messages. Regular members can still read the channel.

- Toggle via the **Read-only** switch in the settings card.
- Useful for announcement channels where only moderators should post.

### Slow Mode

Limits the rate at which non-privileged members can send messages. Options:

| Label | Seconds |
|-------|---------|
| Off | 0 |
| 5s | 5 |
| 10s | 10 |
| 30s | 30 |
| 1m | 60 |
| 5m | 300 |
| 10m | 600 |

Admins and owners are exempt from slow mode.

### Discoverable

When enabled, the channel appears in discovery listings for users browsing available channels.

### Archived

Archived channels cannot receive new messages from anyone. They remain visible but are effectively frozen. Archiving is reversible.

### Voice Background (Voice Channels Only)

Voice channels can have a custom background image that appears behind the video grid during calls. Upload an image via the channel settings card.

### Webhooks

Each channel can have webhooks configured for external integrations. See [Webhooks](../webhooks/README.md) for details.

## Deleting a Channel

From the Channel Settings card, click **Delete Channel**. You will be prompted to confirm. This action is permanent and removes all messages in the channel.

> **Permission:** Only the Group **owner** can delete channels.

## Copying Channel ID

Click **Copy ID** in the settings card to copy the channel's unique identifier to your clipboard. This is useful for API integrations and debugging.

## Who Can Edit Channel Settings

| Setting | Required Role |
|---------|--------------|
| Rename | Owner |
| Edit topic | Admin or Owner |
| Read-only toggle | Admin or Owner |
| Slow mode | Admin or Owner |
| Archive/Unarchive | Admin or Owner |
| Discoverable toggle | Admin or Owner |
| Voice background | Admin or Owner |
| Delete channel | Owner |
| Manage webhooks | Admin or Owner |

## Related Pages

- [Channel Permissions](./channel-permissions.md)
- [Text Channels](./text-channels.md)
- [Voice Channels](./voice-channels.md)
