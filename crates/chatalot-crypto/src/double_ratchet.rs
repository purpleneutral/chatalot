//! Double Ratchet protocol for ongoing message encryption.
//!
//! After X3DH establishes a shared secret, the Double Ratchet provides:
//! - Forward secrecy: compromised keys don't reveal past messages
//! - Break-in recovery: future messages are secure even after key compromise
//! - Out-of-order message decryption via skipped message keys
//!
//! Reference: <https://signal.org/docs/specifications/doubleratchet/>

use std::collections::HashMap;

use chacha20poly1305::{
    ChaCha20Poly1305, Nonce,
    aead::{Aead, KeyInit, Payload},
};
use hkdf::Hkdf;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use x25519_dalek::PublicKey as X25519Public;
use zeroize::Zeroize;

use crate::types::SecretKey;

/// Maximum number of skipped message keys to store.
/// Prevents a DoS where a malicious sender claims a huge message counter.
const MAX_SKIP: u32 = 1000;

/// HKDF info for root key ratchet.
const RATCHET_INFO: &[u8] = b"chatalot-ratchet";

/// HKDF info for message key derivation.
const MSG_KEY_INFO: &[u8] = b"chatalot-msg-key";

/// A message header sent alongside the ciphertext.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    /// Sender's current ratchet public key.
    pub ratchet_key: [u8; 32],
    /// Number of messages in the previous sending chain.
    pub previous_chain_length: u32,
    /// Message number in the current sending chain.
    pub message_number: u32,
}

/// An encrypted message (header + ciphertext).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    pub header: MessageHeader,
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
}

/// Errors during Double Ratchet operations.
#[derive(Debug, thiserror::Error)]
pub enum RatchetError {
    #[error("decryption failed — message may be tampered or key mismatch")]
    DecryptionFailed,
    #[error("too many skipped messages (>{MAX_SKIP})")]
    TooManySkipped,
    #[error("HKDF derivation failed")]
    HkdfError,
    #[error("duplicate message")]
    DuplicateMessage,
}

/// Key used to look up skipped message keys.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
struct SkippedKey {
    ratchet_key: [u8; 32],
    message_number: u32,
}

/// The Double Ratchet session state.
///
/// Each party maintains one of these for each conversation.
#[derive(Serialize, Deserialize)]
pub struct RatchetSession {
    /// Our current DH ratchet key pair (private bytes + public bytes).
    /// Stored as raw bytes for serialization.
    dh_sending_private: Option<[u8; 32]>,
    dh_sending_public: Option<[u8; 32]>,

    /// Their current DH ratchet public key.
    dh_receiving_key: Option<[u8; 32]>,

    /// Root key (32 bytes).
    root_key: [u8; 32],

    /// Sending chain key.
    sending_chain_key: Option<[u8; 32]>,
    /// Receiving chain key.
    receiving_chain_key: Option<[u8; 32]>,

    /// Number of messages sent in the current sending chain.
    send_count: u32,
    /// Number of messages received in the current receiving chain.
    recv_count: u32,
    /// Number of messages in the previous sending chain.
    previous_send_count: u32,

    /// Skipped message keys for out-of-order decryption.
    skipped_keys: HashMap<SkippedKey, [u8; 32]>,
}

impl Drop for RatchetSession {
    fn drop(&mut self) {
        self.root_key.zeroize();
        if let Some(ref mut k) = self.dh_sending_private {
            k.zeroize();
        }
        if let Some(ref mut k) = self.sending_chain_key {
            k.zeroize();
        }
        if let Some(ref mut k) = self.receiving_chain_key {
            k.zeroize();
        }
        for (_, key) in self.skipped_keys.iter_mut() {
            key.zeroize();
        }
    }
}

impl RatchetSession {
    /// Initialize a session as the initiator (Alice).
    ///
    /// After X3DH, Alice knows the shared secret and Bob's signed prekey (ratchet key).
    pub fn init_initiator(
        shared_secret: &SecretKey,
        their_ratchet_key: &X25519Public,
    ) -> Result<Self, RatchetError> {
        // Generate our first ratchet key pair
        let our_secret = x25519_dalek::StaticSecret::random_from_rng(OsRng);
        let our_public = X25519Public::from(&our_secret);

        // Perform the first DH ratchet step
        let dh_output = our_secret.diffie_hellman(their_ratchet_key);

        // Derive root key and initial sending chain key
        let (root_key, chain_key) = kdf_rk(shared_secret.as_bytes(), dh_output.as_bytes())?;

        Ok(Self {
            dh_sending_private: Some(our_secret.to_bytes()),
            dh_sending_public: Some(our_public.to_bytes()),
            dh_receiving_key: Some(their_ratchet_key.to_bytes()),
            root_key,
            sending_chain_key: Some(chain_key),
            receiving_chain_key: None,
            send_count: 0,
            recv_count: 0,
            previous_send_count: 0,
            skipped_keys: HashMap::new(),
        })
    }

