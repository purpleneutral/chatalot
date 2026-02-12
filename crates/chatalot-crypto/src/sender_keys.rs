//! Sender Keys protocol for efficient group messaging.
//!
//! Instead of pairwise Double Ratchet for every group member (O(n^2)),
//! each member generates a Sender Key and distributes it to all other members
//! via their existing pairwise sessions. Messages are encrypted once with
//! the sender's key, and all recipients can decrypt.
//!
//! On member removal, all remaining members regenerate their Sender Keys.
//!
//! Reference: Signal's Sender Keys / libsignal-protocol

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use hkdf::Hkdf;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use zeroize::Zeroize;

use crate::aead;

const SENDER_KEY_INFO: &[u8] = b"chatalot-sender-key-chain";

/// A Sender Key Distribution Message — sent to each group member
/// via their pairwise Double Ratchet session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenderKeyDistribution {
    /// Unique ID for this sender key chain.
    pub chain_id: u32,
    /// Current iteration of the chain.
    pub iteration: u32,
    /// The chain key seed (32 bytes). Recipients use this to derive
    /// message keys for future messages from this sender.
    pub chain_key: Vec<u8>,
    /// The sender's signing public key for this chain (not used for
    /// encryption, but for authenticating the distribution message).
    pub sender_id: Vec<u8>,
}

/// The sender's state for their own Sender Key chain.
#[derive(Serialize, Deserialize)]
pub struct SenderKeyState {
    chain_id: u32,
    chain_key: [u8; 32],
    iteration: u32,
    sender_id: Vec<u8>,
}

impl Drop for SenderKeyState {
    fn drop(&mut self) {
        self.chain_key.zeroize();
    }
}

/// A recipient's state for a particular sender's key chain.
#[derive(Serialize, Deserialize)]
pub struct ReceiverKeyState {
    chain_id: u32,
    chain_key: [u8; 32],
    iteration: u32,
    sender_id: Vec<u8>,
    /// Cached message keys for out-of-order messages.
    /// Maps iteration -> message_key.
    cached_keys: std::collections::HashMap<u32, [u8; 32]>,
}

impl Drop for ReceiverKeyState {
    fn drop(&mut self) {
        self.chain_key.zeroize();
        for (_, key) in self.cached_keys.iter_mut() {
            key.zeroize();
        }
    }
}

/// Errors for Sender Key operations.
#[derive(Debug, thiserror::Error)]
pub enum SenderKeyError {
    #[error("encryption failed")]
    EncryptionFailed,
    #[error("decryption failed")]
    DecryptionFailed,
    #[error("chain key derivation failed")]
    DerivationFailed,
    #[error("too many skipped messages")]
    TooManySkipped,
    #[error("unknown sender key chain")]
    UnknownChain,
}

/// Maximum number of skipped message keys to cache per sender.
const MAX_SKIP: u32 = 2000;

impl SenderKeyState {
    /// Generate a new Sender Key for use in a group.
    pub fn generate(sender_id: &[u8]) -> (Self, SenderKeyDistribution) {
        let mut chain_key = [0u8; 32];
        OsRng.fill_bytes(&mut chain_key);
        let chain_id = rand::random::<u32>();

        let state = Self {
            chain_id,
            chain_key,
            iteration: 0,
            sender_id: sender_id.to_vec(),
        };

        let distribution = SenderKeyDistribution {
            chain_id,
            iteration: 0,
            chain_key: chain_key.to_vec(),
            sender_id: sender_id.to_vec(),
        };

        (state, distribution)
    }

    /// Encrypt a message using this sender key.
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<SenderKeyMessage, SenderKeyError> {
        // Derive message key from chain
        let (msg_key, new_chain_key) = advance_chain(&self.chain_key)?;
        self.chain_key = new_chain_key;
        let iteration = self.iteration;
        self.iteration += 1;

        // Encrypt
        let nonce = aead::generate_nonce();
        let cipher = ChaCha20Poly1305::new((&msg_key).into());
        let ciphertext = cipher
            .encrypt(Nonce::from_slice(&nonce), plaintext)
            .map_err(|_| SenderKeyError::EncryptionFailed)?;

        Ok(SenderKeyMessage {
            chain_id: self.chain_id,
            iteration,
            ciphertext,
            nonce,
        })
    }

