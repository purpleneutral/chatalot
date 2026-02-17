# Encryption Overview

End-to-end encryption in Chatalot: why it matters and how it protects your messages.

> **Status: Beta** -- Infrastructure complete, WASM bridge pending for full client integration.

---

## Why End-to-End Encryption Matters

When you send a message on most chat platforms, the server can read it. The company operating the service, any employee with database access, and any attacker who compromises the server can see your conversations. Even if the connection uses TLS (HTTPS), that only protects messages between your device and the server -- the server itself still handles plaintext.

End-to-end encryption (E2E) changes this. With E2E, messages are encrypted on your device before they leave, and only the intended recipient's device can decrypt them. The server transports encrypted blobs it cannot read. Even if someone gains full access to the server's database, they see ciphertext, not conversations.

For a self-hosted platform like Chatalot, this means:

- **Server administrators** cannot read message content (intentionally or accidentally)
- **Database breaches** do not expose message history
- **Network eavesdroppers** see only encrypted traffic
- **Legal compulsion** cannot produce plaintext messages the server does not possess

## What is the Signal Protocol?

Chatalot's encryption is based on the [Signal Protocol](https://signal.org/docs/), the same cryptographic foundation used by Signal, WhatsApp, and Google Messages. It was chosen for several reasons:

- **Proven security**: The protocol has been formally analyzed by academic cryptographers and has withstood years of scrutiny.
- **Forward secrecy**: Compromising a key does not reveal past messages. Each message uses a unique key that is deleted after use.
- **Break-in recovery**: Even if an attacker compromises your current keys, they lose access once the next key exchange occurs.
- **Asynchronous setup**: Two users can establish a secure session even if one is offline, thanks to the X3DH (Extended Triple Diffie-Hellman) handshake.
- **Efficient group messaging**: The Sender Keys extension allows group messages to be encrypted once rather than once per recipient.

## Privacy Guarantees

When encryption is active, the Chatalot server:

- **Cannot read** the content of your direct messages
- **Cannot read** the content of group channel messages
- **Cannot forge** messages as if they came from you (messages are tied to your identity key)
- **Cannot decrypt** files you share (file encryption keys are exchanged inside encrypted messages)

## What Is Encrypted vs. What Is Not

It is important to understand the boundaries of E2E encryption. Not everything is hidden from the server.

| Encrypted (server cannot read) | Not encrypted (server can see) |
|-------------------------------|-------------------------------|
| Message text content | Message timestamps |
| File contents | Sender and recipient IDs |
| File encryption keys | Channel and community names |
| | File names and sizes |
| | User presence and status |
| | Reactions (which emoji, who reacted) |
| | Message existence and approximate size |
| | Who is in which channel |

Think of it like sending a sealed letter through the postal service: the post office can see the envelope (who sent it, who it is addressed to, when it was mailed, and how heavy it is), but it cannot read what is inside.

See [Limitations](./limitations.md) for a thorough discussion of what the encryption does and does not protect.

## Current Implementation Status

The encryption system is implemented in layers:

1. **Rust crypto crate** (`chatalot-crypto`) -- All cryptographic protocols are implemented and covered by 23 unit tests. This is the core engine.

2. **WASM bridge** (`chatalot-crypto-wasm`) -- The Rust crypto crate is compiled to WebAssembly so it can run in the browser. The WASM module is compiled and bundled with the web client.

3. **Web client integration** (`clients/web/src/lib/crypto/`) -- The KeyManager, SessionManager, and CryptoStorage classes wire the WASM crypto into the Svelte web client. Key generation on registration, prekey replenishment, DM encryption/decryption, and Sender Key group encryption are implemented.

4. **Server key exchange** -- The server provides REST endpoints for uploading and fetching prekey bundles (`/keys/{user_id}/bundle`) and sender key distributions (`/channels/{id}/sender-keys`). The server stores only public keys and encrypted blobs.

During the beta period, the client gracefully falls back to plaintext when the WASM module is unavailable or a session cannot be established. This ensures messages are always deliverable while the integration is completed.

## Next Steps

- [How It Works](./how-it-works.md) -- See the encryption flow step by step
- [Key Management](./key-management.md) -- Learn about the different key types
- [Limitations](./limitations.md) -- Understand the security boundaries
