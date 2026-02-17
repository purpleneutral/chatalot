# Testing

> **Status: Complete**

Chatalot includes a test suite covering cryptographic operations, security utilities, and input validation. This page describes how to run tests, what is tested, and testing conventions.

## Running Tests

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p chatalot-crypto

# Run a specific test by name
cargo test test_basic_exchange

# Run with output visible
cargo test -- --nocapture
```

## Test Suite Overview

### Crypto Tests (chatalot-crypto) -- 23 tests

The crypto crate has the most comprehensive test coverage, validating all cryptographic primitives and protocols.

**AEAD (ChaCha20-Poly1305) -- 3 tests**

| Test | Description |
|------|-------------|
| `test_encrypt_decrypt_roundtrip` | Encrypts and decrypts data, verifies plaintext matches |
| `test_wrong_key_fails` | Decryption with wrong key returns error |
| `test_tampered_ciphertext_fails` | Modified ciphertext is rejected (integrity check) |

**X3DH Key Agreement -- 4 tests**

| Test | Description |
|------|-------------|
| `test_x3dh_initiator_responder` | Full handshake: both sides derive the same shared secret |
| `test_x3dh_without_one_time_prekey` | Handshake succeeds with only 3 DH operations (no OTP) |
| `test_x3dh_invalid_signature` | Rejects prekey bundle with forged signature |
| `test_x3dh_associated_data` | Verifies both sides compute identical associated data |

**Double Ratchet -- 7 tests**

| Test | Description |
|------|-------------|
| `test_basic_exchange` | Alice encrypts, Bob decrypts successfully |
| `test_tampered_ciphertext` | Modified ciphertext is rejected |
| `test_out_of_order` | Messages delivered out of order are decrypted correctly |
| `test_multiple_messages` | Multiple sequential messages in one direction |
| `test_session_serialization` | Session serializes to JSON and deserializes correctly |
| `test_ping_pong` | Alternating send/receive triggers DH ratchet steps |
| `test_large_message` | Large payload (10 KB) encrypts and decrypts correctly |

**Sender Keys (Group Encryption) -- 5 tests**

| Test | Description |
|------|-------------|
| `test_basic_sender_key` | Sender encrypts, receiver decrypts |
| `test_multiple_recipients` | One sender key works for multiple receivers |
| `test_sender_key_serialization` | State serializes and deserializes correctly |
| `test_tampered_sender_key` | Modified ciphertext is rejected |
| `test_out_of_order_sender_key` | Out-of-order messages handled with key caching |

**Identity -- 2 tests**

| Test | Description |
|------|-------------|
| `test_fingerprint_deterministic` | Same key always produces the same fingerprint |
| `test_safety_number_commutative` | safety_number(A, B) == safety_number(B, A) |

### Server Tests

**CSS Sanitizer** -- Tests in `crates/chatalot-server/src/services/css_sanitizer.rs`
- Validates CSS sanitization for custom themes (blocks dangerous properties, allows safe ones)

**File Security** -- Tests in `crates/chatalot-server/src/services/file_security.rs`
- Validates file type detection, content type verification, and security checks

## Linting and Type Checking

```bash
# Rust linting (should pass with 3 acceptable too_many_arguments warnings)
cargo clippy -- -W clippy::all

# Svelte/TypeScript type checking
cd clients/web && npm run check

# Web client build (should produce 0 warnings)
cd clients/web && npm run build
```

## Known Acceptable Warnings

The following clippy warnings are expected and acceptable:
- 3 `too_many_arguments` warnings on database repository functions (these functions naturally take many parameters)

## Testing Conventions

- **Unit tests** are embedded in source files using `#[cfg(test)]` modules
- **Test names** use `test_` prefix with snake_case descriptions
- **Crypto tests** verify both happy path and failure cases (wrong keys, tampered data, invalid signatures)
- **Assertions** use standard `assert!`, `assert_eq!`, and pattern matching on `Result` types
- No external test framework beyond Rust's built-in `#[test]`

## What Is Not Tested

The following areas rely on manual testing:
- WebSocket message handling (integration-level)
- REST API endpoints (integration-level)
- Database repository functions (require live PostgreSQL)
- Web client UI components
- Voice/video WebRTC flows

> **Roadmap:** Integration tests with a test database and API client are planned for a future release.

## Manual Testing Checklist

When making changes, verify these core flows manually:

1. **Registration** -- Create account, receive recovery code
2. **Login/Logout** -- Login, token refresh, logout
3. **Messaging** -- Send, edit, delete, reply, react
4. **Threads** -- Create thread, reply in thread
5. **Voice** -- Join/leave voice channel, mute/unmute
6. **File upload** -- Upload file, view preview, download
7. **Admin panel** -- User management, audit log
8. **Mobile** -- Responsive layout on small screens

## Related Pages

- [Building from Source](./building-from-source.md) -- Build prerequisites and commands
- [Contributing](./contributing.md) -- Contribution guidelines
- [Crypto Implementation](./crypto-implementation.md) -- What the crypto tests verify