    /// Serialize for storage.
    pub fn serialize(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize from storage.
    pub fn deserialize(data: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(data)
    }
}

impl ReceiverKeyState {
    /// Initialize from a received Sender Key Distribution message.
    pub fn from_distribution(dist: &SenderKeyDistribution) -> Self {
        let mut chain_key = [0u8; 32];
        chain_key.copy_from_slice(&dist.chain_key);

        Self {
            chain_id: dist.chain_id,
            chain_key,
            iteration: dist.iteration,
            sender_id: dist.sender_id.clone(),
            cached_keys: std::collections::HashMap::new(),
        }
    }

    /// Decrypt a message from this sender.
    pub fn decrypt(&mut self, message: &SenderKeyMessage) -> Result<Vec<u8>, SenderKeyError> {
        if message.chain_id != self.chain_id {
            return Err(SenderKeyError::UnknownChain);
        }

        // Check if we have a cached key for this iteration
        if let Some(mut msg_key) = self.cached_keys.remove(&message.iteration) {
            let result = decrypt_with_key(&msg_key, &message.nonce, &message.ciphertext);
            msg_key.zeroize();
            return result;
        }

        // If the message is from the future, advance the chain and cache keys
        if message.iteration > self.iteration {
            let skip_count = message.iteration - self.iteration;
            if skip_count > MAX_SKIP {
                return Err(SenderKeyError::TooManySkipped);
            }

            // Advance chain, caching intermediate keys
            for i in self.iteration..message.iteration {
                let (msg_key, new_chain_key) = advance_chain(&self.chain_key)?;
                self.chain_key = new_chain_key;
                self.cached_keys.insert(i, msg_key);
            }
            self.iteration = message.iteration;
        }

        if message.iteration < self.iteration {
            // Message from the past — check cached keys
            if let Some(mut msg_key) = self.cached_keys.remove(&message.iteration) {
                let result = decrypt_with_key(&msg_key, &message.nonce, &message.ciphertext);
                msg_key.zeroize();
                return result;
            }
            return Err(SenderKeyError::DecryptionFailed);
        }

        // Current iteration
        let (msg_key, new_chain_key) = advance_chain(&self.chain_key)?;
        self.chain_key = new_chain_key;
        self.iteration += 1;

        decrypt_with_key(&msg_key, &message.nonce, &message.ciphertext)
    }

