# Direct Messages (DMs)

> **Status: Complete**

Direct messages are private conversations between two users. DMs exist outside the Community/Group/Channel hierarchy and are accessible from the **DMs** tab in the sidebar.

## Starting a DM

There are two ways to start a direct message:

### From the Sidebar

1. Switch to the **DMs** tab in the sidebar.
2. Click the **New DM** button (the `+` icon).
3. Search for a user by username or display name (minimum 2 characters).
4. Click on the user to open the conversation.

### From a User Profile Card

1. Click on any username in a channel (in a message or the member list).
2. In the profile card that appears, click **Message**.
3. A new DM conversation opens (or the existing one, if you have already messaged this user).

> **Tip:** You can only DM users who share at least one Community with you. If you leave all shared Communities, you will no longer be able to send messages in that DM.

## DM List in Sidebar

The DMs tab shows all your existing conversations. Each entry displays:

- The other user's avatar and display name
- An online status indicator (green for online, yellow for idle, red for DND, gray for offline)
- An unread message count badge, if applicable

DMs are sorted by most recent activity.

## End-to-End Encryption

DMs are encrypted using the **Double Ratchet** protocol (the same protocol used by Signal). When encryption is active:

- Messages are encrypted on your device before being sent.
- The server stores only ciphertext -- it cannot read your messages.
- Each message uses a unique encryption key derived from the ratchet state.

If encryption has not been initialized (for example, on first login before key exchange completes), messages fall back to plaintext with a warning logged to the console.

## Block Protection

If either user has blocked the other, messages cannot be sent in the DM. The sender receives an error: "cannot send messages to this user."

## How DMs Differ from Channels

| Feature | Text Channel | DM |
|---------|-------------|-----|
| Encryption | Sender Keys (group) | Double Ratchet (1:1) |
| Membership | Multiple users | Exactly two users |
| Location | Inside a Group | Independent (sidebar DMs tab) |
| Roles/Permissions | Owner, Admin, Member | Equal (no roles) |
| Threads | Supported | Supported |
| Reactions | Supported | Supported |
| File sharing | Supported | Supported |
| Pins | Supported | Supported |

## Related Pages

- [Text Channels](./text-channels.md)
- [Sending Messages](../messaging/sending-messages.md)
- [Security](../security/README.md) -- More on encryption
