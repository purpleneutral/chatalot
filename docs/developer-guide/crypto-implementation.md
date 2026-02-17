# Crypto Implementation

> **Status: Beta** -- Cryptographic primitives and protocols are fully implemented in Rust. WASM compilation works. Client integration is in progress.

The `chatalot-crypto` crate implements the end-to-end encryption protocols used by Chatalot: X3DH for key agreement, Double Ratchet for DM encryption, and Sender Keys for group encryption.

## Architecture

```
chatalot-crypto (pure Rust)
├── aead.rs          -- ChaCha20-Poly1305 symmetric encryption
├── x3dh.rs          -- Extended Triple Diffie-Hellman key agreement
├── double_ratchet.rs -- Double Ratchet protocol for DMs
├── sender_keys.rs   -- Sender Key protocol for groups
├── identity.rs      -- Identity key fingerprints and safety numbers
├── types.rs         -- SecretKey, Fingerprint types
└── lib.rs           -- Public API

chatalot-crypto-wasm (WASM bindings)
├── Cargo.toml       -- crate-type: cdylib + rlib
└── src/lib.rs       -- wasm-bindgen exports
```

## Cryptographic Primitives

| Primitive | Algorithm | Library |
|-----------|-----------|---------|
| Symmetric encryption | ChaCha20-Poly1305 (AEAD) | `chacha20poly1305` |
| Key exchange | X25519 (Curve25519 ECDH) | `x25519-dalek` v2 |
| Signatures | Ed25519 | `ed25519-dalek` |
| Key derivation | HKDF-SHA256 | `hkdf` + `sha2` |
| Random | OS-provided CSPRNG | `rand_core::OsRng` |

## AEAD (aead.rs)

ChaCha20-Poly1305 provides authenticated encryption with associated data.

Functions:
- `generate_key() -> [u8; 32]` -- 256-bit random key
- `generate_nonce() -> [u8; 12]` -- 96-bit random nonce
- `encrypt(key, nonce, plaintext) -> Result<Vec<u8>, AeadError>`
- `decrypt(key, nonce, ciphertext) -> Result<Vec<u8>, AeadError>`

Every message key is used exactly once, then discarded. The nonce is random (not sequential), which is safe because each key is single-use.

## X3DH Key Agreement (x3dh.rs)

X3DH establishes a shared secret between two parties, even when one is offline. Used for the initial handshake of DM conversations.

### Key Types

| Key | Curve | Purpose | Lifetime |
|-----|-------|---------|----------|
| Identity key | Ed25519 | Long-term identity, signs prekeys | Permanent |
| Signed prekey | X25519 | Published to server, rotatable | Weeks/months |
| One-time prekey | X25519 | Single-use, consumed on first contact | One use |
| Ephemeral key | X25519 | Generated per handshake | One handshake |

### Algorithm

The initiator fetches the responder's prekey bundle and performs 4 DH operations:

```
DH1 = X25519(IK_A_x25519, SPK_B)     -- Identity ↔ Signed Prekey
DH2 = X25519(EK_A, IK_B_x25519)      -- Ephemeral ↔ Identity
DH3 = X25519(EK_A, SPK_B)            -- Ephemeral ↔ Signed Prekey
DH4 = X25519(EK_A, OPK_B)            -- Ephemeral ↔ One-Time Prekey (optional)
```

Ed25519 identity keys are converted to X25519 using the birational map between curves.

Shared secret derivation:
```
input = 0xFF[32] || DH1 || DH2 || DH3 || DH4
SK = HKDF-SHA256(salt=0x00[32], ikm=input, info="chatalot-x3dh-shared-secret")
```

Associated data: `AD = IK_A_public || IK_B_public`

### Constants

- `X3DH_INFO = "chatalot-x3dh-shared-secret"`
- `KDF_FILLER = [0xFF; 32]` (per X3DH specification)
- Initial one-time prekeys: 100 per user
- Low threshold: 20 (server sends `keys_low` warning)

## Double Ratchet (double_ratchet.rs)

After X3DH establishes a shared secret, the Double Ratchet provides ongoing encryption with forward secrecy and break-in recovery.

### Session State

| Field | Type | Description |
|-------|------|-------------|
| `root_key` | [u8; 32] | Current root key |
| `sending_chain_key` | [u8; 32] | Sending chain key |
| `receiving_chain_key` | [u8; 32] | Receiving chain key |
| `dh_sending_private` | [u8; 32] | Our current ratchet private key |
| `dh_sending_public` | [u8; 32] | Our current ratchet public key |
| `dh_receiving_key` | [u8; 32] | Their current ratchet public key |
| `send_count` | u32 | Messages sent in current chain |
| `recv_count` | u32 | Messages received in current chain |
| `previous_send_count` | u32 | Messages in previous sending chain |
| `skipped_keys` | HashMap | Cached keys for out-of-order messages |