    /// Initialize a session as the responder (Bob).
    ///
    /// Bob uses his signed prekey as the initial ratchet key.
    pub fn init_responder(
        shared_secret: &SecretKey,
        our_ratchet_private: &x25519_dalek::StaticSecret,
    ) -> Self {
        let our_public = X25519Public::from(our_ratchet_private);

        Self {
            dh_sending_private: Some(our_ratchet_private.to_bytes()),
            dh_sending_public: Some(our_public.to_bytes()),
            dh_receiving_key: None,
            root_key: *shared_secret.as_bytes(),
            sending_chain_key: None,
            receiving_chain_key: None,
            send_count: 0,
            recv_count: 0,
            previous_send_count: 0,
            skipped_keys: HashMap::new(),
        }
    }

    /// Encrypt a plaintext message.
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<EncryptedMessage, RatchetError> {
        // Derive message key from sending chain
        let (msg_key, new_chain_key) = kdf_ck(
            self.sending_chain_key
                .as_ref()
                .ok_or(RatchetError::HkdfError)?,
        )?;
        self.sending_chain_key = Some(new_chain_key);

        let header = MessageHeader {
            ratchet_key: self.dh_sending_public.unwrap_or([0u8; 32]),
            previous_chain_length: self.previous_send_count,
            message_number: self.send_count,
        };

        self.send_count += 1;

        // Encrypt with ChaCha20-Poly1305, using serialized header as AAD
        // per Signal spec: ENCRYPT(mk, plaintext, CONCAT(AD, header))
        let nonce = crate::aead::generate_nonce();
        let header_aad = serde_json::to_vec(&header).unwrap_or_default();
        let cipher = ChaCha20Poly1305::new((&msg_key).into());
        let ciphertext = cipher
            .encrypt(
                Nonce::from_slice(&nonce),
                Payload { msg: plaintext, aad: &header_aad },
            )
            .map_err(|_| RatchetError::DecryptionFailed)?;

        Ok(EncryptedMessage {
            header,
            ciphertext,
            nonce,
        })
    }

    /// Decrypt an incoming encrypted message.
    pub fn decrypt(&mut self, message: &EncryptedMessage) -> Result<Vec<u8>, RatchetError> {
        // First, check if we have a skipped key for this message
        let skip_key = SkippedKey {
            ratchet_key: message.header.ratchet_key,
            message_number: message.header.message_number,
        };

        if let Some(mut msg_key) = self.skipped_keys.remove(&skip_key) {
            let result = decrypt_with_key(&msg_key, &message.nonce, &message.ciphertext, &message.header);
            msg_key.zeroize();
            return result;
        }

        // Check if we need to perform a DH ratchet step
        let their_key_changed = self.dh_receiving_key.as_ref() != Some(&message.header.ratchet_key);

        if their_key_changed {
            // Skip any remaining messages in the current receiving chain
            if self.receiving_chain_key.is_some() {
                self.skip_messages(message.header.previous_chain_length)?;
            }

            // DH ratchet step
            self.dh_ratchet(&message.header.ratchet_key)?;
        }

        // Skip messages in the current receiving chain if needed
        self.skip_messages(message.header.message_number)?;

        // Derive the message key
        let (msg_key, new_chain_key) = kdf_ck(
            self.receiving_chain_key
                .as_ref()
                .ok_or(RatchetError::HkdfError)?,
        )?;
        self.receiving_chain_key = Some(new_chain_key);
        self.recv_count = message.header.message_number + 1;

        decrypt_with_key(&msg_key, &message.nonce, &message.ciphertext, &message.header)
    }

