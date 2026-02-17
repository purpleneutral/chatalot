# Key Management

How Chatalot generates, stores, rotates, and distributes the cryptographic keys that protect your messages.

> **Status: Beta** -- Infrastructure complete, WASM bridge pending for full client integration.

---

## Key Types at a Glance

Chatalot uses several types of cryptographic keys, each serving a different purpose in the encryption protocol:

| Key Type | Algorithm | Lifetime | Where Private Key Lives | Purpose |
|----------|-----------|----------|------------------------|---------|
| Identity Key | Ed25519 | Permanent | Your device only | Your cryptographic identity; signs prekeys |
| Signed Prekey | X25519 | Rotated periodically | Your device only | Enables asynchronous session setup |
| One-Time Prekey | X25519 | Single use | Your device only | Adds forward secrecy to first message |
| Ephemeral Key | X25519 | Single use (per session) | Generated and discarded in memory | Input to X3DH shared secret |
| Ratchet Key | X25519 | Changes every turn | Your device (session state) | Drives the Double Ratchet DH step |
| Chain Key | 256-bit symmetric | Advances per message | Your device (session state) | Derives per-message encryption keys |
| Message Key | 256-bit symmetric | Single use | Derived, used, zeroized | Encrypts one message (ChaCha20-Poly1305) |
| Sender Key | 256-bit symmetric | Per channel, per member | Your device (IndexedDB) | Group channel encryption |

## Identity Key (IK)

The identity key is your long-term cryptographic identity. It is an Ed25519 key pair generated during account registration.

Think of the identity key as your digital signature stamp. It proves that messages and prekeys genuinely came from you, just as a wax seal on a letter proves the sender's identity.

**Public key**: Uploaded to the server and included in your prekey bundle. Other users fetch it when they want to start an encrypted conversation with you.

**Private key**: Stays on your device forever. It never leaves. It is stored in encrypted IndexedDB (web client) and will use the OS keychain on the desktop app.

**What it does**:
- Signs your prekeys so others can verify they are authentic
- Participates in the X3DH key agreement (converted to X25519 via the birational map)
- Computes safety numbers for identity verification

**Important**: If you lose your identity key (e.g., by clearing your browser data), you will need to re-register. Other users who had sessions with you will see a key change warning.

### Key Generation

During registration, the client generates the identity key pair using the WASM-compiled Rust crypto library:

```
signing_key = Ed25519::generate(OsRng)
verifying_key = signing_key.verifying_key()
```

The verifying key (public) is sent to the server as part of the registration request. The signing key (private) is stored locally.

## Signed Prekey (SPK)

The signed prekey is a medium-term X25519 key pair that enables asynchronous session establishment. It is signed with your identity key so that anyone fetching it can verify it is authentic.

Think of the signed prekey as a public mailbox. When someone wants to send you an encrypted message while you are offline, they use your signed prekey (the mailbox) combined with their own keys to compute a shared secret. The signature on the mailbox proves you are the one who put it there.

**Generation**: A random X25519 key pair is generated, and the public key is signed with the Ed25519 identity key.

```
spk_secret = X25519::random(OsRng)
spk_public = X25519Public::from(spk_secret)
signature = identity_signing_key.sign(spk_public.as_bytes())
```

**Storage**:
- Server: public key + signature (so others can fetch and verify it)
- Device: private key (for deriving shared secrets when responding to X3DH)

**Rotation**: Signed prekeys should be rotated periodically. When a new signed prekey is uploaded, the old one is retained briefly to handle in-flight messages that reference it.

## One-Time Prekeys (OPK)

One-time prekeys are single-use X25519 key pairs that provide additional forward secrecy for the initial X3DH handshake. Each one-time prekey is consumed exactly once and then deleted from the server.

Think of one-time prekeys as single-use padlocks you leave at the post office. When someone picks one up to start a conversation with you, that specific padlock is gone forever. Even if your signed prekey were somehow compromised later, the one-time prekey that was used for a particular session is already destroyed.

**Generation**: A batch of key pairs is generated at registration time.

```
for key_id in start..(start + count) {
    secret = X25519::random(OsRng)
    public = X25519Public::from(secret)
    // Store (key_id, secret) locally
    // Upload (key_id, public) to server
}
```

**Initial batch**: 100 one-time prekeys are generated during registration.

**Replenishment**: The client periodically checks how many one-time prekeys remain on the server (via `GET /keys/prekeys/count`). When the count drops below 25, a new batch of 100 is generated and uploaded.

The server also proactively notifies the client via WebSocket (`KeysLow` message) when the remaining count falls below the threshold.

**Consumption**: When another user fetches your prekey bundle, the server includes one of your one-time prekeys and marks it as consumed. If no one-time prekeys are available, the X3DH handshake proceeds without one (DH4 is omitted), which is slightly less secure for the first message but still safe.

## Ephemeral Key (EK)

The ephemeral key is a one-time X25519 key pair generated by the initiator of an X3DH handshake. It is used for the DH computations and then discarded.

This key exists only in memory during the X3DH computation. The public half is sent to the responder as part of the first message's X3DH header.

## Ratchet Keys

