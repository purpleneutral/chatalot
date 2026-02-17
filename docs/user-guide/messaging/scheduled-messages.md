# Scheduled Messages

> **Status: Complete**

Scheduled messages let you compose a message now and have it sent automatically at a future date and time.

## Scheduling a Message

1. Type your message in the message input area.
2. Click the **clock icon** (schedule button) next to the send button to open the schedule picker.
3. Select a **date** and **time** for the message to be sent.
4. Click **Schedule** to confirm.

The message is saved on the server and will be delivered to the channel at the scheduled time.

## Limits

| Constraint | Limit |
|------------|-------|
| Maximum time in advance | **30 days** |
| Maximum scheduled messages per user | **50** |
| Minimum time | Must be in the future |

- You cannot schedule a message for a time in the past.
- If you have reached the 50-message limit, you must cancel an existing scheduled message before creating a new one.

## Viewing Scheduled Messages

1. Click the **clock icon** in the channel header toolbar to open the Scheduled Messages panel.
2. The panel displays all your pending scheduled messages across all channels.
3. Each entry shows the channel it will be sent to and the scheduled delivery time.

## Cancelling a Scheduled Message

1. Open the Scheduled Messages panel.
2. Click the **Cancel** button next to the message you want to remove.
3. The message is permanently deleted and will not be sent.

## How It Works

- When you schedule a message, the text (ciphertext) and nonce are stored on the server.
- At the scheduled time, the server sends the message to the channel on your behalf.
- You must be a member of the target channel when the message is sent -- if you leave the channel before the scheduled time, the message may fail to deliver.
- Scheduled messages are user-specific. Other users cannot see your scheduled messages.

## Panel Behavior

The Scheduled Messages panel is mutually exclusive with other side panels (member list, bookmarks, thread panel). Opening the Scheduled Messages panel closes any other open panel.

## Related Pages

- [Sending Messages](./sending-messages.md)
- [Text Channels](../channels/text-channels.md)
