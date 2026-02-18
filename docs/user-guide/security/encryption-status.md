# Encryption Status

> **Status: Complete** -- E2E encryption is fully active. All messages are encrypted client-side before leaving your device.

Chatalot uses end-to-end encryption (E2E) for all messages. The cryptographic library (`chatalot-crypto`) is compiled to WASM and runs in the browser. The server stores and routes only encrypted ciphertext -- it never sees plaintext message content.

## Current Status

The encryption system is viewable in **Settings > Security** under the **Encryption** section, which shows:

- **Protocol**: X3DH + Double Ratchet (DMs), Sender Keys (groups)
- **Identity key fingerprint**: Your device's public key fingerprint (copyable)
- **Public key hex**: Full hex-encoded public key

### Visual Indicators

Chatalot makes encryption status visible on every message:

| Indicator | Meaning |
|-----------|---------|
| Green lock icon | Message was end-to-end encrypted |
| Red broken lock icon | Message could not be decrypted (session mismatch or corruption) |
| No icon | Legacy plaintext message (sent before encryption was enabled) |

The channel header displays a green **E2E** badge:
- **DMs**: Clickable -- opens the verification modal with safety numbers and fingerprints
- **Group channels**: Static indicator showing encryption is active

### Verification

To verify a contact's identity:

1. Open a DM and click the **E2E** badge in the header.
2. The modal shows your shared **safety number** (monospace, copyable), your fingerprint, and their fingerprint.
3. Compare the safety number with your contact through a separate trusted channel (in person, phone call, etc.).
4. If the numbers match, the encryption is verified.

### Key Change Warnings

When a contact's identity key changes (they re-registered, reinstalled, or switched devices), a yellow warning banner appears:

> "Safety number has changed for this contact. They may have re-registered or switched devices."

You can:
- **Acknowledge** -- accept the change and dismiss the warning.
- **Verify** -- acknowledge and immediately open the verification modal to confirm the new identity.

## What Is Encrypted

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
| **Push notification metadata** | If web push is enabled, the server sends notification metadata (sender name, channel) -- never message content |

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

## Further Reading

- [Encryption Protocol](../../encryption/overview.md) -- full technical specification of the encryption protocol
- [Verification](../../encryption/verification.md) -- safety numbers, fingerprints, and TOFU
- [Account Security](./account-security.md) -- passwords, 2FA, and session management

> **Tip:** Your connection to the Chatalot server is always encrypted in transit via HTTPS/WSS. The E2E layer adds an additional layer of protection -- even a compromised server cannot read message content.
