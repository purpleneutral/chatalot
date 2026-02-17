# Technical Details

Detailed protocol specification, cryptographic primitives, wire formats, and session state for Chatalot's encryption.

> **Status: Beta** -- Infrastructure complete, WASM bridge pending for full client integration.

---

## Cryptographic Primitives

| Purpose | Algorithm | Parameters | Library |
|---------|-----------|------------|---------|
| Identity keys (signing) | Ed25519 | 256-bit keys, 512-bit signatures | `ed25519-dalek` |
| Key exchange | X25519 (ECDH on Curve25519) | 256-bit keys | `x25519-dalek` |
| Symmetric encryption | ChaCha20-Poly1305 (AEAD) | 256-bit key, 96-bit nonce, 128-bit tag | `chacha20poly1305` |
| Key derivation | HKDF-SHA256 | Variable input, 256-bit output | `hkdf` + `sha2` |
| Hashing | SHA-256 | 256-bit output | `sha2` |
| Fingerprints | SHA-256 of Ed25519 public key | 256-bit hash, hex-encoded | `sha2` + `hex` |
| Safety numbers | SHA-256 of sorted key pair | Numeric blocks (5-digit groups) | `sha2` |
| Random generation | OS CSPRNG | Platform-native | `rand` (OsRng) |
| Memory cleanup | Zeroization | All sensitive fields | `zeroize` |

### Why ChaCha20-Poly1305 Instead of AES-256-GCM

ChaCha20-Poly1305 was chosen over AES-256-GCM for several reasons:

- **Constant-time in software**: ChaCha20 is inherently constant-time without hardware support, making it resistant to timing side-channel attacks. AES requires hardware acceleration (AES-NI) for constant-time operation, which may not be available in all WASM environments.
- **Performance in WASM**: ChaCha20 performs well in WebAssembly because it relies on simple arithmetic operations (addition, XOR, rotation) rather than lookup tables.
- **Nonce misuse tolerance**: While both algorithms are vulnerable to nonce reuse, ChaCha20-Poly1305 with random nonces has a comfortable safety margin with 96-bit nonces.
- **Widely adopted**: Used by Signal, WireGuard, TLS 1.3, and many other modern protocols.

### Ed25519 to X25519 Conversion

The X3DH protocol requires both signing (Ed25519) and Diffie-Hellman (X25519) operations with the identity key. Rather than maintaining two separate long-term key pairs, Chatalot converts between the two using the birational map:

- **Public key**: `X25519Public = Ed25519Verifying.to_montgomery()`
- **Private key**: `X25519Secret = SHA-512(Ed25519SigningKey)[0..32]` with clamping applied by `StaticSecret::from()`

This is the standard approach used by the Signal Protocol and libsignal.

## X3DH Protocol Specification

### Constants

| Constant | Value |
|----------|-------|
| `X3DH_INFO` | `b"chatalot-x3dh-shared-secret"` |
| `KDF_FILLER` | `0xFF` repeated 32 times |
| HKDF salt | `0x00` repeated 32 bytes |

### Initiator (Alice)

```
Inputs:
    our_identity_key    : Ed25519 SigningKey
    their_bundle        : PrekeyBundle {
        identity_key           : Ed25519 VerifyingKey
        signed_prekey          : X25519 PublicKey
        signed_prekey_signature: Ed25519 Signature (64 bytes)
        one_time_prekey        : Option<X25519 PublicKey>
    }

Steps:
    1. Verify: their_bundle.identity_key.verify(
           signed_prekey.as_bytes(),
           signed_prekey_signature
       ) -- reject if invalid

    2. ephemeral_secret = X25519::StaticSecret::random(OsRng)
       ephemeral_public = X25519Public::from(ephemeral_secret)

    3. our_x25519    = ed25519_to_x25519_secret(our_identity_key)
       their_x25519  = ed25519_to_x25519_public(their_bundle.identity_key)

    4. DH1 = our_x25519.diffie_hellman(their_bundle.signed_prekey)
       DH2 = ephemeral_secret.diffie_hellman(their_x25519)
       DH3 = ephemeral_secret.diffie_hellman(their_bundle.signed_prekey)
       DH4 = ephemeral_secret.diffie_hellman(their_bundle.one_time_prekey)
              // omitted if one_time_prekey is None

    5. kdf_input = KDF_FILLER || DH1 || DH2 || DH3 [|| DH4]
       SK = HKDF-SHA256(
           salt = 0x00[32],
           ikm  = kdf_input,
           info = X3DH_INFO,
           len  = 32
       )
       zeroize(kdf_input)

    6. AD = our_identity_key.verifying_key().as_bytes()
         || their_bundle.identity_key.as_bytes()

Output:
    shared_secret       : SecretKey (32 bytes)
    ephemeral_public_key: X25519 PublicKey
    associated_data     : Vec<u8> (64 bytes)
```