    /// Perform a DH ratchet step with a new remote public key.
    fn dh_ratchet(&mut self, their_new_key: &[u8; 32]) -> Result<(), RatchetError> {
        self.previous_send_count = self.send_count;
        self.send_count = 0;
        self.recv_count = 0;
        self.dh_receiving_key = Some(*their_new_key);

        let their_public = X25519Public::from(*their_new_key);

        // Derive receiving chain key
        if let Some(our_private_bytes) = &self.dh_sending_private {
            let our_secret = x25519_dalek::StaticSecret::from(*our_private_bytes);
            let dh_output = our_secret.diffie_hellman(&their_public);
            let (new_root, recv_chain) = kdf_rk(&self.root_key, dh_output.as_bytes())?;
            self.root_key = new_root;
            self.receiving_chain_key = Some(recv_chain);
        }

        // Generate new sending ratchet key pair
        let new_secret = x25519_dalek::StaticSecret::random_from_rng(OsRng);
        let new_public = X25519Public::from(&new_secret);
        let dh_output = new_secret.diffie_hellman(&their_public);
        let (new_root, send_chain) = kdf_rk(&self.root_key, dh_output.as_bytes())?;

        self.root_key = new_root;
        self.sending_chain_key = Some(send_chain);
        self.dh_sending_private = Some(new_secret.to_bytes());
        self.dh_sending_public = Some(new_public.to_bytes());

        Ok(())
    }

    /// Store skipped message keys up to the given message number.
    fn skip_messages(&mut self, until: u32) -> Result<(), RatchetError> {
        if self.recv_count + MAX_SKIP < until {
            return Err(RatchetError::TooManySkipped);
        }

        let chain_key = match &self.receiving_chain_key {
            Some(k) => k,
            None => return Ok(()),
        };

        let mut current_chain = *chain_key;
        let ratchet_key = self.dh_receiving_key.unwrap_or([0u8; 32]);

        while self.recv_count < until {
            let (msg_key, new_chain) = kdf_ck(&current_chain)?;
            current_chain = new_chain;

            let key = SkippedKey {
                ratchet_key,
                message_number: self.recv_count,
            };
            self.skipped_keys.insert(key, msg_key);
            self.recv_count += 1;
        }

        self.receiving_chain_key = Some(current_chain);
        Ok(())
    }

