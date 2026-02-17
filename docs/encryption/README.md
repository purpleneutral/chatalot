# Encryption

Chatalot uses Signal Protocol-based end-to-end encryption to protect your messages. The server never sees plaintext message content -- only encrypted blobs pass through it.

> **Status: Beta** -- The cryptographic infrastructure is fully implemented and tested in the `chatalot-crypto` Rust crate. The WASM bridge to the web client is in progress. During this beta period, some messages may fall back to plaintext transport when the WASM module is not yet loaded.

## Pages

| # | Page | Description |
|---|------|-------------|
| 1 | [Overview](./overview.md) | Why E2E encryption matters and what Signal Protocol provides |
| 2 | [How It Works](./how-it-works.md) | High-level encryption flow for DMs and group channels |
| 3 | [Key Management](./key-management.md) | Identity keys, prekeys, key generation, rotation, and storage |
| 4 | [DM Encryption](./dm-encryption.md) | X3DH key agreement and Double Ratchet for private messages |
| 5 | [Group Encryption](./group-encryption.md) | Sender Keys for efficient group message encryption |
| 6 | [Verification](./verification.md) | Safety numbers, fingerprints, and trust-on-first-use |
| 7 | [Limitations](./limitations.md) | What is not encrypted and other security boundaries |
| 8 | [Technical Details](./technical-details.md) | Cryptographic primitives, wire formats, and protocol specification |

## Implementation Status

| Component | Status |
|-----------|--------|
| ChaCha20-Poly1305 AEAD | Implemented and tested |
| Ed25519 identity keys | Implemented and tested |
| X3DH key agreement | Implemented and tested |
| Double Ratchet | Implemented and tested |
| Sender Keys (groups) | Implemented and tested |
| Safety numbers | Implemented and tested |
| WASM bridge | Compiled and bundled |
| Web client integration | In progress |
| Key storage (IndexedDB) | Implemented |
| Key storage (desktop) | Planned (OS keychain) |

The crypto library includes 23 unit tests covering all protocols, including edge cases such as out-of-order messages, tampered ciphertext, invalid signatures, and session serialization round-trips.

## Source Code

The encryption implementation lives in the following locations:

- **Rust crypto crate**: `crates/chatalot-crypto/src/` -- X3DH, Double Ratchet, Sender Keys, AEAD, identity management
- **WASM bindings**: `crates/chatalot-crypto-wasm/` -- wasm-bindgen exports for browser use
- **Web client crypto**: `clients/web/src/lib/crypto/` -- KeyManager, SessionManager, CryptoStorage (IndexedDB)
- **Server key routes**: `crates/chatalot-server/src/routes/keys.rs` -- prekey bundle exchange
- **Server sender key routes**: `crates/chatalot-server/src/routes/sender_keys.rs` -- group sender key distribution