### Responder (Bob)

```
Inputs:
    our_identity_key         : Ed25519 SigningKey
    our_signed_prekey_secret : X25519 StaticSecret
    our_one_time_prekey_secret: Option<X25519 StaticSecret>
    their_identity_key       : Ed25519 VerifyingKey
    their_ephemeral_key      : X25519 PublicKey

Steps:
    1. our_x25519   = ed25519_to_x25519_secret(our_identity_key)
       their_x25519 = ed25519_to_x25519_public(their_identity_key)

    2. DH1 = our_signed_prekey_secret.diffie_hellman(their_x25519)
       DH2 = our_x25519.diffie_hellman(their_ephemeral_key)
       DH3 = our_signed_prekey_secret.diffie_hellman(their_ephemeral_key)
       DH4 = our_one_time_prekey_secret.diffie_hellman(their_ephemeral_key)
              // omitted if no OTP

    3. Same HKDF derivation as initiator

    4. AD = their_identity_key.as_bytes()
         || our_identity_key.verifying_key().as_bytes()

Output:
    shared_secret    : SecretKey (32 bytes)  // identical to initiator's
    associated_data  : Vec<u8> (64 bytes)    // identical to initiator's
```

## Double Ratchet Protocol Specification

### Constants

| Constant | Value |
|----------|-------|
| `RATCHET_INFO` | `b"chatalot-ratchet"` |
| `MSG_KEY_INFO` | `b"chatalot-msg-key"` |
| `MAX_SKIP` | 1000 |

### Session State

```rust
struct RatchetSession {
    dh_sending_private    : Option<[u8; 32]>,  // Our current ratchet private key
    dh_sending_public     : Option<[u8; 32]>,  // Our current ratchet public key
    dh_receiving_key      : Option<[u8; 32]>,  // Their current ratchet public key
    root_key              : [u8; 32],           // Current root key
    sending_chain_key     : Option<[u8; 32]>,   // Current sending chain key
    receiving_chain_key   : Option<[u8; 32]>,   // Current receiving chain key
    send_count            : u32,                // Messages sent in current chain
    recv_count            : u32,                // Messages received in current chain
    previous_send_count   : u32,                // Messages in previous sending chain
    skipped_keys          : HashMap<(ratchet_key, msg_num), [u8; 32]>,
}
```

All sensitive fields (`root_key`, private keys, chain keys, skipped keys) are zeroized when the session is dropped.

### Initialization

**Initiator** (after X3DH):
```
our_secret = X25519::random(OsRng)
our_public = X25519Public::from(our_secret)
dh_output  = our_secret.diffie_hellman(their_ratchet_key)

(root_key, sending_chain_key) = KDF_RK(shared_secret, dh_output)

Session {
    dh_sending_private  = our_secret
    dh_sending_public   = our_public
    dh_receiving_key    = their_ratchet_key  // Bob's signed prekey
    root_key            = root_key
    sending_chain_key   = sending_chain_key
    receiving_chain_key = None
    send_count          = 0
    recv_count          = 0
    previous_send_count = 0
}
```

**Responder**:
```
Session {
    dh_sending_private  = our_signed_prekey_secret
    dh_sending_public   = X25519Public::from(our_signed_prekey_secret)
    dh_receiving_key    = None
    root_key            = shared_secret
    sending_chain_key   = None
    receiving_chain_key = None
    // All counters = 0
}
```

### KDF Functions

**Root Key Ratchet** (`KDF_RK`):
```
(new_root_key, chain_key) = HKDF-SHA256(
    salt = root_key,
    ikm  = dh_output,
    info = "chatalot-ratchet",
    len  = 64           // first 32 bytes = new root key, last 32 = chain key
)
```