    /// Serialize the session state to bytes (for storage).
    pub fn serialize(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize session state from bytes.
    pub fn deserialize(data: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(data)
    }
}

/// KDF for the root key ratchet: derives new root key + chain key.
fn kdf_rk(root_key: &[u8; 32], dh_output: &[u8]) -> Result<([u8; 32], [u8; 32]), RatchetError> {
    let hk = Hkdf::<Sha256>::new(Some(root_key), dh_output);
    let mut output = [0u8; 64];
    hk.expand(RATCHET_INFO, &mut output)
        .map_err(|_| RatchetError::HkdfError)?;

    let mut new_root = [0u8; 32];
    let mut chain_key = [0u8; 32];
    new_root.copy_from_slice(&output[..32]);
    chain_key.copy_from_slice(&output[32..]);
    output.zeroize();

    Ok((new_root, chain_key))
}

/// KDF for the symmetric chain ratchet: derives message key + new chain key.
fn kdf_ck(chain_key: &[u8; 32]) -> Result<([u8; 32], [u8; 32]), RatchetError> {
    let hk = Hkdf::<Sha256>::new(Some(chain_key), &[0x01]);
    let mut msg_key = [0u8; 32];
    hk.expand(MSG_KEY_INFO, &mut msg_key)
        .map_err(|_| RatchetError::HkdfError)?;

    let hk2 = Hkdf::<Sha256>::new(Some(chain_key), &[0x02]);
    let mut new_chain = [0u8; 32];
    hk2.expand(MSG_KEY_INFO, &mut new_chain)
        .map_err(|_| RatchetError::HkdfError)?;

    Ok((msg_key, new_chain))
}

/// Decrypt ciphertext with a given message key, nonce, and header AAD.
///
/// Tries decryption with header-based AAD first (new format), then falls
/// back to no AAD for backwards compatibility with pre-AAD messages.
fn decrypt_with_key(
    msg_key: &[u8; 32],
    nonce: &[u8; 12],
    ciphertext: &[u8],
    header: &MessageHeader,
) -> Result<Vec<u8>, RatchetError> {
    let cipher = ChaCha20Poly1305::new(msg_key.into());
    let n = Nonce::from_slice(nonce);

    // Try with AAD (new format)
    let header_aad = serde_json::to_vec(header).unwrap_or_default();
    if let Ok(plaintext) = cipher.decrypt(n, Payload { msg: ciphertext, aad: &header_aad }) {
        return Ok(plaintext);
    }

    // Fall back to no AAD (pre-AAD messages)
    cipher
        .decrypt(n, ciphertext)
        .map_err(|_| RatchetError::DecryptionFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_sessions() -> (RatchetSession, RatchetSession) {
        // Simulate X3DH output
        let shared_secret = SecretKey(crate::aead::generate_key());

        // Bob's initial ratchet key (his signed prekey in X3DH)
        let bob_ratchet_secret = x25519_dalek::StaticSecret::random_from_rng(OsRng);
        let bob_ratchet_public = X25519Public::from(&bob_ratchet_secret);

        let alice = RatchetSession::init_initiator(&shared_secret, &bob_ratchet_public).unwrap();
        let bob = RatchetSession::init_responder(&shared_secret, &bob_ratchet_secret);

        (alice, bob)
    }

    #[test]
    fn test_basic_message_exchange() {
        let (mut alice, mut bob) = setup_sessions();

        // Alice sends to Bob
        let msg1 = alice.encrypt(b"hello bob").unwrap();
        let pt1 = bob.decrypt(&msg1).unwrap();
        assert_eq!(pt1, b"hello bob");

        // Bob replies to Alice
        let msg2 = bob.encrypt(b"hello alice").unwrap();
        let pt2 = alice.decrypt(&msg2).unwrap();
        assert_eq!(pt2, b"hello alice");
    }

    #[test]
    fn test_multiple_messages_same_direction() {
        let (mut alice, mut bob) = setup_sessions();

        // Alice sends multiple messages before Bob replies
        let m1 = alice.encrypt(b"message 1").unwrap();
        let m2 = alice.encrypt(b"message 2").unwrap();
        let m3 = alice.encrypt(b"message 3").unwrap();

        assert_eq!(bob.decrypt(&m1).unwrap(), b"message 1");
        assert_eq!(bob.decrypt(&m2).unwrap(), b"message 2");
        assert_eq!(bob.decrypt(&m3).unwrap(), b"message 3");
    }

    #[test]
    fn test_out_of_order_messages() {
        let (mut alice, mut bob) = setup_sessions();

        let m1 = alice.encrypt(b"first").unwrap();
        let m2 = alice.encrypt(b"second").unwrap();
        let m3 = alice.encrypt(b"third").unwrap();

        // Receive out of order
        assert_eq!(bob.decrypt(&m3).unwrap(), b"third");
        assert_eq!(bob.decrypt(&m1).unwrap(), b"first");
        assert_eq!(bob.decrypt(&m2).unwrap(), b"second");
    }

    #[test]
    fn test_ping_pong_conversation() {
        let (mut alice, mut bob) = setup_sessions();

        for i in 0..20 {
            let msg_text = format!("message {i}");
            if i % 2 == 0 {
                let enc = alice.encrypt(msg_text.as_bytes()).unwrap();
                let dec = bob.decrypt(&enc).unwrap();
                assert_eq!(dec, msg_text.as_bytes());
            } else {
                let enc = bob.encrypt(msg_text.as_bytes()).unwrap();
                let dec = alice.decrypt(&enc).unwrap();
                assert_eq!(dec, msg_text.as_bytes());
            }
        }
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let (mut alice, mut bob) = setup_sessions();

        let mut msg = alice.encrypt(b"secret").unwrap();
        msg.ciphertext[0] ^= 0xFF;

        assert!(bob.decrypt(&msg).is_err());
    }

    #[test]
    fn test_session_serialization() {
        let (mut alice, mut bob) = setup_sessions();

        let msg1 = alice.encrypt(b"before serialize").unwrap();
        assert_eq!(bob.decrypt(&msg1).unwrap(), b"before serialize");

        // Serialize and deserialize Alice's session
        let alice_bytes = alice.serialize().unwrap();
        let mut alice_restored = RatchetSession::deserialize(&alice_bytes).unwrap();

        // Continue the conversation with restored session
        let msg2 = bob.encrypt(b"after serialize").unwrap();
        let pt2 = alice_restored.decrypt(&msg2).unwrap();
        assert_eq!(pt2, b"after serialize");

        let msg3 = alice_restored.encrypt(b"reply after restore").unwrap();
        let pt3 = bob.decrypt(&msg3).unwrap();
        assert_eq!(pt3, b"reply after restore");
    }

    #[test]
    fn test_large_message() {
        let (mut alice, mut bob) = setup_sessions();

        let large = vec![0x42u8; 65536]; // 64 KiB
        let enc = alice.encrypt(&large).unwrap();
        let dec = bob.decrypt(&enc).unwrap();
        assert_eq!(dec, large);
    }
}
