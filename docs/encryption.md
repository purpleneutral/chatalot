# Encryption Protocol

This document describes the end-to-end encryption protocol used by Chatalot. The design is based on the Signal Protocol with adaptations for group messaging.

## Threat Model

**The server is untrusted.** It is assumed that the server operator (or an attacker who compromises the server) can:

- Read all data stored on the server
- Modify or replay messages in transit
- Observe metadata (who messages whom, when, message sizes)

The server **cannot**:

- Read message plaintext (it only sees ciphertext)
- Forge messages from a user (messages are tied to identity keys)
- Decrypt files (file encryption keys are exchanged via encrypted messages)

**What is not protected:**

- Metadata: the server knows who is in which channel, who messages whom, and timing
- User profiles: usernames, emails, and display names are stored in plaintext
- Message existence: the server knows that a message was sent, its size, and its timestamp

## Cryptographic Primitives

| Purpose | Algorithm | Library |
|---------|-----------|---------|
| Identity keys (signing) | Ed25519 | `ed25519-dalek` |
| Key exchange | X25519 (ECDH) | `x25519-dalek` |
| Symmetric encryption | ChaCha20-Poly1305 (AEAD) | `chacha20poly1305` |
| Key derivation | HKDF-SHA256 | `hkdf` + `sha2` |
| Hashing | SHA-256 | `sha2` |
| Password hashing | Argon2id | `argon2` |
| Random number generation | OS CSPRNG | `rand` |
| Sensitive data cleanup | Zeroization | `zeroize` |

## Key Types

### Identity Key (IK)

A long-lived Ed25519 key pair generated during registration. The public key is uploaded to the server and serves as the user's cryptographic identity. The private key **never leaves the device**.

- **Lifetime**: Permanent (until device is lost/compromised)
- **Storage**: OS keychain (desktop) or encrypted IndexedDB (web)
- **Purpose**: Signing prekeys, X3DH authentication, identity verification

### Signed Prekey (SPK)

An X25519 key pair signed with the identity key. Used in X3DH to establish sessions.

- **Lifetime**: Rotated weekly
- **Storage**: Server stores public key + signature; device stores private key
- **Purpose**: Enables asynchronous session establishment

### One-Time Prekey (OPK)

X25519 key pairs used exactly once. Each user maintains a supply of 100 on the server.

- **Lifetime**: Consumed on first use, never reused
- **Storage**: Server stores public keys; device stores private keys
- **Purpose**: Provides forward secrecy for the initial X3DH exchange

### Ephemeral Key (EK)

A fresh X25519 key pair generated per X3DH session initiation.

- **Lifetime**: Single use, discarded after X3DH
- **Purpose**: Input to X3DH shared secret derivation

### Message Keys

Symmetric keys derived per-message from the Double Ratchet chain. Each message key is used exactly once and then zeroized.

## Protocol Details

### X3DH Key Agreement

When Alice wants to start a conversation with Bob:

1. Alice fetches Bob's **prekey bundle** from the server:
   - Bob's identity key (`IK_B`)
   - Bob's signed prekey (`SPK_B`) + signature
   - One of Bob's one-time prekeys (`OPK_B`) — consumed and removed from server

2. Alice verifies the signed prekey signature using `IK_B`

3. Alice generates an ephemeral key pair (`EK_A`)

4. Alice computes four DH operations:
   ```
   DH1 = X25519(IK_A_private, SPK_B)
   DH2 = X25519(EK_A_private, IK_B)
   DH3 = X25519(EK_A_private, SPK_B)
   DH4 = X25519(EK_A_private, OPK_B)    // omitted if no OPK available
   ```

5. The shared secret is derived via HKDF:
   ```
   SK = HKDF(salt=0, ikm=DH1||DH2||DH3||DH4, info="X3DH")
   ```

6. Associated data for the session:
   ```
   AD = IK_A_public || IK_B_public
   ```

7. Alice initializes a Double Ratchet session with `SK` and sends her first message along with `IK_A_public` and `EK_A_public`.

8. Bob receives the message, performs the same DH operations using his private keys, derives the same `SK`, and initializes his side of the Double Ratchet.

