# Encryption Status

> **Status: Beta** -- E2E encryption infrastructure is complete but the WASM bridge to the web client is pending.

Chatalot is built with end-to-end encryption (E2E) at its core. The cryptographic library (`chatalot-crypto`) implements the full Signal Protocol, but the bridge that connects it to the web and desktop clients is still in progress. This page explains what is encrypted, what is not, and where things stand.

## Current Status

The encryption system is viewable in **Settings > Security** under the **Encryption** section, which shows:

- **End-to-end encryption**: Active
- **Identity key fingerprint**: Stored on device

### What Is Built

The `chatalot-crypto` crate implements the complete cryptographic stack:

| Component | Protocol | Purpose |
|-----------|----------|---------|
| Key exchange | X3DH (Extended Triple Diffie-Hellman) | Establishes shared secrets between two users, even when one is offline |
| DM encryption | Double Ratchet | Provides forward secrecy and break-in recovery for one-on-one messages |
| Group encryption | Sender Keys | Efficient group messaging where each member encrypts once for all recipients |
| Symmetric encryption | ChaCha20-Poly1305 (AEAD) | Encrypts and authenticates individual messages |
| Key derivation | HKDF-SHA256 | Derives per-message keys from ratchet state |
| Identity | Ed25519 | Signs prekeys and authenticates users cryptographically |
| Password hashing | Argon2id | Securely hashes passwords on the server |

### What Remains

The WASM compilation of `chatalot-crypto` and its integration into the Svelte web client and Tauri desktop client have not yet been completed. This means:

- The cryptographic primitives are implemented and tested in Rust.
- The client-side key management, session establishment, and message encrypt/decrypt flows are not yet wired up in the UI.
- Messages are currently transmitted and stored on the server without E2E encryption.

## What Will Be Encrypted

Once the WASM bridge is complete, the following will be end-to-end encrypted:

### Direct Messages (DMs)

DMs use the **Double Ratchet** protocol (the same protocol used by Signal):

- Each conversation establishes a unique session via X3DH key agreement.
- Every message uses a fresh symmetric key derived from the ratchet chain.
- **Forward secrecy**: if a key is compromised, past messages remain secure.
- **Break-in recovery**: future messages become secure again after a key compromise, once a new Diffie-Hellman ratchet step occurs.

### Group Messages

Group channels use **Sender Keys**:

- Each member generates a Sender Key and distributes it to all other group members via their existing pairwise Double Ratchet sessions.
- When sending a message, the sender encrypts once with their Sender Key. All group members can decrypt.
- When a member is removed from a group, all remaining members regenerate their Sender Keys to maintain forward secrecy.

## What Is Not Encrypted

Even with full E2E encryption, certain data remains visible to the server:

| Data | Reason |
|------|--------|
| **Metadata** | The server knows who is in which channel, who messages whom, message timestamps, and message sizes |
| **User profiles** | Usernames, display names, emails, avatars, and bios are stored in plaintext |
| **Presence and status** | Online/offline status and custom status text are visible to the server |
| **File names and sizes** | File metadata is not encrypted (file content encryption is planned) |
| **Community/channel structure** | Community names, channel names, group hierarchy, and membership lists are stored in plaintext |
| **Message existence** | The server knows a message was sent, even though it cannot read the content |

## Encryption Keys

During registration, Chatalot generates the following key material on your device:

| Key | Type | Lifetime |
|-----|------|----------|
| **Identity Key (IK)** | Ed25519 | Permanent (until device is lost) |
| **Signed Prekey (SPK)** | X25519 | Rotated weekly |
| **One-Time Prekeys (OPK)** | X25519 | Single use (100 maintained on server) |

The private portions of these keys **never leave your device**. Only the public keys are uploaded to the server.

## Cryptographic Primitives

| Purpose | Algorithm | Library |
|---------|-----------|---------|
| Identity keys (signing) | Ed25519 | `ed25519-dalek` |
| Key exchange | X25519 (ECDH) | `x25519-dalek` |
| Symmetric encryption | ChaCha20-Poly1305 | `chacha20poly1305` |
| Key derivation | HKDF-SHA256 | `hkdf` + `sha2` |
| Password hashing | Argon2id | `argon2` |
| Random number generation | OS CSPRNG | `rand` |
| Sensitive data cleanup | Zeroization | `zeroize` |

## Roadmap

1. **Compile `chatalot-crypto` to WASM** -- make the Rust crypto available in the browser.
2. **Wire up key management in the client** -- generate, store, and rotate keys from the web and desktop apps.
3. **Implement session establishment** -- X3DH handshake when starting a new DM.
4. **Encrypt/decrypt message flow** -- integrate Double Ratchet into the message send/receive pipeline.
5. **Sender Key distribution for groups** -- distribute Sender Keys via pairwise sessions when joining a group.
6. **File encryption** -- encrypt file contents before upload.

## Further Reading

- [Encryption Protocol](../../encryption/overview.md) -- full technical specification of the encryption protocol
- [Account Security](./account-security.md) -- passwords, 2FA, and session management

> **Tip:** Even while the WASM bridge is pending, your connection to the Chatalot server is always encrypted in transit via HTTPS/WSS. The E2E layer adds protection against a compromised server.
