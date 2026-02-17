# DM Encryption

How Chatalot encrypts direct messages using X3DH key agreement and the Double Ratchet protocol.

> **Status: Beta** -- Infrastructure complete, WASM bridge pending for full client integration.

---

## Overview

Direct messages between two users are encrypted with the full Signal Protocol:

1. **X3DH** (Extended Triple Diffie-Hellman) establishes a shared secret between two users, even if one is offline.
2. **Double Ratchet** uses that shared secret to encrypt every subsequent message with a unique key, providing forward secrecy and break-in recovery.

The combination means that each message is encrypted with a key that is used exactly once and then destroyed. Compromising any single key reveals only that one message.

## X3DH Key Agreement

X3DH solves a fundamental problem: how do two people establish a shared secret when one of them might be offline? The answer is prekeys -- public keys published to the server ahead of time.

### The Handshake

Suppose Alice wants to send her first message to Bob.

**Step 1: Alice fetches Bob's prekey bundle from the server.**

The bundle contains:
- `IK_B` -- Bob's Ed25519 identity public key
- `SPK_B` -- Bob's signed X25519 prekey + Ed25519 signature
- `OPK_B` -- One of Bob's one-time X25519 prekeys (if available)

**Step 2: Alice verifies the signed prekey.**

Alice checks that `SPK_B` was genuinely signed by `IK_B`. If the signature is invalid, the handshake is rejected. This prevents the server from substituting a malicious prekey.

**Step 3: Alice generates an ephemeral key pair.**

```
EK_A_secret = X25519::random()
EK_A_public = X25519Public::from(EK_A_secret)
```

**Step 4: Alice computes four Diffie-Hellman operations.**

Alice converts her Ed25519 identity key to X25519 (using the birational map between the Ed25519 curve and Curve25519), then computes:

```
DH1 = X25519(IK_A_x25519,  SPK_B)       -- Identity <-> Signed Prekey
DH2 = X25519(EK_A,          IK_B_x25519) -- Ephemeral <-> Identity
DH3 = X25519(EK_A,          SPK_B)       -- Ephemeral <-> Signed Prekey
DH4 = X25519(EK_A,          OPK_B)       -- Ephemeral <-> One-Time Prekey
                                             (omitted if no OPK available)
```

Each DH operation provides a different security property:
- **DH1**: Mutual authentication (both long-term keys involved)
- **DH2**: Ensures only Bob's identity can derive the secret
- **DH3**: Ties the ephemeral key to Bob's signed prekey
- **DH4**: Provides forward secrecy for this specific handshake (the OPK is consumed and deleted)

**Step 5: Alice derives the shared secret.**

The DH outputs are concatenated with a 32-byte filler prefix (per the X3DH specification) and fed into HKDF-SHA256:

```
input = 0xFF[32] || DH1 || DH2 || DH3 || DH4
SK = HKDF-SHA256(salt=0x00[32], ikm=input, info="chatalot-x3dh-shared-secret")
```

The result is a 32-byte shared secret (`SK`).

**Step 6: Associated data is computed.**

```
AD = IK_A_public || IK_B_public
```

This associated data is used in the first encrypted message to bind the ciphertext to both identity keys, preventing certain man-in-the-middle attacks.

**Step 7: Alice initializes her Double Ratchet session and sends.**

Alice initializes a Double Ratchet session using `SK` and Bob's signed prekey as the initial remote ratchet key. She encrypts her first message and sends it along with an X3DH header:

```json
{
    "v": 1,
    "x3dh": {
        "identity_key": [/* Alice's identity public key */],
        "ephemeral_key": [/* Alice's ephemeral public key */],
        "signed_prekey_id": 1,
        "one_time_prekey_id": 42
    },
    "header": { /* Double Ratchet message header */ },
    "ciphertext": [/* encrypted bytes */],
    "nonce": [/* 12-byte nonce */]
}
```

**Step 8: Bob receives and processes.**

Bob's client:
1. Looks up his private keys for the prekeys Alice referenced (signed prekey, one-time prekey)
2. Performs the same DH computations (with keys swapped: `DH1 = X25519(SPK_B, IK_A_x25519)`, etc.)
3. Derives the same shared secret `SK`
4. Initializes his side of the Double Ratchet
5. Decrypts Alice's message
6. Deletes the consumed one-time prekey (it was single-use)
7. Stores Alice's identity key under trust-on-first-use (TOFU)

### Without a One-Time Prekey

If Bob has no one-time prekeys remaining on the server, the X3DH handshake proceeds with only three DH operations (DH4 is omitted). This is still secure, but the first message lacks the additional forward secrecy that the one-time prekey provides. The client automatically replenishes one-time prekeys when the count drops low.

## The Double Ratchet

Once X3DH establishes the shared secret, the Double Ratchet takes over for all subsequent messages. It consists of two interlocking ratchets:

### Symmetric Ratchet (Chain Key)

Each message advances a chain key to derive a unique message key:

```
message_key[n]    = HKDF-SHA256(salt=chain_key[n], ikm=0x01, info="chatalot-msg-key")
chain_key[n+1]    = HKDF-SHA256(salt=chain_key[n], ikm=0x02, info="chatalot-msg-key")
```

The message key is used once for ChaCha20-Poly1305 encryption, then zeroized from memory. The chain key advances to the next value. This means:

- **Past message keys cannot be derived**: Once `chain_key[n]` advances to `chain_key[n+1]`, there is no way to go back. This is forward secrecy within a chain.
- **Future message keys can be derived**: If an attacker obtains `chain_key[n]`, they can derive `message_key[n]` and all future keys in that chain. This is why the DH ratchet is needed.

### DH Ratchet

When the conversation direction changes (Alice sends, then Bob replies), a DH ratchet step occurs:

1. Bob generates a new X25519 ratchet key pair
2. Bob computes `DH(new_bob_private, alice_current_public)`
3. The DH output is fed into HKDF with the current root key to derive:
   - A new root key
   - A new receiving chain key
4. Bob generates another new ratchet key pair for sending
5. Bob computes another DH with Alice's key and derives:
   - Another new root key
   - A new sending chain key

This means that after each turn of conversation, entirely new key material is generated. An attacker who compromises Bob's current keys loses access as soon as the next DH ratchet step occurs. This property is called **break-in recovery** (or sometimes "future secrecy" or "post-compromise security").

### Forward Secrecy Explained

Forward secrecy means that compromising a key today does not reveal messages encrypted in the past. In the Double Ratchet:

- Each message uses a unique message key derived from the chain
- Message keys are immediately deleted after use
- Chain keys are advanced and cannot be reversed
- Even the root key changes with each DH ratchet step

Imagine you have a sequence of locked boxes, and each key self-destructs after opening its box. Even if someone steals the key to box #50, they cannot open boxes #1 through #49 because those keys no longer exist.

### Break-in Recovery Explained

Break-in recovery means that even if an attacker compromises your current session state, future messages become secure again after the next key exchange.

In the Double Ratchet, this happens naturally: when the other party sends a message with a new ratchet key, a new DH computation produces fresh key material that the attacker cannot derive (because they do not know the other party's new private key).

Imagine the attacker copies your entire key ring. As soon as you and your contact exchange one more round of messages, you both generate new keys the attacker does not have. The break-in is recovered from automatically.

### Out-of-Order Messages

Network delivery is not always in order. The Double Ratchet handles this gracefully:

- Each message includes a counter (message number within its chain)
- If a message arrives ahead of others, the receiver advances the chain and caches the skipped message keys
- When the skipped message finally arrives, the cached key is used to decrypt it
- A maximum of 1000 skipped keys are cached to prevent denial-of-service attacks

### Message Format

Each encrypted DM is packaged as a wire message:

```json
{
    "v": 1,
    "x3dh": { ... },              // Only on the first message
    "header": {
        "ratchet_key": [/* 32 bytes - sender's current ratchet public key */],
        "previous_chain_length": 5,
        "message_number": 0
    },
    "ciphertext": [/* encrypted bytes */],
    "nonce": [/* 12 bytes */]
}
```

The `v` field is the wire format version (currently 1). The `x3dh` field is present only on the first message in a new session. The `header` carries the ratchet state needed for the recipient to decrypt. The `ciphertext` is the ChaCha20-Poly1305 output. The `nonce` is a random 96-bit value.

### Session Persistence

Double Ratchet sessions are serialized to JSON and stored in IndexedDB (keyed by peer user ID). The session state includes:

- Current root key
- Sending and receiving chain keys
- DH ratchet key pairs (private bytes + public bytes)
- Message counters (send count, receive count, previous chain length)
- Skipped message keys cache

Sessions survive page reloads and browser restarts. They are wiped on logout (via `wipeCrypto()`).

### Decrypted Message Cache

Successfully decrypted messages are cached in IndexedDB (in the `decryptedMessages` store) so that they do not need to be decrypted again when scrolling through message history. The cache is keyed by message ID and stores the plaintext along with the channel ID.

## Fallback Behavior

During the beta period, the client includes a fallback mechanism:

1. Attempt to parse incoming bytes as a wire message (JSON with `v: 1`)
2. If parsing succeeds and a session exists (or an X3DH header is present), decrypt normally
3. If parsing fails or no session can be established, interpret the bytes as plaintext UTF-8

This ensures messages are always readable, even when encryption is not yet active for a particular conversation.

## Next Steps

- [Group Encryption](./group-encryption.md) -- How group channels use Sender Keys instead of Double Ratchet
- [Key Management](./key-management.md) -- Key types, generation, and rotation
- [Technical Details](./technical-details.md) -- Wire formats, primitives, and the session state machine