During an active Double Ratchet session, each party maintains a current X25519 key pair called the ratchet key. When the conversation direction changes (you send after receiving, or vice versa), a new ratchet key pair is generated and a DH ratchet step occurs.

These keys are part of the session state and are stored alongside the session in IndexedDB.

## Chain Keys and Message Keys

Chain keys are 256-bit symmetric keys that advance with each message. From each chain key, a message key is derived for encryption, and a new chain key is derived for the next message.

```
message_key = HKDF(chain_key, 0x01, "chatalot-msg-key")
next_chain_key = HKDF(chain_key, 0x02, "chatalot-msg-key")
```

Message keys are used exactly once for ChaCha20-Poly1305 encryption and then zeroized from memory. This is what provides forward secrecy: even if an attacker obtains the current chain key, they cannot derive past message keys.

## Key Generation on Registration

When you create a Chatalot account, the following keys are generated in a single operation:

1. **Ed25519 identity key pair** -- your permanent cryptographic identity
2. **Signed prekey** (key ID 1) -- signed with the identity key
3. **100 one-time prekeys** (key IDs 1-100) -- single-use keys

The public components are sent to the server as part of the registration request:
- Identity verifying key (32 bytes)
- Signed prekey public key (32 bytes) + signature (64 bytes) + key ID
- 100 one-time prekey public keys (32 bytes each) + key IDs

The private components are stored in the browser's IndexedDB under the `chatalot-crypto` database:
- `identity` store: signing key + verifying key
- `signedPrekeys` store: key ID, public key, private key
- `oneTimePrekeys` store: key ID, public key, private key (per key)

## Key Rotation and Replenishment

### One-Time Prekeys

One-time prekeys are consumed as other users start conversations with you. The client monitors the remaining count:

- **Threshold**: 25 remaining keys
- **Batch size**: 100 new keys per replenishment
- **Trigger**: Periodic check or server `KeysLow` WebSocket notification
- **Upload limit**: Maximum 200 keys per upload request (server-enforced)

### Signed Prekeys

Signed prekeys are rotated periodically. When a new signed prekey is generated:
1. A new X25519 key pair is created
2. The public key is signed with the identity key
3. The new signed prekey is uploaded to the server
4. The old signed prekey's private key is retained temporarily for in-flight sessions

### Sender Keys

Sender Keys for group channels are rotated when the channel membership changes:
- **Member removed or leaves**: All remaining members regenerate their Sender Keys
- **Member joins**: New member receives current Sender Key distributions from the server

See [Group Encryption](./group-encryption.md) for details on Sender Key rotation.

## Prekey Bundle

When a user wants to establish an encrypted session with you, they fetch your prekey bundle from the server via `GET /keys/{user_id}/bundle`. The bundle contains:

```json
{
    "identity_key": [/* 32 bytes */],
    "signed_prekey": {
        "key_id": 1,
        "public_key": [/* 32 bytes */],
        "signature": [/* 64 bytes */]
    },
    "one_time_prekey": {          // null if none available
        "key_id": 42,
        "public_key": [/* 32 bytes */]
    }
}
```

The server validates key sizes (32 bytes for public keys, 64 bytes for signatures) but does not interpret the cryptographic content.

## Key Storage Security

| Store | Contents | Protection |
|-------|----------|------------|
| `identity` | Ed25519 signing key + verifying key | IndexedDB origin isolation |
| `signedPrekeys` | X25519 private keys (by key ID) | IndexedDB origin isolation |
| `oneTimePrekeys` | X25519 private keys (by key ID) | IndexedDB origin isolation |
| `sessions` | Serialized Double Ratchet state (JSON) | IndexedDB origin isolation |
| `peerIdentities` | Peer identity public keys (for TOFU) | IndexedDB origin isolation |
| `senderKeyStates` | Sender Key chain state (per channel) | IndexedDB origin isolation |
| `receiverKeyStates` | Receiver Key state (per channel + sender) | IndexedDB origin isolation |

On the web client, all private key material is stored in IndexedDB, which is isolated by browser origin. The `zeroize` crate is used in the Rust/WASM layer to clear sensitive key material from memory when it is no longer needed.

On the desktop app (planned), private keys will be stored in the operating system's keychain (macOS Keychain, Windows Credential Manager, or Linux Secret Service).

## What Happens If You Lose Your Keys

If you clear your browser data or lose access to your device:

- Your identity key is lost. You will need to re-register.
- Existing sessions with other users will break. They will see a key change warning when your new identity key differs from the one they previously trusted.
- Decrypted message cache is lost. Encrypted messages stored on the server cannot be decrypted with the new keys.
- Sender Key states are lost. You will generate new Sender Keys for group channels on your next message.

This is an inherent property of E2E encryption: if the keys are gone, the messages are gone. There is no server-side recovery mechanism because the server never had your keys.

## Next Steps

- [DM Encryption](./dm-encryption.md) -- How these keys are used in the X3DH and Double Ratchet protocols
- [Group Encryption](./group-encryption.md) -- How Sender Keys work for group channels
- [Technical Details](./technical-details.md) -- Exact algorithms and wire formats