**Chain Key Ratchet** (`KDF_CK`):
```
message_key    = HKDF-SHA256(salt=chain_key, ikm=0x01, info="chatalot-msg-key", len=32)
new_chain_key  = HKDF-SHA256(salt=chain_key, ikm=0x02, info="chatalot-msg-key", len=32)
```

### Encryption

```
1. (msg_key, new_sending_chain_key) = KDF_CK(sending_chain_key)
2. sending_chain_key = new_sending_chain_key
3. header = MessageHeader {
       ratchet_key: dh_sending_public,
       previous_chain_length: previous_send_count,
       message_number: send_count,
   }
4. send_count += 1
5. nonce = random(12 bytes)
6. ciphertext = ChaCha20Poly1305::encrypt(msg_key, nonce, plaintext)
7. return EncryptedMessage { header, ciphertext, nonce }
```

### Decryption

```
1. Check skipped_keys for (header.ratchet_key, header.message_number)
   -> If found, decrypt with cached key and return

2. If header.ratchet_key != dh_receiving_key:
   a. Skip messages in current receiving chain up to header.previous_chain_length
   b. Perform DH ratchet step:
      - previous_send_count = send_count
      - send_count = 0; recv_count = 0
      - dh_receiving_key = header.ratchet_key
      - DH with current dh_sending_private -> derive receiving_chain_key
      - Generate new dh_sending key pair
      - DH with new key pair -> derive sending_chain_key

3. Skip messages up to header.message_number (cache skipped keys)

4. (msg_key, new_receiving_chain_key) = KDF_CK(receiving_chain_key)
5. receiving_chain_key = new_receiving_chain_key
6. recv_count = header.message_number + 1
7. plaintext = ChaCha20Poly1305::decrypt(msg_key, nonce, ciphertext)
```

### Session Serialization

Sessions are serialized to JSON using `serde` and stored in IndexedDB. This allows sessions to persist across page reloads and browser restarts.

```
session_bytes = serde_json::to_vec(session)
session       = serde_json::from_slice(session_bytes)
```

## Sender Key Protocol Specification

### Constants

| Constant | Value |
|----------|-------|
| `SENDER_KEY_INFO` | `b"chatalot-sender-key-chain"` |
| `MAX_SKIP` | 2000 |

### Sender State

```rust
struct SenderKeyState {
    chain_id  : u32,       // Random chain identifier
    chain_key : [u8; 32],  // Current chain key
    iteration : u32,       // Current iteration counter
    sender_id : Vec<u8>,   // Sender's user ID
}
```

### Receiver State

```rust
struct ReceiverKeyState {
    chain_id    : u32,
    chain_key   : [u8; 32],
    iteration   : u32,
    sender_id   : Vec<u8>,
    cached_keys : HashMap<u32, [u8; 32]>,  // iteration -> message_key
}
```

### Chain Advancement

```
message_key    = HKDF-SHA256(salt=chain_key, ikm=0x01, info="chatalot-sender-key-chain", len=32)
new_chain_key  = HKDF-SHA256(salt=chain_key, ikm=0x02, info="chatalot-sender-key-chain", len=32)
```

### Encryption (Sender)

```
1. (msg_key, new_chain_key) = advance_chain(chain_key)
2. chain_key = new_chain_key
3. iteration_used = iteration
4. iteration += 1
5. nonce = random(12 bytes)
6. ciphertext = ChaCha20Poly1305::encrypt(msg_key, nonce, plaintext)
7. return SenderKeyMessage { chain_id, iteration: iteration_used, ciphertext, nonce }
```

### Decryption (Receiver)

```
1. Verify message.chain_id == our chain_id (reject if mismatched)

2. Check cached_keys for message.iteration
   -> If found, decrypt and return

3. If message.iteration > our iteration:
   a. For each step from our iteration to message.iteration:
      - (msg_key, new_chain_key) = advance_chain(chain_key)
      - cached_keys[step] = msg_key
      - chain_key = new_chain_key
   b. iteration = message.iteration

4. If message.iteration < our iteration:
   -> Check cached_keys (must be there from step 3, or message is duplicate)

5. For current iteration:
   (msg_key, new_chain_key) = advance_chain(chain_key)
   chain_key = new_chain_key
   iteration += 1

6. plaintext = ChaCha20Poly1305::decrypt(msg_key, nonce, ciphertext)
```

