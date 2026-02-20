# Limitations

What Chatalot's encryption does not protect, and the security boundaries you should be aware of.

> **Status: Beta** -- Infrastructure complete, WASM bridge pending for full client integration.

---

## What Is NOT Encrypted

End-to-end encryption protects message content, but not everything about your communications. The following metadata is visible to the server:

### Message Metadata

- **Timestamps**: The server knows exactly when each message was sent and delivered.
- **Sender ID**: The server knows who sent each message.
- **Recipient/Channel ID**: The server knows which channel or DM conversation a message belongs to.
- **Message size**: The server can observe the approximate length of messages (the size of the encrypted blob).
- **Message existence**: The server knows that a message was sent, even if it cannot read the content.

### User Activity

- **Presence and status**: Online/offline status, custom status messages, and "last seen" information are stored in plaintext.
- **Typing indicators**: The server relays typing indicators and knows when you are composing a message.
- **Read receipts**: If implemented, the server would know which messages you have read.

### Channel and Community Structure

- **Channel names and descriptions**: These are stored in plaintext on the server.
- **Community names and settings**: Community metadata is not encrypted.
- **Membership lists**: The server knows exactly who is in each channel and community.
- **Roles and permissions**: Organizational structure is plaintext.

### File Metadata

- **File names**: Currently, file names are visible to the server.
- **File sizes**: The server knows how large each uploaded file is.
- **File MIME types**: Content types are visible for serving purposes.
- **Upload timestamps**: The server knows when files were uploaded.

Note: File content encryption (encrypting the file bytes before upload) is part of the design but may not be fully wired in the current beta.

### Reactions

- **Emoji reactions**: The server can see which emoji was used and who reacted to which message. Reactions are not encrypted.

### Profile Information

- **Usernames, display names, bios, pronouns**: All profile data is stored in plaintext.
- **Avatars**: Profile images are not encrypted.
- **Email addresses**: Registration email is stored in plaintext.

## What the Server Operator Can See

A Chatalot server administrator (or an attacker who compromises the server) can see:

- **Who is talking to whom**: The server knows every participant in every conversation.
- **When conversations happen**: Full timing information for all messages.
- **Conversation patterns**: How frequently users communicate, at what times, in which channels.
- **Group membership**: Who is in every channel and community.
- **Encrypted blobs**: The actual ciphertext of messages, which appears as opaque binary data.
- **Public keys**: Identity keys, signed prekeys, and one-time prekeys (public halves only).
- **Sender Key distributions**: These are stored on the server (encrypted distribution data).

A server administrator **cannot**:

- Read message content (the ciphertext is indecipherable without the private keys)
- Forge messages as a specific user (messages are tied to identity keys the server does not possess)
- Decrypt files if file encryption is active (file encryption keys travel inside encrypted messages)

## Sender Keys and Group Forward Secrecy

Group channels use Sender Keys, which have weaker security properties than the pairwise Double Ratchet used for DMs:

- **No automatic break-in recovery**: If an attacker compromises a member's Sender Key, they can decrypt all subsequent messages from that member until the key is rotated. The Double Ratchet's DH ratchet provides automatic recovery on every turn of conversation; Sender Keys do not have this.

- **Rotation is event-driven**: Sender Keys are rotated when membership changes (member joins or leaves), not continuously. During normal operation with a stable membership, the same chain is used for all messages.

- **Forward secrecy within a chain**: Each message advances the chain, so compromising iteration N does not reveal messages 0 through N-1. But it does reveal N+1 and beyond until rotation.

This is the same tradeoff made by Signal, WhatsApp, and other major messaging apps for group conversations. It is considered acceptable for typical group messaging use cases.

## WASM Bridge Status

During the beta period, the WASM bridge between the Rust crypto library and the web client is still being finalized:

- **Fallback to plaintext**: If the WASM module fails to load, the client will send and display messages as plaintext. This means some messages may transit the server unencrypted.
- **Mixed encryption state**: In a channel where some clients have encryption active and others do not, unencrypted messages will be readable by all, while encrypted messages will appear as ciphertext to clients without the WASM module.
- **Graceful degradation**: The client attempts to detect whether incoming bytes are an encrypted wire message (by parsing for `v: 1`) and falls back to UTF-8 if not. This ensures old plaintext messages remain readable.

## Browser-Based Key Storage

On the web client, all private key material is stored in IndexedDB:

- **Not hardware-backed**: Unlike a hardware security module or OS keychain, IndexedDB storage can be accessed by JavaScript running in the same origin. A cross-site scripting (XSS) vulnerability could potentially expose keys.
- **Clearable by the user**: Clearing browser data (cache, cookies, site data) will destroy your keys and all session state. There is no recovery mechanism.
- **Per-browser isolation**: Keys are not synced between browsers or devices. Each browser has its own identity and sessions.

The desktop app uses the OS keychain (GNOME Keyring / KWallet on Linux, Credential Manager on Windows) for identity key storage, providing stronger at-rest protection than IndexedDB.

## No Multi-Device Support (Yet)

Currently, each device (browser) has its own independent identity key and sessions. There is no mechanism to share keys across devices. This means:

- You cannot seamlessly continue conversations on a different device.
- Logging in from a new browser creates a new identity, requiring new sessions.
- Message history encrypted with one device's keys cannot be decrypted by another device.

Multi-device support is a planned future feature.

## No Message Franking or Abuse Reporting

With E2E encryption, the server cannot verify message content. This creates a tension with content moderation:

- The server cannot automatically detect abusive content in encrypted messages.
- Reported messages would need to be forwarded by the reporter, with no server-side proof of authenticity.
- Message franking (a technique that allows recipients to prove to the server what was sent, without the server being able to read messages proactively) is not implemented.

## Trust on First Use (TOFU)

As described in [Verification](./verification.md), Chatalot currently uses trust-on-first-use for identity keys. This means:

- The first contact with a user is vulnerable to man-in-the-middle attacks if the server substitutes keys.
- Key change warnings are not yet displayed in the UI.
- Out-of-band verification (safety numbers) is implemented at the protocol level but not yet exposed to users.

## Summary

| Limitation | Impact | Mitigation |
|-----------|--------|------------|
| Metadata visible to server | Server knows who, when, where -- just not what | Inherent to any server-mediated system |
| No break-in recovery for groups | Compromised Sender Key exposes future messages until rotation | Keys rotate on membership changes |
| WASM fallback to plaintext | Some messages may transit unencrypted during beta | Being resolved as integration completes |
| Browser key storage (IndexedDB) | XSS could expose keys | CSP headers, input sanitization, desktop app uses OS keychain |
| No multi-device | Each browser is an independent identity | Planned feature |
| TOFU without verification UI | First contact vulnerable to MITM | Safety numbers implemented, UI planned |
| Reactions not encrypted | Server sees reaction data | Encryption of reactions is a design consideration |

## Next Steps

- [Verification](./verification.md) -- How verification will address the TOFU limitation
- [Technical Details](./technical-details.md) -- Detailed protocol specification
- [Overview](./overview.md) -- Return to the encryption overview
