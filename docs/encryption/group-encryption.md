# Group Encryption

How Chatalot encrypts group channel messages using the Sender Keys protocol.

> **Status: Beta** -- Infrastructure complete, WASM bridge pending for full client integration.

---

## Why Group Encryption Is Different

In a direct message, there are exactly two participants. The Double Ratchet works perfectly: each party maintains one session, and every message is encrypted and decrypted with that session.

In a group channel with N members, using pairwise Double Ratchet would mean:
- The sender encrypts the message N-1 times (once for each other member)
- The server stores N-1 ciphertext copies
- Each recipient decrypts their own copy

For a channel with 50 members, every single message would require 49 separate encryption operations. This does not scale.

**Sender Keys** solve this. Each member generates a single symmetric key and distributes it to all other members. When that member sends a message, they encrypt it once with their Sender Key. All recipients can decrypt using the copy of that sender's key they received earlier.

```
Without Sender Keys (pairwise):          With Sender Keys:

Alice encrypts for Bob     -> Bob         Alice encrypts once -> Server fans out
Alice encrypts for Carol   -> Carol         Bob decrypts with Alice's Sender Key
Alice encrypts for Dave    -> Dave          Carol decrypts with Alice's Sender Key
Alice encrypts for Eve     -> Eve           Dave decrypts with Alice's Sender Key
  (4 encryptions)                           Eve decrypts with Alice's Sender Key
                                            (1 encryption)
```

## How Sender Keys Work

### Key Generation

When a member sends their first message in a group channel (or after a key rotation), their client generates a new Sender Key:

1. Generate a random 256-bit chain key
2. Assign a random 32-bit chain ID
3. Set the iteration counter to 0

```
chain_key = random(32 bytes)
chain_id  = random(u32)
iteration = 0
```

### Key Distribution

The Sender Key distribution is uploaded to the server via `POST /channels/{id}/sender-keys`:

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

The server:
1. Stores the distribution in the database
2. Broadcasts a `SenderKeyUpdated` WebSocket event to all channel members

When a member receives the `SenderKeyUpdated` event, their client initializes a receiver key state from the distribution, allowing them to decrypt future messages from that sender.

If a member joins the channel later or was offline during the broadcast, they can fetch all current distributions via `GET /channels/{id}/sender-keys`.

### Sending a Message

When a member sends a message:

1. **Advance the chain**: Derive a message key and a new chain key from the current chain key.

```
message_key   = HKDF-SHA256(salt=chain_key, ikm=0x01, info="chatalot-sender-key-chain")
new_chain_key = HKDF-SHA256(salt=chain_key, ikm=0x02, info="chatalot-sender-key-chain")
```

2. **Encrypt**: Use the message key with ChaCha20-Poly1305 to encrypt the plaintext.

3. **Package**: Wrap the ciphertext in a Sender Key wire message.

```json
{
    "v": 1,
    "sk": true,
    "message": {
        "chain_id": 1234567890,
        "iteration": 7,
        "ciphertext": [/* encrypted bytes */],
        "nonce": [/* 12 bytes */]
    }
}
```

4. **Send**: The message is sent to the server, which fans it out to all channel members.

### Receiving a Message

When a member receives a Sender Key message:

1. **Identify the sender**: Look up the receiver key state for this sender in this channel, using the `chain_id` field.

2. **Fetch if missing**: If no receiver state exists (e.g., the member was offline during distribution), fetch the sender's distribution from the server.

3. **Advance to the correct iteration**: If the message iteration is ahead of the receiver's current position, advance the chain and cache intermediate message keys for out-of-order delivery.

4. **Decrypt**: Use the derived message key with ChaCha20-Poly1305.

### Out-of-Order Messages

Just like the Double Ratchet, Sender Keys support out-of-order message delivery:

