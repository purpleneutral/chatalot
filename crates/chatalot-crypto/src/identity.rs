use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

use crate::types::Fingerprint;

/// Generate a new Ed25519 identity key pair.
pub fn generate_identity_key() -> SigningKey {
    SigningKey::generate(&mut OsRng)
}

/// Compute the fingerprint (hex-encoded SHA-256) of a public identity key.
pub fn fingerprint(public_key: &VerifyingKey) -> Fingerprint {
    let mut hasher = Sha256::new();
    hasher.update(public_key.as_bytes());
    let hash = hasher.finalize();
    Fingerprint(hex::encode(hash))
}

/// Compute a safety number for two identity keys (for verification).
/// Result is deterministic regardless of argument order.
pub fn safety_number(key_a: &VerifyingKey, key_b: &VerifyingKey) -> String {
    let (first, second) = if key_a.as_bytes() < key_b.as_bytes() {
        (key_a, key_b)
    } else {
        (key_b, key_a)
    };

    let mut hasher = Sha256::new();
    hasher.update(first.as_bytes());
    hasher.update(second.as_bytes());
    let hash = hasher.finalize();

    // Convert to numeric blocks (5-digit groups)
    hash.chunks(4)
        .map(|chunk| {
            let n = u32::from_be_bytes(chunk.try_into().unwrap_or([0; 4]));
            format!("{:05}", n % 100_000)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_deterministic() {
        let key = generate_identity_key();
        let fp1 = fingerprint(&key.verifying_key());
        let fp2 = fingerprint(&key.verifying_key());
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn test_safety_number_commutative() {
        let key_a = generate_identity_key();
        let key_b = generate_identity_key();
        let sn1 = safety_number(&key_a.verifying_key(), &key_b.verifying_key());
        let sn2 = safety_number(&key_b.verifying_key(), &key_a.verifying_key());
        assert_eq!(sn1, sn2);
    }
}
