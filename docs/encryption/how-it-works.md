# How It Works

A high-level overview of how Chatalot encrypts your messages, from key setup to delivery.

> **Status: Beta** -- Infrastructure complete, WASM bridge pending for full client integration.

---

## The Big Picture

Chatalot uses two different encryption strategies depending on the conversation type:

- **Direct messages (DMs)** use **X3DH + Double Ratchet** -- the full Signal Protocol. Every message gets a unique encryption key, and keys are continuously refreshed as the conversation progresses.

- **Group channels** use **Sender Keys** -- an extension of the Signal Protocol designed for efficiency. Each member generates a single key that all other members can use to decrypt their messages.

Both approaches share a common foundation: all encryption and decryption happens on your device. The server only transports opaque ciphertext.

## DM Encryption Flow

When Alice sends a direct message to Bob, the following happens:

```
  Alice's Device                    Server                     Bob's Device
       |                              |                              |
       |  1. Fetch Bob's prekey bundle |                              |
       |----------------------------->|                              |
       |  (identity key, signed       |                              |
       |   prekey, one-time prekey)   |                              |
       |<-----------------------------|                              |
       |                              |                              |
       |  2. X3DH key agreement       |                              |
       |  (compute shared secret      |                              |
       |   from 3-4 DH operations)    |                              |
       |                              |                              |
       |  3. Initialize Double Ratchet|                              |
       |  with shared secret          |                              |
       |                              |                              |
       |  4. Encrypt message          |                              |
       |  (ChaCha20-Poly1305)         |                              |
       |                              |                              |
       |  5. Send encrypted message   |                              |
       |  + X3DH header (first msg)   |                              |
       |----------------------------->|  6. Forward to Bob           |
       |                              |----------------------------->|
       |                              |                              |
       |                              |   7. X3DH key agreement     |
       |                              |   (derive same shared secret)|
       |                              |                              |
       |                              |   8. Initialize Double      |
       |                              |   Ratchet (responder side)   |
       |                              |                              |
       |                              |   9. Decrypt message         |
       |                              |   (ChaCha20-Poly1305)        |
```

After this initial handshake, subsequent messages skip steps 1-3 and use the established Double Ratchet session directly. The ratchet continuously generates fresh encryption keys, providing forward secrecy and break-in recovery.

See [DM Encryption](./dm-encryption.md) for the detailed protocol.

## Group Channel Encryption Flow

Group encryption works differently because encrypting a message individually for each of 50 members would be expensive. Instead, Sender Keys let each member encrypt once:

```
  Alice's Device                    Server               Bob & Carol's Devices
       |                              |                              |
       |  1. Generate Sender Key      |                              |
       |  (random 256-bit chain key   |                              |
       |   + chain ID)                |                              |
       |                              |                              |
       |  2. Upload Sender Key        |                              |
       |  distribution to server      |                              |
       |----------------------------->|  3. Broadcast distribution   |
       |                              |  to channel members via WS   |
       |                              |----------------------------->|
       |                              |                              |
       |  ... later, Alice sends ...  |                              |
       |                              |                              |
       |  4. Advance chain key,       |                              |
       |  derive message key          |                              |
       |                              |                              |
       |  5. Encrypt message          |                              |
       |  (ChaCha20-Poly1305)         |                              |
       |                              |                              |
       |  6. Send encrypted message   |                              |
       |----------------------------->|  7. Fan out to all members   |
       |                              |----------------------------->|
       |                              |                              |
       |                              |   8. Look up Alice's         |
       |                              |   Sender Key by chain ID     |
       |                              |                              |
       |                              |   9. Advance to correct      |
       |                              |   iteration, derive msg key  |
       |                              |                              |
       |                              |  10. Decrypt message          |
```

When a member leaves or is removed from a channel, all remaining members regenerate their Sender Keys. This ensures the departed member cannot decrypt future messages.

See [Group Encryption](./group-encryption.md) for the detailed protocol.

## Step-by-Step: Alice Sends a DM to Bob

Here is a concrete walkthrough of what happens the first time Alice messages Bob:

### 1. Key Setup (happened during registration)

When Bob registered his account, his client:
- Generated an Ed25519 identity key pair (his permanent cryptographic identity)
- Generated a signed prekey (X25519 key pair, signed with his identity key)
- Generated 100 one-time prekeys (single-use X25519 key pairs)
- Uploaded the public halves to the server, keeping private keys in local IndexedDB

### 2. Alice Fetches Bob's Prekey Bundle

Alice's client calls `GET /keys/{bob_id}/bundle` and receives:
- Bob's Ed25519 identity public key
- Bob's signed prekey (X25519 public key + Ed25519 signature)
- One of Bob's one-time prekeys (X25519 public key) -- consumed and removed from the server

### 3. Alice Verifies and Computes

Alice's client:
- Verifies the signed prekey signature using Bob's identity key (rejects if invalid)
- Generates a fresh ephemeral X25519 key pair
- Performs 3-4 Diffie-Hellman operations to compute a shared secret
- Derives a 256-bit shared secret using HKDF-SHA256

### 4. Alice Encrypts Her Message

Alice's client:
- Initializes a Double Ratchet session with the shared secret
- Derives a per-message key from the ratchet chain
- Encrypts the plaintext with ChaCha20-Poly1305
- Packages the ciphertext with the message header and X3DH metadata

### 5. Message Travels Through the Server

The server receives an opaque blob: a JSON wire message containing the encrypted ciphertext, the message header (ratchet key, counters), and on the first message, the X3DH header (Alice's identity key, ephemeral key, prekey IDs). The server stores and forwards this without being able to read any of it.

### 6. Bob Decrypts

Bob's client:
- Parses the X3DH header from the first message
- Looks up the private keys for the prekeys Alice used (signed prekey, one-time prekey)
- Performs the same DH operations to derive the same shared secret
- Initializes his side of the Double Ratchet
- Decrypts the message using ChaCha20-Poly1305

### 7. Ongoing Conversation

From here on, Alice and Bob use their established Double Ratchet session. Each message advances the ratchet, generating a fresh key. When the conversation direction changes (Alice sends, then Bob replies), a DH ratchet step generates entirely new key material, providing break-in recovery.

## How Keys Are Stored

| Location | What is Stored | Protected By |
|----------|---------------|-------------|
| Server database | Public identity keys, public prekeys, prekey signatures, encrypted message blobs, sender key distributions | Server-side access controls |
| Browser (IndexedDB) | Private identity key, private prekeys, Double Ratchet session state, Sender Key state, decrypted message cache | Browser origin isolation |
| Desktop app (planned) | Same as browser | OS keychain encryption |

Private keys never leave your device. The server only ever sees public keys and ciphertext.

## Next Steps

- [Key Management](./key-management.md) -- Deep dive into the different key types
- [DM Encryption](./dm-encryption.md) -- Full X3DH + Double Ratchet protocol
- [Group Encryption](./group-encryption.md) -- Sender Keys for group channels
- [Technical Details](./technical-details.md) -- Wire formats and cryptographic primitives
