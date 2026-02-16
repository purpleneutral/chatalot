//! X3DH (Extended Triple Diffie-Hellman) key agreement protocol.
//!
//! Implements the Signal protocol's X3DH for establishing shared secrets
//! between two parties who may not be online simultaneously.
//!
//! Reference: <https://signal.org/docs/specifications/x3dh/>

use ed25519_dalek::{Signature, SigningKey, Verifier, VerifyingKey};
use hkdf::Hkdf;
use rand::rngs::OsRng;
use sha2::Sha256;
use x25519_dalek::{PublicKey as X25519Public, StaticSecret};
use zeroize::Zeroize;

use crate::types::SecretKey;

/// Info string for HKDF used in X3DH.
const X3DH_INFO: &[u8] = b"chatalot-x3dh-shared-secret";

/// A 32-byte filler prepended to KDF input per the X3DH spec.
const KDF_FILLER: [u8; 32] = [0xFF; 32];

/// A user's prekey bundle as fetched from the server.
#[derive(Debug, Clone)]
pub struct PrekeyBundle {
    /// The recipient's Ed25519 identity public key.
    pub identity_key: VerifyingKey,
    /// The recipient's signed prekey (X25519 public).
    pub signed_prekey: X25519Public,
    /// Ed25519 signature over the signed prekey, made by the identity key.
    pub signed_prekey_signature: Signature,
    /// An optional one-time prekey (X25519 public).
    pub one_time_prekey: Option<X25519Public>,
}

/// The initiator's output after running X3DH.
pub struct X3dhInitiatorResult {
    /// The derived shared secret (32 bytes).
    pub shared_secret: SecretKey,
    /// The ephemeral public key to send to the recipient.
    pub ephemeral_public_key: X25519Public,
    /// The associated data (AD) for the first message.
    pub associated_data: Vec<u8>,
}

/// The responder's output after processing an X3DH initial message.
pub struct X3dhResponderResult {
    /// The derived shared secret (32 bytes).
    pub shared_secret: SecretKey,
    /// The associated data (AD) for decrypting the first message.
    pub associated_data: Vec<u8>,
}

/// Error types for X3DH operations.
#[derive(Debug, thiserror::Error)]
pub enum X3dhError {
    #[error("signed prekey signature verification failed")]
    InvalidSignature,
    #[error("HKDF expand failed")]
    HkdfError,
}

/// Convert an Ed25519 public key to an X25519 public key.
///
/// This uses the birational map from the Ed25519 curve to Curve25519.
fn ed25519_to_x25519_public(ed_key: &VerifyingKey) -> X25519Public {
    let ed_point = ed_key.to_montgomery();
    X25519Public::from(ed_point.to_bytes())
}

/// Convert an Ed25519 signing key to an X25519 static secret.
fn ed25519_to_x25519_secret(ed_key: &SigningKey) -> StaticSecret {
    use sha2::Digest;
    // The X25519 secret is the first 32 bytes of SHA-512(ed25519_secret_scalar)
    let hash = sha2::Sha512::digest(ed_key.as_bytes());
    let mut secret_bytes = [0u8; 32];
    secret_bytes.copy_from_slice(&hash[..32]);
    // Apply clamping (StaticSecret::from does this internally)
    let secret = StaticSecret::from(secret_bytes);
    secret_bytes.zeroize();
    secret
}

/// Compute the associated data (AD) for X3DH.
///
/// AD = Encode(IK_A) || Encode(IK_B)
fn compute_associated_data(
    initiator_identity: &VerifyingKey,
    responder_identity: &VerifyingKey,
) -> Vec<u8> {
    let mut ad = Vec::with_capacity(64);
    ad.extend_from_slice(initiator_identity.as_bytes());
    ad.extend_from_slice(responder_identity.as_bytes());
    ad
}

