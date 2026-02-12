use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// A 32-byte key that zeroizes on drop.
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct SecretKey(pub [u8; 32]);

impl SecretKey {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Ed25519 identity key fingerprint (hex-encoded SHA-256).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fingerprint(pub String);

impl std::fmt::Display for Fingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Format as "AB12 CD34 EF56 ..." blocks
        let chunks: Vec<&str> = self.0.as_bytes().chunks(4).map(|c| {
            std::str::from_utf8(c).unwrap_or("????")
        }).collect();
        write!(f, "{}", chunks.join(" "))
    }
}
