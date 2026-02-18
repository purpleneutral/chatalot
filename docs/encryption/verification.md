# Verification

How to verify that you are communicating with the right person, and what Chatalot does (and plans to do) to help.

> **Status: Complete** -- Safety numbers, fingerprints, verification modal, and TOFU key change warnings are fully implemented.

---

## Why Verification Matters

End-to-end encryption ensures the server cannot read your messages, but it does not inherently guarantee that you are talking to the right person. An attacker who controls the server could, in theory, substitute their own identity key when you fetch someone's prekey bundle. You would establish an encrypted session -- but with the attacker, not your intended contact.

Verification solves this by letting you confirm, through an independent channel, that the identity key you see matches the one your contact actually has.

## Trust on First Use (TOFU)

Currently, Chatalot uses **Trust on First Use** -- the same approach used by SSH, many XMPP clients, and Signal before verification was added.

Here is how it works:

1. The first time you start an encrypted session with someone, their identity key is stored locally in your browser's IndexedDB (`peerIdentities` store).
2. On subsequent sessions, the client checks whether the stored identity key matches the one in the new session's X3DH header.
3. If the key has changed, it indicates the user has re-registered or their device has changed. (In the future, a warning will be displayed.)

TOFU provides reasonable protection: if you established your first session with the genuine user, future impersonation attempts will be detected. The vulnerability is limited to the very first contact, before any key has been stored.

## Safety Numbers

Chatalot implements **safety numbers** -- a deterministic numeric string computed from two users' identity keys. If both users compute the same safety number, they can be confident they have the correct keys for each other.

### How Safety Numbers Are Computed

The computation is straightforward and deterministic regardless of argument order:

1. Sort the two identity public keys lexicographically (by raw bytes)
2. Concatenate them: `sorted_key_1 || sorted_key_2`
3. Hash with SHA-256
4. Convert the hash to numeric blocks: each 4-byte chunk is converted to a 5-digit decimal number (modulo 100,000)

The result looks like:

```
34821 09472 65130 82947 12083 49271 07384 52910
```

Because the keys are sorted before hashing, both Alice and Bob will compute the same safety number regardless of who initiates the computation.

### How to Verify

To verify a contact:

1. Open a DM conversation and click the **E2E** badge in the channel header.
2. The verification modal displays your shared safety number, your fingerprint, and their fingerprint.
3. Compare the safety number with your contact through an independent channel:
   - **In person**: Read the numbers aloud to each other
   - **Video call**: Show each other your screens
   - **Trusted messaging**: Send the numbers through a different, trusted communication channel
4. Use the **copy** button to easily share the safety number.

If the numbers match, you have confirmed that no one has intercepted or substituted identity keys.

You can also view your own fingerprint in **Settings > Security > Encryption**.

## Fingerprints

In addition to safety numbers, Chatalot implements **fingerprints** -- a hex-encoded SHA-256 hash of a single identity key. Fingerprints identify one user's key, while safety numbers verify the relationship between two users' keys.

A fingerprint is displayed in blocks:

```
a1b2 c3d4 e5f6 7890 1234 5678 9abc def0 1234 5678 9abc def0 1234 5678 9abc def0
```

Fingerprints are useful for publishing your identity out-of-band (e.g., on your website or social media profile), so others can verify your key when they first connect.

## QR Code Verification (Planned)

A planned feature is QR code verification:

1. Your client generates a QR code encoding your identity key fingerprint
2. Your contact scans it with their device's camera
3. The client automatically compares the scanned fingerprint with the stored identity key
4. A confirmation (or warning) is displayed

This is the same approach used by Signal and WhatsApp for contact verification.

## Key Change Detection

When a contact's identity key changes (because they re-registered, reinstalled, or switched devices), the client:

1. Detects the mismatch between the stored peer identity key and the new one in the X3DH header.
2. Displays a yellow warning banner: "Safety number has changed for this contact. They may have re-registered or switched devices."
3. Provides two options:
   - **Acknowledge** -- accept the key change and dismiss the warning.
   - **Verify** -- acknowledge and immediately open the verification modal to confirm the new safety number.

The warning persists until acknowledged. Identity keys are tracked in the `peerIdentities` IndexedDB store.

## Verification Roadmap

| Feature | Status |
|---------|--------|
| Safety number computation | Complete (Rust + WASM) |
| Fingerprint computation | Complete (Rust + WASM) |
| Peer identity storage (TOFU) | Complete (IndexedDB) |
| Safety number display UI | Complete |
| Key change warning UI | Complete |
| Per-message encryption indicators | Complete |
| QR code generation/scanning | Planned |
| Verified contact indicator | Planned |

## Limitations of TOFU

Trust on First Use has a known weakness: if the very first key exchange is intercepted (a man-in-the-middle attack), the attacker's key will be trusted from that point forward. This is why out-of-band verification (safety numbers, QR codes) is important -- it closes this gap.

In practice, for a self-hosted platform where the server is operated by your organization, the risk of a man-in-the-middle attack on the initial key exchange is lower than on a public service. But for maximum security, verification should still be performed.

## External Resources

- [Signal Protocol: Sesame Algorithm (key management)](https://signal.org/docs/specifications/sesame/)
- [How Signal safety numbers work](https://signal.org/blog/safety-number-updates/)
- [X3DH specification](https://signal.org/docs/specifications/x3dh/)

## Next Steps

- [Limitations](./limitations.md) -- Full picture of what the encryption does and does not protect
- [Key Management](./key-management.md) -- How identity keys are generated and stored
- [Overview](./overview.md) -- Return to the encryption overview