    /// Serialize for storage.
    pub fn serialize(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize from storage.
    pub fn deserialize(data: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(data)
    }
}

/// An encrypted Sender Key message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenderKeyMessage {
    pub chain_id: u32,
    pub iteration: u32,
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
}

/// Advance the chain: KDF(chain_key) -> (message_key, new_chain_key).
fn advance_chain(chain_key: &[u8; 32]) -> Result<([u8; 32], [u8; 32]), SenderKeyError> {
    let hk = Hkdf::<Sha256>::new(Some(chain_key), &[0x01]);
    let mut msg_key = [0u8; 32];
    hk.expand(SENDER_KEY_INFO, &mut msg_key)
        .map_err(|_| SenderKeyError::DerivationFailed)?;

    let hk2 = Hkdf::<Sha256>::new(Some(chain_key), &[0x02]);
    let mut new_chain = [0u8; 32];
    hk2.expand(SENDER_KEY_INFO, &mut new_chain)
        .map_err(|_| SenderKeyError::DerivationFailed)?;

    Ok((msg_key, new_chain))
}

/// Decrypt with a specific message key.
fn decrypt_with_key(
    msg_key: &[u8; 32],
    nonce: &[u8; 12],
    ciphertext: &[u8],
) -> Result<Vec<u8>, SenderKeyError> {
    let cipher = ChaCha20Poly1305::new(msg_key.into());
    cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|_| SenderKeyError::DecryptionFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sender_key_basic() {
        let sender_id = b"alice";
        let (mut sender, dist) = SenderKeyState::generate(sender_id);
        let mut receiver = ReceiverKeyState::from_distribution(&dist);

        let msg = sender.encrypt(b"hello group").unwrap();
        let plaintext = receiver.decrypt(&msg).unwrap();
        assert_eq!(plaintext, b"hello group");
    }

    #[test]
    fn test_sender_key_multiple_messages() {
        let (mut sender, dist) = SenderKeyState::generate(b"alice");
        let mut receiver = ReceiverKeyState::from_distribution(&dist);

        for i in 0..50 {
            let text = format!("message {i}");
            let msg = sender.encrypt(text.as_bytes()).unwrap();
            let plaintext = receiver.decrypt(&msg).unwrap();
            assert_eq!(plaintext, text.as_bytes());
        }
    }

    #[test]
    fn test_sender_key_out_of_order() {
        let (mut sender, dist) = SenderKeyState::generate(b"alice");
        let mut receiver = ReceiverKeyState::from_distribution(&dist);

        let m1 = sender.encrypt(b"first").unwrap();
        let m2 = sender.encrypt(b"second").unwrap();
        let m3 = sender.encrypt(b"third").unwrap();

        // Receive out of order
        assert_eq!(receiver.decrypt(&m3).unwrap(), b"third");
        assert_eq!(receiver.decrypt(&m1).unwrap(), b"first");
        assert_eq!(receiver.decrypt(&m2).unwrap(), b"second");
    }

    #[test]
    fn test_sender_key_multiple_recipients() {
        let (mut sender, dist) = SenderKeyState::generate(b"alice");
        let mut bob = ReceiverKeyState::from_distribution(&dist);
        let mut carol = ReceiverKeyState::from_distribution(&dist);

        let msg = sender.encrypt(b"hello everyone").unwrap();
        assert_eq!(bob.decrypt(&msg).unwrap(), b"hello everyone");
        assert_eq!(carol.decrypt(&msg).unwrap(), b"hello everyone");
    }

    #[test]
    fn test_sender_key_tampered_fails() {
        let (mut sender, dist) = SenderKeyState::generate(b"alice");
        let mut receiver = ReceiverKeyState::from_distribution(&dist);

        let mut msg = sender.encrypt(b"secret").unwrap();
        msg.ciphertext[0] ^= 0xFF;
        assert!(receiver.decrypt(&msg).is_err());
    }

    #[test]
    fn test_sender_key_serialization() {
        let (mut sender, dist) = SenderKeyState::generate(b"alice");
        let mut receiver = ReceiverKeyState::from_distribution(&dist);

        let m1 = sender.encrypt(b"before").unwrap();
        assert_eq!(receiver.decrypt(&m1).unwrap(), b"before");

        // Serialize and restore
        let sender_bytes = sender.serialize().unwrap();
        let receiver_bytes = receiver.serialize().unwrap();
        let mut sender2 = SenderKeyState::deserialize(&sender_bytes).unwrap();
        let mut receiver2 = ReceiverKeyState::deserialize(&receiver_bytes).unwrap();

        let m2 = sender2.encrypt(b"after").unwrap();
        assert_eq!(receiver2.decrypt(&m2).unwrap(), b"after");
    }

    #[test]
    fn test_wrong_chain_id_rejected() {
        let (mut sender_a, _) = SenderKeyState::generate(b"alice");
        let (_, dist_b) = SenderKeyState::generate(b"bob");
        let mut receiver = ReceiverKeyState::from_distribution(&dist_b);

        let msg = sender_a.encrypt(b"wrong chain").unwrap();
        assert!(matches!(
            receiver.decrypt(&msg),
            Err(SenderKeyError::UnknownChain)
        ));
    }
}