## Wire Formats

### DM Wire Message (version 1)

```json
{
    "v": 1,
    "x3dh": {                                  // Present only on first message
        "identity_key": [/* 32 bytes as number array */],
        "ephemeral_key": [/* 32 bytes */],
        "signed_prekey_id": 1,
        "one_time_prekey_id": 42               // null if no OPK was available
    },
    "header": {
        "ratchet_key": [/* 32 bytes */],
        "previous_chain_length": 5,
        "message_number": 0
    },
    "ciphertext": [/* variable length */],
    "nonce": [/* 12 bytes */]
}
```

### Sender Key Wire Message (version 1)

```json
{
    "v": 1,
    "sk": true,
    "message": {
        "chain_id": 1234567890,
        "iteration": 7,
        "ciphertext": [/* variable length */],
        "nonce": [/* 12 bytes */]
    }
}
```

### Key Bundle Response (`GET /keys/{user_id}/bundle`)

```json
{
    "identity_key": [/* 32 bytes */],
    "signed_prekey": {
        "key_id": 1,
        "public_key": [/* 32 bytes */],
        "signature": [/* 64 bytes */]
    },
    "one_time_prekey": {
        "key_id": 42,
        "public_key": [/* 32 bytes */]
    }
}
```

### Sender Key Distribution (`POST /channels/{id}/sender-keys`)

```json
{
    "chain_id": 1234567890,
    "distribution": {
        "chain_id": 1234567890,
        "iteration": 0,
        "chain_key": [/* 32 bytes */],
        "sender_id": [/* user ID bytes */]
    }
}
```

## Server API Endpoints

| Method | Path | Purpose |
|--------|------|---------|
| `GET` | `/keys/{user_id}/bundle` | Fetch a user's prekey bundle for X3DH |
| `POST` | `/keys/prekeys/signed` | Upload or rotate a signed prekey |
| `POST` | `/keys/prekeys/one-time` | Upload a batch of one-time prekeys (max 200) |
| `GET` | `/keys/prekeys/count` | Get remaining one-time prekey count |
| `POST` | `/channels/{id}/sender-keys` | Upload Sender Key distribution for a channel |
| `GET` | `/channels/{id}/sender-keys` | Fetch all Sender Key distributions for a channel |

## WebSocket Events

| Event | Direction | Purpose |
|-------|-----------|---------|
| `KeysLow { remaining }` | Server -> Client | One-time prekeys below threshold (25) |
| `SenderKeyUpdated { channel_id, user_id, chain_id, distribution }` | Server -> Client | New/rotated Sender Key for a channel member |

## Client-Side Architecture

### WASM Module

The Rust `chatalot-crypto` crate is compiled to WebAssembly via `wasm-bindgen`. The WASM module exposes the following functions:

| Function | Purpose |
|----------|---------|
| `generate_identity_key()` | Generate Ed25519 key pair |
| `generate_signed_prekey(signing_key, key_id)` | Generate and sign an X25519 prekey |
| `generate_one_time_prekeys(start_id, count)` | Generate batch of X25519 prekeys |
| `x3dh_initiate(signing_key, bundle_json)` | Initiator side of X3DH |
| `x3dh_respond(signing_key, spk, otp, their_ik, their_ek)` | Responder side of X3DH |
| `ratchet_encrypt(session_json, plaintext)` | Double Ratchet encryption |
| `ratchet_decrypt(session_json, message_json)` | Double Ratchet decryption |
| `sender_key_generate(sender_id)` | Generate Sender Key + distribution |
| `sender_key_encrypt(state_json, plaintext)` | Sender Key encryption |
| `sender_key_decrypt(state_json, message_json)` | Sender Key decryption |
| `sender_key_from_distribution(dist_json)` | Initialize receiver state |
| `compute_fingerprint(public_key)` | SHA-256 fingerprint of identity key |
| `compute_safety_number(key_a, key_b)` | Safety number for two identity keys |

### IndexedDB Stores