/// Initiator side of X3DH: Alice wants to establish a session with Bob.
///
/// Alice has her own identity key and fetches Bob's prekey bundle from the server.
pub fn initiate(
    our_identity_key: &SigningKey,
    their_bundle: &PrekeyBundle,
) -> Result<X3dhInitiatorResult, X3dhError> {
    // Step 1: Verify the signed prekey signature
    their_bundle
        .identity_key
        .verify(
            their_bundle.signed_prekey.as_bytes(),
            &their_bundle.signed_prekey_signature,
        )
        .map_err(|_| X3dhError::InvalidSignature)?;

    // Step 2: Generate ephemeral X25519 key pair
    // Using StaticSecret because we need multiple DH operations with the same key.
    // The key is discarded after this function returns.
    let ephemeral_secret = StaticSecret::random_from_rng(OsRng);
    let ephemeral_public = X25519Public::from(&ephemeral_secret);

    // Step 3: Convert our Ed25519 identity key to X25519
    let our_x25519_secret = ed25519_to_x25519_secret(our_identity_key);
    let their_x25519_identity = ed25519_to_x25519_public(&their_bundle.identity_key);

    // Step 4: Compute DH values
    //   DH1 = DH(IK_A, SPK_B)
    //   DH2 = DH(EK_A, IK_B)
    //   DH3 = DH(EK_A, SPK_B)
    //   DH4 = DH(EK_A, OPK_B)  [if one-time prekey available]
    let dh1 = our_x25519_secret.diffie_hellman(&their_bundle.signed_prekey);
    let dh2 = ephemeral_secret.diffie_hellman(&their_x25519_identity);
    let dh3 = ephemeral_secret.diffie_hellman(&their_bundle.signed_prekey);

    // Step 5: Concatenate DH outputs with filler prefix
    let mut kdf_input = Vec::with_capacity(32 + 32 * 4);
    kdf_input.extend_from_slice(&KDF_FILLER);
    kdf_input.extend_from_slice(dh1.as_bytes());
    kdf_input.extend_from_slice(dh2.as_bytes());
    kdf_input.extend_from_slice(dh3.as_bytes());

    if let Some(opk) = &their_bundle.one_time_prekey {
        let dh4 = ephemeral_secret.diffie_hellman(opk);
        kdf_input.extend_from_slice(dh4.as_bytes());
    }

    // Step 6: Derive shared secret using HKDF
    let shared_secret = kdf_derive(&kdf_input)?;
    kdf_input.zeroize();

    // Step 7: Compute associated data
    let associated_data = compute_associated_data(
        &our_identity_key.verifying_key(),
        &their_bundle.identity_key,
    );

    Ok(X3dhInitiatorResult {
        shared_secret,
        ephemeral_public_key: ephemeral_public,
        associated_data,
    })
}

/// Responder side of X3DH: Bob processes Alice's initial message.
///
/// Bob uses his own identity key, signed prekey, and optionally a one-time prekey
/// to derive the same shared secret Alice computed.
pub fn respond(
    our_identity_key: &SigningKey,
    our_signed_prekey_secret: &StaticSecret,
    our_one_time_prekey_secret: Option<&StaticSecret>,
    their_identity_key: &VerifyingKey,
    their_ephemeral_key: &X25519Public,
) -> Result<X3dhResponderResult, X3dhError> {
    // Convert keys
    let our_x25519_identity = ed25519_to_x25519_secret(our_identity_key);
    let their_x25519_identity = ed25519_to_x25519_public(their_identity_key);

    // Compute DH values (mirroring the initiator's computation)
    //   DH1 = DH(SPK_B, IK_A)  -- same shared secret as DH(IK_A, SPK_B)
    //   DH2 = DH(IK_B, EK_A)   -- same as DH(EK_A, IK_B)
    //   DH3 = DH(SPK_B, EK_A)  -- same as DH(EK_A, SPK_B)
    //   DH4 = DH(OPK_B, EK_A)  -- same as DH(EK_A, OPK_B)
    let dh1 = our_signed_prekey_secret.diffie_hellman(&their_x25519_identity);
    let dh2 = our_x25519_identity.diffie_hellman(their_ephemeral_key);
    let dh3 = our_signed_prekey_secret.diffie_hellman(their_ephemeral_key);

    let mut kdf_input = Vec::with_capacity(32 + 32 * 4);
    kdf_input.extend_from_slice(&KDF_FILLER);
    kdf_input.extend_from_slice(dh1.as_bytes());
    kdf_input.extend_from_slice(dh2.as_bytes());
    kdf_input.extend_from_slice(dh3.as_bytes());

    if let Some(opk_secret) = our_one_time_prekey_secret {
        let dh4 = opk_secret.diffie_hellman(their_ephemeral_key);
        kdf_input.extend_from_slice(dh4.as_bytes());
    }

    let shared_secret = kdf_derive(&kdf_input)?;
    kdf_input.zeroize();

    let associated_data =
        compute_associated_data(their_identity_key, &our_identity_key.verifying_key());

    Ok(X3dhResponderResult {
        shared_secret,
        associated_data,
    })
}

