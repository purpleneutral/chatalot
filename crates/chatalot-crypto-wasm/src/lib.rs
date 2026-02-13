use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use x25519_dalek::{PublicKey as X25519Public, StaticSecret};

use chatalot_crypto::double_ratchet::{EncryptedMessage, RatchetSession};
use chatalot_crypto::identity;
use chatalot_crypto::x3dh::{self, PrekeyBundle};

// ─── Identity key generation ───────────────────────────────────────

#[derive(Serialize)]
struct IdentityKeyResult {
    signing_key: Vec<u8>,
    verifying_key: Vec<u8>,
}

/// Generate a new Ed25519 identity keypair.
#[wasm_bindgen]
pub fn generate_identity_key() -> Result<JsValue, JsValue> {
    let key = identity::generate_identity_key();
    let result = IdentityKeyResult {
        signing_key: key.to_bytes().to_vec(),
        verifying_key: key.verifying_key().to_bytes().to_vec(),
    };
    serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Compute the SHA-256 fingerprint of a 32-byte Ed25519 public key.
#[wasm_bindgen]
pub fn compute_fingerprint(public_key: &[u8]) -> Result<String, JsValue> {
    let bytes: [u8; 32] = public_key
        .try_into()
        .map_err(|_| JsValue::from_str("public key must be 32 bytes"))?;
    let vk = VerifyingKey::from_bytes(&bytes)
        .map_err(|e| JsValue::from_str(&format!("invalid public key: {e}")))?;
    let fp = identity::fingerprint(&vk);
    Ok(fp.0)
}

/// Compute the safety number for two identity keys.
#[wasm_bindgen]
pub fn compute_safety_number(key_a: &[u8], key_b: &[u8]) -> Result<String, JsValue> {
    let a: [u8; 32] = key_a
        .try_into()
        .map_err(|_| JsValue::from_str("key_a must be 32 bytes"))?;
    let b: [u8; 32] = key_b
        .try_into()
        .map_err(|_| JsValue::from_str("key_b must be 32 bytes"))?;
    let vk_a =
        VerifyingKey::from_bytes(&a).map_err(|e| JsValue::from_str(&format!("invalid key_a: {e}")))?;
    let vk_b =
        VerifyingKey::from_bytes(&b).map_err(|e| JsValue::from_str(&format!("invalid key_b: {e}")))?;
    Ok(identity::safety_number(&vk_a, &vk_b))
}

// ─── Prekey generation ─────────────────────────────────────────────

#[derive(Serialize)]
struct SignedPrekeyResult {
    key_id: i32,
    public_key: Vec<u8>,
    private_key: Vec<u8>,
    signature: Vec<u8>,
}

/// Generate a signed prekey pair. Signs the public key with the identity signing key.
#[wasm_bindgen]
pub fn generate_signed_prekey(
    identity_signing_key: &[u8],
    key_id: i32,
) -> Result<JsValue, JsValue> {
    let sk_bytes: [u8; 32] = identity_signing_key
        .try_into()
        .map_err(|_| JsValue::from_str("signing key must be 32 bytes"))?;
    let signing_key = SigningKey::from_bytes(&sk_bytes);

    let secret = StaticSecret::random_from_rng(OsRng);
    let public = X25519Public::from(&secret);
    let signature = signing_key.sign(public.as_bytes());

    let result = SignedPrekeyResult {
        key_id,
        public_key: public.as_bytes().to_vec(),
        private_key: secret.to_bytes().to_vec(),
        signature: signature.to_bytes().to_vec(),
    };
    serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[derive(Serialize)]
struct OneTimePrekeyResult {
    key_id: i32,
    public_key: Vec<u8>,
    private_key: Vec<u8>,
}

/// Generate a batch of one-time prekeys.
#[wasm_bindgen]
pub fn generate_one_time_prekeys(start_key_id: i32, count: u32) -> Result<JsValue, JsValue> {
    let mut results = Vec::with_capacity(count as usize);
    for i in 0..count {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = X25519Public::from(&secret);
        results.push(OneTimePrekeyResult {
            key_id: start_key_id + i as i32,
            public_key: public.as_bytes().to_vec(),
            private_key: secret.to_bytes().to_vec(),
        });
    }
    serde_wasm_bindgen::to_value(&results).map_err(|e| JsValue::from_str(&e.to_string()))
}

// ─── X3DH ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct KeyBundleInput {
    identity_key: Vec<u8>,
    signed_prekey: SignedPrekeyInput,
    one_time_prekey: Option<OneTimePrekeyInput>,
}

#[derive(Deserialize)]
struct SignedPrekeyInput {
    #[allow(dead_code)]
    key_id: i32,
    public_key: Vec<u8>,
    signature: Vec<u8>,
}

#[derive(Deserialize)]
struct OneTimePrekeyInput {
    #[allow(dead_code)]
    key_id: i32,
    public_key: Vec<u8>,
}

#[derive(Serialize)]
struct X3dhInitiateResult {
    session_json: String,
    ephemeral_public_key: Vec<u8>,
    associated_data: Vec<u8>,
}

/// Initiator side of X3DH. Establishes a session with a remote user.
///
/// Takes our identity signing key (32 bytes) and the remote user's key bundle (JSON).
/// Returns the initialized Double Ratchet session (JSON), ephemeral public key, and AD.
#[wasm_bindgen]
pub fn x3dh_initiate(
    our_identity_signing_key: &[u8],
    their_bundle_json: &str,
) -> Result<JsValue, JsValue> {
    let sk_bytes: [u8; 32] = our_identity_signing_key
        .try_into()
        .map_err(|_| JsValue::from_str("signing key must be 32 bytes"))?;
    let our_signing_key = SigningKey::from_bytes(&sk_bytes);

    let bundle_input: KeyBundleInput = serde_json::from_str(their_bundle_json)
        .map_err(|e| JsValue::from_str(&format!("invalid bundle JSON: {e}")))?;

    // Reconstruct the PrekeyBundle with proper cryptographic types
    let identity_bytes: [u8; 32] = bundle_input
        .identity_key
        .as_slice()
        .try_into()
        .map_err(|_| JsValue::from_str("identity key must be 32 bytes"))?;
    let identity_key = VerifyingKey::from_bytes(&identity_bytes)
        .map_err(|e| JsValue::from_str(&format!("invalid identity key: {e}")))?;

    let spk_bytes: [u8; 32] = bundle_input
        .signed_prekey
        .public_key
        .as_slice()
        .try_into()
        .map_err(|_| JsValue::from_str("signed prekey must be 32 bytes"))?;
    let signed_prekey = X25519Public::from(spk_bytes);

    let sig_bytes: [u8; 64] = bundle_input
        .signed_prekey
        .signature
        .as_slice()
        .try_into()
        .map_err(|_| JsValue::from_str("signature must be 64 bytes"))?;
    let signature = Signature::from_bytes(&sig_bytes);

    let one_time_prekey = if let Some(otp) = &bundle_input.one_time_prekey {
        let otp_bytes: [u8; 32] = otp
            .public_key
            .as_slice()
            .try_into()
            .map_err(|_| JsValue::from_str("one-time prekey must be 32 bytes"))?;
        Some(X25519Public::from(otp_bytes))
    } else {
        None
    };

    let bundle = PrekeyBundle {
        identity_key,
        signed_prekey,
        signed_prekey_signature: signature,
        one_time_prekey,
    };

    // Run X3DH
    let x3dh_result = x3dh::initiate(&our_signing_key, &bundle)
        .map_err(|e| JsValue::from_str(&format!("X3DH failed: {e}")))?;

    // Initialize Double Ratchet as initiator using the signed prekey as initial ratchet key
    let session = RatchetSession::init_initiator(&x3dh_result.shared_secret, &bundle.signed_prekey)
        .map_err(|e| JsValue::from_str(&format!("ratchet init failed: {e}")))?;

    let session_json = String::from_utf8(session.serialize().map_err(|e| {
        JsValue::from_str(&format!("session serialize failed: {e}"))
    })?)
    .map_err(|e| JsValue::from_str(&format!("session JSON not UTF-8: {e}")))?;

    let result = X3dhInitiateResult {
        session_json,
        ephemeral_public_key: x3dh_result.ephemeral_public_key.as_bytes().to_vec(),
        associated_data: x3dh_result.associated_data,
    };
    serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[derive(Serialize)]
struct X3dhRespondResult {
    session_json: String,
    associated_data: Vec<u8>,
}

/// Responder side of X3DH. Process an initial message from a remote user.
///
/// Takes our identity signing key, our signed prekey private, optionally our OTP private,
/// their identity key, and their ephemeral key.
#[wasm_bindgen]
pub fn x3dh_respond(
    our_identity_signing_key: &[u8],
    our_signed_prekey_private: &[u8],
    our_otp_private: Option<Vec<u8>>,
    their_identity_key: &[u8],
    their_ephemeral_key: &[u8],
) -> Result<JsValue, JsValue> {
    let sk_bytes: [u8; 32] = our_identity_signing_key
        .try_into()
        .map_err(|_| JsValue::from_str("signing key must be 32 bytes"))?;
    let our_signing_key = SigningKey::from_bytes(&sk_bytes);

    let spk_bytes: [u8; 32] = our_signed_prekey_private
        .try_into()
        .map_err(|_| JsValue::from_str("signed prekey private must be 32 bytes"))?;
    let our_spk_secret = StaticSecret::from(spk_bytes);

    let otp_secret = if let Some(ref otp) = our_otp_private {
        let otp_bytes: [u8; 32] = otp
            .as_slice()
            .try_into()
            .map_err(|_| JsValue::from_str("OTP private must be 32 bytes"))?;
        Some(StaticSecret::from(otp_bytes))
    } else {
        None
    };

    let their_ik_bytes: [u8; 32] = their_identity_key
        .try_into()
        .map_err(|_| JsValue::from_str("their identity key must be 32 bytes"))?;
    let their_ik = VerifyingKey::from_bytes(&their_ik_bytes)
        .map_err(|e| JsValue::from_str(&format!("invalid their identity key: {e}")))?;

    let their_ek_bytes: [u8; 32] = their_ephemeral_key
        .try_into()
        .map_err(|_| JsValue::from_str("their ephemeral key must be 32 bytes"))?;
    let their_ek = X25519Public::from(their_ek_bytes);

    // Run X3DH responder
    let x3dh_result = x3dh::respond(
        &our_signing_key,
        &our_spk_secret,
        otp_secret.as_ref(),
        &their_ik,
        &their_ek,
    )
    .map_err(|e| JsValue::from_str(&format!("X3DH respond failed: {e}")))?;

    // Initialize Double Ratchet as responder using our signed prekey as the ratchet key
    let session = RatchetSession::init_responder(&x3dh_result.shared_secret, &our_spk_secret);

    let session_json = String::from_utf8(session.serialize().map_err(|e| {
        JsValue::from_str(&format!("session serialize failed: {e}"))
    })?)
    .map_err(|e| JsValue::from_str(&format!("session JSON not UTF-8: {e}")))?;

    let result = X3dhRespondResult {
        session_json,
        associated_data: x3dh_result.associated_data,
    };
    serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

// ─── Double Ratchet encrypt/decrypt ────────────────────────────────

#[derive(Serialize)]
struct RatchetEncryptResult {
    session_json: String,
    encrypted: EncryptedMessage,
}

/// Encrypt a plaintext message using a Double Ratchet session.
///
/// Takes the serialized session JSON and plaintext bytes.
/// Returns the updated session JSON and the encrypted message.
#[wasm_bindgen]
pub fn ratchet_encrypt(session_json: &str, plaintext: &[u8]) -> Result<JsValue, JsValue> {
    let mut session = RatchetSession::deserialize(session_json.as_bytes())
        .map_err(|e| JsValue::from_str(&format!("session deserialize failed: {e}")))?;

    let encrypted = session
        .encrypt(plaintext)
        .map_err(|e| JsValue::from_str(&format!("encrypt failed: {e}")))?;

    let new_session_json = String::from_utf8(session.serialize().map_err(|e| {
        JsValue::from_str(&format!("session serialize failed: {e}"))
    })?)
    .map_err(|e| JsValue::from_str(&format!("session JSON not UTF-8: {e}")))?;

    let result = RatchetEncryptResult {
        session_json: new_session_json,
        encrypted,
    };
    serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[derive(Serialize)]
struct RatchetDecryptResult {
    session_json: String,
    plaintext: Vec<u8>,
}

/// Decrypt an encrypted message using a Double Ratchet session.
///
/// Takes the serialized session JSON and the encrypted message JSON.
/// Returns the updated session JSON and the plaintext bytes.
#[wasm_bindgen]
pub fn ratchet_decrypt(session_json: &str, encrypted_message_json: &str) -> Result<JsValue, JsValue> {
    let mut session = RatchetSession::deserialize(session_json.as_bytes())
        .map_err(|e| JsValue::from_str(&format!("session deserialize failed: {e}")))?;

    let encrypted: EncryptedMessage = serde_json::from_str(encrypted_message_json)
        .map_err(|e| JsValue::from_str(&format!("invalid encrypted message JSON: {e}")))?;

    let plaintext = session
        .decrypt(&encrypted)
        .map_err(|e| JsValue::from_str(&format!("decrypt failed: {e}")))?;

    let new_session_json = String::from_utf8(session.serialize().map_err(|e| {
        JsValue::from_str(&format!("session serialize failed: {e}"))
    })?)
    .map_err(|e| JsValue::from_str(&format!("session JSON not UTF-8: {e}")))?;

    let result = RatchetDecryptResult {
        session_json: new_session_json,
        plaintext,
    };
    serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}