### Double Ratchet

After X3DH establishes a shared secret, the Double Ratchet provides ongoing key management:

#### Symmetric Ratchet (Chain Key)

Each message advances a chain key:
```
chain_key[n+1] = HMAC-SHA256(chain_key[n], 0x02)
message_key[n] = HMAC-SHA256(chain_key[n], 0x01)
```

Message keys are used once for ChaCha20-Poly1305 encryption, then zeroized. This provides forward secrecy within a chain: compromising `chain_key[n]` reveals `message_key[n]` and all future keys, but not `message_key[0..n-1]`.

#### DH Ratchet

When the conversation direction changes (Alice sends, then Bob replies), a DH ratchet step occurs:

1. The replier generates a new ephemeral X25519 key pair
2. A new DH is computed with the other party's last ephemeral public key
3. The result is fed into HKDF to derive a new root key and chain key
4. This provides **break-in recovery**: even if an attacker compromises current keys, they lose access once a DH ratchet step occurs

#### Out-of-Order Messages

The Double Ratchet handles out-of-order delivery:
- Each message includes a counter within its chain
- Skipped message keys are cached (up to a limit) for later decryption
- This allows messages to arrive out of order without breaking the protocol

#### Session Serialization

Double Ratchet sessions are serialized and stored locally (encrypted) so they persist across app restarts. The session state includes:
- Current root key, sending chain key, receiving chain key
- DH ratchet key pairs
- Message counters
- Skipped message keys cache

### Group Encryption (Sender Keys)

For channels with multiple participants, pairwise Double Ratchet would require O(N) encryptions per message. Sender Keys reduce this to O(1):

1. Each group member generates a **Sender Key**: a random 256-bit symmetric key + a chain ID

2. The Sender Key is **distributed** to all other group members via their existing pairwise Double Ratchet sessions (so the server never sees Sender Keys)

3. To send a message:
   - Advance the sender's chain: `key[n+1] = HMAC-SHA256(key[n], 0x01)`
   - Encrypt with ChaCha20-Poly1305 using the derived key
   - Include the chain ID and message index in the header
   - Send once to the server, which fans out to all subscribers

4. To receive:
   - Look up the sender's key by chain ID
   - Advance to the correct index
   - Decrypt

5. **Member removal**: When a member leaves or is removed, **all remaining members** regenerate their Sender Keys and redistribute them. This ensures the removed member cannot decrypt future messages.

### File Encryption

Files are encrypted client-side before upload:

1. Generate a random 256-bit **File Encryption Key** (FEK)
2. Encrypt the file with ChaCha20-Poly1305 using the FEK
3. Upload the encrypted blob to the server
4. Send a message in the channel containing `{file_id, FEK, nonce, filename}` encrypted with the channel's ratchet/sender key
5. Recipients decrypt the message to get the FEK, download the blob, and decrypt locally

The server never sees the file contents, filename, or encryption key.

## Key Verification

Users can verify each other's identity keys out-of-band using **safety numbers**:

```
fingerprint = SHA-256(sort(IK_A_public || IK_B_public))
```

This is displayed as a series of numeric blocks (similar to Signal's safety numbers) and can be compared in person or via a trusted channel. A QR code encoding the fingerprint can also be scanned for convenience.

If a user's identity key changes (e.g., they reinstalled the app), all their contacts will see a security warning.

## Implementation Status

| Component | Status |
|-----------|--------|
| ChaCha20-Poly1305 AEAD | Implemented + tested |
| Ed25519 identity keys | Implemented + tested |
| X3DH key agreement | Implemented + tested |
| Double Ratchet | Implemented + tested |
| Sender Keys | Implemented + tested |
| Safety numbers | Implemented + tested |
| WASM compilation | Not yet |
| Client integration | Not yet (using plaintext placeholder) |
| Key storage (web) | Not yet |
| Key storage (desktop) | OS keychain ready |

The crypto library has 23 unit tests covering all protocols, including edge cases like out-of-order messages, tampered ciphertext, invalid signatures, and serialization round-trips.