/// HKDF-SHA256 key derivation.
fn kdf_derive(input: &[u8]) -> Result<SecretKey, X3dhError> {
    let hk = Hkdf::<Sha256>::new(Some(&[0u8; 32]), input);
    let mut output = [0u8; 32];
    hk.expand(X3DH_INFO, &mut output)
        .map_err(|_| X3dhError::HkdfError)?;
    Ok(SecretKey(output))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Generate a signed prekey bundle for testing.
    fn make_test_bundle(
        identity_key: &SigningKey,
    ) -> (PrekeyBundle, StaticSecret, Option<StaticSecret>) {
        let signed_prekey_secret = StaticSecret::random_from_rng(OsRng);
        let signed_prekey_public = X25519Public::from(&signed_prekey_secret);

        // Sign the prekey with the identity key
        use ed25519_dalek::Signer;
        let signature = identity_key.sign(signed_prekey_public.as_bytes());

        let otpk_secret = StaticSecret::random_from_rng(OsRng);
        let otpk_public = X25519Public::from(&otpk_secret);

        let bundle = PrekeyBundle {
            identity_key: identity_key.verifying_key(),
            signed_prekey: signed_prekey_public,
            signed_prekey_signature: signature,
            one_time_prekey: Some(otpk_public),
        };

        (bundle, signed_prekey_secret, Some(otpk_secret))
    }

    #[test]
    fn test_x3dh_initiator_responder_agree() {
        let alice_identity = SigningKey::generate(&mut OsRng);
        let bob_identity = SigningKey::generate(&mut OsRng);

        let (bob_bundle, bob_spk_secret, bob_opk_secret) = make_test_bundle(&bob_identity);

        // Alice initiates
        let alice_result = initiate(&alice_identity, &bob_bundle).unwrap();

        // Bob responds
        let bob_result = respond(
            &bob_identity,
            &bob_spk_secret,
            bob_opk_secret.as_ref(),
            &alice_identity.verifying_key(),
            &alice_result.ephemeral_public_key,
        )
        .unwrap();

        // Both should derive the same shared secret
        assert_eq!(
            alice_result.shared_secret.as_bytes(),
            bob_result.shared_secret.as_bytes()
        );
    }

    #[test]
    fn test_x3dh_without_one_time_prekey() {
        let alice_identity = SigningKey::generate(&mut OsRng);
        let bob_identity = SigningKey::generate(&mut OsRng);

        let (mut bob_bundle, bob_spk_secret, _) = make_test_bundle(&bob_identity);
        bob_bundle.one_time_prekey = None; // No OTP available

        let alice_result = initiate(&alice_identity, &bob_bundle).unwrap();

        let bob_result = respond(
            &bob_identity,
            &bob_spk_secret,
            None, // No OTP
            &alice_identity.verifying_key(),
            &alice_result.ephemeral_public_key,
        )
        .unwrap();

        assert_eq!(
            alice_result.shared_secret.as_bytes(),
            bob_result.shared_secret.as_bytes()
        );
    }

    #[test]
    fn test_x3dh_invalid_signature_rejected() {
        let alice_identity = SigningKey::generate(&mut OsRng);
        let bob_identity = SigningKey::generate(&mut OsRng);

        let (mut bob_bundle, _, _) = make_test_bundle(&bob_identity);

        // Tamper with the signature
        let bad_sig_bytes = [0u8; 64];
        bob_bundle.signed_prekey_signature = Signature::from_bytes(&bad_sig_bytes);

        let result = initiate(&alice_identity, &bob_bundle);
        assert!(result.is_err());
    }

    #[test]
    fn test_x3dh_associated_data_matches() {
        let alice_identity = SigningKey::generate(&mut OsRng);
        let bob_identity = SigningKey::generate(&mut OsRng);

        let (bob_bundle, bob_spk_secret, bob_opk_secret) = make_test_bundle(&bob_identity);

        let alice_result = initiate(&alice_identity, &bob_bundle).unwrap();
        let bob_result = respond(
            &bob_identity,
            &bob_spk_secret,
            bob_opk_secret.as_ref(),
            &alice_identity.verifying_key(),
            &alice_result.ephemeral_public_key,
        )
        .unwrap();

        assert_eq!(alice_result.associated_data, bob_result.associated_data);
    }
}