| Store | Key | Contents |
|-------|-----|----------|
| `identity` | `"self"` | `{ signingKey, verifyingKey }` |
| `signedPrekeys` | `keyId` (keyPath) | `{ keyId, publicKey, privateKey }` |
| `oneTimePrekeys` | `keyId` (keyPath) | `{ keyId, publicKey, privateKey }` |
| `sessions` | peer user ID | Serialized RatchetSession JSON string |
| `peerIdentities` | peer user ID | Identity VerifyingKey (Uint8Array) |
| `decryptedMessages` | message ID | `{ messageId, content, channelId }` |
| `senderKeyStates` | channel ID | Serialized SenderKeyState JSON string |
| `receiverKeyStates` | `"channelId:senderId"` | Serialized ReceiverKeyState JSON string |

### Class Structure

```
initCrypto()
  |
  +-- CryptoStorage          -- IndexedDB wrapper for all key/session storage
  +-- KeyManager              -- Key generation, identity access, prekey replenishment
  +-- SessionManager          -- DM encryption/decryption (X3DH + Double Ratchet)
                                 Group encryption/decryption (Sender Keys)
                                 Sender Key rotation and distribution

decryptMessage()             -- Entry point for message decryption
                                Routes to DM or group based on channel type
                                Falls back to UTF-8 for legacy messages
```

## Source Code References

The encryption implementation is spread across these files:

| File | Contents |
|------|----------|
| `crates/chatalot-crypto/src/lib.rs` | Module declarations |
| `crates/chatalot-crypto/src/x3dh.rs` | X3DH key agreement (initiator + responder) |
| `crates/chatalot-crypto/src/double_ratchet.rs` | Double Ratchet session (encrypt, decrypt, ratchet steps) |
| `crates/chatalot-crypto/src/sender_keys.rs` | Sender Key generation, distribution, encrypt, decrypt |
| `crates/chatalot-crypto/src/aead.rs` | ChaCha20-Poly1305 encrypt/decrypt + key/nonce generation |
| `crates/chatalot-crypto/src/identity.rs` | Identity key generation, fingerprints, safety numbers |
| `crates/chatalot-crypto/src/types.rs` | SecretKey (zeroizable) and Fingerprint types |
| `clients/web/src/lib/crypto/index.ts` | Crypto subsystem initialization and access |
| `clients/web/src/lib/crypto/wasm-loader.ts` | Lazy WASM module loading |
| `clients/web/src/lib/crypto/key-manager.ts` | Key generation and prekey replenishment |
| `clients/web/src/lib/crypto/session-manager.ts` | DM and group encryption/decryption orchestration |
| `clients/web/src/lib/crypto/storage.ts` | IndexedDB storage for keys, sessions, and caches |
| `clients/web/src/lib/crypto/decrypt.ts` | Unified message decryption entry point |
| `crates/chatalot-server/src/routes/keys.rs` | Server-side prekey bundle and OTP endpoints |
| `crates/chatalot-server/src/routes/sender_keys.rs` | Server-side Sender Key distribution endpoints |

## External References

- [X3DH Key Agreement Protocol](https://signal.org/docs/specifications/x3dh/) -- Signal Foundation specification
- [Double Ratchet Algorithm](https://signal.org/docs/specifications/doubleratchet/) -- Signal Foundation specification
- [Sender Keys](https://signal.org/blog/private-groups/) -- Signal blog post on group messaging
- [ChaCha20-Poly1305 (RFC 8439)](https://datatracker.ietf.org/doc/html/rfc8439) -- IETF specification
- [HKDF (RFC 5869)](https://datatracker.ietf.org/doc/html/rfc5869) -- HMAC-based Key Derivation Function
- [Ed25519 (RFC 8032)](https://datatracker.ietf.org/doc/html/rfc8032) -- Edwards-Curve Digital Signature Algorithm
- [X25519 (RFC 7748)](https://datatracker.ietf.org/doc/html/rfc7748) -- Elliptic Curves for Security

## Next Steps

- [Overview](./overview.md) -- High-level introduction to encryption in Chatalot
- [DM Encryption](./dm-encryption.md) -- Narrative walkthrough of the DM protocol
- [Group Encryption](./group-encryption.md) -- Narrative walkthrough of Sender Keys
- [Limitations](./limitations.md) -- Security boundaries and tradeoffs