### Key Derivation

Message key derivation from chain key:
```
message_key = HKDF-SHA256(salt=chain_key, ikm=0x01, info="chatalot-msg-key")
next_chain_key = HKDF-SHA256(salt=chain_key, ikm=0x02, info="chatalot-msg-key")
```

DH ratchet step (on conversation turn change):
```
dh_output = X25519(new_private, their_public)
(new_root_key, new_chain_key) = HKDF-SHA256(root_key, dh_output, "chatalot-ratchet")
```

### Message Header

```json
{
  "ratchet_key": [/* 32 bytes */],
  "previous_chain_length": 5,
  "message_number": 0
}
```

### Constants

- `MAX_SKIP = 1000` (maximum cached out-of-order keys)
- `RATCHET_INFO = "chatalot-ratchet"` (HKDF info for DH ratchet)
- `MSG_KEY_INFO = "chatalot-msg-key"` (HKDF info for message keys)

### Security Properties

- **Forward secrecy:** Past message keys cannot be derived from current state
- **Break-in recovery:** Compromised keys become stale after the next DH ratchet step
- **Out-of-order tolerance:** Up to 1000 skipped messages handled via key caching

## Sender Keys (sender_keys.rs)

Sender Keys provide efficient group encryption. Instead of encrypting each message N times (once per recipient), each sender distributes a symmetric chain key to all group members.

### How It Works

1. Each group member generates a Sender Key (chain_id + chain_key + sender_id)
2. The Sender Key distribution is sent to all other members via their existing pairwise Double Ratchet sessions
3. To send a group message: encrypt once with sender's chain key
4. All recipients decrypt with the sender's distributed key
5. Chain key advances after each message (HKDF)

### Data Structures

**SenderKeyDistribution** (sent to each group member):
```json
{
  "chain_id": 1,
  "iteration": 0,
  "chain_key": [/* 32 bytes */],
  "sender_id": [/* sender's public key */]
}
```

**SenderKeyState** (sender side): chain_id, chain_key, iteration, sender_id
**ReceiverKeyState** (receiver side): same + cached_keys HashMap for out-of-order

### Constants

- `SENDER_KEY_INFO = "chatalot-sender-key-chain"` (HKDF info)
- `MAX_SKIP = 2000` (higher than Double Ratchet, groups have more out-of-order potential)

### Key Rotation

Sender Keys are rotated when:
- A member is removed from the group (all remaining members regenerate)
- The server sends a `sender_key_rotation_required` message
- A distribution is suspected compromised

## Identity and Fingerprints (identity.rs)

- **Fingerprint:** SHA-256 hash of Ed25519 public key, formatted as hex blocks ("AB12 CD34 ...")
- **Safety number:** Commutative hash of two identity keys, used for out-of-band verification
  - `safety_number(A, B) == safety_number(B, A)`

## Secure Memory

All key types implement `Zeroize` from the `zeroize` crate. Private keys, chain keys, and message keys are zeroed from memory when dropped. The `SecretKey` wrapper type (`types.rs`) enforces this for 32-byte secrets.

## WASM Compilation

The `chatalot-crypto-wasm` crate wraps the pure Rust crypto library for browser use:

- Built with `wasm-pack build --target web`
- Uses `wasm-bindgen` for JavaScript bindings
- Uses `serde-wasm-bindgen` for serializing types across the JS/WASM boundary
- Uses `getrandom` with `js` feature for `crypto.getRandomValues()` in browsers
- Output: `clients/web/src/lib/crypto/wasm/`

Build profile:
- `opt-level = "s"` (size optimization)
- `lto = true` (link-time optimization)

### Build Command

```bash
./scripts/build-wasm.sh          # Release (optimized)
./scripts/build-wasm.sh --dev    # Development (faster)
```

## Client Integration

In the browser, crypto sessions are:
- Stored in IndexedDB (keyed by peer user ID for DMs, channel ID for groups)
- Decrypted messages cached in IndexedDB (`decryptedMessages` store) to avoid re-decryption
- Wiped on logout via `wipeCrypto()`

## Related Pages

- [Encryption Overview](../encryption/overview.md) -- User-facing encryption documentation
- [DM Encryption](../encryption/dm-encryption.md) -- How DM encryption works for users
- [Group Encryption](../encryption/group-encryption.md) -- Sender Keys from the user perspective
- [Authentication](./authentication.md) -- JWT and identity key management