- If a message arrives from the future (higher iteration than expected), the receiver advances the chain and caches the intermediate keys
- When the skipped message arrives, the cached key is used
- A maximum of 2000 skipped keys are cached per sender to prevent abuse

## Sender Key Rotation

Sender Keys must be rotated when the channel membership changes to maintain security guarantees.

### When a Member Leaves or Is Removed

If a member departs the channel (voluntarily or by removal):

1. All remaining members **delete** their Sender Key state for that channel
2. All remaining members **delete** all receiver key states for that channel
3. On the next message send, each member generates a fresh Sender Key and distributes it

This ensures the removed member cannot decrypt any messages sent after their departure. Even though they had everyone's previous Sender Keys, the new keys are entirely independent.

### When a Member Joins

When a new member joins the channel:

1. They fetch all current Sender Key distributions from the server
2. Existing members do not need to rotate their keys (the new member receives the current distributions)
3. The new member generates their own Sender Key and uploads it

Note: The new member can decrypt messages sent after they received the Sender Key distributions, but not messages sent before they joined (because they did not have the Sender Keys at those earlier iterations).

## Tradeoffs vs. Pairwise Double Ratchet

Sender Keys make a deliberate tradeoff: efficiency for weaker security properties.

| Property | DM (Double Ratchet) | Group (Sender Keys) |
|----------|-------------------|-------------------|
| Encryption operations per message | 1 | 1 |
| Forward secrecy | Per-message (chain ratchet + DH ratchet) | Per-message (chain ratchet only) |
| Break-in recovery | Yes (DH ratchet) | Only on key rotation |
| Key rotation trigger | Every turn of conversation | Membership changes |
| Out-of-order support | Yes (up to 1000 skipped) | Yes (up to 2000 skipped) |

The key differences:

- **No DH ratchet in groups**: Sender Keys use a symmetric chain ratchet but not a DH ratchet. This means there is no automatic break-in recovery during normal messaging. If an attacker compromises a Sender Key, they can decrypt all future messages from that sender until the key is rotated.

- **Rotation is event-driven, not continuous**: Sender Keys are rotated when membership changes, not on every turn of conversation. This is a practical tradeoff -- rotating keys on every message in a 50-person channel would negate the efficiency benefit.

- **Forward secrecy is per-chain**: Within a single Sender Key chain, each message uses a unique derived key and the chain advances forward. Compromising iteration N does not reveal iterations 0 through N-1. However, it does reveal all future iterations until rotation.

For most group messaging scenarios, this tradeoff is acceptable. The Signal app, WhatsApp, and other major messaging apps use similar Sender Key approaches for group messaging.

## Sender Key Wire Format

### Distribution Message (stored on server)

```json
{
    "chain_id": 1234567890,
    "iteration": 0,
    "chain_key": [/* 32 bytes */],
    "sender_id": [/* user ID bytes */]
}
```

### Encrypted Message (sent to channel)

```json
{
    "v": 1,
    "sk": true,
    "message": {
        "chain_id": 1234567890,
        "iteration": 7,
        "ciphertext": [/* ChaCha20-Poly1305 output */],
        "nonce": [/* 12 bytes */]
    }
}
```

The `v: 1` and `sk: true` fields identify this as a Sender Key wire message (as opposed to a Double Ratchet DM wire message which has `v: 1` but no `sk` field).

## Fallback Behavior

The client falls back to plain UTF-8 decoding if:
- The message bytes cannot be parsed as a Sender Key wire message
- No receiver key state exists and cannot be fetched from the server
- Decryption fails for any reason

This ensures backward compatibility with messages sent before encryption was enabled on a channel.

## Next Steps

- [DM Encryption](./dm-encryption.md) -- The full Double Ratchet protocol for direct messages
- [Key Management](./key-management.md) -- How all key types are generated and stored
- [Limitations](./limitations.md) -- Security boundaries of the Sender Key approach
- [Technical Details](./technical-details.md) -- Cryptographic primitives and exact algorithms
